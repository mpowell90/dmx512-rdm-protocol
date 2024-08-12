pub mod error;
pub mod parameter;
pub mod request;
pub mod response;

use error::RdmError;

pub const RDM_START_CODE_BYTE: u8 = 0xcc;
pub const RDM_SUB_START_CODE_BYTE: u8 = 0x01;

pub const DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE: u8 = 0xfe;
pub const DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE: u8 = 0xaa;

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
    pub const ALL_MANUFACTURERS_ID: u16 = 0xffff;
    pub const ALL_DEVICES_ID: u32 = 0xffffffff;

    pub fn new(manufacturer_id: u16, device_id: u32) -> Self {
        DeviceUID {
            manufacturer_id,
            device_id,
        }
    }

    pub fn broadcast_to_all_devices() -> Self {
        DeviceUID {
            manufacturer_id: Self::ALL_MANUFACTURERS_ID,
            device_id: Self::ALL_DEVICES_ID,
        }
    }

    pub fn broadcast_to_devices_with_manufacturer_id(manufacturer_id: u16) -> Self {
        DeviceUID {
            manufacturer_id,
            device_id: Self::ALL_DEVICES_ID,
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
