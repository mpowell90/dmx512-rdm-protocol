pub mod codec;
pub mod device;
pub mod parameter;
pub mod request;
pub mod response;
pub mod sensor;

use bytes::BytesMut;
use thiserror::Error;

pub const MIN_PACKET_LEN: usize = 26;

pub const SC_RDM: u8 = 0xcc;
pub const SC_SUB_MESSAGE: u8 = 0x01;

pub const DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE: u8 = 0xfe;
pub const DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE: u8 = 0xaa;

pub const BROADCAST_ALL_DEVICES_ID: u64 = 0xffffffffffff;
pub const SUB_DEVICE_ALL_CALL: u16 = 0xffff;
pub const ROOT_DEVICE: u16 = 0x0000;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ProtocolError {
    #[error("Invalid Start Code: {0}")]
    InvalidStartByte(u8),
    #[error("Invalid Sub Start Code: {0}")]
    InvalidSubStartByte(u8),
    #[error("Invalid message length: {0}, must be >= 24 and <= 255")]
    InvalidMessageLength(u8),
    #[error("Invalid checksum: {0}, expected: {1}")]
    InvalidChecksum(u16, u16), 
    #[error("Invalid ResponseType: {0}")]
    InvalidResponseType(u8),
    #[error("Invalid CommandClass: {0}")]
    InvalidCommandClass(u8),
    #[error("Unsupported ParameterId: {0}")]
    UnsupportedParameterId(u16),
    #[error("Invalid parameter data length: {0}, must be >= 0 and <= 231")]
    InvalidParameterDataLength(u8),
    #[error("Invalid discovery unique branch preamble")]
    InvalidDiscoveryUniqueBranchPreamble,
    #[error("Invalid stop byte: {0}")]
    InvalidStopByte(u8),
    #[error("Invalid PacketResponseType: {0}")]
    InvalidPacketResponseType(u8),
    #[error("Malformed packet")]
    MalformedPacket,
}

#[derive(Debug, Error)]
pub enum PacketTypeError {
    #[error("Unsupported PacketType: {0}")]
    UnsupportedPacketType(u16),
}

#[derive(Debug, PartialEq)]
pub enum PacketType {
    RdmResponse = 0xcc01,
    DiscoveryUniqueBranchResponse = 0xfefe,
}

impl TryFrom<u16> for PacketType {
    type Error = PacketTypeError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0xcc01 => Ok(Self::RdmResponse),
            0xfefe => Ok(Self::DiscoveryUniqueBranchResponse),
            _ => Err(PacketTypeError::UnsupportedPacketType(value)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CommandClass {
    DiscoveryCommand = 0x10,
    DiscoveryCommandResponse = 0x11,
    GetCommand = 0x20,
    GetCommandResponse = 0x21,
    SetCommand = 0x30,
    SetCommandResponse = 0x31,
}

impl TryFrom<u8> for CommandClass {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x10 => Ok(Self::DiscoveryCommand),
            0x11 => Ok(Self::DiscoveryCommandResponse),
            0x20 => Ok(Self::GetCommand),
            0x21 => Ok(Self::GetCommandResponse),
            0x30 => Ok(Self::SetCommand),
            0x31 => Ok(Self::SetCommandResponse),
            _ => Err(ProtocolError::InvalidCommandClass(value)),
        }
    }
}

pub fn bsd_16_crc_bytes_mut(packet: &mut BytesMut) -> u16 {
    packet
        .iter()
        .fold(0_u16, |sum, byte| (sum.overflowing_add(*byte as u16).0))
}

pub fn bsd_16_crc(packet: &[u8]) -> u16 {
    packet
        .iter()
        .fold(0_u16, |sum, byte| (sum.overflowing_add(*byte as u16).0))
}

pub fn is_checksum_valid(packet: &[u8]) -> bool {
    let packet_checksum =
        u16::from_be_bytes(packet[packet.len() - 2..packet.len()].try_into().unwrap());

    let calculated_checksum = bsd_16_crc(&packet[..packet.len() - 2]);

    packet_checksum == calculated_checksum
}
