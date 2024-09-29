//! Data types and functionality for decoding RDM responses
//!
//! ### RdmResponse
//!
//! ```rust
//! use dmx512_rdm_protocol::rdm::{
//!     parameter::ParameterId,
//!     response::{
//!         RdmFrameResponse, RdmResponse, ResponseData, ResponseParameterData, ResponseType,
//!     },
//!     CommandClass, DeviceUID, SubDeviceId,
//! };
//!
//! let decoded = RdmResponse::decode(&[
//!     0xcc, // Start Code
//!     0x01, // Sub Start Code
//!     0x19, // Message Length
//!     0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // Destination UID
//!     0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // Source UID
//!     0x00, // Transaction Number
//!     0x00, // Response Type = Ack
//!     0x00, // Message Count
//!     0x00, 0x00, // Sub-Device ID = Root Device
//!     0x21, // Command Class = GetCommandResponse
//!     0x10, 0x00, // Parameter ID = Identify Device
//!     0x01, // PDL
//!     0x01, // Identifying = true
//!     0x01, 0x43, // Checksum
//! ]);
//!
//! let expected = Ok(RdmResponse::RdmFrame(RdmFrameResponse {
//!     destination_uid: DeviceUID::new(0x0102, 0x03040506),
//!     source_uid: DeviceUID::new(0x0605, 0x04030201),
//!     transaction_number: 0x00,
//!     response_type: ResponseType::Ack,
//!     message_count: 0x00,
//!     sub_device_id: SubDeviceId::RootDevice,
//!     command_class: CommandClass::GetCommandResponse,
//!     parameter_id: ParameterId::IdentifyDevice,
//!     parameter_data: ResponseData::ParameterData(Some(
//!         ResponseParameterData::GetIdentifyDevice(true),
//!     )),
//! }));
//!
//! assert_eq!(decoded, expected);
//! ```

use super::{
    bsd_16_crc,
    parameter::{
        decode_string_bytes, DefaultSlotValue, DisplayInvertMode, LampOnMode, LampState, MergeMode,
        ParameterDescription, ParameterId, PinCode, PowerState, PresetPlaybackMode,
        PresetProgrammed, ProductCategory, ProductDetail, SelfTest, SensorDefinition, SensorValue,
        SlotInfo, StatusMessage, StatusType, SupportedTimes, TimeMode,
    },
    CommandClass, DeviceUID, RdmError, SubDeviceId, DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
    DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE, RDM_START_CODE_BYTE, RDM_SUB_START_CODE_BYTE,
};
use core::{fmt::Display, result::Result};

#[cfg(not(feature = "alloc"))]
use core::str::FromStr;
#[cfg(not(feature = "alloc"))]
use heapless::{String, Vec};

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
    SubDeviceIdOutOfRange = 0x0009,
    ProxyBufferFull = 0x000a,
}

impl TryFrom<u16> for ResponseNackReasonCode {
    type Error = RdmError;

    fn try_from(value: u16) -> Result<Self, RdmError> {
        match value {
            0x0000 => Ok(Self::UnknownPid),
            0x0001 => Ok(Self::FormatError),
            0x0002 => Ok(Self::HardwareFault),
            0x0003 => Ok(Self::ProxyReject),
            0x0004 => Ok(Self::WriteProtect),
            0x0005 => Ok(Self::UnsupportedCommandClass),
            0x0006 => Ok(Self::DataOutOfRange),
            0x0007 => Ok(Self::BufferFull),
            0x0008 => Ok(Self::PacketSizeUnsupported),
            0x0009 => Ok(Self::SubDeviceIdOutOfRange),
            0x000a => Ok(Self::ProxyBufferFull),
            value => Err(RdmError::InvalidNackReasonCode(value)),
        }
    }
}

impl Display for ResponseNackReasonCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Self::UnknownPid => "The responder cannot comply with request because the message is not implemented in responder.",
            Self::FormatError => "The responder cannot interpret request as controller data was not formatted correctly.",
            Self::HardwareFault => "The responder cannot comply due to an internal hardware fault.",
            Self::ProxyReject => "Proxy is not the RDM line master and cannot comply with message.",
            Self::WriteProtect => "Command normally allowed but being blocked currently.",
            Self::UnsupportedCommandClass => "Not valid for Command Class attempted. May be used where GET allowed but SET is not supported.",
            Self::DataOutOfRange => "Value for given Parameter out of allowable range or not supported.",
            Self::BufferFull => "Buffer or Queue space currently has no free space to store data.",
            Self::PacketSizeUnsupported => "Incoming message exceeds buffer capacity.",
            Self::SubDeviceIdOutOfRange => "Sub-Device is out of range or unknown.",
            Self::ProxyBufferFull => "The proxy buffer is full and can not store any more Queued Message or Status Message responses.",
        };

        f.write_str(message)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ResponseType {
    Ack = 0x00,
    AckTimer = 0x01,
    NackReason = 0x02,
    AckOverflow = 0x03,
}

impl TryFrom<u8> for ResponseType {
    type Error = RdmError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Ack),
            0x01 => Ok(Self::AckTimer),
            0x02 => Ok(Self::NackReason),
            0x03 => Ok(Self::AckOverflow),
            _ => Err(RdmError::InvalidResponseType(value)),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum ResponseData {
    ParameterData(Option<ResponseParameterData>),
    EstimateResponseTime(u16),
    NackReason(ResponseNackReasonCode),
}

