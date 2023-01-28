#![allow(unused)]
use byteorder::{LittleEndian, WriteBytesExt};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use serialport::{DataBits, SerialPort, StopBits};
use std::{
    cmp::PartialEq,
    io::{self, Write},
};
use std::{str};
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};

const DMX_START_CODE: u8 = 0x00;
const DMX_UNIVERSE_MAX_CHANNELS: usize = 512;

const ENTTEC_PACKET_START_BYTE: u8 = 0x7e;
const ENTTEC_PACKET_STOP_BYTE: u8 = 0xe7;

#[derive(Debug, Error)]
pub enum DriverError {
    #[error("invalid data length")]
    InvalidDataLength,
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

        Ok((packet_type, packet_data))
    }
}

#[derive(Default)]
pub struct EnttecDmxUsbProCodec;

impl EnttecDmxUsbProCodec {
    const SC_RDM: u8 = 0xcc;
    const SC_SUB_MESSAGE: u8 = 0x01;
    const START_BYTE: u8 = 0x7e;
    const STOP_BYTE: u8 = 0xe7;
    const FRAME_HEADER_FOOTER_SIZE: usize = 5;

    pub fn new() -> Self {
        EnttecDmxUsbProCodec
    }

    fn get_footer_position(src: &BytesMut) -> Option<usize> {
        let mut iter = src.iter().rev().enumerate().peekable(); //search from end (footer should be right at the end per spec)
        loop {
            let cur = iter.next();
            let next = iter.peek();

            match (cur, next) {
                (Some((_, cur_ele)), Some((i, next_ele))) => {
                    // TODO what do I need to do here??
                    // // both current and next ele are avail
                    // if cur_ele == &EnttecDmxUsbProCodec::BLOCK_FOOTER[1]
                    //     && *next_ele == &EnttecDmxUsbProCodec::BLOCK_FOOTER[0]
                    // {
                    //     //if the bytes are our footer
                    //     let index = src.len() - i - 1; //need an extra byte removed
                    //     dbg!("MLLP: Found footer at index {}", index);
                    //     return Some(index);
                    // }
                }
                (_, None) => {
                    dbg!("MLLP: Unable to find footer...");
                    return None;
                }
                _ => {} // keep looping
            }
        }
    }

    fn decode_packet(buf_to_process: &mut BytesMut) -> Result<Option<BytesMut>, std::io::Error> {
        if let Some(start_offset) = buf_to_process.iter().position(|b| *b == Self::START_BYTE) {
            //yes we do, do we have a footer?

            //trace!("MLLP: Found message header at index {}", start_offset);

            if let Some(end_offset) = EnttecDmxUsbProCodec::get_footer_position(buf_to_process) {
                //Is it worth passing a slice of src so we don't search the header chars?
                //Most of the time the start_offset == 0, so not sure it's worth it.

                let mut result = buf_to_process
                    .split_to(end_offset + 2) //get the footer bytes
                    .split_to(end_offset); // grab our data from the buffer, consuming (and losing) the footer

                result.advance(start_offset + 1); //move to start of data

                return Ok(Some(result));
            }
        }

        Ok(None)
    }
}

enum MessageLabel {
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

#[derive(Clone, Debug)]
pub enum EnttecRequestMessage {
    SendRdmPacketRequest(Option<BytesMut>), // TODO change to bytes
    SendRdmDiscoveryMessage(Option<BytesMut>), // TODO change to bytes
}

#[derive(Clone, Debug, PartialEq)]
pub enum EnttecResponseMessage {
    SuccessResponse(Option<Bytes>),
    NullResponse,
}

impl Encoder<EnttecRequestMessage> for EnttecDmxUsbProCodec {
    type Error = io::Error;

    fn encode(
        &mut self,
        item: EnttecRequestMessage,
        dst: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        let (label, data) = match item {
            EnttecRequestMessage::SendRdmPacketRequest(data) => {
                (MessageLabel::SendRdmPacketRequest, data)
            }
            EnttecRequestMessage::SendRdmDiscoveryMessage(data) => {
                (MessageLabel::SendRdmDiscoveryRequest, data)
            }
            _ => panic!("Unknown EnttecRequestMessage type"),
        };

        let data_length = if let Some(bytes) = data.clone() {
            bytes.len()
        } else {
            0
        };

        dst.reserve(data_length + 5);

        dst.put_u8(ENTTEC_PACKET_START_BYTE);
        dst.put_u8(label as u8);
        dst.put_u16_le(data_length as u16);

        if let Some(bytes) = data.clone() {
            dst.put(Bytes::from(bytes));
        }

        dst.put_u8(ENTTEC_PACKET_STOP_BYTE);

        Ok(())
    }
}

impl Decoder for EnttecDmxUsbProCodec {
    // type Item = (PacketResponseType, Vec<u8>);
    // type Item = Bytes;
    type Item = EnttecResponseMessage;
    type Error = io::Error;

