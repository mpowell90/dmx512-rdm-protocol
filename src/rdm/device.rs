use std::collections::HashMap;

use super::{
    parameter::{
        CurveDescriptionGetResponse, CurveGetResponse, DeviceHoursGetResponse, DeviceInfoResponse,
        DeviceModelDescriptionGetResponse, DevicePowerCyclesGetResponse, DimmerInfoResponse,
        DisplayInvertGetResponse, DmxPersonalityDescriptionGetResponse, DmxPersonalityGetResponse,
        IdentifyDeviceResponse, LampHoursGetResponse, LampOnModeGetResponse, LampStateGetResponse,
        LampStrikesGetResponse, ManufacturerLabelResponse, ManufacturerSpecificParameter,
        MaximumLevelGetResponse, MinimumLevelGetResponse,
        ModulationFrequencyDescriptionGetResponse, ModulationFrequencyGetResponse,
        OutputResponseTimeDescriptionGetResponse, OutputResponseTimeGetResponse,
        ParameterDescriptionGetResponse, PerformSelfTestGetResponse, PowerStateGetResponse,
        PresetPlaybackGetResponse, ProductDetailIdListGetResponse, SelfTestDescriptionGetResponse,
        SensorDefinitionResponse, SlotInfoResponse, SoftwareVersionLabelGetResponse,
        SupportedParametersGetResponse,
    },
    DisplayInvertMode, LampOnMode, LampState, ParameterId, PowerState, ProductCategory,
};

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
pub struct DmxSlot {
    pub id: u16,
    pub kind: u8, // TODO use enum
    pub label_id: u16,
    pub description: Option<String>,
}

