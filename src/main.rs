mod enttecdmxusbpro;

use std::{
    io::{self, Write},
    time::Duration,
};

use serialport::available_ports;

use enttecdmxusbpro::Driver;

fn main() {
    let serialports = available_ports().unwrap();

    dbg!(&serialports);

    let port_info = serialports
        .iter()
        .find(|serialport| serialport.port_name.contains("usbserial"))
        .unwrap();

    dbg!(&port_info);

    let mut driver = Driver::open(&port_info.port_name);

    let packet = &[0xff, 0xff, 0xff];

    loop {
        match driver.send_dmx_packet(packet) {
            Ok(_) => {
                println!("{:?}", &packet);
                std::io::stdout().flush().unwrap();
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => eprintln!("{:?}", e),
        }
        std::thread::sleep(Duration::from_millis(1000_u64));
    }
}
