use crate::{
    parameter::{
        DisplayInvertMode, LampOnMode, LampState, ManufacturerSpecificParameter, ParameterId,
        PowerState, ProductCategory,
    },
    sensor::Sensor,
    StatusType,
};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
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

impl From<u64> for DeviceUID {
    fn from(value: u64) -> Self {
        DeviceUID {
            manufacturer_id: ((value >> 32_u64) & 0xffff) as u16,
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
            manufacturer_id: u16::from_be_bytes(buffer[..2].try_into().unwrap()),
            device_id: u32::from_be_bytes(buffer[2..].try_into().unwrap()),
        }
    }
}

impl From<&[u8]> for DeviceUID {
    fn from(buffer: &[u8]) -> Self {
        DeviceUID {
            manufacturer_id: u16::from_be_bytes(buffer[..2].try_into().unwrap()),
            device_id: u32::from_be_bytes(buffer[2..].try_into().unwrap()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusMessage {
    pub sub_device_id: u16,
    pub status_type: StatusType,
    pub status_message_id: u16, // TODO reference appendix B for status message IDs
    pub data_value1: u16,
    pub data_value2: u16,
}

impl StatusMessage {
    pub fn new(
        sub_device_id: u16,
        status_type: StatusType,
        status_message_id: u16,
        data_value1: u16,
        data_value2: u16,
    ) -> Self {
        StatusMessage {
            sub_device_id,
            status_type,
            status_message_id,
            data_value1,
            data_value2,
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

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct OutputResponseTime {
    pub id: u8,
    pub description: String,
}

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SlotInfo {
    pub id: u16,
    pub kind: u8, // TODO use enum
    pub label_id: u16,
}

impl SlotInfo {
    pub fn new(id: u16, kind: u8, label_id: u16) -> Self {
        Self { id, kind, label_id }
    }
}

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DefaultSlotValue {
    pub id: u16,
    pub value: u8,
}

impl DefaultSlotValue {
    pub fn new(id: u16, value: u8) -> Self {
        Self { id, value }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DmxSlot {
    pub id: u16,
    pub kind: u8, // TODO use enum
    pub label_id: u16,
    pub description: Option<String>,
}

impl DmxSlot {
    pub fn new(id: u16, kind: u8, label_id: u16, description: Option<String>) -> Self {
        Self {
            id,
            kind,
            label_id,
            description,
        }
    }
}

// TODO add minimum_level, maximum_level, output_response_time etc
#[derive(Clone, Debug, Default)]
pub struct Device {
    pub uid: DeviceUID,
    pub protocol_version: Option<String>,
    pub model_id: Option<u16>,
    pub model_description: Option<String>,
    pub product_category: Option<ProductCategory>,
    pub software_version_id: Option<u32>,
    pub software_version_label: Option<String>,
    pub footprint: u16,
    pub current_personality: Option<u8>,
    pub personality_count: u8,
    pub personalities: Option<HashMap<u8, DmxPersonality>>,
    pub start_address: u16,
    pub dmx_slots: Option<HashMap<u16, DmxSlot>>,
    pub sub_device_id: u16,
    pub sub_device_count: u16,
    pub sensor_count: u8,
    pub sensors: Option<HashMap<u8, Sensor>>,
    pub supported_standard_parameters: Option<Vec<ParameterId>>,
    pub supported_manufacturer_specific_parameters:
        Option<HashMap<u16, ManufacturerSpecificParameter>>,
    pub is_identifying: Option<bool>,
    pub manufacturer_label: Option<String>,
    pub device_label: Option<String>,
    pub product_detail_id_list: Option<Vec<u16>>, // TODO use enum types
    pub device_model_description: Option<String>,
    pub device_hours: Option<u32>,
    pub lamp_hours: Option<u32>,
    pub lamp_strikes: Option<u32>,
    pub lamp_state: Option<LampState>,
    pub lamp_on_mode: Option<LampOnMode>,
    pub power_cycle_count: Option<u32>,
    pub display_invert_mode: Option<DisplayInvertMode>,
    pub minimum_level_lower_limit: Option<u16>,
    pub minimum_level_upper_limit: Option<u16>,
    pub maximum_level_lower_limit: Option<u16>,
    pub maximum_level_upper_limit: Option<u16>,
    pub num_of_supported_curves: Option<u8>,
    pub levels_resolution: Option<u8>,
    pub minimum_levels_split_levels_supports: Option<u8>,
    pub minimum_level_increasing: Option<u16>,
    pub minimum_level_decreasing: Option<u16>,
    pub on_below_minimum: Option<u8>,
    pub maximum_level: Option<u16>,
    pub current_curve: Option<u8>,
    pub curve_count: u8,
    pub curves: Option<HashMap<u8, Curve>>,
    pub current_modulation_frequency: Option<u8>,
    pub modulation_frequency_count: u8,
    pub modulation_frequencies: Option<HashMap<u8, ModulationFrequency>>,
    pub current_output_response_time: Option<u8>,
    pub output_response_time_count: u8,
    pub output_response_times: Option<HashMap<u8, OutputResponseTime>>,
    pub power_state: Option<PowerState>,
    pub self_test_is_active: Option<bool>,
    pub preset_playback_mode: Option<u16>,
    pub preset_playback_level: Option<u8>,
}

impl From<DeviceUID> for Device {
    fn from(device_uid: DeviceUID) -> Self {
        Device {
            uid: device_uid,
            ..Default::default()
        }
    }
}

impl Device {
    pub fn new(uid: DeviceUID, sub_device_id: u16) -> Self {
        Device {
            uid,
            sub_device_id,
            ..Default::default()
        }
    }

    pub fn print(self) {
        println!("UID: {:#02X?}", self.uid);

        println!("\n> Production Info:");
        if let Some(manufacturer_label) = self.manufacturer_label {
            println!("Manufacturer Label: {:?}", manufacturer_label);
        }
        if let Some(model_id) = self.model_id {
            println!("Model ID: {:#02X?}", model_id);
        }
        if let Some(model_description) = self.model_description {
            println!("Model Description: {:#02X?}", model_description);
        }
        if let Some(device_model_description) = self.device_model_description {
            println!(
                "Device Model Description: {:#02X?}",
                device_model_description
            );
        }
        if let Some(protocol_version) = self.protocol_version {
            println!("RDM Protocol Version: {:?}", protocol_version);
        }
        if let Some(software_version_id) = self.software_version_id {
            println!("Software Version ID: {:?}", software_version_id);
        }
        if let Some(software_version_label) = self.software_version_label {
            println!("Software Version: {:?}", software_version_label);
        }
        if let Some(device_hours) = self.device_hours {
            println!("Device Hours: {:?}", device_hours);
        }
        if let Some(product_category) = self.product_category {
            println!("Product Category: {:?}", product_category);
        }

        if let Some(manufacturer_specific) = self.supported_manufacturer_specific_parameters {
            for parameter in manufacturer_specific.into_values() {
                if let Some(description) = parameter.description {
                    println!("{:?}", description);
                }
            }
        }

        println!("\n> Supported Parameters:");
        if let Some(standard_parameters) = self.supported_standard_parameters {
            for parameter in standard_parameters {
                println!("{:?}", parameter);
            }
        }
        if let Some(product_detail_id_list) = self.product_detail_id_list {
            println!("Product Detail ID List: {:#?}", product_detail_id_list);
        }

        println!("\n> DMX Setup:");
        println!("Start Address: {:?}", self.start_address);

        if self.personality_count > 0 {
            println!("\n> Personalities:");
            if let (Some(current_personality), Some(personalities)) =
                (self.current_personality, self.personalities)
            {
                let mut personalities_vec =
                    personalities.into_values().collect::<Vec<DmxPersonality>>();
                personalities_vec.sort_by(|a, b| a.to_owned().id.cmp(&b.id));

                for personality in personalities_vec {
                    let current: &str = if personality.id == current_personality {
                        "- Current"
                    } else {
                        ""
                    };
                    println!(
                        "[{}] {:#?} {}",
                        personality.id, personality.description, current
                    );
                }
            }
        }

        if self.footprint > 0 {
            println!("DMX Footprint: {:?}", self.footprint);
            if let Some(dmx_slots) = self.dmx_slots {
                println!("\nDMX Slots: {:#?}", dmx_slots);
            }
        }

        if let (
            Some(minimum_level_lower_limit),
            Some(minimum_level_upper_limit),
            Some(maximum_level_lower_limit),
            Some(maximum_level_upper_limit),
        ) = (
            self.minimum_level_lower_limit,
            self.minimum_level_upper_limit,
            self.maximum_level_lower_limit,
            self.maximum_level_upper_limit,
        ) {
            println!("minimum_level_lower_limit: {:?}", minimum_level_lower_limit);
            println!("minimum_level_upper_limit: {:?}", minimum_level_upper_limit);
            println!("maximum_level_lower_limit: {:?}", maximum_level_lower_limit);
            println!("maximum_level_upper_limit: {:?}", maximum_level_upper_limit);
        }
        // pub num_of_supported_curves: Option<u8>,
        // pub levels_resolution: Option<u8>,
        // pub minimum_levels_split_levels_supports: Option<u8>,
        // pub minimum_level_increasing: Option<u16>,
        // pub minimum_level_decreasing: Option<u16>,
        // pub on_below_minimum: Option<u8>,
        // pub maximum_level: Option<u16>,

        if self.sensor_count > 0 {
            println!("\n> Sensors:");
            println!("Sensor Count: {:?}", self.sensor_count);
            if let Some(sensors) = self.sensors {
                println!("Sensors: {:#?}", sensors);
            }
        }

        println!("\n> Lamp Details:");
        if let Some(lamp_hours) = self.lamp_hours {
            println!("lamp_hours: {:?}", lamp_hours);
        }
        if let Some(lamp_strikes) = self.lamp_strikes {
            println!("lamp_strikes: {:?}", lamp_strikes);
        }
        if let Some(lamp_state) = self.lamp_state {
            println!("lamp_state: {:?}", lamp_state);
        }
        if let Some(lamp_on_mode) = self.lamp_on_mode {
            println!("lamp_on_mode: {:?}", lamp_on_mode);
        }
        if let Some(power_cycle_count) = self.power_cycle_count {
            println!("power_cycle_count: {:?}", power_cycle_count);
        }
        if let Some(display_invert_mode) = self.display_invert_mode {
            println!("display_invert_mode: {:?}", display_invert_mode);
        }

        if self.curve_count > 0 {
            println!("\n> Curves:");

            if let (Some(current_curve), Some(curves)) = (self.current_curve, self.curves) {
                let mut curves_vec = curves.into_values().collect::<Vec<Curve>>();
                curves_vec.sort_by(|a, b| a.to_owned().id.cmp(&b.id));

                for curve in curves_vec {
                    let current: &str = if curve.id == current_curve {
                        "- Current"
                    } else {
                        ""
                    };
                    println!("[{}] {:#?} {}", curve.id, curve.description, current);
                }
            }
        }

        if self.modulation_frequency_count > 0 {
            println!("\n> Modulation Frequencies:");

            if let (Some(current_modulation_frequency), Some(modulation_frequencies)) = (
                self.current_modulation_frequency,
                self.modulation_frequencies,
            ) {
                let mut modulation_frequencies_vec = modulation_frequencies
                    .into_values()
                    .collect::<Vec<ModulationFrequency>>();
                modulation_frequencies_vec.sort_by(|a, b| a.to_owned().id.cmp(&b.id));

                for modulation_frequency in modulation_frequencies_vec {
                    let current: &str = if modulation_frequency.id == current_modulation_frequency {
                        "- Current"
                    } else {
                        ""
                    };
                    println!(
                        "[{}] {:#?} {}",
                        modulation_frequency.id, modulation_frequency.description, current
                    );
                }
            }
        }

        if self.output_response_time_count > 0 {
            println!("\n> Output Response Time:");
            if let (Some(current_output_response_time), Some(output_response_times)) = (
                self.current_output_response_time,
                self.output_response_times,
            ) {
                let mut output_response_times_vec = output_response_times
                    .into_values()
                    .collect::<Vec<OutputResponseTime>>();
                output_response_times_vec.sort_by(|a, b| a.to_owned().id.cmp(&b.id));

                for output_response_time in output_response_times_vec {
                    let current: &str = if output_response_time.id == current_output_response_time {
                        "- Current"
                    } else {
                        ""
                    };
                    println!(
                        "[{}] {:#?} {}",
                        output_response_time.id, output_response_time.description, current
                    );
                }
            }
        }

        println!("\n> Control:");
        if let Some(is_identifying) = self.is_identifying {
            println!("Identifying: {:?}", is_identifying);
        }
        if let Some(power_state) = self.power_state {
            println!("Power State: {:?}", power_state);
        }
        if let Some(self_test_is_active) = self.self_test_is_active {
            println!("Self Test Active: {:?}", self_test_is_active);
        }
        if let Some(preset_playback_mode) = self.preset_playback_mode {
            println!("Preset Playback Mode: {:?}", preset_playback_mode);
        }
        if let Some(preset_playback_level) = self.preset_playback_level {
            println!("Preset Playback Level: {:?}", preset_playback_level);
        }
    }
}
