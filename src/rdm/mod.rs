pub mod parameter;
pub mod request;
pub mod response;

use std::array::TryFromSliceError;
use thiserror::Error;

pub const RDM_START_CODE_BYTE: u8 = 0xcc;
pub const RDM_SUB_START_CODE_BYTE: u8 = 0x01;

pub const DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE: u8 = 0xfe;
pub const DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE: u8 = 0xaa;

#[derive(Clone, Debug, Error, PartialEq)]
pub enum RdmError {
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
    #[error("Invalid NackReasonCode: {0}")]
    InvalidNackReasonCode(u16),
    #[error("Invalid StatusType: {0}")]
    InvalidStatusType(u8),
    #[error("Invalid CommandClass: {0}")]
    InvalidCommandClass(u8),
    #[error("Invalid CommandClass Implementation: {0}")]
    InvalidCommandClassImplementation(u8),
    #[error("Invalid ProductDetail: {0}")]
    InvalidProductDetail(u16),
    #[error("Unsupported Parameter, CommandClass: {0}, ParameterId: {1}")]
    UnsupportedParameter(u8, u16),
    #[error("Unsupported ParameterId: {0}")]
    UnsupportedParameterId(u16),
    #[error("Invalid parameter data length: {0}, must be >= 0 and <= 231")]
    InvalidParameterDataLength(u8),
    #[error("Invalid ParameterDataType: {0}")]
    InvalidParameterDataType(u8),
    #[error("Invalid SensorUnit: {0}")]
    InvalidSensorUnit(u8),
    #[error("Invalid SensorUnitPrefix: {0}")]
    InvalidSensorUnitPrefix(u8),
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
    #[error("Invalid SlotType: {0}")]
    InvalidSlotType(u8),
    #[error("Unsupported SlotIdDefinition: {0}")]
    UnsupportedSlotIdDefinition(u16),
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

impl From<TryFromSliceError> for RdmError {
    fn from(_: TryFromSliceError) -> Self {
        RdmError::TryFromSliceError
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CommandClass {
    DiscoveryCommand = 0x10,
    DiscoveryCommandResponse = 0x11,
    GetCommand = 0x20,
    GetCommandResponse = 0x21,
    SetCommand = 0x30,
    SetCommandResponse = 0x31,
}

impl TryFrom<u8> for CommandClass {
    type Error = RdmError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x10 => Ok(Self::DiscoveryCommand),
            0x11 => Ok(Self::DiscoveryCommandResponse),
            0x20 => Ok(Self::GetCommand),
            0x21 => Ok(Self::GetCommandResponse),
            0x30 => Ok(Self::SetCommand),
            0x31 => Ok(Self::SetCommandResponse),
            _ => Err(RdmError::InvalidCommandClass(value)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DeviceUID {
    pub manufacturer_id: u16,
    pub device_id: u32,
}

impl DeviceUID {
    pub fn new(manufacturer_id: u16, device_id: u32) -> Self {
        DeviceUID {
            manufacturer_id,
            device_id,
        }
    }

    pub fn broadcast_all_devices() -> Self {
        DeviceUID {
            manufacturer_id: 0xffff,
            device_id: 0xffffffff,
        }
    }
}

pub fn bsd_16_crc(packet: &[u8]) -> u16 {
    packet
        .iter()
        .fold(0_u16, |sum, byte| (sum.overflowing_add(*byte as u16).0))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SubDeviceId {
    RootDevice,
    Id(u16),
    AllDevices,
}

impl From<u16> for SubDeviceId {
    fn from(value: u16) -> SubDeviceId {
        match value {
            0x0000 => SubDeviceId::RootDevice,
            0xffff => SubDeviceId::AllDevices,
            _ => SubDeviceId::Id(value),
        }
    }
}

impl From<SubDeviceId> for u16 {
    fn from(sub_device: SubDeviceId) -> u16 {
        match sub_device {
            SubDeviceId::RootDevice => 0x0000,
            SubDeviceId::AllDevices => 0xffff,
            SubDeviceId::Id(id) => id,
        }
    }
}
