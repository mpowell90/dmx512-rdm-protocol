mod enttecdmxusbpro;
mod rdm;

use futures::{stream::StreamExt, SinkExt};
use std::{
    collections::{HashMap, VecDeque},
    str,
    sync::{Arc, Mutex},
};

use bytes::{BufMut, BytesMut};
use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::Decoder;
use ux::u48;
use yansi::Paint;

use crate::enttecdmxusbpro::{EnttecDmxUsbProCodec, EnttecRequestMessage, EnttecResponseMessage};
use crate::rdm::device::{Curve, DmxPersonality, DmxSlot, ModulationFrequency, OutputResponseTime};
use crate::rdm::{
    device::{Device, DeviceUID},
    parameter::{
        create_standard_parameter_get_request_packet, CurveDescriptionGetRequest, DiscMuteRequest,
        DiscUniqueBranchRequest, DiscUnmuteRequest, DmxPersonalityDescriptionGetRequest,
        ModulationFrequencyDescriptionGetRequest, OutputResponseTimeDescriptionGetRequest,
        ParameterDescriptionGetRequest, ParameterId, SensorDefinitionRequest, REQUIRED_PARAMETERS,
    },
    DiscoveryRequest, GetRequest, GetResponseParameterData, RdmCodec, RdmResponseMessage,
    ROOT_DEVICE,
};

// const DEFAULT_TTY: &str = "/dev/tty.usbserial-EN137670";
const DEFAULT_TTY: &str = "/dev/ttyUSB0";

