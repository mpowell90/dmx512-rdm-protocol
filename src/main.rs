mod enttecdmxusbpro;
mod rdm;

use std::{
    collections::VecDeque,
    io::{self, Write},
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use serialport::available_ports;
use ux::u48;

use enttecdmxusbpro::{Driver, PacketRequestType, PacketResponseDataType, PacketResponseType};
use rdm::{
    device::{Device, DeviceUID},
    request::{
        DeviceInfoRequest, DeviceLabelRequest, DiscUniqueBranchRequest, DiscUnmuteRequest, Request,
    },
    response::{DeviceInfoResponse, DeviceLabelResponse, DiscUniqueBranchResponse, Response},
    CommandClass, ParameterId,
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
    let mut devices: Vec<Device> = Vec::new();

    // This is the known uid for the test Enttec DMXUSBPRO device
    let source_uid = DeviceUID::new(0x454e, 0x02137670);

    // Broadcast DiscUnmute to all devices so they accept DiscUniqueBranch messages
    let disc_unmute: Request<DiscUnmuteRequest> = Request::new(
        DeviceUID::broadcast_all_devices(),
        source_uid,
        0x00,
        0x01,
        0x0000, // Root Sub Device
        CommandClass::DiscoveryCommand,
        ParameterId::DiscUnMute,
        None,
    );

    let rdm_packet: Vec<u8> = disc_unmute.into();
    // println!("{:02X?}", &rdm_packet);
    queue.push_back(Driver::create_discovery_packet(&rdm_packet));

    // Broadcast DiscUniqueBranch to find all devices destination uids
    // TODO improve algorithm to handle branching properly
    let disc_unique_branch = Request::new(
        DeviceUID::broadcast_all_devices(),
        source_uid,
        0x00,
        0x01,
        0x0000,
        CommandClass::DiscoveryCommand,
        ParameterId::DiscUniqueBranch,
        Some(DiscUniqueBranchRequest::new(
            u48::new(0x000000000000),
            u48::new(0xffffffffffff),
        )),
    );

    let rdm_packet: Vec<u8> = disc_unique_branch.into();
    // println!("{:02X?}", &rdm_packet);
    queue.push_back(Driver::create_discovery_packet(&rdm_packet));

    // Send out 4 bytes very second
    thread::spawn(move || loop {
        // More complex solution allowing us to send data between threads using channels
        match rx.recv() {
            Ok(packet) => {
                transmitter
                    .write_all(&packet)
                    .expect("Failed to write to serial port");
                println!("Sent: {:02X?}", &packet);
            }
            Err(message) => println!("TX Error: {}", message),
        }
    });

    let mut last_device_count = 0;

    // TODO consider how we manage sending data over the channel
    // an option could be to pause this
    let mut waiting_response = false;

    loop {
        // Log any changes in devices
        if last_device_count != devices.len() {
            println!("Found device count: {:?}", devices);
            last_device_count = devices.len();
        }

        if !waiting_response && queue.len() > 0 {
            if let Some(packet) = queue.pop_front() {
                tx.send(packet).unwrap();
                waiting_response = true;
            }

            // match queue.pop_front() {
            //     Some(packet) => tx.send(packet),
            //     Err(message) => println!("Queue Error:", message)
            // }
        }

        // Pre-sized buffer
        let mut serial_buf: Vec<u8> = vec![0; 600];

        match driver.port.read(serial_buf.as_mut_slice()) {
            Ok(bytes) => {
                // println!("Bytes: {}", bytes);
                println!("Recv: {:02X?}", &serial_buf[..bytes]);

                let (response_type, packet_data_type, packet_data) =
                    Driver::parse_packet(&serial_buf[..bytes]);

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

                println!("Packet Data: {:02X?}", packet_data);

                match packet_data_type {
                    PacketResponseDataType::DiscoveryResponse => {
                        match DiscUniqueBranchResponse::try_from(packet_data.clone()) {
                            Ok(disc_unique_response) => {
                                println!("Parsed Discovery Response: {:#?}", &disc_unique_response);

                                devices.push(Device::from(disc_unique_response.device_uid));

                                // Set up subsequent messages to find for the newly found device
                                let get_device_info: Request<DeviceInfoRequest> = Request::new(
                                    disc_unique_response.device_uid,
                                    source_uid,
                                    0x00,
                                    0x01,
                                    0x0000,
                                    CommandClass::GetCommand,
                                    ParameterId::DeviceInfo,
                                    None,
                                );

                                let get_device_info_packet: Vec<u8> = get_device_info.into();

                                queue.push_back(Driver::create_rdm_packet(&get_device_info_packet));
                            }
                            Err(message) => {
                                println!("Error Message: {}", message);
                                println!("Unknown Discovery Packet: {:02X?}", packet_data);
                            }
                        }
                    }
                    PacketResponseDataType::RdmResponse => {
                        let parameter_id = ParameterId::from(&packet_data[21..=22]);
                        println!("ParameterId: {:?}", parameter_id);

                        match parameter_id {
                            ParameterId::DeviceInfo => {
                                match Response::<DeviceInfoResponse>::try_from(packet_data.clone())
                                {
                                    Ok(device_info_response) => {
                                        println!(
                                            "Parsed DeviceInfo Response: {:#02X?}",
                                            &device_info_response
                                        );
                                    }
                                    Err(message) => {
                                        println!("Error Message: {}", message);
                                    }
                                }
                            }
                            _ => println!("Unsupported Parameter Id: {:?}", parameter_id),
                        }
                    }
                    _ => println!("Null Response"),
                }

                // On next loop send the next packet in the queue
                waiting_response = false;
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        }
    }
}
