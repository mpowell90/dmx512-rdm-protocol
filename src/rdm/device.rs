#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DeviceUID {
    manufacturer_id: u16,
    device_id: u32,
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

impl From<u64> for DeviceUID {
    fn from(value: u64) -> Self {
        println!("{:02X?}", ((value >> 32_u64) & (0xffff as u64)));

        DeviceUID {
            manufacturer_id: ((value >> 32_u64) & (0xffff as u64)) as u16,
            device_id: (value & 0xffffffff) as u32,
        }
    }
}

impl From<DeviceUID> for u64 {
    fn from(device_uid: DeviceUID) -> u64 {
        ((device_uid.manufacturer_id as u64) << 32u64) + device_uid.device_id as u64
    }
}

impl From<Vec<u8>> for DeviceUID {
    fn from(buffer: Vec<u8>) -> Self {
        DeviceUID {
            // TODO we might be able to simplify this use .read_u16()
            manufacturer_id: u16::from_be_bytes(buffer[..2].try_into().unwrap()),
            device_id: u32::from_be_bytes(buffer[2..].try_into().unwrap()),
        }
    }
}

impl From<&[u8]> for DeviceUID {
    fn from(buffer: &[u8]) -> Self {
        DeviceUID {
            // TODO we might be able to simplify this use .read_u16()
            manufacturer_id: u16::from_be_bytes(buffer[..2].try_into().unwrap()),
            device_id: u32::from_be_bytes(buffer[2..].try_into().unwrap()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Device {
    pub uid: DeviceUID,
    pub protocol_version: Option<String>,
    pub model_id: Option<u16>,
    pub model_description: Option<String>,
    pub product_category_coarse: Option<u8>,
    pub product_category_fine: Option<u8>,
    pub software_version_id: Option<u32>,
    pub footprint: Option<u16>,
    pub current_personality: Option<u8>,
    pub total_personalities: Option<u8>,
    pub start_address: Option<u16>,
    pub sub_device_count: Option<u16>,
    pub sensor_count: Option<u8>,
}

impl From<DeviceUID> for Device {
    fn from(device_uid: DeviceUID) -> Self {
        Device {
            uid: device_uid,
            protocol_version: None,
            model_id: None,
            model_description: None,
            product_category_coarse: None,
            product_category_fine: None,
            software_version_id: None,
            footprint: None,
            current_personality: None,
            total_personalities: None,
            start_address: None,
            sub_device_count: None,
            sensor_count: None,
        }
    }
}
