use crate::{
    device::{DeviceUID, SlotInfo},
    parameter::{
        DisplayInvertMode, LampOnMode, LampState, ManufacturerSpecificParameter, ParameterId,
        PowerState, ProductCategory,
    },
    sensor::Sensor,
    CommandClass, ProtocolError,
};
use std::{collections::HashMap, ffi::CStr};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GetResponseParameterData {
    ProxiedDeviceCount {
        device_count: u16,
        list_change: bool,
    },
    ProxiedDevices {
        device_uids: Vec<DeviceUID>,
    },
    ParameterDescription {
        parameter_id: u16,
        parameter_data_size: u8,
        data_type: u8,
        command_class: CommandClass,
        prefix: u8,
        minimum_valid_value: u32,
        maximum_valid_value: u32,
        default_value: u32,
        description: String,
    },
    DeviceLabel {
        device_label: String,
    },
    DeviceInfo {
        protocol_version: String,
        model_id: u16,
        product_category: ProductCategory,
        software_version_id: u32,
        footprint: u16,
        current_personality: u8,
        personality_count: u8,
        start_address: u16,
        sub_device_count: u16,
        sensor_count: u8,
    },
    SoftwareVersionLabel {
        software_version_label: String,
    },
    SupportedParameters {
        standard_parameters: Vec<ParameterId>,
        manufacturer_specific_parameters: HashMap<u16, ManufacturerSpecificParameter>,
    },
    SensorDefinition {
        sensor: Sensor,
    },
    SensorValue {
        sensor_id: u8,
        current_value: i16,
        lowest_detected_value: i16,
        highest_detected_value: i16,
        recorded_value: i16,
    },
    IdentifyDevice {
        is_identifying: bool,
    },
    ManufacturerLabel {
        manufacturer_label: String,
    },
    FactoryDefaults {
        factory_default: bool,
    },
    DeviceModelDescription {
        device_model_description: String,
    },
    ProductDetailIdList {
        product_detail_id_list: Vec<u16>,
    },
    DmxPersonality {
        current_personality: u8,
        personality_count: u8,
    },
    DmxPersonalityDescription {
        id: u8,
        dmx_slots_required: u16,
        description: String,
    },
    DmxStartAddress {
        dmx_start_address: u16,
    },
    SlotInfo {
        dmx_slots: Vec<SlotInfo>,
    },
    SlotDescription {
        slot_id: u16,
        description: String,
    },
    DeviceHours {
        device_hours: u32,
    },
    LampHours {
        lamp_hours: u32,
    },
    LampStrikes {
        lamp_strikes: u32,
    },
    LampState {
        lamp_state: LampState,
    },
    LampOnMode {
        lamp_on_mode: LampOnMode,
    },
    DevicePowerCycles {
        power_cycle_count: u32,
    },
    DisplayInvert {
        display_invert_mode: DisplayInvertMode,
    },
    Curve {
        current_curve: u8,
        curve_count: u8,
    },
    CurveDescription {
        id: u8,
        description: String,
    },
    ModulationFrequency {
        current_modulation_frequency: u8,
        modulation_frequency_count: u8,
    },
    ModulationFrequencyDescription {
        id: u8,
        frequency: u32,
        description: String,
    },
    DimmerInfo {
        minimum_level_lower_limit: u16,
        minimum_level_upper_limit: u16,
        maximum_level_lower_limit: u16,
        maximum_level_upper_limit: u16,
        num_of_supported_curves: u8,
        levels_resolution: u8,
        minimum_levels_split_levels_supports: u8,
    },
    MinimumLevel {
        minimum_level_increasing: u16,
        minimum_level_decreasing: u16,
        on_below_minimum: u8, // TODO could be bool
    },
    MaximumLevel {
        maximum_level: u16,
    },
    OutputResponseTime {
        current_output_response_time: u8,
        output_response_time_count: u8,
    },
    OutputResponseTimeDescription {
        id: u8,
        description: String,
    },
    PowerState {
        power_state: PowerState,
    },
    PerformSelfTest {
        is_active: bool,
    },
    SelfTestDescription {
        self_test_id: u8,
        description: String,
    },
    PresetPlayback {
        mode: u16,
        level: u8,
    },
}