impl From<&[u8]> for DmxSlot {
    fn from(bytes: &[u8]) -> Self {
        DmxSlot {
            id: u16::from_be_bytes(bytes[0..=1].try_into().unwrap()),
            kind: bytes[2], // TODO use enum
            label_id: u16::from_be_bytes(bytes[3..=4].try_into().unwrap()),
            description: None,
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
    pub footprint: Option<u16>,
    pub current_personality: Option<u8>,
    pub personality_count: u8,
    pub personalities: Option<HashMap<u8, DmxPersonality>>,
    pub start_address: Option<u16>,
    pub dmx_slots: Option<HashMap<u16, DmxSlot>>,
    pub sub_device_id: u16,
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
        if let Some(start_address) = self.start_address {
            println!("Start Address: {:?}", start_address);
        }

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

        if let Some(footprint) = self.footprint {
            println!("DMX Footprint: {:?}", footprint);

            if footprint > 0 {
                if let Some(dmx_slots) = self.dmx_slots {
                    println!("\nDMX Slots: {:#?}", dmx_slots);
                }
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

        if self.sub_device_count > 0 {
            println!("\n> Sub devices:");
            if let Some(sub_devices) = self.sub_devices {
                for sub_device in sub_devices.into_values() {
                    sub_device.print();
                }
            }
        }

        println!("");
    }

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

    pub fn update_sensor_definition(&mut self, data: SensorDefinitionResponse) {
        let sensor = Sensor {
            id: data.sensor_id,
            kind: data.kind,
            unit: data.unit,
            prefix: data.prefix,
            range_minimum_value: data.range_minimum_value,
            range_maximum_value: data.range_maximum_value,
            normal_minimum_value: data.normal_minimum_value,
            normal_maximum_value: data.normal_maximum_value,
            recorded_value_support: data.recorded_value_support,
            description: data.description,
        };
        self.sensors = if let Some(sensors) = self.sensors.as_mut() {
            sensors.insert(data.sensor_id, sensor);
            Some(sensors.to_owned())
        } else {
            Some(HashMap::from([(data.sensor_id, sensor)]))
        }
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

    pub fn update_slot_info(&mut self, data: SlotInfoResponse) {
        let mut hash_map: HashMap<u16, DmxSlot> = HashMap::new();
        for dmx_slot in data.dmx_slots {
            hash_map.insert(dmx_slot.id, dmx_slot);
        }
        self.dmx_slots = Some(hash_map);
    }

    pub fn update_device_hours(&mut self, data: DeviceHoursGetResponse) {
        self.device_hours = Some(data.device_hours);
    }

    pub fn update_lamp_hours(&mut self, data: LampHoursGetResponse) {
        self.lamp_hours = Some(data.lamp_hours);
    }

    pub fn update_lamp_strikes(&mut self, data: LampStrikesGetResponse) {
        self.lamp_strikes = Some(data.lamp_strikes);
    }

    pub fn update_lamp_state(&mut self, data: LampStateGetResponse) {
        self.lamp_state = Some(data.lamp_state);
    }

    pub fn update_lamp_on_mode(&mut self, data: LampOnModeGetResponse) {
        self.lamp_on_mode = Some(data.lamp_on_mode);
    }

    pub fn update_device_power_cycles(&mut self, data: DevicePowerCyclesGetResponse) {
        self.power_cycle_count = Some(data.power_cycle_count);
    }

    pub fn update_display_invert(&mut self, data: DisplayInvertGetResponse) {
        self.display_invert_mode = Some(data.display_invert_mode);
    }

    pub fn update_dimmer_info(&mut self, data: DimmerInfoResponse) {
        self.minimum_level_lower_limit = Some(data.minimum_level_lower_limit);
        self.minimum_level_upper_limit = Some(data.minimum_level_upper_limit);
        self.maximum_level_lower_limit = Some(data.maximum_level_lower_limit);
        self.maximum_level_upper_limit = Some(data.maximum_level_upper_limit);
        self.num_of_supported_curves = Some(data.num_of_supported_curves);
        self.levels_resolution = Some(data.levels_resolution);
        self.minimum_levels_split_levels_supports = Some(data.minimum_levels_split_levels_supports);
    }

    pub fn update_minimum_level(&mut self, data: MinimumLevelGetResponse) {
        self.minimum_level_increasing = Some(data.minimum_level_increasing);
        self.minimum_level_decreasing = Some(data.minimum_level_decreasing);
        self.on_below_minimum = Some(data.on_below_minimum);
    }

    pub fn update_maximum_level(&mut self, data: MaximumLevelGetResponse) {
        self.maximum_level = Some(data.maximum_level);
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
        self.modulation_frequencies =
            if let Some(modulation_frequencies) = self.modulation_frequencies.as_mut() {
                modulation_frequencies.insert(data.modulation_frequency, modulation_frequency);
                Some(modulation_frequencies.to_owned())
            } else {
                Some(HashMap::from([(
                    data.modulation_frequency,
                    modulation_frequency,
                )]))
            }
    }

    pub fn update_output_response_time_info(&mut self, data: OutputResponseTimeGetResponse) {
        self.current_output_response_time = Some(data.current_output_response_time);
        self.output_response_time_count = data.output_response_time_count;
    }

    pub fn update_output_response_time_description(
        &mut self,
        data: OutputResponseTimeDescriptionGetResponse,
    ) {
        let output_response_time = OutputResponseTime {
            id: data.output_response_time,
            description: data.description,
        };
        self.output_response_times =
            if let Some(output_response_times) = self.output_response_times.as_mut() {
                output_response_times.insert(data.output_response_time, output_response_time);
                Some(output_response_times.to_owned())
            } else {
                Some(HashMap::from([(
                    data.output_response_time,
                    output_response_time,
                )]))
            }
    }

    pub fn update_power_state(&mut self, data: PowerStateGetResponse) {
        self.power_state = Some(data.power_state);
    }

    pub fn update_perform_self_state(&mut self, data: PerformSelfTestGetResponse) {
        self.self_test_is_active = Some(data.is_active);
    }

    pub fn update_self_test_description(&mut self, data: SelfTestDescriptionGetResponse) {
        todo!()
    }

    pub fn update_preset_playback(&mut self, data: PresetPlaybackGetResponse) {
        self.preset_playback_mode = Some(data.mode);
        self.preset_playback_level = Some(data.level);
    }
}
