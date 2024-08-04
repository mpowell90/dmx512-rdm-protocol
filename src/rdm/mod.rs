pub mod parameter;
pub mod sensor;
pub mod response;
pub mod request;
pub mod codec;
pub mod device;

use std::array::TryFromSliceError;
use thiserror::Error;

pub const SC_RDM: u8 = 0xcc;
pub const SC_SUB_MESSAGE: u8 = 0x01;

pub const DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE: u8 = 0xfe;
pub const DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE: u8 = 0xaa;

pub const BROADCAST_ALL_DEVICES_ID: u64 = 0xffffffffffff;
pub const SUB_DEVICE_ALL_CALL: u16 = 0xffff;
pub const ROOT_DEVICE: u16 = 0x0000;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ProtocolError {
    #[error("Invalid start code")]
    InvalidStartCode,
    #[error("Invalid frame length: {0}")]
    InvalidFrameLength(u8),
    #[error("Invalid message length: {0}, must be >= 24 and <= 255")]
    InvalidMessageLength(u8),
    #[error("Invalid checksum: {0}, expected: {1}")]
    InvalidChecksum(u16, u16),
    #[error("Invalid ResponseType: {0}")]
    InvalidResponseType(u8),
    #[error("Invalid StatusType: {0}")]
    InvalidStatusType(u8),
    #[error("Invalid CommandClass: {0}")]
    InvalidCommandClass(u8),
    #[error("Unsupported Parameter, CommandClass: {0}, ParameterId: {1}")]
    UnsupportedParameter(u8, u16),
    #[error("Unsupported ParameterId: {0}")]
    UnsupportedParameterId(u16),
    #[error("Invalid parameter data length: {0}, must be >= 0 and <= 231")]
    InvalidParameterDataLength(u8),
    #[error("Invalid discovery unique branch preamble")]
    InvalidDiscoveryUniqueBranchPreamble,
    #[error("Failed to convert from bytes with nul")]
    FromBytesWithNulError {
        #[from]
        source: std::ffi::FromBytesWithNulError,
    },
    #[error("Invalid utf-8 sequence")]
    Utf8Error {
        #[from]
        source: std::str::Utf8Error,
    },
    #[error("Could not convert slice to array")]
    TryFromSliceError,
    #[error("Invalid ProductCategory: {0}")]
    InvalidProductCategory(u16),
    #[error("Invalid LampState: {0}")]
    InvalidLampState(u8),
    #[error("Invalid LampOnMode: {0}")]
    InvalidLampOnMode(u8),
    #[error("Invalid PowerState: {0}")]
    InvalidPowerState(u8),
    #[error("Invalid OnOffStates: {0}")]
    InvalidOnOffStates(u8),
    #[error("Invalid DisplayInvertMode: {0}")]
    InvalidDisplayInvertMode(u8),
    #[error("Invalid ResetDeviceMode: {0}")]
    InvalidResetDeviceMode(u8),
    #[error("Invalid SensorType: {0}")]
    InvalidSensorType(u8),
    #[error("Malformed packet")]
    MalformedPacket,
}

impl From<TryFromSliceError> for ProtocolError {
    fn from(_: TryFromSliceError) -> Self {
        ProtocolError::TryFromSliceError
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
