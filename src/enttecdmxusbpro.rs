#![allow(unused)]
use byteorder::{LittleEndian, WriteBytesExt};
use serialport::{DataBits, SerialPort, StopBits};
use std::io::{self, Write};

const DMX_START_CODE: u8 = 0x00;
const DMX_UNIVERSE_MAX_CHANNELS: usize = 512;

const ENTTEC_PACKET_START_BYTE: u8 = 0x7e;
const ENTTEC_PACKET_STOP_BYTE: u8 = 0xe7;

enum EnttecDmxUsbProPacketType {
    ReprogramFirmwareRequest = 0x01,
    ProgramFlashPageRequest = 0x02,
    GetWidgetParametersRequest = 0x03,
    SetWidgetParametersRequest = 0x04,
    ReceivedDmxPacket = 0x05,
    SendDmxPacketRequest = 0x06, // Output only
    SendRdmPacketRequest = 0x07,
    ReceiveDmxOnChange = 0x08,
    ReceiveDmxOnChangeOfStatePacket = 0x09,
    GetWidgetSerialNumber = 0x0a,
    SendRdmDiscoveryRequest = 0x0b,
}

pub struct Driver {
    port: Box<dyn SerialPort>,
    path: Option<String>,
    serial_number: Option<String>,
}

impl Driver {
    // TODO Should return result
    pub fn open(path: &str) -> Driver {
        let builder = serialport::new(path, 115200)
            .data_bits(DataBits::Eight)
            .stop_bits(StopBits::One);

        let port = builder.open().unwrap_or_else(|e| {
            eprintln!("Failed to open \"{}\". Error: {}", &path, e);
            ::std::process::exit(1);
        });

        Driver {
            port,
            path: Some(path.to_string()),
            serial_number: None,
        }
    }

    pub fn path(&self) -> Option<String> {
        self.path.clone()
    }

    fn create_packet(buf: &[u8]) -> Vec<u8> {
        let mut packet: Vec<u8> = Vec::new();
        packet.write_u8(ENTTEC_PACKET_START_BYTE).unwrap();
        packet
            .write_u8(EnttecDmxUsbProPacketType::SendDmxPacketRequest as u8)
            .unwrap();
        packet
            .write_u16::<LittleEndian>((buf.len() + 1) as u16)
            .unwrap();
        packet.write_u8(DMX_START_CODE).unwrap();
        packet.write(buf).unwrap();
        packet.write_u8(ENTTEC_PACKET_STOP_BYTE).unwrap();
        packet
    }

    pub fn send_dmx_packet(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        let len = buf.len();
        if len > DMX_UNIVERSE_MAX_CHANNELS {
            return Err(std::io::Error::new(
                io::ErrorKind::Other,
                "Dmx packet out of bound. Must be smaller than 512 bytes",
            ));
        }
        let mut dmx: Vec<u8> = Vec::new();
        dmx.write_u8(DMX_START_CODE).unwrap();
        dmx.write(buf).unwrap();
        self.port.write(Self::create_packet(dmx.as_slice()).as_slice())
    }

    pub fn send_rdm_packet(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        self.port.write(Self::create_packet(buf).as_slice())
    }
}