#[tokio::main]
async fn main() -> tokio_serial::Result<()> {
    let tty_path: String = DEFAULT_TTY.into();
    let port = tokio_serial::new(tty_path, 115_200).open_native_async()?;

    let stream = EnttecDmxUsbProCodec.framed(port);
    let (mut tx, mut rx) = stream.split();

    // let ready_to_send = Arc::new(AtomicBool::new(true));
    let ready_to_send = Arc::new(Mutex::new(true));
    let rx_ready_to_send = ready_to_send.clone();

    // Setup initial state
    let queue: Arc<Mutex<VecDeque<EnttecRequestMessage>>> = Arc::new(Mutex::new(VecDeque::new()));
    let queue_clone = queue.clone();
    // let mut queue: VecDeque<EnttecRequestMessage> = VecDeque::new();

    let devices: Arc<Mutex<HashMap<DeviceUID, Device>>> = Arc::new(Mutex::new(HashMap::new()));
    let devices_clone = devices.clone();
    // let mut devices: HashMap<DeviceUID, Device> = HashMap::new();

    // This is the known uid for the test Enttec DMXUSBPRO device
    let source_uid = DeviceUID::new(0x454e, 0x02137670);
    let port_id: u8 = 0x01;
    // TODO improve algorithm to handle branching properly
    let upper_bound_uid = u48::new(0xffffffffffff);
    let lower_bound_uid = u48::new(0x000000000000);

    tokio::spawn(async move {
        loop {
            let item = rx
                .next()
                .await
                .expect("Error awaiting future in RX stream.")
                .expect("Reading stream resulted in an error");

            println!(
                "{} {:02X?}",
                Paint::green("RX:"),
                Paint::green(item.clone())
            );

            let mut ready_to_send = rx_ready_to_send.lock().unwrap();

            let enttec_frame = match item {
                EnttecResponseMessage::NullResponse => {
                    *ready_to_send = true;
                    continue;
                }
                EnttecResponseMessage::SuccessResponse(bytes) => bytes,
            };

            let mut rdm_packet = if let Some(bytes) = enttec_frame {
                let mut bytesmut = BytesMut::new();
                bytesmut.put(bytes);
                bytesmut
            } else {
                *ready_to_send = true;
                continue;
            };

            let mut rdm_codec = RdmCodec;

            let rdm_frame = if let Ok(Some(rdm_frame)) = rdm_codec.decode(&mut rdm_packet) {
                rdm_frame
            } else {
                *ready_to_send = true;
                continue;
            };

            match rdm_frame {
                RdmResponseMessage::DiscoveryUniqueBranchResponse(device_uid) => {
                    // Device has been discovered!
                    println!("{:#02X?}", device_uid);

                    // Broadcast DiscUnmute to all devices so they accept DiscUniqueBranch messages
                    let disc_unmute: Vec<u8> = DiscMuteRequest
                        .discovery_request(device_uid, source_uid, 0x00, port_id, 0x0000)
                        .into();

                    queue_clone.lock().unwrap().push_back(
                        EnttecRequestMessage::SendRdmDiscoveryMessage(Some(disc_unmute)),
                    );

                    devices_clone
                        .lock()
                        .unwrap()
                        .insert(device_uid, Device::from(device_uid));

                    // Push subsequent required parameter requests for root device
                    for parameter_id in REQUIRED_PARAMETERS {
                        let packet = create_standard_parameter_get_request_packet(
                            parameter_id,
                            device_uid,
                            source_uid,
                            0x00,
                            port_id,
                            0x0000,
                        )
                        .unwrap();

                        queue_clone
                            .lock()
                            .unwrap()
                            .push_back(EnttecRequestMessage::SendRdmPacketRequest(Some(packet)));
                    }

                    // Retry same branch
                    let disc_unique_branch: Vec<u8> =
                        DiscUniqueBranchRequest::new(lower_bound_uid, upper_bound_uid)
                            .discovery_request(
                                DeviceUID::broadcast_all_devices(),
                                source_uid,
                                0x00,
                                port_id,
                                0x0000,
                            )
                            .try_into()
                            .unwrap();

                    queue_clone.lock().unwrap().push_back(
                        EnttecRequestMessage::SendRdmDiscoveryMessage(Some(disc_unique_branch)),
                    );
                }
                RdmResponseMessage::DiscoveryResponse(response) => {
                    // dbg!(response);

                    match response.parameter_id {
                        ParameterId::DiscMute => {
                            dbg!(response.parameter_data);
                        }
                        ParameterId::DiscUnMute => {
                            dbg!(response.parameter_data);
                        }
                        _ => todo!(),
                    }
                }
                RdmResponseMessage::GetResponse(response) => {
                    // // As we run discovery first, the device we have sent messages to will likely be available
                    // if !Response::is_checksum_valid(&rdm_packet) {
                    //     // TODO should retry sending packets here
                    //     println!("Checksum failed");
                    //     waiting_response = false;
                    //     continue;
                    // }

                    if let Some(found_device) =
                        devices_clone.lock().unwrap().get_mut(&response.source_uid)
                    {
                        let mut device;
                        if response.sub_device == ROOT_DEVICE {
                            device = found_device;
                        } else {
                            // find a sub_device if the received message was a response to a sub_device request
                            if let Some(sub_devices) = found_device.sub_devices.as_mut() {
                                if let Some(found_sub_device) =
                                    sub_devices.get_mut(&response.sub_device)
                                {
                                    device = found_sub_device;
                                } else {
                                    continue;
                                }
                            } else {
                                continue;
                            }
                        }

                        if let Some(parameter_data) = response.parameter_data {
                            match parameter_data {
                                GetResponseParameterData::DeviceInfo {
                                    protocol_version,
                                    model_id,
                                    product_category,
                                    software_version_id,
                                    footprint,
                                    current_personality,
                                    personality_count,
                                    start_address,
                                    sub_device_count,
                                    sensor_count,
                                } => {
                                    device.sub_device_count = sub_device_count;
                                    device.sensor_count = sensor_count;
                                    device.personality_count = personality_count;
                                    device.current_personality = Some(current_personality);
                                    device.software_version_id = Some(software_version_id);
                                    device.model_id = Some(model_id);
                                    device.product_category = Some(product_category);
                                    device.protocol_version = Some(protocol_version);
                                    device.footprint = Some(footprint);
                                    device.start_address = Some(start_address);

                                    if device.sub_device_id == ROOT_DEVICE
                                        && device.sub_device_count > 0
                                    {
                                        // initialise device.sub_device
                                        let mut sub_devices: HashMap<u16, Device> = HashMap::new();

                                        for sub_device_id in 1..=device.sub_device_count {
                                            sub_devices.insert(
                                                sub_device_id,
                                                Device::new(device.uid, sub_device_id),
                                            );
                                            // Push subsequent required parameter requests for root device
                                            for parameter_id in REQUIRED_PARAMETERS {
                                                let packet =
                                                    create_standard_parameter_get_request_packet(
                                                        parameter_id,
                                                        device.uid,
                                                        source_uid,
                                                        0x00,
                                                        port_id,
                                                        sub_device_id,
                                                    )
                                                    .unwrap();
                                                queue_clone.lock().unwrap().push_back(
                                                    EnttecRequestMessage::SendRdmPacketRequest(
                                                        Some(packet),
                                                    ),
                                                );
                                            }
                                        }

                                        device.sub_devices = Some(sub_devices);
                                    }

                                    if device.sensor_count > 0 {
                                        for idx in 0..device.sensor_count {
                                            let packet: Vec<u8> = SensorDefinitionRequest::new(idx)
                                                .get_request(
                                                    device.uid,
                                                    source_uid,
                                                    0x00,
                                                    port_id,
                                                    response.sub_device,
                                                )
                                                .into();

                                            queue_clone.lock().unwrap().push_back(
                                                EnttecRequestMessage::SendRdmPacketRequest(Some(
                                                    packet,
                                                )),
                                            );
                                        }
                                    }

                                    // TODO causes a nack response
                                    // if let Some(footprint) = device.footprint {
                                    //     if footprint > 0 {
                                    //         let packet = create_standard_parameter_get_request_packet(
                                    //             ParameterId::SlotInfo,
                                    //             device.uid,
                                    //             source_uid,
                                    //             0x00,
                                    //             port_id,
                                    //             response.sub_device,
                                    //         )
                                    //         .unwrap();
                                    //         queue_clone.lock().unwrap().push_back(EnttecRequestMessage::SendRdmPacketRequest(Some(packet)));
                                    //     }
                                    // }
                                }
                                GetResponseParameterData::SensorDefinition { sensor } => {
                                    device.sensors = if let Some(sensors) = device.sensors.as_mut()
                                    {
                                        sensors.insert(sensor.id, sensor);
                                        Some(sensors.to_owned())
                                    } else {
                                        Some(HashMap::from([(sensor.id, sensor)]))
                                    }
                                }
                                GetResponseParameterData::SoftwareVersionLabel {
                                    software_version_label,
                                } => {
                                    device.software_version_label = Some(software_version_label);
                                }
                                GetResponseParameterData::SupportedParameters {
                                    standard_parameters,
                                    manufacturer_specific_parameters,
                                } => {
                                    device.supported_standard_parameters =
                                        Some(standard_parameters);
                                    device.supported_manufacturer_specific_parameters =
                                        Some(manufacturer_specific_parameters);

                                    if let Some(standard_parameters) =
                                        device.supported_standard_parameters.clone()
                                    {
                                        for parameter_id in standard_parameters {
                                            match create_standard_parameter_get_request_packet(
                                                parameter_id,
                                                device.uid,
                                                source_uid,
                                                0x00,
                                                port_id,
                                                response.sub_device,
                                            ) {
                                                Ok(packet) => {
                                                    queue_clone.lock().unwrap().push_back(
                                                        EnttecRequestMessage::SendRdmPacketRequest(
                                                            Some(packet),
                                                        ),
                                                    );
                                                }
                                                Err(error) => {
                                                    println!(
                                                        "Error whilst creating packet: {}",
                                                        error
                                                    )
                                                }
                                            }
                                        }
                                    }

                                    // Iterate over manufacturer specific parameters to get their parameter descriptions
                                    if let Some(manufacturer_specific_parameters) =
                                        device.supported_manufacturer_specific_parameters.clone()
                                    {
                                        for parameter_id in
                                            manufacturer_specific_parameters.into_keys()
                                        {
                                            let get_manufacturer_label: Vec<u8> =
                                                ParameterDescriptionGetRequest::new(parameter_id)
                                                    .get_request(
                                                        device.uid,
                                                        source_uid,
                                                        0x00,
                                                        port_id,
                                                        response.sub_device,
                                                    )
                                                    .into();
                                            queue_clone.lock().unwrap().push_back(
                                                EnttecRequestMessage::SendRdmPacketRequest(Some(
                                                    get_manufacturer_label,
                                                )),
                                            );
                                        }
                                    }
                                }
                                GetResponseParameterData::ParameterDescription {
                                    parameter_id,
                                    parameter_data_size,
                                    data_type,
                                    command_class,
                                    prefix,
                                    minimum_valid_value,
                                    maximum_valid_value,
                                    default_value,
                                    description,
                                } => {
                                    device.supported_manufacturer_specific_parameters = device
                                        .supported_manufacturer_specific_parameters
                                        .as_mut()
                                        .and_then(|parameter_hash_map| {
                                            parameter_hash_map.get_mut(&parameter_id).and_then(
                                                |parameter| {
                                                    parameter.parameter_data_size =
                                                        Some(parameter_data_size);
                                                    parameter.data_type = Some(data_type);
                                                    parameter.command_class = Some(command_class);
                                                    parameter.prefix = Some(prefix);
                                                    parameter.minimum_valid_value =
                                                        Some(minimum_valid_value);
                                                    parameter.maximum_valid_value =
                                                        Some(maximum_valid_value);
                                                    parameter.default_value = Some(default_value);
                                                    parameter.description = Some(description);
                                                    Some(parameter)
                                                },
                                            );
                                            Some(parameter_hash_map.to_owned())
                                        })
                                }
                                GetResponseParameterData::IdentifyDevice { is_identifying } => {
                                    device.is_identifying = Some(is_identifying);
                                }
                                GetResponseParameterData::ManufacturerLabel {
                                    manufacturer_label,
                                } => {
                                    device.manufacturer_label = Some(manufacturer_label);
                                }
                                GetResponseParameterData::FactoryDefaults { factory_default } => {
                                    dbg!(factory_default);
                                }
                                GetResponseParameterData::ProductDetailIdList {
                                    product_detail_id_list,
                                } => {
                                    device.product_detail_id_list = Some(product_detail_id_list);
                                }
                                GetResponseParameterData::DeviceModelDescription {
                                    device_model_description,
                                } => {
                                    device.device_model_description =
                                        Some(device_model_description);
                                }
                                GetResponseParameterData::DmxPersonality {
                                    current_personality,
                                    personality_count,
                                } => {
                                    device.personality_count = personality_count;
                                    device.current_personality = Some(current_personality);

                                    for idx in 1..device.personality_count + 1 {
                                        let packet: Vec<u8> =
                                            DmxPersonalityDescriptionGetRequest::new(idx)
                                                .get_request(
                                                    device.uid,
                                                    source_uid,
                                                    0x00,
                                                    port_id,
                                                    response.sub_device,
                                                )
                                                .into();

                                        queue_clone.lock().unwrap().push_back(
                                            EnttecRequestMessage::SendRdmPacketRequest(Some(
                                                packet,
                                            )),
                                        );
                                    }
                                }
                                GetResponseParameterData::DmxPersonalityDescription {
                                    id,
                                    dmx_slots_required,
                                    description,
                                } => {
                                    let personality = DmxPersonality {
                                        id,
                                        dmx_slots_required,
                                        description,
                                    };
                                    device.personalities =
                                        if let Some(personalities) = device.personalities.as_mut() {
                                            personalities.insert(id, personality);
                                            Some(personalities.to_owned())
                                        } else {
                                            Some(HashMap::from([(id, personality)]))
                                        }
                                }
                                GetResponseParameterData::SlotInfo { dmx_slots } => {
                                    let mut hash_map: HashMap<u16, DmxSlot> = HashMap::new();
                                    for dmx_slot in dmx_slots {
                                        hash_map.insert(dmx_slot.id, dmx_slot);
                                    }
                                    device.dmx_slots = Some(hash_map);
                                }
                                GetResponseParameterData::DeviceHours { device_hours } => {
                                    device.device_hours = Some(device_hours);
                                }
                                GetResponseParameterData::LampHours { lamp_hours } => {
                                    device.lamp_hours = Some(lamp_hours);
                                }
                                GetResponseParameterData::LampStrikes { lamp_strikes } => {
                                    device.lamp_strikes = Some(lamp_strikes);
                                }
                                GetResponseParameterData::LampState { lamp_state } => {
                                    device.lamp_state = Some(lamp_state);
                                }
                                GetResponseParameterData::LampOnMode { lamp_on_mode } => {
                                    device.lamp_on_mode = Some(lamp_on_mode);
                                }
                                GetResponseParameterData::DevicePowerCycles {
                                    power_cycle_count,
                                } => {
                                    device.power_cycle_count = Some(power_cycle_count);
                                }
                                GetResponseParameterData::DisplayInvert {
                                    display_invert_mode,
                                } => {
                                    device.display_invert_mode = Some(display_invert_mode);
                                }
                                GetResponseParameterData::DimmerInfo {
                                    minimum_level_lower_limit,
                                    minimum_level_upper_limit,
                                    maximum_level_lower_limit,
                                    maximum_level_upper_limit,
                                    num_of_supported_curves,
                                    levels_resolution,
                                    minimum_levels_split_levels_supports,
                                } => {
                                    device.minimum_level_lower_limit =
                                        Some(minimum_level_lower_limit);
                                    device.minimum_level_upper_limit =
                                        Some(minimum_level_upper_limit);
                                    device.maximum_level_lower_limit =
                                        Some(maximum_level_lower_limit);
                                    device.maximum_level_upper_limit =
                                        Some(maximum_level_upper_limit);
                                    device.num_of_supported_curves = Some(num_of_supported_curves);
                                    device.levels_resolution = Some(levels_resolution);
                                    device.minimum_levels_split_levels_supports =
                                        Some(minimum_levels_split_levels_supports);
                                }
                                GetResponseParameterData::MinimumLevel {
                                    minimum_level_increasing,
                                    minimum_level_decreasing,
                                    on_below_minimum,
                                } => {
                                    device.minimum_level_increasing =
                                        Some(minimum_level_increasing);
                                    device.minimum_level_decreasing =
                                        Some(minimum_level_decreasing);
                                    device.on_below_minimum = Some(on_below_minimum);
                                }
                                GetResponseParameterData::MaximumLevel { maximum_level } => {
                                    device.maximum_level = Some(maximum_level);
                                }
                                GetResponseParameterData::Curve {
                                    current_curve,
                                    curve_count,
                                } => {
                                    device.curve_count = curve_count;
                                    device.current_curve = Some(current_curve);

                                    for idx in 1..device.curve_count + 1 {
                                        let packet: Vec<u8> = CurveDescriptionGetRequest::new(idx)
                                            .get_request(
                                                device.uid,
                                                source_uid,
                                                0x00,
                                                port_id,
                                                response.sub_device,
                                            )
                                            .into();

                                        queue_clone.lock().unwrap().push_back(
                                            EnttecRequestMessage::SendRdmPacketRequest(Some(
                                                packet,
                                            )),
                                        );
                                    }
                                }
                                GetResponseParameterData::CurveDescription { id, description } => {
                                    let curve = Curve { id, description };
                                    device.curves = if let Some(curves) = device.curves.as_mut() {
                                        curves.insert(id, curve);
                                        Some(curves.to_owned())
                                    } else {
                                        Some(HashMap::from([(id, curve)]))
                                    }
                                }
                                GetResponseParameterData::ModulationFrequency {
                                    current_modulation_frequency,
                                    modulation_frequency_count,
                                } => {
                                    device.modulation_frequency_count = modulation_frequency_count;
                                    device.current_modulation_frequency =
                                        Some(current_modulation_frequency);

                                    for idx in 1..device.modulation_frequency_count + 1 {
                                        let packet: Vec<u8> =
                                            ModulationFrequencyDescriptionGetRequest::new(idx)
                                                .get_request(
                                                    device.uid,
                                                    source_uid,
                                                    0x00,
                                                    port_id,
                                                    response.sub_device,
                                                )
                                                .into();

                                        queue_clone.lock().unwrap().push_back(
                                            EnttecRequestMessage::SendRdmPacketRequest(Some(
                                                packet,
                                            )),
                                        );
                                    }
                                }
                                GetResponseParameterData::ModulationFrequencyDescription {
                                    id,
                                    frequency,
                                    description,
                                } => {
                                    let modulation_frequency = ModulationFrequency {
                                        id,
                                        frequency,
                                        description,
                                    };
                                    device.modulation_frequencies =
                                        if let Some(modulation_frequencies) =
                                            device.modulation_frequencies.as_mut()
                                        {
                                            modulation_frequencies.insert(id, modulation_frequency);
                                            Some(modulation_frequencies.to_owned())
                                        } else {
                                            Some(HashMap::from([(id, modulation_frequency)]))
                                        }
                                }
                                GetResponseParameterData::OutputResponseTime {
                                    current_output_response_time,
                                    output_response_time_count,
                                } => {
                                    device.output_response_time_count = output_response_time_count;
                                    device.current_output_response_time =
                                        Some(current_output_response_time);

                                    for idx in 1..device.output_response_time_count + 1 {
                                        let packet: Vec<u8> =
                                            OutputResponseTimeDescriptionGetRequest::new(idx)
                                                .get_request(
                                                    device.uid,
                                                    source_uid,
                                                    0x00,
                                                    port_id,
                                                    response.sub_device,
                                                )
                                                .into();

                                        queue_clone.lock().unwrap().push_back(
                                            EnttecRequestMessage::SendRdmPacketRequest(Some(
                                                packet,
                                            )),
                                        );
                                    }
                                }
                                GetResponseParameterData::OutputResponseTimeDescription {
                                    id,
                                    description,
                                } => {
                                    let output_response_time =
                                        OutputResponseTime { id, description };
                                    device.output_response_times =
                                        if let Some(output_response_times) =
                                            device.output_response_times.as_mut()
                                        {
                                            output_response_times.insert(id, output_response_time);
                                            Some(output_response_times.to_owned())
                                        } else {
                                            Some(HashMap::from([(id, output_response_time)]))
                                        }
                                }
                                GetResponseParameterData::PowerState { power_state } => {
                                    device.power_state = Some(power_state);
                                }
                                GetResponseParameterData::PerformSelfTest { is_active } => {
                                    device.self_test_is_active = Some(is_active);
                                }
                                GetResponseParameterData::SelfTestDescription {
                                    self_test_id,
                                    description,
                                } => {
                                    todo!();
                                }
                                GetResponseParameterData::PresetPlayback { mode, level } => {
                                    device.preset_playback_mode = Some(mode);
                                    device.preset_playback_level = Some(level);
                                }
                                _ => todo!(),
                            }
                        }
                    } else {
                        println!("Device not found!");
                    }
                }
            }

            *ready_to_send = true;
        }
    });

    // Broadcast DiscUnmute to all devices so they accept DiscUniqueBranch messages
    let disc_unmute: Vec<u8> = DiscUnmuteRequest
        .discovery_request(
            DeviceUID::broadcast_all_devices(),
            source_uid,
            0x00,
            port_id,
            0x0000,
        )
        .try_into()
        .unwrap();

    queue
        .lock()
        .unwrap()
        .push_back(EnttecRequestMessage::SendRdmDiscoveryMessage(Some(
            disc_unmute,
        )));

    // Broadcast DiscUniqueBranch to find all devices destination uids
    let disc_unique_branch: Vec<u8> =
        DiscUniqueBranchRequest::new(lower_bound_uid, upper_bound_uid)
            .discovery_request(
                DeviceUID::broadcast_all_devices(),
                source_uid,
                0x00,
                port_id,
                0x0000,
            )
            .try_into()
            .unwrap();

    queue
        .lock()
        .unwrap()
        .push_back(EnttecRequestMessage::SendRdmDiscoveryMessage(Some(
            disc_unique_branch,
        )));

    loop {
        let mut is_ready = ready_to_send.lock().unwrap();
        let queue_length = queue.lock().unwrap().len();
        if *is_ready && queue_length > 0 {
            if let Some(packet) = queue.lock().unwrap().pop_front() {
                println!(
                    "{} {:02X?}",
                    Paint::yellow("TX:"),
                    Paint::yellow(packet.clone())
                );
                let write_result = tx.send(packet).await;

                match write_result {
                    Ok(_) => {
                        *is_ready = false;
                    }
                    Err(err) => println!("{:?}", err),
                }
            }
        } else if *is_ready && queue_length == 0 {
            break;
        }
    }

    println!("Devices: {:?}", devices.lock().unwrap());

    Ok(())
}

