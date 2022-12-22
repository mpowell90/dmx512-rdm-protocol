mod enttecdmxusbpro;
mod rdm;

use std::{
    collections::{HashMap, VecDeque},
    io::{self, Write},
    process,
    sync::mpsc,
    thread,
};

use serialport::available_ports;
use ux::u48;
use yansi::Paint;

use crate::enttecdmxusbpro::{Driver, PacketResponseType};
use crate::rdm::{
    device::{Device, DeviceUID},
    parameter::{
        create_standard_parameter_get_request_packet, CurveDescriptionGetRequest, DiscMuteRequest,
        DiscMuteResponse, DiscUniqueBranchRequest, DiscUniqueBranchResponse, DiscUnmuteRequest,
        DmxPersonalityDescriptionGetRequest, ModulationFrequencyDescriptionGetRequest,
        OutputResponseTimeDescriptionGetRequest, ParameterDescriptionGetRequest, ParameterId,
        SensorDefinitionRequest, REQUIRED_PARAMETERS,
    },
    DiscoveryRequest, GetRequest, PacketType, Response, ROOT_DEVICE,
};

fn main() {
    let serialports = available_ports().unwrap();
    // dbg!(&serialports);

    let port_info = serialports
        .iter()
        .find(|serialport| serialport.port_name.contains("usbserial"))
        .expect("Cannot connect to device");
    // dbg!(&port_info);

    let mut driver = Driver::open(&port_info.port_name);

    // Clone the port so we can pass it to a different thread
    let mut transmitter = driver.port.try_clone().expect("Failed to clone");

    // Initial cross thread communication channel
    let (tx, rx) = mpsc::channel::<Vec<u8>>();

    // Setup initial state
    let mut queue: VecDeque<Vec<u8>> = VecDeque::new();
    let mut devices: HashMap<DeviceUID, Device> = HashMap::new();

    // This is the known uid for the test Enttec DMXUSBPRO device
    let source_uid = DeviceUID::new(0x454e, 0x02137670);

    let port_id: u8 = 0x01;

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

    queue.push_back(Driver::create_discovery_packet(&disc_unmute));

    // TODO improve algorithm to handle branching properly
    let upper_bound_uid = u48::new(0xffffffffffff);
    let lower_bound_uid = u48::new(0x000000000000);

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

    queue.push_back(Driver::create_discovery_packet(&disc_unique_branch));

    // Data sent between threads using a channel, on channel recv, send to serialport
    let tx_handle = thread::spawn(move || loop {
        match rx.recv() {
            Ok(packet) => {
                transmitter
                    .write_all(&packet)
                    .expect("Failed to write to serial port");
                println!("{} {:02X?}", Paint::yellow("TX:"), Paint::yellow(&packet));
            }
            Err(message) => {
                println!("TX Error: {}", message);
                process::exit(1);
            }
        }
    });

    let mut waiting_response = false;
    let mut packet: Vec<u8> = Vec::new();

    loop {
        // Its possible for responses to be split across multiple serialport reads
        // However we must initialise the packet after each full packet has been handled
        if !waiting_response && packet.len() > 0 {
            packet = Vec::new();
        }

        // Send the next message to the transmitter if there are any in the queue
        if !waiting_response && queue.len() > 0 {
            if let Some(packet) = queue.pop_front() {
                tx.send(packet).unwrap();
                waiting_response = true;
            }
        }

        // Pre-sized buffer sized to max size of packet that can be returned from the enttec driver
        // In practice it is always around 64 bytes
        let mut serial_buf: Vec<u8> = vec![0; 600];

        match driver.port.read(serial_buf.as_mut_slice()) {
            Ok(bytes) => {
                // println!("Bytes: {}", bytes);
                println!(
                    "{} {:02X?}",
                    Paint::green("RX:"),
                    Paint::green(&serial_buf[..bytes])
                );

                packet.extend(&serial_buf[..bytes]);

                let (response_type, rdm_packet) = match Driver::parse_packet(&packet) {
                    Ok((response_type, rdm_packet)) => {
                        println!(
                            "{} {:02X?}",
                            Paint::magenta("RDM Packet:"),
                            Paint::magenta(&rdm_packet)
                        );
                        (response_type, rdm_packet)
                    }
                    Err(message) => {
                        println!("{:?}", message);
                        waiting_response = true;
                        continue;
                    }
                };

                // We can ignore null responses
                if response_type == PacketResponseType::NullResponse {
                    println!("Null Response");
                    waiting_response = false;
                    continue;
                } else if response_type != PacketResponseType::SuccessResponse {
                    println!("Unknown enttec packet type: {:02X?}", &serial_buf[1]);
                    waiting_response = false;
                    continue;
                }

                let rdm_packet_type =
                    PacketType::try_from(u16::from_be_bytes(rdm_packet[0..=1].try_into().unwrap()))
                        .unwrap();

                match rdm_packet_type {
                    PacketType::DiscoveryResponse => {
                        match DiscUniqueBranchResponse::try_from(rdm_packet.as_slice()) {
                            Ok(disc_unique_response) => {
                                // Device has been discovered!
                                println!("{:#02X?}", disc_unique_response);

                                // Broadcast DiscUnmute to all devices so they accept DiscUniqueBranch messages
                                let disc_unmute: Vec<u8> = DiscMuteRequest
                                    .discovery_request(
                                        disc_unique_response.device_uid,
                                        source_uid,
                                        0x00,
                                        port_id,
                                        0x0000,
                                    )
                                    .into();

                                // println!("{:02X?}", &rdm_packet);
                                queue.push_back(Driver::create_discovery_packet(&disc_unmute));

                                devices.insert(
                                    disc_unique_response.device_uid,
                                    Device::from(disc_unique_response.device_uid),
                                );

                                // Push subsequent required parameter requests for root device
                                for parameter_id in REQUIRED_PARAMETERS {
                                    let packet = create_standard_parameter_get_request_packet(
                                        parameter_id,
                                        disc_unique_response.device_uid,
                                        source_uid,
                                        0x00,
                                        port_id,
                                        0x0000,
                                    )
                                    .unwrap();
                                    queue.push_back(Driver::create_rdm_packet(&packet));
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

                                // println!("{:02X?}", &rdm_packet);
                                queue.push_back(Driver::create_discovery_packet(
                                    &disc_unique_branch,
                                ));
                            }
                            Err(message) => {
                                println!("Error Message: {}", message);
                                println!("Unknown Discovery Packet: {:02X?}", rdm_packet);
                            }
                        }
                    }
                    PacketType::RdmResponse => {
                        if !Response::is_checksum_valid(&rdm_packet) {
                            // TODO should retry sending packets here
                            println!("Checksum failed");
                            waiting_response = false;
                            continue;
                        }

                        let parameter_id = ParameterId::from(&rdm_packet[21..=22]);
                        println!("ParameterId: {:?}", parameter_id);

                        let response = Response::from(rdm_packet.as_slice());

                        // As we run discovery first, the device we have sent messages to will likely be available
                        if let Some(found_device) = devices.get_mut(&response.source_uid) {
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

                            match response.parameter_id {
                                ParameterId::DiscMute => {
                                    let disc_mute_response: DiscMuteResponse =
                                        response.parameter_data.into();
                                    println!("{:#02X?}", disc_mute_response);
                                    // TODO handle updating device
                                    // device.update_software_version_label(
                                    //     response.parameter_data.into(),
                                    // );
                                }
                                ParameterId::DeviceInfo => {
                                    device.update_device_info(response.parameter_data.into());

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
                                                queue.push_back(Driver::create_rdm_packet(&packet));
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

                                            queue.push_back(Driver::create_rdm_packet(&packet));
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
                                    //         queue.push_back(Driver::create_rdm_packet(&packet));
                                    //     }
                                    // }
                                }
                                ParameterId::SensorDefinition => {
                                    device.update_sensor_definition(response.parameter_data.into());
                                }
                                ParameterId::SoftwareVersionLabel => {
                                    device.update_software_version_label(
                                        response.parameter_data.into(),
                                    );
                                }
                                ParameterId::SupportedParameters => {
                                    device.update_supported_parameters(
                                        response.parameter_data.into(),
                                    );

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
                                                    queue.push_back(Driver::create_rdm_packet(
                                                        &packet,
                                                    ));
                                                }
                                                Err(error) => println!(
                                                    "Error whilst creating packet: {}",
                                                    error
                                                ),
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
                                            queue.push_back(Driver::create_rdm_packet(
                                                &get_manufacturer_label,
                                            ));
                                        }
                                    }
                                }
                                ParameterId::ParameterDescription => {
                                    device.update_parameter_description(
                                        response.parameter_data.into(),
                                    );
                                }
                                ParameterId::IdentifyDevice => {
                                    device.update_identify_device(response.parameter_data.into());
                                }
                                ParameterId::ManufacturerLabel => {
                                    device
                                        .update_manufacturer_label(response.parameter_data.into());
                                }
                                ParameterId::ProductDetailIdList => {
                                    device.update_product_detail_id_list(
                                        response.parameter_data.into(),
                                    );
                                }
                                ParameterId::DeviceModelDescription => {
                                    device.update_device_model_description(
                                        response.parameter_data.into(),
                                    );
                                }
                                ParameterId::DmxPersonality => {
                                    device.update_dmx_personality_info(
                                        response.parameter_data.into(),
                                    );

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

                                        queue.push_back(Driver::create_rdm_packet(&packet));
                                    }
                                }
                                ParameterId::DmxPersonalityDescription => {
                                    device.update_dmx_personality_description(
                                        response.parameter_data.into(),
                                    );
                                }
                                ParameterId::SlotInfo => {
                                    device.update_slot_info(response.parameter_data.into());
                                }
                                ParameterId::DeviceHours => {
                                    device.update_device_hours(response.parameter_data.into());
                                }
                                ParameterId::LampHours => {
                                    device.update_lamp_hours(response.parameter_data.into());
                                }
                                ParameterId::LampStrikes => {
                                    device.update_lamp_strikes(response.parameter_data.into());
                                }
                                ParameterId::LampState => {
                                    device.update_lamp_state(response.parameter_data.into());
                                }
                                ParameterId::LampOnMode => {
                                    device.update_lamp_on_mode(response.parameter_data.into());
                                }
                                ParameterId::DevicePowerCycles => {
                                    device
                                        .update_device_power_cycles(response.parameter_data.into());
                                }
                                ParameterId::DisplayInvert => {
                                    device.update_display_invert(response.parameter_data.into());
                                }
                                ParameterId::DimmerInfo => {
                                    device.update_dimmer_info(response.parameter_data.into());
                                }
                                ParameterId::MinimumLevel => {
                                    device.update_minimum_level(response.parameter_data.into());
                                }
                                ParameterId::MaximumLevel => {
                                    device.update_maximum_level(response.parameter_data.into());
                                }
                                ParameterId::Curve => {
                                    device.update_curve_info(response.parameter_data.into());

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

                                        queue.push_back(Driver::create_rdm_packet(&packet));
                                    }
                                }
                                ParameterId::CurveDescription => {
                                    device.update_curve_description(response.parameter_data.into());
                                }
                                ParameterId::ModulationFrequency => {
                                    device.update_modulation_frequency_info(
                                        response.parameter_data.into(),
                                    );

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

                                        queue.push_back(Driver::create_rdm_packet(&packet));
                                    }
                                }
                                ParameterId::ModulationFrequencyDescription => {
                                    device.update_modulation_frequency_description(
                                        response.parameter_data.into(),
                                    );
                                }
                                ParameterId::OutputResponseTime => {
                                    device.update_output_response_time_info(
                                        response.parameter_data.into(),
                                    );

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

                                        queue.push_back(Driver::create_rdm_packet(&packet));
                                    }
                                }
                                ParameterId::OutputResponseTimeDescription => {
                                    device.update_output_response_time_description(
                                        response.parameter_data.into(),
                                    );
                                }
                                ParameterId::PowerState => {
                                    device.update_power_state(response.parameter_data.into());
                                }
                                ParameterId::PerformSelfTest => {
                                    device.update_perform_self_state(response.parameter_data.into());
                                }
                                ParameterId::SelfTestDescription => {
                                    device.update_self_test_description(response.parameter_data.into());
                                }
                                ParameterId::PresetPlayback => {
                                    device.update_preset_playback(response.parameter_data.into());
                                }
                                _ => println!(
                                    "Unsupported Parameter Id: {:?}",
                                    response.parameter_id
                                ),
                            }

                            // println!("Devices: {:#02X?}", devices);
                        } else {
                            // TODO consider if we should remove it from the devices array
                            println!("Device can't be found, skipping...");
                        }
                    }
                }

                // On next loop send the next packet in the queue
                waiting_response = false;
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        }

        if !waiting_response && queue.len() == 0 {
            // println!("Devices: {:#02X?}", devices);
            for device in devices.into_values() {
                device.print();
            }
            break;
        }
    }

    drop(tx);

    tx_handle.join().unwrap();

    drop(driver.port);
}
