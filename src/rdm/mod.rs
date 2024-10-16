//! Data types and functionality for encoding and decoding RDM packets

pub mod error;
pub mod parameter;
pub mod request;
pub mod response;

use error::RdmError;
pub use macaddr;

pub const RDM_START_CODE_BYTE: u8 = 0xcc;
pub const RDM_SUB_START_CODE_BYTE: u8 = 0x01;

pub const DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE: u8 = 0xfe;
pub const DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE: u8 = 0xaa;

pub const MAX_RDM_FRAME_LENGTH: usize = 257;
pub const MAX_RDM_PARAMETER_DATA_LENGTH: usize = 231;

#[cfg(not(feature = "alloc"))]
use heapless::Vec;

#[cfg(feature = "alloc")]
pub type EncodedFrame = Vec<u8>;
#[cfg(not(feature = "alloc"))]
pub type EncodedFrame = Vec<u8, MAX_RDM_FRAME_LENGTH>;

#[cfg(feature = "alloc")]
pub type EncodedParameterData = Vec<u8>;
#[cfg(not(feature = "alloc"))]
pub type EncodedParameterData = Vec<u8, MAX_RDM_PARAMETER_DATA_LENGTH>;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceUID {
    pub manufacturer_id: u16,
    pub device_id: u32,
}

impl DeviceUID {
    pub const ALL_MANUFACTURERS_ID: u16 = 0xffff;
    pub const ALL_DEVICES_ID: u32 = 0xffffffff;

    pub const fn new(manufacturer_id: u16, device_id: u32) -> Self {
        Self {
            manufacturer_id,
            device_id,
        }
    }

    pub const fn new_dynamic(mut manufacturer_id: u16, device_id: u32) -> Self {
        manufacturer_id |= 0x8000;

        Self {
            manufacturer_id,
            device_id,
        }
    }

    pub const fn broadcast_to_devices_with_manufacturer_id(manufacturer_id: u16) -> Self {
        Self {
            manufacturer_id,
            device_id: Self::ALL_DEVICES_ID,
        }
    }

    pub const fn broadcast_to_devices_with_manufacturer_id_dynamic(
        mut manufacturer_id: u16,
        device_id: u32,
    ) -> Self {
        manufacturer_id |= 0x8000;

        Self {
            manufacturer_id,
            device_id,
        }
    }

    pub const fn broadcast_to_all_devices() -> Self {
        Self {
            manufacturer_id: Self::ALL_MANUFACTURERS_ID,
            device_id: Self::ALL_DEVICES_ID,
        }
    }

    pub fn is_dynamic(&self) -> bool {
        self.manufacturer_id & 0x8000 != 0
    }
}

impl From<[u8; 6]> for DeviceUID {
    fn from(bytes: [u8; 6]) -> Self {
        let manufacturer_id = u16::from_be_bytes([bytes[0], bytes[1]]);
        let device_id = u32::from_be_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]);

        DeviceUID {
            manufacturer_id,
            device_id,
        }
    }
}

impl From<DeviceUID> for [u8; 6] {
    fn from(uid: DeviceUID) -> Self {
        let manufacturer_id_bytes = uid.manufacturer_id.to_be_bytes();
        let device_id_bytes = uid.device_id.to_be_bytes();

        [
            manufacturer_id_bytes[0],
            manufacturer_id_bytes[1],
            device_id_bytes[0],
            device_id_bytes[1],
            device_id_bytes[2],
            device_id_bytes[3],
        ]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_static_device_uid() {
        let device_uid = DeviceUID::new(0x1234, 0x56789abc);

        assert_eq!(
            device_uid,
            DeviceUID {
                manufacturer_id: 0x1234,
                device_id: 0x56789abc
            }
        );
        assert!(!device_uid.is_dynamic());
    }

    #[test]
    fn should_create_dynamic_device_uid() {
        let device_uid = DeviceUID::new_dynamic(0x1234, 0x56789abc);

        assert_eq!(
            device_uid,
            DeviceUID {
                manufacturer_id: 0x9234,
                device_id: 0x56789abc
            }
        );
        assert!(device_uid.is_dynamic());
    }

    #[test]
    fn should_array_to_convert_device_uid() {
        assert_eq!(
            DeviceUID::from([0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc]),
            DeviceUID::new(0x1234, 0x56789abc)
        );
    }

    #[test]
    fn should_convert_device_uid_to_array() {
        assert_eq!(
            <[u8; 6]>::from(DeviceUID::new(0x1234, 0x56789abc)),
            [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc]
        );
    }
}