// //! A "tiny database" and accompanying protocol
// //!
// //! This example shows the usage of shared state amongst all connected clients,
// //! namely a database of key/value pairs. Each connected client can send a
// //! series of GET/SET commands to query the current value of a key or set the
// //! value of a key.
// //!
// //! This example has a simple protocol you can use to interact with the server.
// //! To run, first run this in one terminal window:
// //!
// //!     cargo run --example tinydb
// //!
// //! and next in another windows run:
// //!
// //!     cargo run --example connect 127.0.0.1:8080
// //!
// //! In the `connect` window you can type in commands where when you hit enter
// //! you'll get a response from the server for that command. An example session
// //! is:
// //!
// //!
// //!     $ cargo run --example connect 127.0.0.1:8080
// //!     GET foo
// //!     foo = bar
// //!     GET FOOBAR
// //!     error: no key FOOBAR
// //!     SET FOOBAR my awesome string
// //!     set FOOBAR = `my awesome string`, previous: None
// //!     SET foo tokio
// //!     set foo = `tokio`, previous: Some("bar")
// //!     GET foo
// //!     foo = tokio
// //!
// //! Namely you can issue two forms of commands:
// //!
// //! * `GET $key` - this will fetch the value of `$key` from the database and
// //!   return it. The server's database is initially populated with the key `foo`
// //!   set to the value `bar`
// //! * `SET $key $value` - this will set the value of `$key` to `$value`,
// //!   returning the previous value, if any.

