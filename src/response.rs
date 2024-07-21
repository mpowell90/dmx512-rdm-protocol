use crate::{
    bsd_16_crc,
    device::{DeviceUID, DmxSlot},
    parameter::{
        DisplayInvertMode, LampOnMode, LampState, ManufacturerSpecificParameter, ParameterId,
        PowerState, ProductCategory,
    },
    sensor::Sensor,
    CommandClass, ProtocolError, DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
    DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE, SC_RDM, SC_SUB_MESSAGE,
};
use std::collections::HashMap;

pub enum ResponseNackReasonCode {
    UnknownPid = 0x0000,
    FormatError = 0x0001,
    HardwareFault = 0x0002,
    ProxyReject = 0x0003,
    WriteProtect = 0x0004,
    UnsupportedCommandClass = 0x0005,
    DataOutOfRange = 0x0006,
    BufferFull = 0x0007,
    PacketSizeUnsupported = 0x0008,
    SubDeviceOutOfRange = 0x0009,
    ProxyBufferFull = 0x000a,
}

#[derive(Clone, Debug)]
pub enum ResponseType {
    Ack = 0x00,
    AckTimer = 0x01,
    NackReason = 0x02,
    AckOverflow = 0x03,
}

impl TryFrom<u8> for ResponseType {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Ack),
            0x01 => Ok(Self::AckTimer),
            0x02 => Ok(Self::NackReason),
            0x03 => Ok(Self::AckOverflow),
            _ => Err(ProtocolError::InvalidResponseType(value)),
        }
    }
}

// TODO the following is a quick and dirty test
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PacketResponseType {
    SuccessResponse = 0x05,
    NullResponse = 0x0c,
}

impl TryFrom<u8> for PacketResponseType {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x05 => Ok(Self::SuccessResponse),
            0x0c => Ok(Self::NullResponse),
            _ => Err(ProtocolError::InvalidPacketResponseType(value)),
        }
    }
}