#[allow(clippy::large_enum_variant)]
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum ResponseParameterData {
    // E1.20
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
    GetProxiedDevices(
        #[cfg(feature = "alloc")] Vec<DeviceUID>,
        #[cfg(not(feature = "alloc"))] Vec<DeviceUID, 38>,
    ),
    GetCommsStatus {
        short_message: u16,
        length_mismatch: u16,
        checksum_fail: u16,
    },
    GetStatusMessages(
        #[cfg(feature = "alloc")] Vec<StatusMessage>,
        #[cfg(not(feature = "alloc"))] Vec<StatusMessage, 25>,
    ),
    GetStatusIdDescription(
        #[cfg(feature = "alloc")] String,
        #[cfg(not(feature = "alloc"))] String<32>,
    ),
    GetSubDeviceIdStatusReportThreshold(StatusType),
    GetSupportedParameters(
        #[cfg(feature = "alloc")] Vec<u16>,
        #[cfg(not(feature = "alloc"))] Vec<u16, 115>,
    ),
    GetParameterDescription(ParameterDescription),
    GetDeviceInfo {
        #[cfg(feature = "alloc")]
        protocol_version: String,
        #[cfg(not(feature = "alloc"))]
        protocol_version: String<32>,
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
    GetProductDetailIdList(
        #[cfg(feature = "alloc")] Vec<ProductDetail>,
        #[cfg(not(feature = "alloc"))] Vec<ProductDetail, 115>,
    ),
    GetDeviceModelDescription(
        #[cfg(feature = "alloc")] String,
        #[cfg(not(feature = "alloc"))] String<32>,
    ),
    GetManufacturerLabel(
        #[cfg(feature = "alloc")] String,
        #[cfg(not(feature = "alloc"))] String<32>,
    ),
    GetDeviceLabel(
        #[cfg(feature = "alloc")] String,
        #[cfg(not(feature = "alloc"))] String<32>,
    ),
    GetFactoryDefaults(bool),
    GetLanguageCapabilities(
        #[cfg(feature = "alloc")] Vec<String>,
        #[cfg(not(feature = "alloc"))] Vec<String<2>, 115>,
    ),
    GetLanguage(
        #[cfg(feature = "alloc")] String,
        #[cfg(not(feature = "alloc"))] String<2>,
    ),
    GetSoftwareVersionLabel(
        #[cfg(feature = "alloc")] String,
        #[cfg(not(feature = "alloc"))] String<32>,
    ),
    GetBootSoftwareVersionId(u32),
    GetBootSoftwareVersionLabel(
        #[cfg(feature = "alloc")] String,
        #[cfg(not(feature = "alloc"))] String<32>,
    ),
    GetDmxPersonality {
        current_personality: u8,
        personality_count: u8,
    },
    GetDmxPersonalityDescription {
        id: u8,
        dmx_slots_required: u16,
        #[cfg(feature = "alloc")]
        description: String,
        #[cfg(not(feature = "alloc"))]
        description: String<32>,
    },
    GetDmxStartAddress(u16),
    GetSlotInfo(
        #[cfg(feature = "alloc")] Vec<SlotInfo>,
        #[cfg(not(feature = "alloc"))] Vec<SlotInfo, 46>,
    ),
    GetSlotDescription {
        slot_id: u16,
        #[cfg(feature = "alloc")]
        description: String,
        #[cfg(not(feature = "alloc"))]
        description: String<32>,
    },
    GetDefaultSlotValue(
        #[cfg(feature = "alloc")] Vec<DefaultSlotValue>,
        #[cfg(not(feature = "alloc"))] Vec<DefaultSlotValue, 77>,
    ),
    GetSensorDefinition(SensorDefinition),
    GetSensorValue(SensorValue),
    SetSensorValue(SensorValue),
    GetDeviceHours(u32),
    GetLampHours(u32),
    GetLampStrikes(u32),
    GetLampState(LampState),
    GetLampOnMode(LampOnMode),
    GetDevicePowerCycles(u32),
    GetDisplayInvert(DisplayInvertMode),
    GetDisplayLevel(u8),
    GetPanInvert(bool),
    GetTiltInvert(bool),
    GetPanTiltSwap(bool),
    GetRealTimeClock {
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    },
    GetIdentifyDevice(bool),
    GetPowerState(PowerState),
    GetPerformSelfTest(bool),
    GetSelfTestDescription {
        self_test_id: SelfTest,
        #[cfg(feature = "alloc")]
        description: String,
        #[cfg(not(feature = "alloc"))]
        description: String<32>,
    },
    GetPresetPlayback {
        mode: PresetPlaybackMode,
        level: u8,
    },
    // E1.37-1
    GetDmxBlockAddress {
        total_sub_device_footprint: u16,
        base_dmx_address: u16,
    },
    GetDmxFailMode {
        scene_id: PresetPlaybackMode,
        loss_of_signal_delay: TimeMode,
        hold_time: TimeMode,
        level: u8,
    },
    GetDmxStartupMode {
        scene_id: PresetPlaybackMode,
        startup_delay: TimeMode,
        hold_time: TimeMode,
        level: u8,
    },
    GetPowerOnSelfTest(bool),
    GetLockState {
        lock_state_id: u8,
        lock_state_count: u8,
    },
    GetLockStateDescription {
        lock_state_id: u8,
        #[cfg(feature = "alloc")]
        description: String,
        #[cfg(not(feature = "alloc"))]
        description: String<32>,
    },
    GetLockPin(PinCode),
    GetBurnIn(u8),
    GetDimmerInfo {
        minimum_level_lower_limit: u16,
        minimum_level_upper_limit: u16,
        maximum_level_lower_limit: u16,
        maximum_level_upper_limit: u16,
        number_of_supported_curves: u8,
        levels_resolution: u8,
        minimum_level_split_levels_supported: bool,
    },
    GetMinimumLevel {
        minimum_level_increasing: u16,
        minimum_level_decreasing: u16,
        on_below_minimum: bool,
    },
    GetMaximumLevel(u16),
    GetCurve {
        curve_id: u8,
        curve_count: u8,
    },
    GetCurveDescription {
        curve_id: u8,
        #[cfg(feature = "alloc")]
        description: String,
        #[cfg(not(feature = "alloc"))]
        description: String<32>,
    },
    GetOutputResponseTime {
        response_time_id: u8,
        response_time_count: u8,
    },
    GetOutputResponseTimeDescription {
        response_time_id: u8,
        #[cfg(feature = "alloc")]
        description: String,
        #[cfg(not(feature = "alloc"))]
        description: String<32>,
    },
    GetModulationFrequency {
        modulation_frequency_id: u8,
        modulation_frequency_count: u8,
    },
    GetModulationFrequencyDescription {
        modulation_frequency_id: u8,
        frequency: u32,
        #[cfg(feature = "alloc")]
        description: String,
        #[cfg(not(feature = "alloc"))]
        description: String<32>,
    },
    GetPresetInfo {
        level_field_supported: bool,
        preset_sequence_supported: bool,
        split_times_supported: bool,
        dmx_fail_infinite_delay_time_supported: bool,
        dmx_fail_infinite_hold_time_supported: bool,
        startup_infinite_hold_time_supported: bool,
        maximum_scene_number: u16,
        minimum_preset_fade_time_supported: u16,
        maximum_preset_fade_time_supported: u16,
        minimum_preset_wait_time_supported: u16,
        maximum_preset_wait_time_supported: u16,
        minimum_dmx_fail_delay_time_supported: SupportedTimes,
        maximum_dmx_fail_delay_time_supported: SupportedTimes,
        minimum_dmx_fail_hold_time_supported: SupportedTimes,
        maximum_dmx_fail_hold_time_supported: SupportedTimes,
        minimum_startup_delay_time_supported: SupportedTimes,
        maximum_startup_delay_time_supported: SupportedTimes,
        minimum_startup_hold_time_supported: SupportedTimes,
        maximum_startup_hold_time_supported: SupportedTimes,
    },
    GetPresetStatus {
        scene_id: u16,
        up_fade_time: u16,
        down_fade_time: u16,
        wait_time: u16,
        programmed: PresetProgrammed,
    },
    GetPresetMergeMode(MergeMode),
    ManufacturerSpecific(
        #[cfg(feature = "alloc")] Vec<u8>,
        #[cfg(not(feature = "alloc"))] Vec<u8, 231>,
    ),
    Unsupported(
        #[cfg(feature = "alloc")] Vec<u8>,
        #[cfg(not(feature = "alloc"))] Vec<u8, 231>,
    ),
}

impl ResponseParameterData {
    pub fn decode(
        command_class: CommandClass,
        parameter_id: ParameterId,
        bytes: &[u8],
    ) -> Result<Self, RdmError> {
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
                Ok(Self::GetProxiedDevices(
                    #[cfg(feature = "alloc")]
                    bytes
                        .chunks(6)
                        .map(|chunk| {
                            Ok(DeviceUID::new(
                                u16::from_be_bytes(chunk[0..=1].try_into()?),
                                u32::from_be_bytes(chunk[2..=5].try_into()?),
                            ))
                        })
                        .collect::<Result<Vec<DeviceUID>, RdmError>>()?,
                    #[cfg(not(feature = "alloc"))]
                    bytes
                        .chunks(6)
                        .map(|chunk| {
                            Ok(DeviceUID::new(
                                u16::from_be_bytes(chunk[0..=1].try_into()?),
                                u32::from_be_bytes(chunk[2..=5].try_into()?),
                            ))
                        })
                        .collect::<Result<Vec<DeviceUID, 38>, RdmError>>()?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::CommsStatus) => {
                Ok(Self::GetCommsStatus {
                    short_message: u16::from_be_bytes(bytes[0..=1].try_into()?),
                    length_mismatch: u16::from_be_bytes(bytes[2..=3].try_into()?),
                    checksum_fail: u16::from_be_bytes(bytes[4..=5].try_into()?),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::StatusMessages) => {
                Ok(Self::GetStatusMessages(
                    #[cfg(feature = "alloc")]
                    bytes
                        .chunks(9)
                        .map(|chunk| {
                            Ok(StatusMessage::new(
                                u16::from_be_bytes(chunk[0..=1].try_into()?).into(),
                                chunk[2].try_into()?,
                                u16::from_be_bytes(chunk[3..=4].try_into()?),
                                u16::from_be_bytes(chunk[5..=6].try_into()?),
                                u16::from_be_bytes(chunk[7..=8].try_into()?),
                            ))
                        })
                        .collect::<Result<Vec<StatusMessage>, RdmError>>()?,
                    #[cfg(not(feature = "alloc"))]
                    bytes
                        .chunks(9)
                        .map(|chunk| {
                            Ok(StatusMessage::new(
                                u16::from_be_bytes(chunk[0..=1].try_into()?).into(),
                                chunk[2].try_into()?,
                                u16::from_be_bytes(chunk[3..=4].try_into()?),
                                u16::from_be_bytes(chunk[5..=6].try_into()?),
                                u16::from_be_bytes(chunk[7..=8].try_into()?),
                            ))
                        })
                        .collect::<Result<Vec<StatusMessage, 25>, RdmError>>()?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::StatusIdDescription) => {
                Ok(Self::GetStatusIdDescription(decode_string_bytes(bytes)?))
            }
            (CommandClass::GetCommandResponse, ParameterId::SubDeviceIdStatusReportThreshold) => {
                Ok(Self::GetSubDeviceIdStatusReportThreshold(
                    bytes[0].try_into()?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::SupportedParameters) => {
                let parameters = bytes
                    .chunks(2)
                    .map(|chunk| Ok(u16::from_be_bytes(chunk.try_into()?)))
                    .filter_map(|parameter_id: Result<u16, RdmError>| parameter_id.ok());

                Ok(Self::GetSupportedParameters(
                    #[cfg(feature = "alloc")]
                    parameters.collect::<Vec<u16>>(),
                    #[cfg(not(feature = "alloc"))]
                    parameters.collect::<Vec<u16, 115>>(),
                ))
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
                    description: decode_string_bytes(&bytes[20..])?,
                }))
            }
            (CommandClass::GetCommandResponse, ParameterId::DeviceInfo) => {
                Ok(Self::GetDeviceInfo {
                    #[cfg(feature = "alloc")]
                    protocol_version: format!("{}.{}", bytes[0], bytes[1]),
                    #[cfg(not(feature = "alloc"))]
                    protocol_version: String::<32>::from_str(
                        format_args!("{}.{}", bytes[0], bytes[1]).as_str().unwrap(),
                    )
                    .unwrap(),
                    model_id: u16::from_be_bytes(bytes[2..=3].try_into()?),
                    product_category: u16::from_be_bytes(bytes[4..=5].try_into()?).into(),
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
                Ok(Self::GetProductDetailIdList(
                    #[cfg(feature = "alloc")]
                    bytes
                        .chunks(2)
                        .map(|chunk| Ok(u16::from_be_bytes(chunk.try_into()?).into()))
                        .collect::<Result<Vec<ProductDetail>, RdmError>>()?,
                    #[cfg(not(feature = "alloc"))]
                    bytes
                        .chunks(2)
                        .map(|chunk| Ok(u16::from_be_bytes(chunk.try_into()?).into()))
                        .collect::<Result<Vec<ProductDetail, 115>, RdmError>>()?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::DeviceModelDescription) => {
                Ok(Self::GetDeviceModelDescription(decode_string_bytes(bytes)?))
            }
            (CommandClass::GetCommandResponse, ParameterId::ManufacturerLabel) => {
                Ok(Self::GetManufacturerLabel(decode_string_bytes(bytes)?))
            }
            (CommandClass::GetCommandResponse, ParameterId::DeviceLabel) => {
                Ok(Self::GetDeviceLabel(decode_string_bytes(bytes)?))
            }
            (CommandClass::GetCommandResponse, ParameterId::FactoryDefaults) => {
                Ok(Self::GetFactoryDefaults(bytes[0] == 1))
            }
            (CommandClass::GetCommandResponse, ParameterId::LanguageCapabilities) => {
                Ok(Self::GetLanguageCapabilities(
                    #[cfg(feature = "alloc")]
                    bytes
                        .chunks(2)
                        .map(|chunk| Ok(core::str::from_utf8(chunk)?.to_string()))
                        .collect::<Result<Vec<String>, RdmError>>()?,
                    #[cfg(not(feature = "alloc"))]
                    bytes
                        .chunks(2)
                        .map(|chunk| {
                            Ok(String::from_utf8(Vec::<u8, 2>::from_slice(chunk).unwrap())?)
                        })
                        .collect::<Result<Vec<String<2>, 115>, RdmError>>()?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::Language) => Ok(Self::GetLanguage(
                #[cfg(feature = "alloc")]
                core::str::from_utf8(&bytes[0..=1])?.to_string(),
                #[cfg(not(feature = "alloc"))]
                String::from_utf8(Vec::<u8, 2>::from_slice(&bytes[0..=1]).unwrap())?,
            )),
            (CommandClass::GetCommandResponse, ParameterId::SoftwareVersionLabel) => {
                Ok(Self::GetSoftwareVersionLabel(decode_string_bytes(bytes)?))
            }
            (CommandClass::GetCommandResponse, ParameterId::BootSoftwareVersionId) => Ok(
                Self::GetBootSoftwareVersionId(u32::from_be_bytes(bytes.try_into()?)),
            ),
            (CommandClass::GetCommandResponse, ParameterId::BootSoftwareVersionLabel) => Ok(
                Self::GetBootSoftwareVersionLabel(decode_string_bytes(bytes)?),
            ),
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
                    description: decode_string_bytes(&bytes[3..])?,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::DmxStartAddress) => Ok(
                Self::GetDmxStartAddress(u16::from_be_bytes(bytes[0..=1].try_into()?)),
            ),
            (CommandClass::GetCommandResponse, ParameterId::SlotInfo) => Ok(Self::GetSlotInfo(
                #[cfg(feature = "alloc")]
                bytes
                    .chunks(5)
                    .map(|chunk| {
                        Ok(SlotInfo::new(
                            u16::from_be_bytes(chunk[0..=1].try_into()?),
                            chunk[2].into(),
                            u16::from_be_bytes(chunk[3..=4].try_into()?),
                        ))
                    })
                    .collect::<Result<Vec<SlotInfo>, RdmError>>()?,
                #[cfg(not(feature = "alloc"))]
                bytes
                    .chunks(5)
                    .map(|chunk| {
                        Ok(SlotInfo::new(
                            u16::from_be_bytes(chunk[0..=1].try_into()?),
                            chunk[2].into(),
                            u16::from_be_bytes(chunk[3..=4].try_into()?),
                        ))
                    })
                    .collect::<Result<Vec<SlotInfo, 46>, RdmError>>()?,
            )),
            (CommandClass::GetCommandResponse, ParameterId::SlotDescription) => {
                Ok(Self::GetSlotDescription {
                    slot_id: u16::from_be_bytes(bytes[0..=1].try_into()?),
                    description: decode_string_bytes(&bytes[2..])?,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::DefaultSlotValue) => {
                Ok(Self::GetDefaultSlotValue(
                    #[cfg(feature = "alloc")]
                    bytes
                        .chunks(3)
                        .map(|chunk| {
                            Ok(DefaultSlotValue::new(
                                u16::from_be_bytes(chunk[0..=1].try_into()?),
                                chunk[2],
                            ))
                        })
                        .collect::<Result<Vec<DefaultSlotValue>, RdmError>>()?,
                    #[cfg(not(feature = "alloc"))]
                    bytes
                        .chunks(3)
                        .map(|chunk| {
                            Ok(DefaultSlotValue::new(
                                u16::from_be_bytes(chunk[0..=1].try_into()?),
                                chunk[2],
                            ))
                        })
                        .collect::<Result<Vec<DefaultSlotValue, 77>, RdmError>>()?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::SensorDefinition) => {
                Ok(Self::GetSensorDefinition(SensorDefinition {
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
                    description: decode_string_bytes(&bytes[13..])?,
                }))
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
            (CommandClass::GetCommandResponse, ParameterId::DeviceHours) => Ok(
                Self::GetDeviceHours(u32::from_be_bytes(bytes[0..=3].try_into()?)),
            ),
            (CommandClass::GetCommandResponse, ParameterId::LampHours) => Ok(Self::GetLampHours(
                u32::from_be_bytes(bytes[0..=3].try_into()?),
            )),
            (CommandClass::GetCommandResponse, ParameterId::LampStrikes) => Ok(
                Self::GetLampStrikes(u32::from_be_bytes(bytes[0..=3].try_into()?)),
            ),
            (CommandClass::GetCommandResponse, ParameterId::LampState) => {
                Ok(Self::GetLampState(bytes[0].try_into()?))
            }
            (CommandClass::GetCommandResponse, ParameterId::LampOnMode) => {
                Ok(Self::GetLampOnMode(bytes[0].try_into()?))
            }
            (CommandClass::GetCommandResponse, ParameterId::DevicePowerCycles) => Ok(
                Self::GetDevicePowerCycles(u32::from_be_bytes(bytes[0..=3].try_into()?)),
            ),
            (CommandClass::GetCommandResponse, ParameterId::DisplayInvert) => {
                Ok(Self::GetDisplayInvert(bytes[0].try_into()?))
            }
            (CommandClass::GetCommandResponse, ParameterId::DisplayLevel) => {
                Ok(Self::GetDisplayLevel(bytes[0]))
            }
            (CommandClass::GetCommandResponse, ParameterId::PanInvert) => {
                Ok(Self::GetPanInvert(bytes[0] == 1))
            }
            (CommandClass::GetCommandResponse, ParameterId::TiltInvert) => {
                Ok(Self::GetTiltInvert(bytes[0] == 1))
            }
            (CommandClass::GetCommandResponse, ParameterId::PanTiltSwap) => {
                Ok(Self::GetPanTiltSwap(bytes[0] == 1))
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
                Ok(Self::GetIdentifyDevice(bytes[0] == 1))
            }
            (CommandClass::GetCommandResponse, ParameterId::PowerState) => {
                Ok(Self::GetPowerState(bytes[0].try_into()?))
            }
            (CommandClass::GetCommandResponse, ParameterId::PerformSelfTest) => {
                Ok(Self::GetPerformSelfTest(bytes[0] == 1))
            }
            (CommandClass::GetCommandResponse, ParameterId::SelfTestDescription) => {
                Ok(Self::GetSelfTestDescription {
                    self_test_id: bytes[0].into(),
                    description: decode_string_bytes(&bytes[1..])?,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::PresetPlayback) => {
                Ok(Self::GetPresetPlayback {
                    mode: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                    level: bytes[2],
                })
            }
            // E1.37-1
            (CommandClass::GetCommandResponse, ParameterId::DmxBlockAddress) => {
                Ok(Self::GetDmxBlockAddress {
                    total_sub_device_footprint: u16::from_be_bytes(bytes[0..=1].try_into()?),
                    base_dmx_address: u16::from_be_bytes(bytes[2..=3].try_into()?),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::DmxFailMode) => {
                Ok(Self::GetDmxFailMode {
                    scene_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                    loss_of_signal_delay: u16::from_be_bytes(bytes[2..=3].try_into()?).into(),
                    hold_time: u16::from_be_bytes(bytes[4..=5].try_into()?).into(),
                    level: bytes[6],
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::DmxStartupMode) => {
                Ok(Self::GetDmxStartupMode {
                    scene_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                    startup_delay: u16::from_be_bytes(bytes[2..=3].try_into()?).into(),
                    hold_time: u16::from_be_bytes(bytes[4..=5].try_into()?).into(),
                    level: bytes[6],
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::PowerOnSelfTest) => {
                Ok(Self::GetPowerOnSelfTest(bytes[0] == 1))
            }
            (CommandClass::GetCommandResponse, ParameterId::LockState) => Ok(Self::GetLockState {
                lock_state_id: bytes[0],
                lock_state_count: bytes[1],
            }),
            (CommandClass::GetCommandResponse, ParameterId::LockStateDescription) => {
                Ok(Self::GetLockStateDescription {
                    lock_state_id: bytes[0],
                    description: decode_string_bytes(&bytes[1..])?,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::LockPin) => Ok(Self::GetLockPin(
                PinCode::try_from(u16::from_be_bytes(bytes[0..=1].try_into()?))?,
            )),
            (CommandClass::GetCommandResponse, ParameterId::BurnIn) => {
                Ok(Self::GetBurnIn(bytes[0]))
            }
            (CommandClass::GetCommandResponse, ParameterId::DimmerInfo) => {
                Ok(Self::GetDimmerInfo {
                    minimum_level_lower_limit: u16::from_be_bytes(bytes[0..=1].try_into()?),
                    minimum_level_upper_limit: u16::from_be_bytes(bytes[2..=3].try_into()?),
                    maximum_level_lower_limit: u16::from_be_bytes(bytes[4..=5].try_into()?),
                    maximum_level_upper_limit: u16::from_be_bytes(bytes[6..=7].try_into()?),
                    number_of_supported_curves: bytes[8],
                    levels_resolution: bytes[9],
                    minimum_level_split_levels_supported: bytes[10] == 1,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::MinimumLevel) => {
                Ok(Self::GetMinimumLevel {
                    minimum_level_increasing: u16::from_be_bytes(bytes[0..=1].try_into()?),
                    minimum_level_decreasing: u16::from_be_bytes(bytes[2..=3].try_into()?),
                    on_below_minimum: bytes[4] == 1,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::MaximumLevel) => Ok(
                Self::GetMaximumLevel(u16::from_be_bytes(bytes[0..=1].try_into()?)),
            ),
            (CommandClass::GetCommandResponse, ParameterId::Curve) => Ok(Self::GetCurve {
                curve_id: bytes[0],
                curve_count: bytes[1],
            }),
            (CommandClass::GetCommandResponse, ParameterId::CurveDescription) => {
                Ok(Self::GetCurveDescription {
                    curve_id: bytes[0],
                    description: decode_string_bytes(&bytes[1..])?,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::OutputResponseTime) => {
                Ok(Self::GetOutputResponseTime {
                    response_time_id: bytes[0],
                    response_time_count: bytes[1],
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::OutputResponseTimeDescription) => {
                Ok(Self::GetOutputResponseTimeDescription {
                    response_time_id: bytes[0],
                    description: decode_string_bytes(&bytes[1..])?,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::ModulationFrequency) => {
                Ok(Self::GetModulationFrequency {
                    modulation_frequency_id: bytes[0],
                    modulation_frequency_count: bytes[1],
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::ModulationFrequencyDescription) => {
                Ok(Self::GetModulationFrequencyDescription {
                    modulation_frequency_id: bytes[0],
                    frequency: u32::from_be_bytes(bytes[1..=4].try_into()?),
                    description: decode_string_bytes(&bytes[5..])?,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::PresetInfo) => {
                Ok(Self::GetPresetInfo {
                    level_field_supported: bytes[0] == 1,
                    preset_sequence_supported: bytes[1] == 1,
                    split_times_supported: bytes[2] == 1,
                    dmx_fail_infinite_delay_time_supported: bytes[3] == 1,
                    dmx_fail_infinite_hold_time_supported: bytes[4] == 1,
                    startup_infinite_hold_time_supported: bytes[5] == 1,
                    maximum_scene_number: u16::from_be_bytes(bytes[6..=7].try_into()?),
                    minimum_preset_fade_time_supported: u16::from_be_bytes(
                        bytes[8..=9].try_into()?,
                    ),
                    maximum_preset_fade_time_supported: u16::from_be_bytes(
                        bytes[10..=11].try_into()?,
                    ),
                    minimum_preset_wait_time_supported: u16::from_be_bytes(
                        bytes[12..=13].try_into()?,
                    ),
                    maximum_preset_wait_time_supported: u16::from_be_bytes(
                        bytes[14..=15].try_into()?,
                    ),
                    minimum_dmx_fail_delay_time_supported: SupportedTimes::from(
                        u16::from_be_bytes(bytes[16..=17].try_into()?),
                    ),
                    maximum_dmx_fail_delay_time_supported: SupportedTimes::from(
                        u16::from_be_bytes(bytes[18..=19].try_into()?),
                    ),
                    minimum_dmx_fail_hold_time_supported: SupportedTimes::from(u16::from_be_bytes(
                        bytes[20..=21].try_into()?,
                    )),
                    maximum_dmx_fail_hold_time_supported: SupportedTimes::from(u16::from_be_bytes(
                        bytes[22..=23].try_into()?,
                    )),
                    minimum_startup_delay_time_supported: SupportedTimes::from(u16::from_be_bytes(
                        bytes[24..=25].try_into()?,
                    )),
                    maximum_startup_delay_time_supported: SupportedTimes::from(u16::from_be_bytes(
                        bytes[26..=27].try_into()?,
                    )),
                    minimum_startup_hold_time_supported: SupportedTimes::from(u16::from_be_bytes(
                        bytes[28..=29].try_into()?,
                    )),
                    maximum_startup_hold_time_supported: SupportedTimes::from(u16::from_be_bytes(
                        bytes[30..=31].try_into()?,
                    )),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::PresetStatus) => {
                Ok(Self::GetPresetStatus {
                    scene_id: u16::from_be_bytes(bytes[0..=1].try_into()?),
                    up_fade_time: u16::from_be_bytes(bytes[2..=3].try_into()?),
                    down_fade_time: u16::from_be_bytes(bytes[4..=5].try_into()?),
                    wait_time: u16::from_be_bytes(bytes[6..=7].try_into()?),
                    programmed: PresetProgrammed::try_from(bytes[8])?,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::PresetMergeMode) => {
                Ok(Self::GetPresetMergeMode(MergeMode::try_from(bytes[0])?))
            }
            (_, ParameterId::ManufacturerSpecific(_)) => Ok(Self::ManufacturerSpecific(
                #[cfg(feature = "alloc")]
                bytes.to_vec(),
                #[cfg(not(feature = "alloc"))]
                Vec::<u8, 231>::from_slice(bytes).unwrap(),
            )),
            (_, _) => Ok(Self::Unsupported(
                #[cfg(feature = "alloc")]
                bytes.to_vec(),
                #[cfg(not(feature = "alloc"))]
                Vec::<u8, 231>::from_slice(bytes).unwrap(),
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RdmFrameResponse {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub response_type: ResponseType,
    pub message_count: u8,
    pub sub_device_id: SubDeviceId,
    pub command_class: CommandClass,
    pub parameter_id: ParameterId,
    pub parameter_data: ResponseData,
}

impl RdmFrameResponse {
    pub fn decode(bytes: &[u8]) -> Result<Self, RdmError> {
        let message_length = bytes[2];

        if message_length < 24 {
            return Err(RdmError::InvalidMessageLength(message_length));
        }

        let packet_checksum = u16::from_be_bytes(
            bytes[message_length as usize..=message_length as usize + 1].try_into()?,
        );

        let decoded_checksum = bsd_16_crc(&bytes[..message_length as usize]);

        if decoded_checksum != packet_checksum {
            return Err(RdmError::InvalidChecksum(decoded_checksum, packet_checksum));
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

        let sub_device_id = u16::from_be_bytes(bytes[18..=19].try_into()?).into();

        let command_class = CommandClass::try_from(bytes[20])?;

        let parameter_id = u16::from_be_bytes(bytes[21..=22].try_into()?).into();

        let parameter_data_length = bytes[23];

        if parameter_data_length > 231 {
            return Err(RdmError::InvalidParameterDataLength(parameter_data_length));
        }

        let parameter_data = match response_type {
            ResponseType::Ack | ResponseType::AckOverflow => {
                let parameter_data = if parameter_data_length > 0 {
                    Some(ResponseParameterData::decode(
                        command_class,
                        parameter_id,
                        &bytes[24..=(24 + parameter_data_length as usize - 1)],
                    )?)
                } else {
                    None
                };

                ResponseData::ParameterData(parameter_data)
            }
            ResponseType::AckTimer => {
                let estimated_response_time = u16::from_be_bytes(bytes[24..=25].try_into()?);

                ResponseData::EstimateResponseTime(estimated_response_time)
            }
            ResponseType::NackReason => {
                let nack_reason = u16::from_be_bytes(bytes[24..=25].try_into()?).try_into()?;

                ResponseData::NackReason(nack_reason)
            }
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

impl TryFrom<&[u8]> for RdmFrameResponse {
    type Error = RdmError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        RdmFrameResponse::decode(bytes)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DiscoveryUniqueBranchFrameResponse(pub DeviceUID);

impl DiscoveryUniqueBranchFrameResponse {
    pub fn decode(bytes: &[u8]) -> Result<Self, RdmError> {
        let Some(frame_start_index) = bytes.iter().position(|&x| x == 0xaa) else {
            return Err(RdmError::InvalidDiscoveryUniqueBranchPreamble);
        };

        let euid = &bytes[(frame_start_index + 1)..=(frame_start_index + 12)];

        let ecs = &bytes[(frame_start_index + 13)..=(frame_start_index + 16)];

        let decoded_checksum = bsd_16_crc(euid);

        let checksum = u16::from_be_bytes([ecs[0] & ecs[1], ecs[2] & ecs[3]]);

        if checksum != decoded_checksum {
            return Err(RdmError::InvalidChecksum(decoded_checksum, checksum));
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

impl TryFrom<&[u8]> for DiscoveryUniqueBranchFrameResponse {
    type Error = RdmError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        DiscoveryUniqueBranchFrameResponse::decode(bytes)
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum RdmResponse {
    RdmFrame(RdmFrameResponse),
    DiscoveryUniqueBranchFrame(DiscoveryUniqueBranchFrameResponse),
}

impl RdmResponse {
    pub fn decode(bytes: &[u8]) -> Result<Self, RdmError> {
        if bytes[0] == RDM_START_CODE_BYTE && bytes[1] == RDM_SUB_START_CODE_BYTE {
            if bytes.len() < 25 {
                return Err(RdmError::InvalidFrameLength(bytes.len() as u8));
            }

            return RdmFrameResponse::decode(bytes).map(RdmResponse::RdmFrame);
        }

        if bytes[0] == DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE
            || bytes[0] == DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE
        {
            if bytes.len() < 17 {
                return Err(RdmError::InvalidFrameLength(bytes.len() as u8));
            }

            return DiscoveryUniqueBranchFrameResponse::decode(bytes)
                .map(RdmResponse::DiscoveryUniqueBranchFrame);
        }

        Err(RdmError::InvalidStartCode)
    }
}

impl TryFrom<&[u8]> for RdmResponse {
    type Error = RdmError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        RdmResponse::decode(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_valid_rdm_ack_response() {
        let decoded = RdmResponse::decode(&[
            0xcc, // Start Code
            0x01, // Sub Start Code
            25,   // Message Length
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // Destination UID
            0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // Source UID
            0x00, // Transaction Number
            0x00, // Response Type = Ack
            0x00, // Message Count
            0x00, 0x00, // Sub-Device ID = Root Device
            0x21, // Command Class = GetCommandResponse
            0x10, 0x00, // Parameter ID = Identify Device
            0x01, // PDL
            0x01, // Identifying = true
            0x01, 0x43, // Checksum
        ]);

        let expected = Ok(RdmResponse::RdmFrame(RdmFrameResponse {
            destination_uid: DeviceUID::new(0x0102, 0x03040506),
            source_uid: DeviceUID::new(0x0605, 0x04030201),
            transaction_number: 0x00,
            response_type: ResponseType::Ack,
            message_count: 0x00,
            sub_device_id: SubDeviceId::RootDevice,
            command_class: CommandClass::GetCommandResponse,
            parameter_id: ParameterId::IdentifyDevice,
            parameter_data: ResponseData::ParameterData(Some(
                ResponseParameterData::GetIdentifyDevice(true),
            )),
        }));

        assert_eq!(decoded, expected);
    }

    #[test]
    fn should_decode_valid_rdm_ack_manufacturer_specific_response() {
        let decoded = RdmResponse::decode(&[
            0xcc, // Start Code
            0x01, // Sub Start Code
            28,   // Message Length
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // Destination UID
            0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // Source UID
            0x00, // Transaction Number
            0x00, // Response Type = Ack
            0x00, // Message Count
            0x00, 0x00, // Sub-Device ID = Root Device
            0x31, // Command Class = SetCommandResponse
            0x80, 0x80, // Parameter ID = Identify Device
            0x04, // PDL
            0x04, 0x03, 0x02, 0x01, // Arbitrary manufacturer specific data
            0x02, 0x52, // Checksum
        ]);

        let expected = Ok(RdmResponse::RdmFrame(RdmFrameResponse {
            destination_uid: DeviceUID::new(0x0102, 0x03040506),
            source_uid: DeviceUID::new(0x0605, 0x04030201),
            transaction_number: 0x00,
            response_type: ResponseType::Ack,
            message_count: 0x00,
            sub_device_id: SubDeviceId::RootDevice,
            command_class: CommandClass::SetCommandResponse,
            parameter_id: ParameterId::ManufacturerSpecific(0x8080),
            #[cfg(feature = "alloc")]
            parameter_data: ResponseData::ParameterData(Some(
                ResponseParameterData::ManufacturerSpecific(vec![0x04, 0x03, 0x02, 0x01]),
            )),
            #[cfg(not(feature = "alloc"))]
            parameter_data: ResponseData::ParameterData(Some(
                ResponseParameterData::ManufacturerSpecific(
                    Vec::<u8, 231>::from_slice(&[0x04, 0x03, 0x02, 0x01]).unwrap(),
                ),
            )),
        }));

        assert_eq!(decoded, expected);
    }

    #[test]
    fn should_decode_valid_rdm_ack_timer_response() {
        let decoded = RdmResponse::decode(&[
            0xcc, // Start Code
            0x01, // Sub Start Code
            26,   // Message Length
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // Destination UID
            0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // Source UID
            0x00, // Transaction Number
            0x01, // Response Type = AckTimer
            0x00, // Message Count
            0x00, 0x00, // Sub-Device ID = Root Device
            0x21, // Command Class = GetCommandResponse
            0x10, 0x00, // Parameter ID = Identify Device
            0x02, // PDL
            0x00, 0x0a, // Estimated Response Time = 10x 100ms = 1 second
            0x01, 0x4f, // Checksum
        ]);

        let expected = Ok(RdmResponse::RdmFrame(RdmFrameResponse {
            destination_uid: DeviceUID::new(0x0102, 0x03040506),
            source_uid: DeviceUID::new(0x0605, 0x04030201),
            transaction_number: 0x00,
            response_type: ResponseType::AckTimer,
            message_count: 0x00,
            sub_device_id: SubDeviceId::RootDevice,
            command_class: CommandClass::GetCommandResponse,
            parameter_id: ParameterId::IdentifyDevice,
            parameter_data: ResponseData::EstimateResponseTime(0x0a),
        }));

        assert_eq!(decoded, expected);
    }

    #[test]
    fn should_decode_valid_rdm_nack_reason_response() {
        let decoded = RdmResponse::decode(&[
            0xcc, // Start Code
            0x01, // Sub Start Code
            26,   // Message Length
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // Destination UID
            0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // Source UID
            0x00, // Transaction Number
            0x02, // Response Type = Nack_Reason
            0x00, // Message Count
            0x00, 0x00, // Sub-Device ID = Root Device
            0x21, // Command Class = GetCommandResponse
            0x10, 0x00, // Parameter ID = Identify Device
            0x02, // PDL
            0x00, 0x01, // Nack Reason = FormatError
            0x01, 0x47, // Checksum
        ]);

        let expected = Ok(RdmResponse::RdmFrame(RdmFrameResponse {
            destination_uid: DeviceUID::new(0x0102, 0x03040506),
            source_uid: DeviceUID::new(0x0605, 0x04030201),
            transaction_number: 0x00,
            response_type: ResponseType::NackReason,
            message_count: 0x00,
            sub_device_id: SubDeviceId::RootDevice,
            command_class: CommandClass::GetCommandResponse,
            parameter_id: ParameterId::IdentifyDevice,
            parameter_data: ResponseData::NackReason(ResponseNackReasonCode::FormatError),
        }));

        assert_eq!(decoded, expected);
    }

    #[test]
    fn should_decode_valid_rdm_ack_overflow_response() {
        let decoded = RdmResponse::decode(&[
            0xcc, // Start Code
            0x01, // Sub Start Code
            25,   // Message Length
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // Destination UID
            0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // Source UID
            0x00, // Transaction Number
            0x03, // Response Type = Ack_Overflow
            0x00, // Message Count
            0x00, 0x00, // Sub-Device ID = Root Device
            0x21, // Command Class = GetCommandResponse
            0x10, 0x00, // Parameter ID = Identify Device
            0x01, // PDL
            0x01, // Identifying = true
            0x01, 0x46, // Checksum
        ]);

        let expected = Ok(RdmResponse::RdmFrame(RdmFrameResponse {
            destination_uid: DeviceUID::new(0x0102, 0x03040506),
            source_uid: DeviceUID::new(0x0605, 0x04030201),
            transaction_number: 0x00,
            response_type: ResponseType::AckOverflow,
            message_count: 0x00,
            sub_device_id: SubDeviceId::RootDevice,
            command_class: CommandClass::GetCommandResponse,
            parameter_id: ParameterId::IdentifyDevice,
            parameter_data: ResponseData::ParameterData(Some(
                ResponseParameterData::GetIdentifyDevice(true),
            )),
        }));

        assert_eq!(decoded, expected);
    }

    #[test]
    fn should_decode_valid_discovery_unique_branch_response() {
        // includes preamble bytes
        let decoded = RdmResponse::decode(&[
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
        ]);

        let expected = Ok(RdmResponse::DiscoveryUniqueBranchFrame(
            DiscoveryUniqueBranchFrameResponse(DeviceUID::new(0x0102, 0x03040506)),
        ));

        assert_eq!(decoded, expected);

        // does not include preamble bytes
        let decoded = RdmResponse::decode(&[
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
        ]);

        let expected = Ok(RdmResponse::DiscoveryUniqueBranchFrame(
            DiscoveryUniqueBranchFrameResponse(DeviceUID::new(0x0102, 0x03040506)),
        ));

        assert_eq!(decoded, expected);
    }
}
