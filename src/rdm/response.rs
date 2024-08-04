use super::{
    bsd_16_crc,
    parameter::{
        DefaultSlotValue, DisplayInvertMode, LampOnMode, LampState, ManufacturerSpecificParameter,
        ParameterDescription, ParameterId, PowerState, PresetPlaybackMode, ProductCategory, SensorDefinition,
        SensorValue, SlotInfo, StatusMessage, StatusType,
    },
    CommandClass, DeviceUID, ProtocolError, DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
    DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE, SC_RDM, SC_SUB_MESSAGE,
};
use std::{collections::HashMap, ffi::CStr};

#[derive(Copy, Clone, Debug, PartialEq)]
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

#[derive(Copy, Clone, Debug, PartialEq)]
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

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum ResponseParameterData {
    DiscMute {
        control_field: u16,
        binding_uid: Option<DeviceUID>,
    },
    DiscUnMute {
        control_field: u16,
        binding_uid: Option<DeviceUID>,
    },
    GetProxiedDeviceCount {
        device_count: u16,
        list_change: bool,
    },
    GetProxiedDevices {
        device_uids: Vec<DeviceUID>,
    },
    GetCommsStatus {
        short_message: u16,
        length_mismatch: u16,
        checksum_fail: u16,
    },
    GetStatusMessages {
        status_messages: Vec<StatusMessage>,
    },
    GetStatusIdDescription {
        status_id_description: String,
    },
    GetSubDeviceStatusReportThreshold {
        status_type: StatusType,
    },
    GetSupportedParameters {
        standard_parameters: Vec<ParameterId>,
        manufacturer_specific_parameters: HashMap<u16, ManufacturerSpecificParameter>,
    },
    GetParameterDescription(ParameterDescription),
    GetDeviceInfo {
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
    GetProductDetailIdList {
        product_detail_id_list: Vec<u16>,
    },
    GetDeviceModelDescription {
        device_model_description: String,
    },
    GetManufacturerLabel {
        manufacturer_label: String,
    },
    GetDeviceLabel {
        device_label: String,
    },
    GetFactoryDefaults {
        factory_default: bool,
    },
    GetLanguageCapabilities {
        language_capabilities: Vec<String>,
    },
    GetLanguage {
        current_language: String,
    },
    GetSoftwareVersionLabel {
        software_version_label: String,
    },
    GetBootSoftwareVersionId {
        boot_software_version_id: u32,
    },
    GetBootSoftwareVersionLabel {
        boot_software_version_label: String,
    },
    GetDmxPersonality {
        current_personality: u8,
        personality_count: u8,
    },
    GetDmxPersonalityDescription {
        id: u8,
        dmx_slots_required: u16,
        description: String,
    },
    GetDmxStartAddress {
        dmx_start_address: u16,
    },
    GetSlotInfo {
        dmx_slots: Vec<SlotInfo>,
    },
    GetSlotDescription {
        slot_id: u16,
        description: String,
    },
    GetDefaultSlotValue {
        default_slot_values: Vec<DefaultSlotValue>,
    },
    GetSensorDefinition {
        sensor: SensorDefinition,
    },
    GetSensorValue(SensorValue),
    SetSensorValue(SensorValue),
    GetDeviceHours {
        device_hours: u32,
    },
    GetLampHours {
        lamp_hours: u32,
    },
    GetLampStrikes {
        lamp_strikes: u32,
    },
    GetLampState {
        lamp_state: LampState,
    },
    GetLampOnMode {
        lamp_on_mode: LampOnMode,
    },
    GetDevicePowerCycles {
        power_cycle_count: u32,
    },
    GetDisplayInvert {
        display_invert_mode: DisplayInvertMode,
    },
    GetDisplayLevel {
        display_level: u8,
    },
    GetPanInvert {
        pan_invert: bool,
    },
    GetTiltInvert {
        tilt_invert: bool,
    },
    GetPanTiltSwap {
        pan_tilt_swap: bool,
    },
    GetRealTimeClock {
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    },
    GetIdentifyDevice {
        is_identifying: bool,
    },
    GetPowerState {
        power_state: PowerState,
    },
    GetPerformSelfTest {
        is_active: bool,
    },
    GetSelfTestDescription {
        self_test_id: u8,
        description: String,
    },
    GetPresetPlayback {
        mode: PresetPlaybackMode,
        level: u8,
    },
}

impl ResponseParameterData {
    pub fn decode(
        command_class: CommandClass,
        parameter_id: ParameterId,
        bytes: &[u8],
    ) -> Result<Self, ProtocolError> {
        match (command_class, parameter_id) {
            (CommandClass::DiscoveryCommandResponse, ParameterId::DiscMute) => {
                let binding_uid = if bytes.len() > 2 {
                    let manufacturer_id = u16::from_be_bytes(bytes[2..=3].try_into()?);
                    let device_id = u32::from_be_bytes(bytes[4..=7].try_into()?);
                    Some(DeviceUID::new(manufacturer_id, device_id))
                } else {
                    None
                };

                Ok(Self::DiscMute {
                    control_field: u16::from_be_bytes(bytes[..=1].try_into()?),
                    binding_uid,
                })
            }
            (CommandClass::DiscoveryCommandResponse, ParameterId::DiscUnMute) => {
                let binding_uid = if bytes.len() > 2 {
                    let manufacturer_id = u16::from_be_bytes(bytes[2..=3].try_into()?);
                    let device_id = u32::from_be_bytes(bytes[4..=7].try_into()?);
                    Some(DeviceUID::new(manufacturer_id, device_id))
                } else {
                    None
                };

                Ok(Self::DiscUnMute {
                    control_field: u16::from_be_bytes(bytes[..=1].try_into()?),
                    binding_uid,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::ProxiedDeviceCount) => {
                Ok(Self::GetProxiedDeviceCount {
                    device_count: u16::from_be_bytes(bytes[0..=1].try_into()?),
                    list_change: bytes[2] == 1,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::ProxiedDevices) => {
                Ok(Self::GetProxiedDevices {
                    device_uids: bytes
                        .chunks(6)
                        .map(|chunk| {
                            Ok(DeviceUID::new(
                                u16::from_be_bytes(chunk[0..=1].try_into()?),
                                u32::from_be_bytes(chunk[2..=5].try_into()?),
                            ))
                        })
                        .collect::<Result<Vec<DeviceUID>, ProtocolError>>()?,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::CommsStatus) => {
                Ok(Self::GetCommsStatus {
                    short_message: u16::from_be_bytes(bytes[0..=1].try_into()?),
                    length_mismatch: u16::from_be_bytes(bytes[2..=3].try_into()?),
                    checksum_fail: u16::from_be_bytes(bytes[4..=5].try_into()?),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::StatusMessages) => {
                Ok(Self::GetStatusMessages {
                    status_messages: bytes
                        .chunks(9)
                        .map(|chunk| {
                            Ok(StatusMessage::new(
                                u16::from_be_bytes(chunk[0..=1].try_into()?),
                                chunk[2].try_into()?,
                                u16::from_be_bytes(chunk[3..=4].try_into()?),
                                u16::from_be_bytes(chunk[5..=6].try_into()?),
                                u16::from_be_bytes(chunk[7..=8].try_into()?),
                            ))
                        })
                        .collect::<Result<Vec<StatusMessage>, ProtocolError>>()?,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::StatusIdDescription) => {
                Ok(Self::GetStatusIdDescription {
                    status_id_description: CStr::from_bytes_with_nul(bytes)?
                        .to_string_lossy()
                        .to_string(),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::SubDeviceStatusReportThreshold) => {
                Ok(Self::GetSubDeviceStatusReportThreshold {
                    status_type: bytes[0].try_into()?,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::SupportedParameters) => {
                let parameters = bytes
                    .chunks(2)
                    .map(|chunk| u16::from_be_bytes(chunk.try_into().unwrap()));

                Ok(Self::GetSupportedParameters {
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
            (CommandClass::GetCommandResponse, ParameterId::ParameterDescription) => {
                Ok(Self::GetParameterDescription(ParameterDescription {
                    parameter_id: u16::from_be_bytes(bytes[0..=1].try_into()?),
                    parameter_data_length: bytes[2],
                    data_type: bytes[3].try_into()?,
                    command_class: bytes[4].try_into()?,
                    unit_type: bytes[6].try_into()?,
                    prefix: bytes[7].try_into()?,
                    raw_minimum_valid_value: bytes[8..=11].try_into()?,
                    raw_maximum_valid_value: bytes[12..=15].try_into()?,
                    raw_default_value: bytes[16..=19].try_into()?,
                    description: CStr::from_bytes_with_nul(&bytes[20..])?
                        .to_string_lossy()
                        .to_string(),
                }))
            }
            (CommandClass::GetCommandResponse, ParameterId::DeviceInfo) => {
                Ok(Self::GetDeviceInfo {
                    protocol_version: format!("{}.{}", bytes[0], bytes[1]),
                    model_id: u16::from_be_bytes(bytes[2..=3].try_into()?),
                    product_category: u16::from_be_bytes(bytes[4..=5].try_into()?).try_into()?,
                    software_version_id: u32::from_be_bytes(bytes[6..=9].try_into()?),
                    footprint: u16::from_be_bytes(bytes[10..=11].try_into()?),
                    current_personality: bytes[12],
                    personality_count: bytes[13],
                    start_address: u16::from_be_bytes(bytes[14..=15].try_into()?),
                    sub_device_count: u16::from_be_bytes(bytes[16..=17].try_into()?),
                    sensor_count: u8::from_be(bytes[18]),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::ProductDetailIdList) => {
                Ok(Self::GetProductDetailIdList {
                    product_detail_id_list: bytes
                        .chunks(2)
                        .map(|chunk| Ok(u16::from_be_bytes(chunk.try_into()?)))
                        .collect::<Result<Vec<u16>, ProtocolError>>()?,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::DeviceModelDescription) => {
                Ok(Self::GetDeviceModelDescription {
                    device_model_description: CStr::from_bytes_with_nul(bytes)?
                        .to_string_lossy()
                        .to_string(),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::ManufacturerLabel) => {
                Ok(Self::GetManufacturerLabel {
                    manufacturer_label: CStr::from_bytes_with_nul(bytes)?
                        .to_string_lossy()
                        .to_string(),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::DeviceLabel) => {
                Ok(Self::GetDeviceLabel {
                    device_label: CStr::from_bytes_with_nul(bytes)?
                        .to_string_lossy()
                        .to_string(),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::FactoryDefaults) => {
                Ok(Self::GetFactoryDefaults {
                    factory_default: bytes[0] == 1,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::LanguageCapabilities) => {
                Ok(Self::GetLanguageCapabilities {
                    language_capabilities: bytes
                        .chunks(2)
                        .map(|chunk| Ok(std::str::from_utf8(chunk)?.to_string()))
                        .collect::<Result<Vec<String>, ProtocolError>>()?,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::Language) => Ok(Self::GetLanguage {
                current_language: std::str::from_utf8(&bytes[0..=1])?.to_string(),
            }),
            (CommandClass::GetCommandResponse, ParameterId::SoftwareVersionLabel) => {
                Ok(Self::GetSoftwareVersionLabel {
                    software_version_label: CStr::from_bytes_with_nul(bytes)?
                        .to_string_lossy()
                        .to_string(),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::BootSoftwareVersionId) => {
                Ok(Self::GetBootSoftwareVersionId {
                    boot_software_version_id: u32::from_be_bytes(bytes.try_into()?),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::BootSoftwareVersionLabel) => {
                Ok(Self::GetBootSoftwareVersionLabel {
                    boot_software_version_label: CStr::from_bytes_with_nul(bytes)?
                        .to_string_lossy()
                        .to_string(),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::DmxPersonality) => {
                Ok(Self::GetDmxPersonality {
                    current_personality: bytes[0],
                    personality_count: bytes[1],
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::DmxPersonalityDescription) => {
                Ok(Self::GetDmxPersonalityDescription {
                    id: bytes[0],
                    dmx_slots_required: u16::from_be_bytes(bytes[1..=2].try_into()?),
                    description: CStr::from_bytes_with_nul(&bytes[3..])?
                        .to_string_lossy()
                        .to_string(),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::DmxStartAddress) => {
                Ok(Self::GetDmxStartAddress {
                    dmx_start_address: u16::from_be_bytes(bytes[0..=1].try_into()?),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::SlotInfo) => Ok(Self::GetSlotInfo {
                dmx_slots: bytes
                    .chunks(5)
                    .map(|chunk| {
                        Ok(SlotInfo::new(
                            u16::from_be_bytes(chunk[0..=1].try_into()?),
                            chunk[2].try_into()?,
                            u16::from_be_bytes(chunk[3..=4].try_into()?),
                        ))
                    })
                    .collect::<Result<Vec<SlotInfo>, ProtocolError>>()?,
            }),
            (CommandClass::GetCommandResponse, ParameterId::SlotDescription) => {
                Ok(Self::GetSlotDescription {
                    slot_id: u16::from_be_bytes(bytes[0..=1].try_into()?),
                    description: CStr::from_bytes_with_nul(&bytes[2..])?
                        .to_string_lossy()
                        .to_string(),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::DefaultSlotValue) => {
                Ok(Self::GetDefaultSlotValue {
                    default_slot_values: bytes
                        .chunks(3)
                        .map(|chunk| {
                            Ok(DefaultSlotValue::new(
                                u16::from_be_bytes(chunk[0..=1].try_into()?),
                                chunk[2],
                            ))
                        })
                        .collect::<Result<Vec<DefaultSlotValue>, ProtocolError>>()?,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::SensorDefinition) => {
                Ok(Self::GetSensorDefinition {
                    sensor: SensorDefinition {
                        id: bytes[0],
                        kind: bytes[1].try_into()?,
                        unit: bytes[2].try_into()?,
                        prefix: bytes[3].try_into()?,
                        range_minimum_value: i16::from_be_bytes(bytes[4..=5].try_into()?),
                        range_maximum_value: i16::from_be_bytes(bytes[6..=7].try_into()?),
                        normal_minimum_value: i16::from_be_bytes(bytes[8..=9].try_into()?),
                        normal_maximum_value: i16::from_be_bytes(bytes[10..=11].try_into()?),
                        is_lowest_highest_detected_value_supported: bytes[12] >> 1 & 1 == 1,
                        is_recorded_value_supported: bytes[12] & 1 == 1,
                        description: CStr::from_bytes_with_nul(&bytes[13..])?
                            .to_string_lossy()
                            .to_string(),
                    },
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::SensorValue) => {
                Ok(Self::GetSensorValue(SensorValue::new(
                    bytes[0],
                    i16::from_be_bytes(bytes[1..=2].try_into()?),
                    i16::from_be_bytes(bytes[3..=4].try_into()?),
                    i16::from_be_bytes(bytes[5..=6].try_into()?),
                    i16::from_be_bytes(bytes[7..=8].try_into()?),
                )))
            }
            (CommandClass::SetCommandResponse, ParameterId::SensorValue) => {
                Ok(Self::SetSensorValue(SensorValue::new(
                    bytes[0],
                    i16::from_be_bytes(bytes[1..=2].try_into()?),
                    i16::from_be_bytes(bytes[3..=4].try_into()?),
                    i16::from_be_bytes(bytes[5..=6].try_into()?),
                    i16::from_be_bytes(bytes[7..=8].try_into()?),
                )))
            }
            (CommandClass::GetCommandResponse, ParameterId::DeviceHours) => {
                Ok(Self::GetDeviceHours {
                    device_hours: u32::from_be_bytes(bytes[0..=3].try_into()?),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::LampHours) => Ok(Self::GetLampHours {
                lamp_hours: u32::from_be_bytes(bytes[0..=3].try_into()?),
            }),
            (CommandClass::GetCommandResponse, ParameterId::LampStrikes) => {
                Ok(Self::GetLampStrikes {
                    lamp_strikes: u32::from_be_bytes(bytes[0..=3].try_into()?),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::LampState) => Ok(Self::GetLampState {
                lamp_state: bytes[0].try_into()?,
            }),
            (CommandClass::GetCommandResponse, ParameterId::LampOnMode) => {
                Ok(Self::GetLampOnMode {
                    lamp_on_mode: bytes[0].try_into()?,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::DevicePowerCycles) => {
                Ok(Self::GetDevicePowerCycles {
                    power_cycle_count: u32::from_be_bytes(bytes[0..=3].try_into()?),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::DisplayInvert) => {
                Ok(Self::GetDisplayInvert {
                    display_invert_mode: bytes[0].try_into()?,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::DisplayLevel) => {
                Ok(Self::GetDisplayLevel {
                    display_level: bytes[0],
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::PanInvert) => Ok(Self::GetPanInvert {
                pan_invert: bytes[0] == 1,
            }),
            (CommandClass::GetCommandResponse, ParameterId::TiltInvert) => {
                Ok(Self::GetTiltInvert {
                    tilt_invert: bytes[0] == 1,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::PanTiltSwap) => {
                Ok(Self::GetPanTiltSwap {
                    pan_tilt_swap: bytes[0] == 1,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::RealTimeClock) => {
                Ok(Self::GetRealTimeClock {
                    year: u16::from_be_bytes(bytes[0..=1].try_into()?),
                    month: bytes[2],
                    day: bytes[3],
                    hour: bytes[4],
                    minute: bytes[5],
                    second: bytes[6],
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::IdentifyDevice) => {
                Ok(Self::GetIdentifyDevice {
                    is_identifying: bytes[0] == 1,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::PowerState) => {
                Ok(Self::GetPowerState {
                    power_state: bytes[0].try_into()?,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::PerformSelfTest) => {
                Ok(Self::GetPerformSelfTest {
                    is_active: bytes[0] == 1,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::SelfTestDescription) => {
                Ok(Self::GetSelfTestDescription {
                    self_test_id: bytes[0],
                    description: CStr::from_bytes_with_nul(&bytes[1..])?
                        .to_string_lossy()
                        .to_string(),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::PresetPlayback) => {
                Ok(Self::GetPresetPlayback {
                    mode: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                    level: bytes[2],
                })
            }
            (_, _) => Err(ProtocolError::UnsupportedParameter(
                command_class as u8,
                parameter_id as u16,
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RdmResponse {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub response_type: ResponseType,
    pub message_count: u8,
    pub sub_device_id: u16,
    pub command_class: CommandClass,
    pub parameter_id: ParameterId,
    pub parameter_data: Option<ResponseParameterData>,
}

impl RdmResponse {
    pub fn parse(bytes: &mut [u8]) -> Result<Self, ProtocolError> {
        let message_length = bytes[2];

        if message_length < 24 {
            return Err(ProtocolError::InvalidMessageLength(message_length));
        }

        let packet_checksum = u16::from_be_bytes(
            bytes[message_length as usize..=message_length as usize + 1].try_into()?,
        );

        let decoded_checksum = bsd_16_crc(&bytes[..message_length as usize - 1]);

        if decoded_checksum != packet_checksum {
            return Err(ProtocolError::InvalidChecksum(
                decoded_checksum,
                packet_checksum,
            ));
        }

        let destination_manufacturer_id = u16::from_be_bytes(bytes[3..=4].try_into()?);
        let destination_device_id = u32::from_be_bytes(bytes[5..=8].try_into()?);
        let destination_uid = DeviceUID::new(destination_manufacturer_id, destination_device_id);

        let source_manufacturer_id = u16::from_be_bytes(bytes[9..=10].try_into()?);
        let source_device_id = u32::from_be_bytes(bytes[11..=14].try_into()?);
        let source_uid = DeviceUID::new(source_manufacturer_id, source_device_id);

        let transaction_number = bytes[15];

        let response_type = ResponseType::try_from(bytes[16])?;

        let message_count = bytes[17];

        let sub_device_id = u16::from_be_bytes(bytes[18..=19].try_into()?);

        let command_class = CommandClass::try_from(bytes[20])?;

        let parameter_id = ParameterId::try_from(u16::from_be_bytes(bytes[21..=22].try_into()?))?;

        let parameter_data_length = bytes[23];

        if parameter_data_length > 231 {
            return Err(ProtocolError::InvalidParameterDataLength(
                parameter_data_length,
            ));
        }

        let parameter_data = if parameter_data_length > 0 {
            Some(ResponseParameterData::decode(
                command_class,
                parameter_id,
                &bytes[25..=message_length as usize + 1],
            )?)
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

#[derive(Clone, Debug, PartialEq)]
pub struct DiscoveryUniqueBranchResponse(DeviceUID);

impl DiscoveryUniqueBranchResponse {
    pub fn parse(bytes: &mut [u8]) -> Result<Self, ProtocolError> {
        let Some(frame_start_index) = bytes.iter().position(|&x| x == 0xaa) else {
            return Err(ProtocolError::InvalidDiscoveryUniqueBranchPreamble);
        };

        let euid = &bytes[(frame_start_index + 1)..=(frame_start_index + 12)];

        let ecs = &bytes[(frame_start_index + 13)..=(frame_start_index + 16)];

        let decoded_checksum = bsd_16_crc(euid);

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

#[derive(Clone, Debug, PartialEq)]
pub enum RdmFrame {
    Rdm(RdmResponse),
    DiscoveryUniqueBranch(DiscoveryUniqueBranchResponse),
}

impl RdmFrame {
    pub fn parse(bytes: &mut [u8]) -> Result<Option<Self>, ProtocolError> {
        if bytes[0] == SC_RDM && bytes[1] == SC_SUB_MESSAGE {
            if bytes.len() < 25 {
                return Err(ProtocolError::InvalidFrameLength(bytes.len() as u8));
            }

            return RdmResponse::parse(bytes).map(|frame| Some(RdmFrame::Rdm(frame)));
        }

        if bytes[0] == DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE
            || bytes[0] == DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE
        {
            if bytes.len() < 17 {
                return Err(ProtocolError::InvalidFrameLength(bytes.len() as u8));
            }

            return DiscoveryUniqueBranchResponse::parse(bytes)
                .map(|frame| Some(RdmFrame::DiscoveryUniqueBranch(frame)));
        }

        Err(ProtocolError::InvalidStartCode)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_valid_rdm_response() {
        let bytes = &mut [
            SC_RDM,
            SC_SUB_MESSAGE,
            25,   // message length
            0x01, // destination uid
            0x02,
            0x03,
            0x04,
            0x05,
            0x06,
            0x06, // source uid
            0x05,
            0x04,
            0x03,
            0x02,
            0x01,
            0x00, // transaction number
            0x00, // response type = Ack
            0x00, // message count
            0x00, // sub device id = root device
            0x00,
            0x21, // command class = get command response
            0x10, // parameter id = identify device
            0x00,
            0x01, // parameter data length
            0x01, // identifying = true
            0x01,
            0x42, // checksum
        ];

        assert_eq!(
            RdmFrame::parse(bytes),
            Ok(Some(RdmFrame::Rdm(RdmResponse {
                destination_uid: DeviceUID::new(0x0102, 0x03040506),
                source_uid: DeviceUID::new(0x0605, 0x04030201),
                transaction_number: 0x00,
                response_type: ResponseType::Ack,
                message_count: 0x00,
                sub_device_id: 0x0000,
                command_class: CommandClass::GetCommandResponse,
                parameter_id: ParameterId::IdentifyDevice,
                parameter_data: Some(ResponseParameterData::GetIdentifyDevice {
                    is_identifying: true,
                }),
            })))
        );
    }

    #[test]
    fn should_parse_valid_discovery_unique_branch_response() {
        // includes preamble bytes
        let bytes = &mut [
            DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
            DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
            DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
            DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
            DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
            DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
            DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
            DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE,
            0xab, // euid 11 = manufacturer id 1 (MSB)
            0x55, // euid 10 = manufacturer id 1 (MSB)
            0xaa, // euid 9 = manufacturer id 0 (LSB)
            0x57, // euid 8 = manufacturer id 0 (LSB)
            0xab, // euid 7 = device id 3 (MSB)
            0x57, // euid 6 = device id 3 (MSB)
            0xae, // euid 5 = device id 2
            0x55, // euid 4 = device id 2
            0xaf, // euid 3 = device id 1
            0x55, // euid 2 = device id 1
            0xae, // euid 1 = device id 0 (LSB)
            0x57, // euid 0 = device id 0 (LSB)
            0xae, // ecs 3 = Checksum1 (MSB)
            0x57, // ecs 2 = Checksum1 (MSB)
            0xaf, // ecs 1 = Checksum0 (LSB)
            0x5f, // ecs 0 = Checksum0 (LSB)
        ];

        assert_eq!(
            RdmFrame::parse(bytes),
            Ok(Some(RdmFrame::DiscoveryUniqueBranch(
                DiscoveryUniqueBranchResponse(DeviceUID::new(0x0102, 0x03040506))
            )))
        );

        // does not include preamble bytes
        let bytes = &mut [
            DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE,
            0xab, // euid 11 = manufacturer id 1 (MSB)
            0x55, // euid 10 = manufacturer id 1 (MSB)
            0xaa, // euid 9 = manufacturer id 0 (LSB)
            0x57, // euid 8 = manufacturer id 0 (LSB)
            0xab, // euid 7 = device id 3 (MSB)
            0x57, // euid 6 = device id 3 (MSB)
            0xae, // euid 5 = device id 2
            0x55, // euid 4 = device id 2
            0xaf, // euid 3 = device id 1
            0x55, // euid 2 = device id 1
            0xae, // euid 1 = device id 0 (LSB)
            0x57, // euid 0 = device id 0 (LSB)
            0xae, // ecs 3 = Checksum1 (MSB)
            0x57, // ecs 2 = Checksum1 (MSB)
            0xaf, // ecs 1 = Checksum0 (LSB)
            0x5f, // ecs 0 = Checksum0 (LSB)
        ];

        assert_eq!(
            RdmFrame::parse(bytes),
            Ok(Some(RdmFrame::DiscoveryUniqueBranch(
                DiscoveryUniqueBranchResponse(DeviceUID::new(0x0102, 0x03040506))
            )))
        );
    }
}