#[derive(Clone, Debug)]
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
        dmx_slots: Option<Vec<DmxSlot>>,
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
    pub fn parse(parameter_id: ParameterId, bytes: &[u8]) -> Result<Option<Self>, ProtocolError> {
        match parameter_id {
            ParameterId::ProxiedDeviceCount => {
                Ok(Some(GetResponseParameterData::ProxiedDeviceCount {
                    device_count: u16::from_be_bytes(bytes[..=1].try_into().unwrap()),
                    list_change: bytes[2] != 0,
                }))
            }
            ParameterId::ProxiedDevices => Ok(Some(GetResponseParameterData::ProxiedDevices {
                device_uids: bytes.chunks(6).map(DeviceUID::from).collect(),
            })),
            ParameterId::ParameterDescription => {
                Ok(Some(GetResponseParameterData::ParameterDescription {
                    parameter_id: u16::from_be_bytes(bytes[0..=1].try_into().unwrap()),
                    parameter_data_size: bytes[2],
                    data_type: bytes[3],
                    command_class: CommandClass::try_from(bytes[4]).unwrap(),
                    prefix: bytes[5],
                    minimum_valid_value: u32::from_be_bytes(bytes[8..=11].try_into().unwrap()),
                    maximum_valid_value: u32::from_be_bytes(bytes[12..=15].try_into().unwrap()),
                    default_value: u32::from_be_bytes(bytes[16..=19].try_into().unwrap()),
                    description: String::from_utf8_lossy(&bytes[20..])
                        .trim_end_matches('\0')
                        .to_string(),
                }))
            }
            ParameterId::DeviceLabel => Ok(Some(GetResponseParameterData::DeviceLabel {
                device_label: String::from_utf8_lossy(&bytes)
                    .trim_end_matches('\0')
                    .to_string(),
            })),
            ParameterId::DeviceInfo => Ok(Some(GetResponseParameterData::DeviceInfo {
                protocol_version: format!("{}.{}", bytes[0], bytes[1]),
                model_id: u16::from_be_bytes(bytes[2..=3].try_into().unwrap()),
                product_category: ProductCategory::from(&bytes[4..=5]),
                software_version_id: u32::from_be_bytes(bytes[6..=9].try_into().unwrap()),
                footprint: u16::from_be_bytes(bytes[10..=11].try_into().unwrap()),
                current_personality: bytes[12],
                personality_count: bytes[13],
                start_address: u16::from_be_bytes(bytes[14..=15].try_into().unwrap()),
                sub_device_count: u16::from_be_bytes(bytes[16..=17].try_into().unwrap()),
                sensor_count: u8::from_be(bytes[18]),
            })),
            ParameterId::SoftwareVersionLabel => {
                Ok(Some(GetResponseParameterData::SoftwareVersionLabel {
                    software_version_label: String::from_utf8_lossy(&bytes)
                        .trim_end_matches('\0')
                        .to_string(),
                }))
            }
            ParameterId::SupportedParameters => {
                let parameters = bytes
                    .chunks(2)
                    .map(|chunk| u16::from_be_bytes(chunk.try_into().unwrap()));

                Ok(Some(GetResponseParameterData::SupportedParameters {
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
                }))
            }
            ParameterId::SensorDefinition => Ok(Some(GetResponseParameterData::SensorDefinition {
                sensor: Sensor {
                    id: bytes[0],
                    kind: bytes[1].into(),
                    unit: bytes[2],
                    prefix: bytes[3],
                    range_minimum_value: i16::from_be_bytes(bytes[4..=5].try_into().unwrap()),
                    range_maximum_value: i16::from_be_bytes(bytes[6..=7].try_into().unwrap()),
                    normal_minimum_value: i16::from_be_bytes(bytes[8..=9].try_into().unwrap()),
                    normal_maximum_value: i16::from_be_bytes(bytes[10..=11].try_into().unwrap()),
                    recorded_value_support: bytes[12],
                    description: String::from_utf8_lossy(&bytes[13..])
                        .trim_end_matches('\0')
                        .to_string(),
                },
            })),
            ParameterId::IdentifyDevice => Ok(Some(GetResponseParameterData::IdentifyDevice {
                is_identifying: bytes[0] != 0,
            })),
            ParameterId::ManufacturerLabel => {
                Ok(Some(GetResponseParameterData::ManufacturerLabel {
                    manufacturer_label: String::from_utf8_lossy(&bytes)
                        .trim_end_matches('\0')
                        .to_string(),
                }))
            }
            ParameterId::FactoryDefaults => Ok(Some(GetResponseParameterData::FactoryDefaults {
                factory_default: bytes[0] != 0,
            })),
            ParameterId::DeviceModelDescription => {
                Ok(Some(GetResponseParameterData::DeviceModelDescription {
                    device_model_description: String::from_utf8_lossy(&bytes)
                        .trim_end_matches('\0')
                        .to_string(),
                }))
            }
            ParameterId::ProductDetailIdList => {
                Ok(Some(GetResponseParameterData::ProductDetailIdList {
                    product_detail_id_list: bytes
                        .chunks(2)
                        .map(|id| u16::from_be_bytes(id.try_into().unwrap()))
                        .collect(),
                }))
            }
            ParameterId::DmxPersonality => Ok(Some(GetResponseParameterData::DmxPersonality {
                current_personality: bytes[0],
                personality_count: bytes[1],
            })),
            ParameterId::DmxPersonalityDescription => {
                Ok(Some(GetResponseParameterData::DmxPersonalityDescription {
                    id: bytes[0],
                    dmx_slots_required: u16::from_be_bytes(bytes[1..=2].try_into().unwrap()),
                    description: String::from_utf8_lossy(&bytes[3..])
                        .trim_end_matches('\0')
                        .to_string(),
                }))
            }
            ParameterId::DmxStartAddress => Ok(Some(GetResponseParameterData::DmxStartAddress {
                dmx_start_address: u16::from_be_bytes(bytes[0..=1].try_into().unwrap()),
            })),
            ParameterId::SlotInfo => {
                let dmx_slots = if bytes.len() >= 5 {
                    Some(bytes.chunks(5).map(DmxSlot::from).collect())
                } else {
                    None
                };

                Ok(Some(GetResponseParameterData::SlotInfo { dmx_slots }))
            }
            ParameterId::SlotDescription => Ok(Some(GetResponseParameterData::SlotDescription {
                slot_id: u16::from_be_bytes(bytes[0..=1].try_into().unwrap()),
                description: String::from_utf8_lossy(&bytes[2..])
                    .trim_end_matches('\0')
                    .to_string(),
            })),
            ParameterId::DeviceHours => Ok(Some(GetResponseParameterData::DeviceHours {
                device_hours: u32::from_be_bytes(bytes[0..=3].try_into().unwrap()),
            })),
            ParameterId::LampHours => Ok(Some(GetResponseParameterData::LampHours {
                lamp_hours: u32::from_be_bytes(bytes[0..=3].try_into().unwrap()),
            })),
            ParameterId::LampStrikes => Ok(Some(GetResponseParameterData::LampStrikes {
                lamp_strikes: u32::from_be_bytes(bytes[0..=3].try_into().unwrap()),
            })),
            ParameterId::LampState => Ok(Some(GetResponseParameterData::LampState {
                lamp_state: LampState::from(bytes[0]),
            })),
            ParameterId::LampOnMode => Ok(Some(GetResponseParameterData::LampOnMode {
                lamp_on_mode: LampOnMode::from(bytes[0]),
            })),
            ParameterId::DevicePowerCycles => {
                Ok(Some(GetResponseParameterData::DevicePowerCycles {
                    power_cycle_count: u32::from_be_bytes(bytes[0..=3].try_into().unwrap()),
                }))
            }
            ParameterId::DisplayInvert => Ok(Some(GetResponseParameterData::DisplayInvert {
                display_invert_mode: DisplayInvertMode::from(bytes[0]),
            })),
            ParameterId::Curve => Ok(Some(GetResponseParameterData::Curve {
                current_curve: bytes[0],
                curve_count: bytes[1],
            })),
            ParameterId::CurveDescription => Ok(Some(GetResponseParameterData::CurveDescription {
                id: bytes[0],
                description: String::from_utf8_lossy(&bytes[1..])
                    .trim_end_matches('\0')
                    .to_string(),
            })),
            ParameterId::ModulationFrequency => {
                Ok(Some(GetResponseParameterData::ModulationFrequency {
                    current_modulation_frequency: bytes[0],
                    modulation_frequency_count: bytes[1],
                }))
            }
            ParameterId::ModulationFrequencyDescription => Ok(Some(
                GetResponseParameterData::ModulationFrequencyDescription {
                    id: bytes[0],
                    frequency: u32::from_be_bytes(bytes[1..=4].try_into().unwrap()),
                    description: String::from_utf8_lossy(&bytes[5..])
                        .trim_end_matches('\0')
                        .to_string(),
                },
            )),
            ParameterId::DimmerInfo => Ok(Some(GetResponseParameterData::DimmerInfo {
                minimum_level_lower_limit: u16::from_be_bytes(bytes[0..=1].try_into().unwrap()),
                minimum_level_upper_limit: u16::from_be_bytes(bytes[2..=3].try_into().unwrap()),
                maximum_level_lower_limit: u16::from_be_bytes(bytes[4..=5].try_into().unwrap()),
                maximum_level_upper_limit: u16::from_be_bytes(bytes[6..=7].try_into().unwrap()),
                num_of_supported_curves: bytes[8],
                levels_resolution: bytes[9],
                minimum_levels_split_levels_supports: bytes[10], // TODO could be bool
            })),
            ParameterId::MinimumLevel => Ok(Some(GetResponseParameterData::MinimumLevel {
                minimum_level_increasing: u16::from_be_bytes(bytes[0..=1].try_into().unwrap()),
                minimum_level_decreasing: u16::from_be_bytes(bytes[2..=3].try_into().unwrap()),
                on_below_minimum: bytes[4],
            })),
            ParameterId::MaximumLevel => Ok(Some(GetResponseParameterData::MaximumLevel {
                maximum_level: u16::from_be_bytes(bytes[0..=1].try_into().unwrap()),
            })),
            ParameterId::OutputResponseTime => {
                Ok(Some(GetResponseParameterData::OutputResponseTime {
                    current_output_response_time: bytes[0],
                    output_response_time_count: bytes[1],
                }))
            }
            ParameterId::OutputResponseTimeDescription => Ok(Some(
                GetResponseParameterData::OutputResponseTimeDescription {
                    id: bytes[0],
                    description: String::from_utf8_lossy(&bytes[1..])
                        .trim_end_matches('\0')
                        .to_string(),
                },
            )),
            ParameterId::PowerState => Ok(Some(GetResponseParameterData::PowerState {
                power_state: PowerState::from(bytes[0]),
            })),
            ParameterId::PerformSelfTest => Ok(Some(GetResponseParameterData::PerformSelfTest {
                is_active: bytes[0] != 0,
            })),
            ParameterId::SelfTestDescription => {
                Ok(Some(GetResponseParameterData::SelfTestDescription {
                    self_test_id: bytes[0],
                    description: String::from_utf8_lossy(&bytes[1..])
                        .trim_end_matches('\0')
                        .to_string(),
                }))
            }
            ParameterId::PresetPlayback => Ok(Some(GetResponseParameterData::PresetPlayback {
                mode: u16::from_be_bytes(bytes[..=1].try_into().unwrap()),
                level: bytes[2],
            })),
            _ => Ok(None),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SetResponseParameterData {
    DeviceLabel,
    DmxPersonality,
    DmxStartAddress,
    Curve,
    ModulationFrequency,
    OutputResponseTime,
    IdentifyDevice,
}

impl SetResponseParameterData {
    pub fn parse(_parameter_id: ParameterId, _bytes: &[u8]) -> Result<Option<Self>, ProtocolError> {
        Ok(None)
    }
}

#[derive(Clone, Debug)]
pub enum DiscoveryResponseParameterData {
    DiscMute {
        control_field: u16,
        binding_uid: Option<DeviceUID>,
    },
    DiscUnmute {
        control_field: u16,
        binding_uid: Option<DeviceUID>,
    },
}

impl DiscoveryResponseParameterData {
    pub fn parse(parameter_id: ParameterId, bytes: &[u8]) -> Result<Option<Self>, ProtocolError> {
        match parameter_id {
            ParameterId::DiscMute => {
                let binding_uid = if bytes.len() > 2 {
                    Some(DeviceUID::from(&bytes[2..]))
                } else {
                    None
                };

                Ok(Some(DiscoveryResponseParameterData::DiscMute {
                    control_field: u16::from_be_bytes(bytes[..=1].try_into().unwrap()),
                    binding_uid,
                }))
            }
            ParameterId::DiscUnMute => {
                let binding_uid = if bytes.len() > 2 {
                    Some(DeviceUID::from(&bytes[2..]))
                } else {
                    None
                };

                Ok(Some(DiscoveryResponseParameterData::DiscMute {
                    control_field: u16::from_be_bytes(bytes[..=1].try_into().unwrap()),
                    binding_uid,
                }))
            }
            _ => Ok(None),
        }
    }
}

#[derive(Clone, Debug)]
pub enum RdmResponseParameterData {
    GetResponse(GetResponseParameterData),
    SetResponse(SetResponseParameterData),
    DiscoveryResponse(DiscoveryResponseParameterData),
}

impl RdmResponseParameterData {
    pub fn parse(
        command_class: CommandClass,
        parameter_id: ParameterId,
        bytes: &[u8],
    ) -> Result<Option<Self>, ProtocolError> {
        match command_class {
            CommandClass::GetCommandResponse => {
                Ok(GetResponseParameterData::parse(parameter_id, bytes)?.map(Self::GetResponse))
            }
            CommandClass::SetCommandResponse => {
                Ok(SetResponseParameterData::parse(parameter_id, bytes)?.map(Self::SetResponse))
            }
            CommandClass::DiscoveryCommandResponse => {
                Ok(DiscoveryResponseParameterData::parse(parameter_id, bytes)?
                    .map(Self::DiscoveryResponse))
            }
            _ => Err(ProtocolError::InvalidCommandClass(command_class as u8)),
        }
    }
}

pub struct DiscoveryUniqueBranchResponse(DeviceUID);

impl DiscoveryUniqueBranchResponse {
    pub fn parse(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let euid_start_index = bytes.iter().position(|x| *x == 0xaa).unwrap();

        let euid = Vec::from(&bytes[(euid_start_index + 1)..=(euid_start_index + 12)]);

        let ecs = Vec::from(&bytes[(euid_start_index + 13)..=(euid_start_index + 16)]);

        let decoded_checksum = bsd_16_crc(&euid);

        let checksum = u16::from_be_bytes([ecs[0] & ecs[1], ecs[2] & ecs[3]]);

        if checksum != decoded_checksum {
            return Err(ProtocolError::InvalidChecksum(decoded_checksum, checksum));
        }

        let manufacturer_id = u16::from_be_bytes([euid[0] & euid[1], euid[2] & euid[3]]);

        let device_id = u32::from_be_bytes([
            euid[4] & euid[5],
            euid[6] & euid[7],
            euid[8] & euid[9],
            euid[10] & euid[11],
        ]);

        Ok(Self(DeviceUID::new(manufacturer_id, device_id)))
    }
}

pub struct RdmResponse {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub response_type: ResponseType,
    pub message_count: u8,
    pub sub_device_id: u16,
    pub command_class: CommandClass,
    pub parameter_id: ParameterId,
    pub parameter_data: Option<RdmResponseParameterData>,
}

impl RdmResponse {
    pub fn parse(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let message_length = bytes[2];

        if message_length < 24 {
            return Err(ProtocolError::InvalidMessageLength(message_length));
        }

        let packet_checksum = u16::from_be_bytes(
            bytes[message_length as usize..=message_length as usize + 2]
                .try_into()
                .unwrap(),
        );

        let decoded_checksum = bsd_16_crc(&bytes[..message_length as usize - 1]);

        if decoded_checksum != packet_checksum {
            return Err(ProtocolError::InvalidChecksum(
                decoded_checksum,
                packet_checksum,
            ));
        }

        let destination_manufacturer_id = u16::from_be_bytes(bytes[3..=4].try_into().unwrap());
        let destination_device_id = u32::from_be_bytes(bytes[5..=8].try_into().unwrap());
        let destination_uid = DeviceUID::new(destination_manufacturer_id, destination_device_id);

        let source_manufacturer_id = u16::from_be_bytes(bytes[9..=10].try_into().unwrap());
        let source_device_id = u32::from_be_bytes(bytes[11..=14].try_into().unwrap());
        let source_uid = DeviceUID::new(source_manufacturer_id, source_device_id);

        let transaction_number = bytes[15];

        let response_type = ResponseType::try_from(bytes[16])?;

        let message_count = bytes[17];

        let sub_device_id = u16::from_be_bytes(bytes[18..=19].try_into().unwrap());

        let command_class = CommandClass::try_from(bytes[20])?;

        let parameter_id =
            ParameterId::try_from(u16::from_be_bytes(bytes[21..=22].try_into().unwrap()))?;

        let parameter_data_length = bytes[23];

        if parameter_data_length > 231 {
            return Err(ProtocolError::InvalidParameterDataLength(
                parameter_data_length,
            ));
        }

        let parameter_data = if parameter_data_length > 0 {
            RdmResponseParameterData::parse(
                command_class,
                parameter_id,
                &bytes[24..(message_length as usize - 2)],
            )?
        } else {
            None
        };

        Ok(Self {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device_id,
            command_class,
            parameter_id,
            parameter_data,
        })
    }
}

pub enum Frame {
    Rdm(RdmResponse),
    DiscoveryUniqueBranch(DiscoveryUniqueBranchResponse),
}

impl Frame {
    pub fn parse(bytes: &[u8]) -> Result<Option<Self>, ProtocolError> {
        if bytes[0] == SC_RDM && bytes[1] == SC_SUB_MESSAGE {
            if bytes.len() < 25 {
                return Ok(None);
            }

            return Ok(Some(Frame::Rdm(RdmResponse::parse(bytes)?)));
        }

        if bytes[0] == DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE
            || bytes[1] == DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE
        {
            if bytes.len() < 17 {
                return Ok(None);
            }

            return Ok(Some(Frame::DiscoveryUniqueBranch(
                DiscoveryUniqueBranchResponse::parse(bytes)?,
            )));
        }

        Ok(None)
    }
}
