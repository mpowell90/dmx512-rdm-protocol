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

use enttecdmxusbpro::{Driver, PacketResponseType};
use rdm::{
    bsd_16_crc,
    device::{Device, DeviceUID},
    parameter::{
        DeviceInfoRequest, DeviceInfoResponse, DiscUniqueBranchRequest, DiscUniqueBranchResponse,
        DiscUnmuteRequest, IdentifyDeviceGetRequest, IdentifyDeviceResponse,
        ManufacturerLabelGetRequest, ManufacturerLabelResponse, ParameterDescriptionResponse,
        ProductDetailIdListResponse, SoftwareVersionLabelRequest, SoftwareVersionLabelResponse,
        SupportedParametersRequest, SupportedParametersResponse,
    },
    DiscoveryRequest, GetRequest, PacketType, ParameterId, Protocol,
};

use crate::rdm::parameter::ParameterDescriptionRequest;

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

    // println!("{:02X?}", &rdm_packet);
    queue.push_back(Driver::create_discovery_packet(&disc_unmute));

    // Broadcast DiscUniqueBranch to find all devices destination uids
    // TODO improve algorithm to handle branching properly
    let disc_unique_branch: Vec<u8> =
        DiscUniqueBranchRequest::new(u48::new(0x000000000000), u48::new(0xffffffffffff))
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
    queue.push_back(Driver::create_discovery_packet(&disc_unique_branch));

    // Data sent between threads using a channel, on channel recv, send to serialport
    thread::spawn(move || loop {
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

    let mut last_device_count = 0;

    // TODO consider how we manage sending data over the channel
    // an option could be to pause this
    let mut waiting_response = false;
    // let mut clear_packet = false;

    let mut packet: Vec<u8> = Vec::new();

    loop {
        // Log any changes in devices
        if last_device_count != devices.len() {
            println!("Found device count: {:#?}", devices);
            last_device_count = devices.len();
        }

        if !waiting_response && packet.len() > 0 {
            // println!("Packet Cleared!");
            packet = Vec::new();
        }

        // Send the next message to the transmitter if there are any in the queue
        if !waiting_response && queue.len() > 0 {
            if let Some(packet) = queue.pop_front() {
                tx.send(packet).unwrap();
                waiting_response = true;
            }
        }

        // Pre-sized buffer
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
                                devices.insert(
                                    disc_unique_response.device_uid,
                                    Device::from(disc_unique_response.device_uid),
                                );

                                // Set up subsequent messages to find for the newly found device
                                let get_device_info: Vec<u8> = DeviceInfoRequest
                                    .get_request(
                                        disc_unique_response.device_uid,
                                        source_uid,
                                        0x00,
                                        port_id,
                                        0x0000,
                                    )
                                    .into();
                                queue.push_back(Driver::create_rdm_packet(&get_device_info));

                                let get_software_version_label: Vec<u8> =
                                    SoftwareVersionLabelRequest
                                        .get_request(
                                            disc_unique_response.device_uid,
                                            source_uid,
                                            0x00,
                                            port_id,
                                            0x0000,
                                        )
                                        .into();

                                queue.push_back(Driver::create_rdm_packet(
                                    &get_software_version_label,
                                ));

                                let get_supported_parameters: Vec<u8> = SupportedParametersRequest
                                    .get_request(
                                        disc_unique_response.device_uid,
                                        source_uid,
                                        0x00,
                                        port_id,
                                        0x0000,
                                    )
                                    .into();

                                queue.push_back(Driver::create_rdm_packet(
                                    &get_supported_parameters,
                                ));

                                let get_identify_device: Vec<u8> = IdentifyDeviceGetRequest
                                    .get_request(
                                        disc_unique_response.device_uid,
                                        source_uid,
                                        0x00,
                                        port_id,
                                        0x0000,
                                    )
                                    .into();
                                queue.push_back(Driver::create_rdm_packet(&get_identify_device));

                                let get_manufacturer_label: Vec<u8> = ManufacturerLabelGetRequest
                                    .get_request(
                                        disc_unique_response.device_uid,
                                        source_uid,
                                        0x00,
                                        port_id,
                                        0x0000,
                                    )
                                    .into();
                                queue.push_back(Driver::create_rdm_packet(&get_manufacturer_label));
                            }
                            Err(message) => {
                                println!("Error Message: {}", message);
                                println!("Unknown Discovery Packet: {:02X?}", rdm_packet);
                            }
                        }
                    }
                    PacketType::RdmResponse => {
                        // TODO check the checksum here, and attempt retry
                        // TODO also be good to lift this outside of this check
                        let packet_checksum = u16::from_be_bytes(
                            rdm_packet[rdm_packet.len() - 2..rdm_packet.len()]
                                .try_into()
                                .unwrap(),
                        );

                        let calculated_checksum =
                            bsd_16_crc(&rdm_packet[..rdm_packet.len() - 2].try_into().unwrap());

                        if packet_checksum != calculated_checksum {
                            // TODO should retry sending packets here
                            println!("Checksum failed");
                            waiting_response = false;
                            continue;
                        }

                        let parameter_id = ParameterId::from(&rdm_packet[21..=22]);
                        println!("ParameterId: {:?}", parameter_id);

                        match parameter_id {
                            ParameterId::DeviceInfo => {
                                match DeviceInfoResponse::parse_response(rdm_packet) {
                                    Ok(device_info_response) => {
                                        if let (Some(device), Some(data)) = (
                                            devices.get_mut(&device_info_response.source_uid),
                                            device_info_response.parameter_data,
                                        ) {
                                            device.update_device_info(data);
                                            println!("Device: {:#02X?}", device);
                                        } else {
                                            println!("Device can't be found, skipping...");
                                        }

                                        // TODO trigger more messages based on this data
                                    }
                                    Err(message) => {
                                        println!("Error Message: {}", message);
                                    }
                                }
                            }
                            ParameterId::SoftwareVersionLabel => {
                                match SoftwareVersionLabelResponse::parse_response(rdm_packet) {
                                    Ok(software_version_label_response) => {
                                        if let (Some(device), Some(data)) = (
                                            devices.get_mut(
                                                &software_version_label_response.source_uid,
                                            ),
                                            software_version_label_response.parameter_data,
                                        ) {
                                            device.update_software_version_label(data);
                                            println!("Device: {:#02X?}", device);
                                        } else {
                                            println!("Device can't be found, skipping...");
                                        }
                                    }
                                    Err(message) => {
                                        println!("Error Message: {}", message);
                                    }
                                }
                            }
                            ParameterId::SupportedParameters => {
                                match SupportedParametersResponse::parse_response(rdm_packet) {
                                    Ok(supported_parameters_response) => {
                                        if let (Some(device), Some(data)) = (
                                            devices
                                                .get_mut(&supported_parameters_response.source_uid),
                                            supported_parameters_response.parameter_data,
                                        ) {
                                            device.update_supported_parameters(data);
                                            println!("Device: {:#02X?}", device);

                                            // TODO iterate over parameters here
                                            if let Some(manufacturer_specific_parameters) = device
                                                .supported_manufacturer_specific_parameters
                                                .clone()
                                            {
                                                for parameter_id in
                                                    manufacturer_specific_parameters.into_keys()
                                                {
                                                    println!("PID: {:02X?}", parameter_id);
                                                    let get_manufacturer_label: Vec<u8> =
                                                        ParameterDescriptionRequest::new(
                                                            parameter_id,
                                                        )
                                                        .get_request(
                                                            device.uid, source_uid, 0x00, port_id,
                                                            0x0000,
                                                        )
                                                        .into();
                                                    queue.push_back(Driver::create_rdm_packet(
                                                        &get_manufacturer_label,
                                                    ));
                                                }
                                            }
                                        } else {
                                            println!("Device can't be found, skipping...");
                                        }
                                    }
                                    Err(message) => {
                                        println!("Error Message: {}", message);
                                    }
                                }
                            }
                            ParameterId::ParameterDescription => {
                                match ParameterDescriptionResponse::parse_response(rdm_packet) {
                                    Ok(parameter_description_response) => {
                                        if let (Some(device), Some(data)) = (
                                            devices.get_mut(
                                                &parameter_description_response.source_uid,
                                            ),
                                            parameter_description_response.parameter_data,
                                        ) {
                                            device.update_parameter_description(data);
                                            println!("Device: {:#02X?}", device);
                                        } else {
                                            println!("Device can't be found, skipping...");
                                        }
                                    }
                                    Err(message) => {
                                        println!("Error Message: {}", message);
                                    }
                                }
                            }
                            ParameterId::IdentifyDevice => {
                                match IdentifyDeviceResponse::parse_response(rdm_packet) {
                                    Ok(identify_device_response) => {
                                        if let (Some(device), Some(data)) = (
                                            devices.get_mut(&identify_device_response.source_uid),
                                            identify_device_response.parameter_data,
                                        ) {
                                            device.update_identify_device(data);
                                            println!("Device: {:#02X?}", device);
                                        } else {
                                            println!("Device can't be found, skipping...");
                                        }
                                    }
                                    Err(message) => {
                                        println!("Error Message: {}", message);
                                    }
                                }
                            }
                            ParameterId::ManufacturerLabel => {
                                match ManufacturerLabelResponse::parse_response(rdm_packet) {
                                    Ok(manufacturer_label_response) => {
                                        if let (Some(device), Some(data)) = (
                                            devices
                                                .get_mut(&manufacturer_label_response.source_uid),
                                            manufacturer_label_response.parameter_data,
                                        ) {
                                            device.update_manufacturer_label(data);
                                            println!("Device: {:#02X?}", device);
                                        } else {
                                            println!("Device can't be found, skipping...");
                                        }
                                    }
                                    Err(_) => {
                                        println!("Error occur whilst parsing");
                                    }
                                }
                            }
                            ParameterId::ProductDetailIdList => {
                                match ProductDetailIdListResponse::parse_response(rdm_packet) {
                                    Ok(product_detail_id_list_response) => {
                                        if let (Some(device), Some(data)) = (
                                            devices.get_mut(
                                                &product_detail_id_list_response.source_uid,
                                            ),
                                            product_detail_id_list_response.parameter_data,
                                        ) {
                                            device.update_product_detail_id_list(data);
                                            println!("Device: {:#02X?}", device);
                                        } else {
                                            println!("Device can't be found, skipping...");
                                        }
                                    }
                                    Err(_) => {
                                        println!("Error occur whilst parsing");
                                    }
                                }
                            }
                            _ => println!("Unsupported Parameter Id: {:?}", parameter_id),
                        }
                    }
                }

                // On next loop send the next packet in the queue
                waiting_response = false;
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        }
    }
}