impl GetResponseParameterData {
    pub fn parse(parameter_id: ParameterId, bytes: &[u8]) -> Result<Self, ProtocolError> {
        match parameter_id {
            ParameterId::ProxiedDeviceCount => Ok(GetResponseParameterData::ProxiedDeviceCount {
                device_count: u16::from_be_bytes(
                    bytes[0..=1]
                        .try_into()
                        .map_err(|_| ProtocolError::TryFromSliceError)?,
                ),
                list_change: bytes[2] != 0,
            }),
            ParameterId::ProxiedDevices => Ok(GetResponseParameterData::ProxiedDevices {
                device_uids: bytes
                    .chunks(6)
                    .map(|chunk| {
                        Ok(DeviceUID::new(
                            u16::from_be_bytes(
                                chunk[0..=1]
                                    .try_into()
                                    .map_err(|_| ProtocolError::TryFromSliceError)?,
                            ),
                            u32::from_be_bytes(
                                chunk[2..=5]
                                    .try_into()
                                    .map_err(|_| ProtocolError::TryFromSliceError)?,
                            ),
                        ))
                    })
                    .collect::<Result<Vec<DeviceUID>, ProtocolError>>()?,
            }),
            ParameterId::ParameterDescription => {
                Ok(GetResponseParameterData::ParameterDescription {
                    parameter_id: u16::from_be_bytes(
                        bytes[0..=1]
                            .try_into()
                            .map_err(|_| ProtocolError::TryFromSliceError)?,
                    ),
                    parameter_data_size: bytes[2],
                    data_type: bytes[3],
                    command_class: CommandClass::try_from(bytes[4])
                        .map_err(|_| ProtocolError::TryFromSliceError)?,
                    prefix: bytes[5],
                    minimum_valid_value: u32::from_be_bytes(
                        bytes[8..=11]
                            .try_into()
                            .map_err(|_| ProtocolError::TryFromSliceError)?,
                    ),
                    maximum_valid_value: u32::from_be_bytes(
                        bytes[12..=15]
                            .try_into()
                            .map_err(|_| ProtocolError::TryFromSliceError)?,
                    ),
                    default_value: u32::from_be_bytes(
                        bytes[16..=19]
                            .try_into()
                            .map_err(|_| ProtocolError::TryFromSliceError)?,
                    ),
                    description: CStr::from_bytes_with_nul(&bytes[20..])?
                        .to_string_lossy()
                        .to_string(),
                })
            }
            ParameterId::DeviceLabel => Ok(GetResponseParameterData::DeviceLabel {
                device_label: CStr::from_bytes_with_nul(bytes)?
                    .to_string_lossy()
                    .to_string(),
            }),
            ParameterId::DeviceInfo => Ok(GetResponseParameterData::DeviceInfo {
                protocol_version: format!("{}.{}", bytes[0], bytes[1]),
                model_id: u16::from_be_bytes(
                    bytes[2..=3]
                        .try_into()
                        .map_err(|_| ProtocolError::TryFromSliceError)?,
                ),
                product_category: u16::from_be_bytes(
                    bytes[4..=5]
                        .try_into()
                        .map_err(|_| ProtocolError::TryFromSliceError)?,
                )
                .try_into()?,
                software_version_id: u32::from_be_bytes(
                    bytes[6..=9]
                        .try_into()
                        .map_err(|_| ProtocolError::TryFromSliceError)?,
                ),
                footprint: u16::from_be_bytes(
                    bytes[10..=11]
                        .try_into()
                        .map_err(|_| ProtocolError::TryFromSliceError)?,
                ),
                current_personality: bytes[12],
                personality_count: bytes[13],
                start_address: u16::from_be_bytes(
                    bytes[14..=15]
                        .try_into()
                        .map_err(|_| ProtocolError::TryFromSliceError)?,
                ),
                sub_device_count: u16::from_be_bytes(
                    bytes[16..=17]
                        .try_into()
                        .map_err(|_| ProtocolError::TryFromSliceError)?,
                ),
                sensor_count: u8::from_be(bytes[18]),
            }),
            ParameterId::SoftwareVersionLabel => {
                Ok(GetResponseParameterData::SoftwareVersionLabel {
                    software_version_label: CStr::from_bytes_with_nul(bytes)?
                        .to_string_lossy()
                        .to_string(),
                })
            }
            ParameterId::SupportedParameters => {
                let parameters = bytes
                    .chunks(2)
                    .map(|chunk| u16::from_be_bytes(chunk.try_into().unwrap()));

                Ok(GetResponseParameterData::SupportedParameters {
                    standard_parameters: parameters
                        .clone()
                        .filter(|parameter_id| {
                            // TODO consider if we should filter parameters here or before we add to the queue
                            let parameter_id = *parameter_id;
                            (0x0060_u16..0x8000_u16).contains(&parameter_id)
                        })
                        .map(ParameterId::try_from)
                        .collect::<Result<Vec<ParameterId>, ProtocolError>>()
                        .unwrap(), // TODO handle this error properly
                    manufacturer_specific_parameters: parameters
                        .filter(|parameter_id| *parameter_id >= 0x8000_u16)
                        .map(|parameter_id| {
                            (
                                parameter_id,
                                ManufacturerSpecificParameter {
                                    parameter_id,
                                    ..Default::default()
                                },
                            )
                        })
                        .collect(),
                })
            }
            ParameterId::SensorDefinition => Ok(GetResponseParameterData::SensorDefinition {
                sensor: Sensor {
                    id: bytes[0],
                    kind: bytes[1].try_into()?,
                    unit: bytes[2],
                    prefix: bytes[3],
                    range_minimum_value: i16::from_be_bytes(
                        bytes[4..=5]
                            .try_into()
                            .map_err(|_| ProtocolError::TryFromSliceError)?,
                    ),
                    range_maximum_value: i16::from_be_bytes(
                        bytes[6..=7]
                            .try_into()
                            .map_err(|_| ProtocolError::TryFromSliceError)?,
                    ),
                    normal_minimum_value: i16::from_be_bytes(
                        bytes[8..=9]
                            .try_into()
                            .map_err(|_| ProtocolError::TryFromSliceError)?,
                    ),
                    normal_maximum_value: i16::from_be_bytes(
                        bytes[10..=11]
                            .try_into()
                            .map_err(|_| ProtocolError::TryFromSliceError)?,
                    ),
                    recorded_value_support: bytes[12],
                    description: CStr::from_bytes_with_nul(&bytes[13..])?
                        .to_string_lossy()
                        .to_string(),
                },
            }),
            ParameterId::IdentifyDevice => Ok(GetResponseParameterData::IdentifyDevice {
                is_identifying: bytes[0] != 0,
            }),
            ParameterId::ManufacturerLabel => Ok(GetResponseParameterData::ManufacturerLabel {
                manufacturer_label: CStr::from_bytes_with_nul(bytes)?
                    .to_string_lossy()
                    .to_string(),
            }),
            ParameterId::FactoryDefaults => Ok(GetResponseParameterData::FactoryDefaults {
                factory_default: bytes[0] != 0,
            }),
            ParameterId::DeviceModelDescription => {
                Ok(GetResponseParameterData::DeviceModelDescription {
                    device_model_description: CStr::from_bytes_with_nul(bytes)?
                        .to_string_lossy()
                        .to_string(),
                })
            }
            ParameterId::ProductDetailIdList => Ok(GetResponseParameterData::ProductDetailIdList {
                product_detail_id_list: bytes
                    .chunks(2)
                    .map(|chunk| {
                        Ok(u16::from_be_bytes(
                            chunk
                                .try_into()
                                .map_err(|_| ProtocolError::TryFromSliceError)?,
                        ))
                    })
                    .collect::<Result<Vec<u16>, ProtocolError>>()?,
            }),
            ParameterId::DmxPersonality => Ok(GetResponseParameterData::DmxPersonality {
                current_personality: bytes[0],
                personality_count: bytes[1],
            }),
            ParameterId::DmxPersonalityDescription => {
                Ok(GetResponseParameterData::DmxPersonalityDescription {
                    id: bytes[0],
                    dmx_slots_required: u16::from_be_bytes(
                        bytes[1..=2]
                            .try_into()
                            .map_err(|_| ProtocolError::TryFromSliceError)?,
                    ),
                    description: CStr::from_bytes_with_nul(&bytes[3..])?
                        .to_string_lossy()
                        .to_string(),
                })
            }
            ParameterId::DmxStartAddress => Ok(GetResponseParameterData::DmxStartAddress {
                dmx_start_address: u16::from_be_bytes(
                    bytes[0..=1]
                        .try_into()
                        .map_err(|_| ProtocolError::TryFromSliceError)?,
                ),
            }),
            ParameterId::SlotInfo => Ok(GetResponseParameterData::SlotInfo {
                dmx_slots: bytes
                    .chunks(5)
                    .map(|chunk| {
                        Ok(SlotInfo::new(
                            u16::from_be_bytes(
                                chunk[0..=1]
                                    .try_into()
                                    .map_err(|_| ProtocolError::TryFromSliceError)?,
                            ),
                            chunk[2],
                            u16::from_be_bytes(
                                chunk[3..=4]
                                    .try_into()
                                    .map_err(|_| ProtocolError::TryFromSliceError)?,
                            ),
                        ))
                    })
                    .collect::<Result<Vec<SlotInfo>, ProtocolError>>()?,
            }),
            ParameterId::SlotDescription => Ok(GetResponseParameterData::SlotDescription {
                slot_id: u16::from_be_bytes(
                    bytes[0..=1]
                        .try_into()
                        .map_err(|_| ProtocolError::TryFromSliceError)?,
                ),
                description: CStr::from_bytes_with_nul(&bytes[2..])?
                    .to_string_lossy()
                    .to_string(),
            }),
            ParameterId::DeviceHours => Ok(GetResponseParameterData::DeviceHours {
                device_hours: u32::from_be_bytes(
                    bytes[0..=3]
                        .try_into()
                        .map_err(|_| ProtocolError::TryFromSliceError)?,
                ),
            }),
            ParameterId::LampHours => Ok(GetResponseParameterData::LampHours {
                lamp_hours: u32::from_be_bytes(
                    bytes[0..=3]
                        .try_into()
                        .map_err(|_| ProtocolError::TryFromSliceError)?,
                ),
            }),
            ParameterId::LampStrikes => Ok(GetResponseParameterData::LampStrikes {
                lamp_strikes: u32::from_be_bytes(
                    bytes[0..=3]
                        .try_into()
                        .map_err(|_| ProtocolError::TryFromSliceError)?,
                ),
            }),
            ParameterId::LampState => Ok(GetResponseParameterData::LampState {
                lamp_state: bytes[0].try_into()?,
            }),
            ParameterId::LampOnMode => Ok(GetResponseParameterData::LampOnMode {
                lamp_on_mode: bytes[0].try_into()?,
            }),
            ParameterId::DevicePowerCycles => Ok(GetResponseParameterData::DevicePowerCycles {
                power_cycle_count: u32::from_be_bytes(
                    bytes[0..=3]
                        .try_into()
                        .map_err(|_| ProtocolError::TryFromSliceError)?,
                ),
            }),
            ParameterId::DisplayInvert => Ok(GetResponseParameterData::DisplayInvert {
                display_invert_mode: bytes[0].try_into()?,
            }),
            ParameterId::Curve => Ok(GetResponseParameterData::Curve {
                current_curve: bytes[0],
                curve_count: bytes[1],
            }),
            ParameterId::CurveDescription => Ok(GetResponseParameterData::CurveDescription {
                id: bytes[0],
                description: CStr::from_bytes_with_nul(&bytes[1..])?
                    .to_string_lossy()
                    .to_string(),
            }),
            ParameterId::ModulationFrequency => Ok(GetResponseParameterData::ModulationFrequency {
                current_modulation_frequency: bytes[0],
                modulation_frequency_count: bytes[1],
            }),
            ParameterId::ModulationFrequencyDescription => {
                Ok(GetResponseParameterData::ModulationFrequencyDescription {
                    id: bytes[0],
                    frequency: u32::from_be_bytes(
                        bytes[1..=4]
                            .try_into()
                            .map_err(|_| ProtocolError::TryFromSliceError)?,
                    ),
                    description: CStr::from_bytes_with_nul(&bytes[5..])?
                        .to_string_lossy()
                        .to_string(),
                })
            }
            ParameterId::DimmerInfo => Ok(GetResponseParameterData::DimmerInfo {
                minimum_level_lower_limit: u16::from_be_bytes(
                    bytes[0..=1]
                        .try_into()
                        .map_err(|_| ProtocolError::TryFromSliceError)?,
                ),
                minimum_level_upper_limit: u16::from_be_bytes(
                    bytes[2..=3]
                        .try_into()
                        .map_err(|_| ProtocolError::TryFromSliceError)?,
                ),
                maximum_level_lower_limit: u16::from_be_bytes(
                    bytes[4..=5]
                        .try_into()
                        .map_err(|_| ProtocolError::TryFromSliceError)?,
                ),
                maximum_level_upper_limit: u16::from_be_bytes(
                    bytes[6..=7]
                        .try_into()
                        .map_err(|_| ProtocolError::TryFromSliceError)?,
                ),
                num_of_supported_curves: bytes[8],
                levels_resolution: bytes[9],
                minimum_levels_split_levels_supports: bytes[10], // TODO could be bool
            }),
            ParameterId::MinimumLevel => Ok(GetResponseParameterData::MinimumLevel {
                minimum_level_increasing: u16::from_be_bytes(
                    bytes[0..=1]
                        .try_into()
                        .map_err(|_| ProtocolError::TryFromSliceError)?,
                ),
                minimum_level_decreasing: u16::from_be_bytes(
                    bytes[2..=3]
                        .try_into()
                        .map_err(|_| ProtocolError::TryFromSliceError)?,
                ),
                on_below_minimum: bytes[4],
            }),
            ParameterId::MaximumLevel => Ok(GetResponseParameterData::MaximumLevel {
                maximum_level: u16::from_be_bytes(
                    bytes[0..=1]
                        .try_into()
                        .map_err(|_| ProtocolError::TryFromSliceError)?,
                ),
            }),
            ParameterId::OutputResponseTime => Ok(GetResponseParameterData::OutputResponseTime {
                current_output_response_time: bytes[0],
                output_response_time_count: bytes[1],
            }),
            ParameterId::OutputResponseTimeDescription => {
                Ok(GetResponseParameterData::OutputResponseTimeDescription {
                    id: bytes[0],
                    description: CStr::from_bytes_with_nul(&bytes[1..])?
                        .to_string_lossy()
                        .to_string(),
                })
            }
            ParameterId::PowerState => Ok(GetResponseParameterData::PowerState {
                power_state: bytes[0].try_into()?,
            }),
            ParameterId::PerformSelfTest => Ok(GetResponseParameterData::PerformSelfTest {
                is_active: bytes[0] != 0,
            }),
            ParameterId::SelfTestDescription => Ok(GetResponseParameterData::SelfTestDescription {
                self_test_id: bytes[0],
                description: CStr::from_bytes_with_nul(&bytes[1..])?
                    .to_string_lossy()
                    .to_string(),
            }),
            ParameterId::PresetPlayback => Ok(GetResponseParameterData::PresetPlayback {
                mode: u16::from_be_bytes(
                    bytes[0..=1]
                        .try_into()
                        .map_err(|_| ProtocolError::TryFromSliceError)?,
                ),
                level: bytes[2],
            }),
            _ => Err(ProtocolError::UnsupportedParameterId(parameter_id as u16)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_proxied_count() {
        assert_eq!(
            GetResponseParameterData::parse(ParameterId::ProxiedDeviceCount, &[0x00, 0x01, 0x00]),
            Ok(GetResponseParameterData::ProxiedDeviceCount {
                device_count: 1,
                list_change: false
            })
        );
        assert_eq!(
            GetResponseParameterData::parse(ParameterId::ProxiedDeviceCount, &[0x00, 0xff, 0x01]),
            Ok(GetResponseParameterData::ProxiedDeviceCount {
                device_count: 255,
                list_change: true
            })
        );
    }

    #[test]
    fn should_parse_proxied_devices() {
        assert_eq!(
            GetResponseParameterData::parse(
                ParameterId::ProxiedDevices,
                &[
                    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // DeviceUID 1
                    0x02, 0x03, 0x04, 0x05, 0x06, 0x07, // DeviceUID 2
                    0x03, 0x04, 0x05, 0x06, 0x07, 0x08, // DeviceUID 3
                    0x04, 0x05, 0x06, 0x07, 0x08, 0x09, // DeviceUID 4
                ]
            ),
            Ok(GetResponseParameterData::ProxiedDevices {
                device_uids: vec![
                    DeviceUID::new(0x0102, 0x03040506),
                    DeviceUID::new(0x0203, 0x04050607),
                    DeviceUID::new(0x0304, 0x05060708),
                    DeviceUID::new(0x0405, 0x06070809),
                ]
            })
        );
    }

    #[test]
    fn should_parse_device_label() {
        assert_eq!(
            GetResponseParameterData::parse(
                ParameterId::DeviceLabel,
                &[
                    0x54, 0x68, 0x69, 0x73, 0x20, 0x69, 0x73, 0x20, 0x61, 0x20, 0x74, 0x65, 0x73,
                    0x74, 0x20, 0x64, 0x65, 0x76, 0x69, 0x63, 0x65, 0x20, 0x6c, 0x61, 0x62, 0x65,
                    0x6c, 0x00
                ]
            ),
            Ok(GetResponseParameterData::DeviceLabel {
                device_label: "This is a test device label".to_string()
            })
        );
    }

    #[test]
    fn should_parse_software_version_label() {
        assert_eq!(
            GetResponseParameterData::parse(
                ParameterId::SoftwareVersionLabel,
                &[0x31, 0x2e, 0x32, 0x2e, 0x33, 0x00]
            ),
            Ok(GetResponseParameterData::SoftwareVersionLabel {
                software_version_label: "1.2.3".to_string()
            })
        );
    }

    #[test]
    fn should_parse_identify_device() {
        assert_eq!(
            GetResponseParameterData::parse(ParameterId::IdentifyDevice, &[0x01]),
            Ok(GetResponseParameterData::IdentifyDevice {
                is_identifying: true
            })
        );
    }

    #[test]
    fn should_parse_manufacturer_label() {
        assert_eq!(
            GetResponseParameterData::parse(
                ParameterId::ManufacturerLabel,
                &[
                    0x47, 0x65, 0x6e, 0x65, 0x72, 0x69, 0x63, 0x20, 0x43, 0x6f, 0x6d, 0x70, 0x61,
                    0x6e, 0x79, 0x20, 0x41, 0x20, 0x6c, 0x74, 0x64, 0x00
                ]
            ),
            Ok(GetResponseParameterData::ManufacturerLabel {
                manufacturer_label: "Generic Company A ltd".to_string()
            })
        );
    }

    #[test]
    fn should_parse_factory_defaults() {
        assert_eq!(
            GetResponseParameterData::parse(ParameterId::FactoryDefaults, &[0x01]),
            Ok(GetResponseParameterData::FactoryDefaults {
                factory_default: true
            })
        );
    }

    #[test]
    fn should_parse_device_model_description() {
        assert_eq!(
            GetResponseParameterData::parse(
                ParameterId::DeviceModelDescription,
                &[
                    0x47, 0x65, 0x6e, 0x65, 0x72, 0x69, 0x63, 0x20, 0x50, 0x72, 0x6f, 0x64, 0x75,
                    0x63, 0x74, 0x20, 0x41, 0x00
                ]
            ),
            Ok(GetResponseParameterData::DeviceModelDescription {
                device_model_description: "Generic Product A".to_string()
            })
        );
    }

    #[test]
    fn should_parse_dmx_personality() {
        assert_eq!(
            GetResponseParameterData::parse(ParameterId::DmxPersonality, &[0x02, 0x04]),
            Ok(GetResponseParameterData::DmxPersonality {
                current_personality: 2,
                personality_count: 4
            })
        );
    }

    #[test]
    fn should_parse_dmx_personality_description() {
        assert_eq!(
            GetResponseParameterData::parse(
                ParameterId::DmxPersonalityDescription,
                &[
                    0x01, 0x00, 0x04, 0x34, 0x20, 0x43, 0x68, 0x61, 0x6e, 0x6e, 0x65, 0x6c, 0x20,
                    0x52, 0x47, 0x42, 0x57, 0x00
                ]
            ),
            Ok(GetResponseParameterData::DmxPersonalityDescription {
                id: 1,
                dmx_slots_required: 4,
                description: "4 Channel RGBW".to_string(),
            })
        );
    }

    #[test]
    fn should_parse_dmx_start_address() {
        assert_eq!(
            GetResponseParameterData::parse(ParameterId::DmxStartAddress, &[0x00, 0x01]),
            Ok(GetResponseParameterData::DmxStartAddress {
                dmx_start_address: 0x01
            })
        );
    }

    #[test]
    fn should_parse_device_hours() {
        assert_eq!(
            GetResponseParameterData::parse(ParameterId::DeviceHours, &[0x01, 0x02, 0x03, 0x04]),
            Ok(GetResponseParameterData::DeviceHours {
                device_hours: 0x01020304
            })
        );
    }

    #[test]
    fn should_parse_lamp_hours() {
        assert_eq!(
            GetResponseParameterData::parse(ParameterId::LampHours, &[0x01, 0x02, 0x03, 0x04]),
            Ok(GetResponseParameterData::LampHours {
                lamp_hours: 0x01020304
            })
        );
    }

    #[test]
    fn should_parse_lamp_strikes() {
        assert_eq!(
            GetResponseParameterData::parse(ParameterId::LampStrikes, &[0x01, 0x02, 0x03, 0x04]),
            Ok(GetResponseParameterData::LampStrikes {
                lamp_strikes: 0x01020304
            })
        );
    }

    #[test]
    fn should_parse_lamp_state() {
        assert_eq!(
            GetResponseParameterData::parse(ParameterId::LampState, &[0x01]),
            Ok(GetResponseParameterData::LampState {
                lamp_state: LampState::LampOn
            })
        );
    }

    #[test]
    fn should_parse_lamp_on_mode() {
        assert_eq!(
            GetResponseParameterData::parse(ParameterId::LampOnMode, &[0x01]),
            Ok(GetResponseParameterData::LampOnMode {
                lamp_on_mode: LampOnMode::DmxMode
            })
        );
    }

    #[test]
    fn should_parse_device_power_cycles() {
        assert_eq!(
            GetResponseParameterData::parse(
                ParameterId::DevicePowerCycles,
                &[0x01, 0x02, 0x03, 0x04]
            ),
            Ok(GetResponseParameterData::DevicePowerCycles {
                power_cycle_count: 0x01020304
            })
        );
    }

    #[test]
    fn should_parse_display_invert() {
        assert_eq!(
            GetResponseParameterData::parse(ParameterId::DisplayInvert, &[0x01]),
            Ok(GetResponseParameterData::DisplayInvert {
                display_invert_mode: DisplayInvertMode::On
            })
        );
    }

    #[test]
    fn should_parse_curve() {
        assert_eq!(
            GetResponseParameterData::parse(ParameterId::Curve, &[0x01, 0x04]),
            Ok(GetResponseParameterData::Curve {
                current_curve: 1,
                curve_count: 4,
            })
        );
    }

    #[test]
    fn should_parse_curve_description() {
        assert_eq!(
            GetResponseParameterData::parse(
                ParameterId::CurveDescription,
                &[0x01, 0x50, 0x6f, 0x77, 0x65, 0x72, 0x20, 0x4c, 0x61, 0x77, 0x00]
            ),
            Ok(GetResponseParameterData::CurveDescription {
                id: 1,
                description: "Power Law".to_string(),
            })
        );
    }

    #[test]
    fn should_parse_modulation_frequency() {
        assert_eq!(
            GetResponseParameterData::parse(ParameterId::ModulationFrequency, &[0x01, 0x04]),
            Ok(GetResponseParameterData::ModulationFrequency {
                current_modulation_frequency: 1,
                modulation_frequency_count: 4,
            })
        );
    }

    #[test]
    fn should_parse_modulation_frequency_description() {
        assert_eq!(
            GetResponseParameterData::parse(
                ParameterId::ModulationFrequencyDescription,
                &[
                    0x01, 0x01, 0x02, 0x03, 0x04, 0x46, 0x61, 0x73, 0x74, 0x20, 0x52, 0x65, 0x66,
                    0x72, 0x65, 0x73, 0x68, 0x00
                ]
            ),
            Ok(GetResponseParameterData::ModulationFrequencyDescription {
                id: 1,
                frequency: 0x01020304,
                description: "Fast Refresh".to_string(),
            })
        );
    }
}
