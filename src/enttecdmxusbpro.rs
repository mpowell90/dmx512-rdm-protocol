#![allow(unused)]
use byteorder::{LittleEndian, WriteBytesExt};
use serialport::{DataBits, SerialPort, StopBits};
use std::{
    cmp::PartialEq,
    io::{self, Write},
};
use thiserror::Error;

const DMX_START_CODE: u8 = 0x00;
const DMX_UNIVERSE_MAX_CHANNELS: usize = 512;

const ENTTEC_PACKET_START_BYTE: u8 = 0x7e;
const ENTTEC_PACKET_STOP_BYTE: u8 = 0xe7;

#[derive(Debug, Error)]
pub enum DriverError {
    #[error("invalid data length")]
    InvalidDataLength, // TODO should be used when input data is not correct
    #[error("invalid start byte")]
    InvalidStartByte,
    #[error("invalid stop byte")]
    InvalidStopByte,
    #[error("invalid packet type")]
    UnsupportedPacketType,
    #[error("malformed packet")]
    MalformedPacket,
    #[error("unknown driver error")]
    Unknown,
}

#[derive(PartialEq)]
pub enum PacketRequestType {
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

impl TryFrom<u8> for PacketRequestType {
    type Error = DriverError;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        let packet_type = match byte {
            0x01 => PacketRequestType::ReprogramFirmwareRequest,
            0x02 => PacketRequestType::ProgramFlashPageRequest,
            0x03 => PacketRequestType::GetWidgetParametersRequest,
            0x04 => PacketRequestType::SetWidgetParametersRequest,
            0x05 => PacketRequestType::ReceivedDmxPacket,
            0x06 => PacketRequestType::SendDmxPacketRequest, // Output only
            0x07 => PacketRequestType::SendRdmPacketRequest,
            0x08 => PacketRequestType::ReceiveDmxOnChange,
            0x09 => PacketRequestType::ReceiveDmxOnChangeOfStatePacket,
            0x0a => PacketRequestType::GetWidgetSerialNumber,
            0x0b => PacketRequestType::SendRdmDiscoveryRequest,
            _ => return Err(DriverError::UnsupportedPacketType),
        };
        Ok(packet_type)
    }
}

#[derive(Debug, PartialEq)]
pub enum PacketResponseType {
    SuccessResponse = 0x05,
    NullResponse = 0x0c,
}

impl TryFrom<u8> for PacketResponseType {
    type Error = DriverError;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        let packet_type = match byte {
            0x05 => PacketResponseType::SuccessResponse,
            0x0c => PacketResponseType::NullResponse,
            _ => return Err(DriverError::UnsupportedPacketType),
        };
        Ok(packet_type)
    }
}

pub struct Driver {
    pub port: Box<dyn SerialPort>,
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

    pub fn create_packet(label: PacketRequestType, buf: &[u8]) -> Vec<u8> {
        let mut packet: Vec<u8> = Vec::new();
        packet.write_u8(ENTTEC_PACKET_START_BYTE).unwrap();
        packet.write_u8(label as u8).unwrap();
        packet
            .write_u16::<LittleEndian>((buf.len() + 1) as u16)
            .unwrap();
        packet.write_u8(DMX_START_CODE).unwrap();
        packet.write(buf).unwrap();
        packet.write_u8(ENTTEC_PACKET_STOP_BYTE).unwrap();
        packet
    }

    pub fn create_rdm_packet(buf: &[u8]) -> Vec<u8> {
        let mut packet: Vec<u8> = Vec::new();
        packet.write_u8(ENTTEC_PACKET_START_BYTE).unwrap();
        packet
            .write_u8(PacketRequestType::SendRdmPacketRequest as u8)
            .unwrap();
        packet.write_u16::<LittleEndian>(buf.len() as u16).unwrap();
        packet.write(buf).unwrap();
        packet.write_u8(ENTTEC_PACKET_STOP_BYTE).unwrap();
        packet
    }

    pub fn create_discovery_packet(buf: &[u8]) -> Vec<u8> {
        let mut packet: Vec<u8> = Vec::new();
        packet.write_u8(ENTTEC_PACKET_START_BYTE).unwrap();
        packet
            .write_u8(PacketRequestType::SendRdmDiscoveryRequest as u8)
            .unwrap();
        packet.write_u16::<LittleEndian>(buf.len() as u16).unwrap();
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
        self.port.write(
            Self::create_packet(PacketRequestType::SendDmxPacketRequest, dmx.as_slice()).as_slice(),
        )
    }

    pub fn send_rdm_packet(&mut self, buf: &[u8]) -> Result<(), std::io::Error> {
        self.port
            .write_all(Self::create_packet(PacketRequestType::SendRdmPacketRequest, buf).as_slice())
    }

    pub fn is_complete_packet(packet: Vec<u8>) -> bool {
        if packet[0] != ENTTEC_PACKET_START_BYTE {
            return false;
            // return Err(DriverError::InvalidStartByte)
        }

        let parsed_packet_length = u16::from_le_bytes(packet[2..=3].try_into().unwrap()) as usize;

        println!("Length: {}, Parsed: {}", packet.len(), parsed_packet_length);

        if packet.len() < parsed_packet_length {
            return false;
        }

        if packet[packet.len() - 1] != ENTTEC_PACKET_STOP_BYTE {
            return false;
        }

        true
    }

    pub fn parse_packet(packet: &[u8]) -> Result<(PacketResponseType, Vec<u8>), DriverError> {
        if packet[0] != ENTTEC_PACKET_START_BYTE {
            return Err(DriverError::InvalidStartByte);
        }

        let packet_type = PacketResponseType::try_from(packet[1])?;

        let packet_length = if let Ok(bytes) = packet[2..=3].try_into() {
            u16::from_le_bytes(bytes)
        } else {
            return Err(DriverError::MalformedPacket);
        };

        // Minus the Enttec packet header and stop byte
        if (packet.len() - 4) < packet_length as usize {
            return Err(DriverError::InvalidDataLength);
        }

        if packet[packet.len() - 1] != ENTTEC_PACKET_STOP_BYTE {
            return Err(DriverError::InvalidStopByte);
        }

        let packet_data = if packet_type == PacketResponseType::NullResponse {
            Vec::new()
        } else {
            packet[5..(packet.len() - 1)].to_vec()
        };

        // let packet_data_type = if packet_type == PacketResponseType::NullResponse {
        //     // TODO consider if there is a better approach to this
        //     PacketResponseDataType::NullResponse
        // } else {
        //     let data_type_u16 = u16::from_be_bytes(packet[5..=6].try_into().unwrap());
        //     PacketResponseDataType::try_from(data_type_u16).unwrap()
        // };

        // println!("{:02X?}", packet_data_type);
        // // TODO consider if we should try from array slice instead of the additional u16 conversion
        // let packet_data_type =
        //     PacketResponseDataType::try_from(u16::from_be_bytes(packet[5..=6].try_into().unwrap()))
        //         .unwrap();

        Ok((packet_type, packet_data))
    }
}
