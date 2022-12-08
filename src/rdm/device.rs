use super::{
    response::{DeviceInfoResponse, SoftwareVersionLabelResponse, SupportParametersResponse, IdentifyDeviceResponse},
    ParameterId, ProductCategory,
};

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
    pub product_category: Option<ProductCategory>,
    pub software_version_id: Option<u32>,
    pub software_version_label: Option<String>,
    pub footprint: Option<u16>,
    pub current_personality: Option<u8>,
    pub total_personalities: Option<u8>,
    pub start_address: Option<u16>,
    pub sub_device_count: Option<u16>,
    pub sensor_count: Option<u8>,
    pub supported_parameters: Option<Vec<ParameterId>>,
    pub is_identifying: Option<bool>,
}

impl From<DeviceUID> for Device {
    fn from(device_uid: DeviceUID) -> Self {
        Device {
            uid: device_uid,
            protocol_version: None,
            model_id: None,
            model_description: None,
            product_category: None,
            software_version_id: None,
            software_version_label: None,
            footprint: None,
            current_personality: None,
            total_personalities: None,
            start_address: None,
            sub_device_count: None,
            sensor_count: None,
            supported_parameters: None,
            is_identifying: None,
        }
    }
}

impl Device {
    pub fn update_device_info(&mut self, data: DeviceInfoResponse) {
        self.protocol_version = Some(data.protocol_version);
        self.model_id = Some(data.model_id);
        self.product_category = Some(data.product_category);
        self.software_version_id = Some(data.software_version_id);
        self.footprint = Some(data.footprint);
        self.current_personality = Some(data.current_personality);
        self.total_personalities = Some(data.total_personalities);
        self.start_address = Some(data.start_address);
        self.sub_device_count = Some(data.sub_device_count);
        self.sensor_count = Some(data.sensor_count);
    }

    pub fn update_software_version_label(&mut self, data: SoftwareVersionLabelResponse) {
        self.software_version_label = Some(data.software_version_label);
    }

    pub fn update_supported_parameters(&mut self, data: SupportParametersResponse) {
        self.supported_parameters = Some(data.supported_parameters);
    }

    pub fn update_identify_device(&mut self, data: IdentifyDeviceResponse) {
        self.is_identifying = Some(data.is_identifying);
    }
}