    // Application Message Format
    // The PC based application program communicates with the Widget via the FTDI driver. The table below specifies
    // the general format of the messages between the application program and the FTDI driver.
    // Size in Bytes Description
    // [0] Start of message delimiter, 0x7E
    // [1] Label to identify type of message
    // [2] Data length LSB - valid range for data length is 0 to 600
    // [3] Data length MSB
    // [3 + Data length]
    // [3 + Data length + 1] End of message delimiter, 0xE7
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let len = src.len();

        if let Some(start_byte) = src.iter().position(|b| *b == Self::START_BYTE) {
            // We can safely ignore any bytes before and including the START_BYTE
            let _ = src.split_to(start_byte);

            if src.len() < Self::FRAME_HEADER_FOOTER_SIZE {
                return Ok(None);
            }

            let packet_length = if let Ok(bytes) = src[2..=3].try_into() {
                u16::from_le_bytes(bytes) as usize
            } else {
                return Ok(None);
                // println!("Malformed Packet");
                // return Err(io::Error::new(io::ErrorKind::Other, "Malformed Packet"));
            };

            if src.len() < packet_length + Self::FRAME_HEADER_FOOTER_SIZE {
                return Ok(None);
            }

            if src[packet_length + Self::FRAME_HEADER_FOOTER_SIZE - 1] != Self::STOP_BYTE {
                return Err(io::Error::new(io::ErrorKind::Other, "Invalid Stop Byte"));
            }

            let frame = src
                .split_to(packet_length + Self::FRAME_HEADER_FOOTER_SIZE)
                .freeze();

            let frame_type = PacketResponseType::try_from(frame[1]).unwrap();

            let frame = match frame_type {
                PacketResponseType::NullResponse => EnttecResponseMessage::NullResponse,
                PacketResponseType::SuccessResponse => EnttecResponseMessage::SuccessResponse(
                    Some(frame.slice(Self::FRAME_HEADER_FOOTER_SIZE..frame.len() - 1)),
                ),
            };

            Ok(Some(frame))
        } else {
            // TODO might need to return Err() here
            return Ok(None);
        }
    }
}

pub fn bsd_16_crc(packet: BytesMut) -> u16 {
    packet
        .iter()
        .fold(0_u16, |sum, byte| (sum.overflowing_add(*byte as u16).0))
}

#[cfg(test)]
mod tests {
    use crate::rdm::{CommandClass, ResponseType};

    use super::*;
    use bytes::Bytes;

    fn mock_enttec_frame(packet_type: PacketResponseType, data: BytesMut) -> BytesMut {
        let mut packet = BytesMut::new();
        packet.put_u8(EnttecDmxUsbProCodec::START_BYTE);
        packet.put_u8(packet_type as u8);
        packet.put_u16_le(data.len() as u16); // no bytes in response
        if data.len() > 0 {
            packet.put(data);
        }
        packet.put_u8(EnttecDmxUsbProCodec::STOP_BYTE);
        packet
    }

    fn mock_valid_null_response() -> BytesMut {
        mock_enttec_frame(PacketResponseType::NullResponse, BytesMut::new())
    }

    fn encoded_enttec_packet(rdm_packet: Bytes) -> Bytes {
        let mut packet = BytesMut::new();
        packet.put_u8(EnttecDmxUsbProCodec::START_BYTE);
        packet.put_u8(PacketRequestType::SendRdmPacketRequest as u8);
        packet.put_u16_le(rdm_packet.len() as u16);
        packet.put(rdm_packet);
        packet.put_u8(EnttecDmxUsbProCodec::STOP_BYTE);
        packet.freeze()
    }

    // RDM PID: Identify
    fn mock_valid_raw_rdm_request() -> Bytes {
        let manufacturer_id: u16 = 0x0001;
        let source_device_id: u32 = 0x00000001;
        let target_device_id: u32 = 0x00000002;

        let mut packet = BytesMut::new();
        packet.put_u8(EnttecDmxUsbProCodec::SC_RDM); // RDM Start Code
        packet.put_u8(EnttecDmxUsbProCodec::SC_SUB_MESSAGE); // RDM Sub Start Code
        packet.put_u8(25_u8); // Message Length: Range 24 to 255 excluding the checksum
        packet.put_u16(manufacturer_id);
        packet.put_u32(source_device_id);
        packet.put_u16(manufacturer_id);
        packet.put_u32(target_device_id);
        packet.put_u8(0x01); // Transaction Number
        packet.put_u8(0x01); // Port Id / Response Type
        packet.put_u8(0x00); // Message Count
        packet.put_u16(0x0000); // Sub Device (Root)
        packet.put_u8(CommandClass::SetCommand as u8);
        packet.put_u16(0x1000); // IdentifyDevice PID
        packet.put_u8(0x01); // Parameter Data Length
        packet.put_u8(0x01); // Set Target Device Identify to True
        packet.put_u16(bsd_16_crc(packet.clone()));
        packet.freeze()
    }

