use std::collections::HashMap;

use super::{
    parameter::{
        DeviceInfoResponse, IdentifyDeviceResponse, ManufacturerLabelResponse,
        ParameterDescriptionResponse, ProductDetailIdListResponse, SoftwareVersionLabelResponse,
        SupportedParametersResponse,
    },
    ManufacturerSpecificParameter, ParameterId, ProductCategory,
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
    pub supported_standard_parameters: Option<Vec<ParameterId>>,
    pub supported_manufacturer_specific_parameters:
        Option<HashMap<u16, ManufacturerSpecificParameter>>,
    pub is_identifying: Option<bool>,
    pub manufacturer_label: Option<String>,
    pub product_detail_id_list: Option<Vec<u16>>, // TODO use enum types
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
            supported_standard_parameters: None,
            supported_manufacturer_specific_parameters: None,
            is_identifying: None,
            manufacturer_label: None,
            product_detail_id_list: None,
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

    pub fn update_supported_parameters(&mut self, data: SupportedParametersResponse) {
        self.supported_standard_parameters = Some(data.standard_parameters);
        self.supported_manufacturer_specific_parameters =
            Some(data.manufacturer_specific_parameters);
    }

    pub fn update_identify_device(&mut self, data: IdentifyDeviceResponse) {
        self.is_identifying = Some(data.is_identifying);
    }

    pub fn update_parameter_description(&mut self, data: ParameterDescriptionResponse) {
        self.supported_manufacturer_specific_parameters = self
            .supported_manufacturer_specific_parameters
            .as_mut()
            .and_then(|parameter_hash_map| {
                parameter_hash_map
                    .get_mut(&data.parameter_id)
                    .and_then(|parameter| {
                        parameter.parameter_data_size = Some(data.parameter_data_size);
                        parameter.data_type = Some(data.data_type);
                        parameter.command_class = Some(data.command_class);
                        parameter.prefix = Some(data.prefix);
                        parameter.minimum_valid_value = Some(data.minimum_valid_value);
                        parameter.maximum_valid_value = Some(data.maximum_valid_value);
                        parameter.default_value = Some(data.default_value);
                        parameter.description = Some(data.description);
                        Some(parameter)
                    });
                Some(parameter_hash_map.to_owned())
            })
    }

    pub fn update_manufacturer_label(&mut self, data: ManufacturerLabelResponse) {
        self.manufacturer_label = Some(data.manufacturer_label);
    }

    pub fn update_product_detail_id_list(&mut self, data: ProductDetailIdListResponse) {
        self.product_detail_id_list = Some(data.product_detail_id_list);
    }
}