// #![warn(rust_2018_idioms)]

// use tokio::net::TcpListener;
// use tokio_stream::StreamExt;
// use tokio_util::codec::{Framed, LinesCodec};

// use futures::SinkExt;
// use std::collections::HashMap;
// use std::env;
// use std::error::Error;
// use std::sync::{Arc, Mutex};

// /// The in-memory database shared amongst all clients.
// ///
// /// This database will be shared via `Arc`, so to mutate the internal map we're
// /// going to use a `Mutex` for interior mutability.
// struct Database {
//     map: Mutex<HashMap<String, String>>,
// }

// /// Possible requests our clients can send us
// enum Request {
//     Get { key: String },
//     Set { key: String, value: String },
// }

// /// Responses to the `Request` commands above
// enum Response {
//     Value {
//         key: String,
//         value: String,
//     },
//     Set {
//         key: String,
//         value: String,
//         previous: Option<String>,
//     },
//     Error {
//         msg: String,
//     },
// }

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn Error>> {
//     // Parse the address we're going to run this server on
//     // and set up our TCP listener to accept connections.
//     let addr = env::args()
//         .nth(1)
//         .unwrap_or_else(|| "127.0.0.1:8080".to_string());

//     let listener = TcpListener::bind(&addr).await?;
//     println!("Listening on: {}", addr);

//     // Create the shared state of this server that will be shared amongst all
//     // clients. We populate the initial database and then create the `Database`
//     // structure. Note the usage of `Arc` here which will be used to ensure that
//     // each independently spawned client will have a reference to the in-memory
//     // database.
//     let mut initial_db = HashMap::new();
//     initial_db.insert("foo".to_string(), "bar".to_string());
//     let db = Arc::new(Database {
//         map: Mutex::new(initial_db),
//     });

//     loop {
//         match listener.accept().await {
//             Ok((socket, _)) => {
//                 // After getting a new connection first we see a clone of the database
//                 // being created, which is creating a new reference for this connected
//                 // client to use.
//                 let db = db.clone();

//                 // Like with other small servers, we'll `spawn` this client to ensure it
//                 // runs concurrently with all other clients. The `move` keyword is used
//                 // here to move ownership of our db handle into the async closure.
//                 tokio::spawn(async move {
//                     // Since our protocol is line-based we use `tokio_codecs`'s `LineCodec`
//                     // to convert our stream of bytes, `socket`, into a `Stream` of lines
//                     // as well as convert our line based responses into a stream of bytes.
//                     let mut lines = Framed::new(socket, LinesCodec::new());

//                     // Here for every line we get back from the `Framed` decoder,
//                     // we parse the request, and if it's valid we generate a response
//                     // based on the values in the database.
//                     while let Some(result) = lines.next().await {
//                         match result {
//                             Ok(line) => {
//                                 let response = handle_request(&line, &db);

//                                 let response = response.serialize();

//                                 if let Err(e) = lines.send(response.as_str()).await {
//                                     println!("error on sending response; error = {:?}", e);
//                                 }
//                             }
//                             Err(e) => {
//                                 println!("error on decoding from socket; error = {:?}", e);
//                             }
//                         }
//                     }

//                     // The connection will be closed at this point as `lines.next()` has returned `None`.
//                 });
//             }
//             Err(e) => println!("error accepting socket; error = {:?}", e),
//         }
//     }
// }

// fn handle_request(line: &str, db: &Arc<Database>) -> Response {
//     let request = match Request::parse(line) {
//         Ok(req) => req,
//         Err(e) => return Response::Error { msg: e },
//     };

//     let mut db = db.map.lock().unwrap();
//     match request {
//         Request::Get { key } => match db.get(&key) {
//             Some(value) => Response::Value {
//                 key,
//                 value: value.clone(),
//             },
//             None => Response::Error {
//                 msg: format!("no key {}", key),
//             },
//         },
//         Request::Set { key, value } => {
//             let previous = db.insert(key.clone(), value.clone());
//             Response::Set {
//                 key,
//                 value,
//                 previous,
//             }
//         }
//     }
// }

// impl Request {
//     fn parse(input: &str) -> Result<Request, String> {
//         let mut parts = input.splitn(3, ' ');
//         match parts.next() {
//             Some("GET") => {
//                 let key = parts.next().ok_or("GET must be followed by a key")?;
//                 if parts.next().is_some() {
//                     return Err("GET's key must not be followed by anything".into());
//                 }
//                 Ok(Request::Get {
//                     key: key.to_string(),
//                 })
//             }
//             Some("SET") => {
//                 let key = match parts.next() {
//                     Some(key) => key,
//                     None => return Err("SET must be followed by a key".into()),
//                 };
//                 let value = match parts.next() {
//                     Some(value) => value,
//                     None => return Err("SET needs a value".into()),
//                 };
//                 Ok(Request::Set {
//                     key: key.to_string(),
//                     value: value.to_string(),
//                 })
//             }
//             Some(cmd) => Err(format!("unknown command: {}", cmd)),
//             None => Err("empty input".into()),
//         }
//     }
// }

// impl Response {
//     fn serialize(&self) -> String {
//         match *self {
//             Response::Value { ref key, ref value } => format!("{} = {}", key, value),
//             Response::Set {
//                 ref key,
//                 ref value,
//                 ref previous,
//             } => format!("set {} = `{}`, previous: {:?}", key, value, previous),
//             Response::Error { ref msg } => format!("error: {}", msg),
//         }
//     }
// }