    // RDM PID: Identify
    fn mock_valid_enttec_rdm_response() -> BytesMut {
        let manufacturer_id: u16 = 0x0001;
        let source_device_id: u32 = 0x00000001;
        let target_device_id: u32 = 0x00000002;

        let mut packet = BytesMut::new();
        packet.put_u8(ENTTEC_PACKET_START_BYTE);
        packet.put_u8(PacketResponseType::SuccessResponse as u8);
        packet.put_u16_le(27_u16); // 0x1b00
        packet.put_u8(EnttecDmxUsbProCodec::SC_RDM); // RDM Start Code
        packet.put_u8(EnttecDmxUsbProCodec::SC_SUB_MESSAGE); // RDM Sub Start Code
        packet.put_u8(25_u8); // Message Length: Range 24 to 255 excluding the checksum
        packet.put_u16(manufacturer_id);
        packet.put_u32(target_device_id);
        packet.put_u16(manufacturer_id);
        packet.put_u32(source_device_id);
        packet.put_u8(0x01); // Transaction Number
        packet.put_u8(ResponseType::Ack as u8); // Port Id / Response Type
        packet.put_u8(0x00); // Message Count
        packet.put_u16(0x0000); // Sub Device (Root)
        packet.put_u8(CommandClass::SetCommandResponse as u8);
        packet.put_u16(0x1000); // IdentifyDevice PID
        packet.put_u8(0x01); // Parameter Data Length
        packet.put_u8(0x01); // Set Target Device Identify to True
        packet.put_u16(bsd_16_crc(packet.clone()));
        packet.put_u8(ENTTEC_PACKET_STOP_BYTE);
        packet
    }

    #[test]
    fn can_construct_without_error() {
        let _m = EnttecDmxUsbProCodec::new();
    }

    #[test]
    fn implements_default() {
        let _m = EnttecDmxUsbProCodec::default();
    }

    #[test]
    fn empty_buffer_correct_frame_single_decode() {
        let mut data = mock_valid_null_response();

        let mut codec = EnttecDmxUsbProCodec::new();

        let result = codec.decode(&mut data);

        match result {
            Ok(Some(frame)) => assert_eq!(frame, EnttecResponseMessage::NullResponse),
            _ => panic!("Failure for message with illegal trailing data"),
        }
    }

    #[test]
    fn empty_buffer_ignore_beginning_bytes_correct_frame_single_decode() {
        let mut data = BytesMut::new();
        data.put_u8(0xff);
        data.put(mock_valid_null_response());

        println!("data.len() before decode: {}", data.len());

        let mut codec = EnttecDmxUsbProCodec::new();

        let result = codec.decode(&mut data);

        println!("data.len() after decode: {}", data.len());

        match result {
            Ok(Some(frame)) => assert_eq!(frame, EnttecResponseMessage::NullResponse),
            _ => panic!("Failure for message with illegal trailing data"),
        }
    }

    #[test]
    fn empty_buffer_correct_frame_multiple_decodes() {
        let mut data = BytesMut::new();
        data.put_u8(EnttecDmxUsbProCodec::START_BYTE);
        data.put_u8(PacketResponseType::NullResponse as u8);

        let mut codec = EnttecDmxUsbProCodec::new();

        let result = codec.decode(&mut data);

        match result {
            Ok(None) => {
                println!("Full frame not found, awaiting more data");
            }
            _ => panic!("Failure for message with illegal trailing data"),
        }

        // Add packet_length but no stop byte
        data.put_u16_le(0); // no bytes in frame data

        let result = codec.decode(&mut data);

        match result {
            Ok(None) => {
                println!("Full frame not found, awaiting more data");
            }
            _ => panic!("Failure for message with illegal trailing data"),
        }

        // Add packet_length but no stop byte
        data.put_u8(EnttecDmxUsbProCodec::STOP_BYTE); // no bytes in frame data

        let result = codec.decode(&mut data);

        match result {
            Ok(Some(frame)) => assert_eq!(frame, EnttecResponseMessage::NullResponse),
            _ => panic!("Failure for message with illegal trailing data"),
        }
    }
}
