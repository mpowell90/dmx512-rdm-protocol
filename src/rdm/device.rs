use std::collections::HashMap;

use super::{
    parameter::{
        CurveDescriptionGetResponse, CurveGetResponse, DeviceHoursGetResponse, DeviceInfoResponse,
        DeviceModelDescriptionGetResponse, DmxPersonalityDescriptionGetResponse,
        DmxPersonalityGetResponse, IdentifyDeviceResponse, ManufacturerLabelResponse,
        ModulationFrequencyDescriptionGetResponse, ModulationFrequencyGetResponse,
        ParameterDescriptionGetResponse, ProductDetailIdListGetResponse, SoftwareVersionLabelGetResponse,
        SupportedParametersGetResponse,
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
pub struct DmxPersonality {
    pub id: u8,
    pub dmx_slots_required: u16,
    pub description: String,
}

#[derive(Clone, Debug)]
pub struct Curve {
    pub id: u8,
    pub description: String,
}

#[derive(Clone, Debug)]
pub struct ModulationFrequency {
    pub id: u8,
    pub frequency: u32,
    pub description: String,
}

#[derive(Clone, Debug)]
pub struct Sensor {
    pub id: u8,
    pub kind: u8,
    pub unit: u8,
    pub prefix: u8,
    pub range_minimum_value: u16,
    pub range_maximum_value: u16,
    pub normal_minimum_value: u16,
    pub normal_maximum_value: u16,
    pub recorded_value_support: u8,
    pub description: String,
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
    pub personality_count: u8,
    pub personalities: Option<HashMap<u8, DmxPersonality>>,
    pub start_address: Option<u16>,
    pub sub_device_count: u16,
    pub sub_devices: Option<HashMap<u16, Device>>,
    pub sensor_count: u8,
    pub sensors: Option<HashMap<u8, Sensor>>,
    pub supported_standard_parameters: Option<Vec<ParameterId>>,
    pub supported_manufacturer_specific_parameters:
        Option<HashMap<u16, ManufacturerSpecificParameter>>,
    pub is_identifying: Option<bool>,
    pub manufacturer_label: Option<String>,
    pub product_detail_id_list: Option<Vec<u16>>, // TODO use enum types
    pub device_model_description: Option<String>,
    pub device_hours: Option<u32>,
    pub current_curve: Option<u8>,
    pub curve_count: u8,
    pub curves: Option<HashMap<u8, Curve>>,
    pub current_modulation_frequency: Option<u8>,
    pub modulation_frequency_count: u8,
    pub modulation_frequencies: Option<HashMap<u8, ModulationFrequency>>,
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
            personality_count: 0,
            personalities: None,
            start_address: None,
            sub_device_count: 0,
            sub_devices: None,
            sensor_count: 0,
            sensors: None,
            supported_standard_parameters: None,
            supported_manufacturer_specific_parameters: None,
            is_identifying: None,
            manufacturer_label: None,
            product_detail_id_list: None,
            device_model_description: None,
            device_hours: None,
            current_curve: None,
            curve_count: 0,
            curves: None,
            current_modulation_frequency: None,
            modulation_frequency_count: 0,
            modulation_frequencies: None,
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
        self.personality_count = data.personality_count;
        self.start_address = Some(data.start_address);
        self.sub_device_count = data.sub_device_count;
        self.sensor_count = data.sensor_count;
    }

    pub fn update_software_version_label(&mut self, data: SoftwareVersionLabelGetResponse) {
        self.software_version_label = Some(data.software_version_label);
    }

    pub fn update_supported_parameters(&mut self, data: SupportedParametersGetResponse) {
        self.supported_standard_parameters = Some(data.standard_parameters);
        self.supported_manufacturer_specific_parameters =
            Some(data.manufacturer_specific_parameters);
    }

    pub fn update_identify_device(&mut self, data: IdentifyDeviceResponse) {
        self.is_identifying = Some(data.is_identifying);
    }

    pub fn update_parameter_description(&mut self, data: ParameterDescriptionGetResponse) {
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

    pub fn update_product_detail_id_list(&mut self, data: ProductDetailIdListGetResponse) {
        self.product_detail_id_list = Some(data.product_detail_id_list);
    }

    pub fn update_device_model_description(&mut self, data: DeviceModelDescriptionGetResponse) {
        self.device_model_description = Some(data.device_model_description);
    }

    pub fn update_dmx_personality_info(&mut self, data: DmxPersonalityGetResponse) {
        self.current_personality = Some(data.current_personality);
        self.personality_count = data.personality_count;
    }

    pub fn update_dmx_personality_description(
        &mut self,
        data: DmxPersonalityDescriptionGetResponse,
    ) {
        let personality = DmxPersonality {
            id: data.personality,
            dmx_slots_required: data.dmx_slots_required,
            description: data.description,
        };
        self.personalities = if let Some(personalities) = self.personalities.as_mut() {
            personalities.insert(data.personality, personality);
            Some(personalities.to_owned())
        } else {
            Some(HashMap::from([(data.personality, personality)]))
        }
    }

    pub fn update_device_hours(&mut self, data: DeviceHoursGetResponse) {
        self.device_hours = Some(data.device_hours);
    }

    pub fn update_curve_info(&mut self, data: CurveGetResponse) {
        self.current_curve = Some(data.current_curve);
        self.curve_count = data.curve_count;
    }

    pub fn update_curve_description(&mut self, data: CurveDescriptionGetResponse) {
        let curve = Curve {
            id: data.curve,
            description: data.description,
        };
        self.curves = if let Some(curves) = self.curves.as_mut() {
            curves.insert(data.curve, curve);
            Some(curves.to_owned())
        } else {
            Some(HashMap::from([(data.curve, curve)]))
        }
    }

    pub fn update_modulation_frequency_info(&mut self, data: ModulationFrequencyGetResponse) {
        self.current_modulation_frequency = Some(data.current_modulation_frequency);
        self.modulation_frequency_count = data.modulation_frequency_count;
    }

    pub fn update_modulation_frequency_description(
        &mut self,
        data: ModulationFrequencyDescriptionGetResponse,
    ) {
        let modulation_frequency = ModulationFrequency {
            id: data.modulation_frequency,
            frequency: data.frequency,
            description: data.description,
        };
        self.modulation_frequencies = if let Some(modulation_frequencies) = self.modulation_frequencies.as_mut() {
            modulation_frequencies.insert(data.modulation_frequency, modulation_frequency);
            Some(modulation_frequencies.to_owned())
        } else {
            Some(HashMap::from([(
                data.modulation_frequency,
                modulation_frequency,
            )]))
        }
    }
}
