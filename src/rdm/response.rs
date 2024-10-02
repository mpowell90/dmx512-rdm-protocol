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
        decode_string_bytes, DefaultSlotValue, DhcpMode, DisplayInvertMode, Ipv4Address, Ipv4Route,
        LampOnMode, LampState, MergeMode, NetworkInterface, ParameterDescription, ParameterId,
        PinCode, PowerState, PresetPlaybackMode, PresetProgrammed, ProductCategory, ProductDetail,
        SelfTest, SensorDefinition, SensorValue, SlotInfo, StatusMessage, StatusType,
        SupportedTimes, TimeMode,
    },
    CommandClass, DeviceUID, EncodedFrame, EncodedParameterData, RdmError, SubDeviceId,
    DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE, DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE,
    RDM_START_CODE_BYTE, RDM_SUB_START_CODE_BYTE,
};
use core::{fmt::Display, iter, result::Result};
use macaddr::MacAddr6;

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
    ActionNotSupported = 0x000b,
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
            0x000b => Ok(Self::ActionNotSupported),
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
            Self::ActionNotSupported => "The parameter data is valid but the SET operation cannot be performed with the current configuration.",
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

impl ResponseData {
    pub fn encode(&self) -> EncodedParameterData {
        #[cfg(feature = "alloc")]
        let mut buf = Vec::new();

        #[cfg(not(feature = "alloc"))]
        let mut buf = Vec::new();

        match self {
            Self::ParameterData(Some(data)) => {
                let data = data.encode();

                #[cfg(feature = "alloc")]
                buf.reserve(data.len());

                buf.extend(data);
            }
            Self::ParameterData(None) => {}
            Self::EstimateResponseTime(time) => {
                #[cfg(feature = "alloc")]
                buf.reserve(2);

                buf.extend(time.to_be_bytes());
            }
            Self::NackReason(reason) => {
                #[cfg(feature = "alloc")]
                buf.reserve(2);

                buf.extend((*reason as u16).to_be_bytes());
            }
        }

        buf
    }

    pub fn decode(
        response_type: ResponseType,
        command_class: CommandClass,
        parameter_data_length: u8,
        parameter_id: ParameterId,
        bytes: &[u8],
    ) -> Result<Self, RdmError> {
        match response_type {
            ResponseType::Ack | ResponseType::AckOverflow => {
                let parameter_data = if parameter_data_length > 0 {
                    Some(ResponseParameterData::decode(
                        command_class,
                        parameter_id,
                        bytes,
                    )?)
                } else {
                    None
                };

                Ok(ResponseData::ParameterData(parameter_data))
            }
            ResponseType::AckTimer => {
                let estimated_response_time = u16::from_be_bytes(bytes[0..=1].try_into()?);

                Ok(ResponseData::EstimateResponseTime(estimated_response_time))
            }
            ResponseType::NackReason => {
                let nack_reason = u16::from_be_bytes(bytes[0..=1].try_into()?).try_into()?;

                Ok(ResponseData::NackReason(nack_reason))
            }
        }
    }
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
    // E1.37-2
    GetListInterfaces(
        #[cfg(feature = "alloc")] Vec<NetworkInterface>,
        #[cfg(not(feature = "alloc"))] Vec<NetworkInterface, 38>,
    ),
    GetInterfaceLabel {
        interface_id: u32,
        #[cfg(feature = "alloc")]
        interface_label: String,
        #[cfg(not(feature = "alloc"))]
        interface_label: String<32>,
    },
    GetInterfaceHardwareAddressType1 {
        interface_id: u32,
        hardware_address: MacAddr6,
    },
    GetIpV4DhcpMode {
        interface_id: u32,
        dhcp_mode: bool,
    },
    GetIpV4ZeroConfMode {
        interface_id: u32,
        zero_conf_mode: bool,
    },
    GetIpV4CurrentAddress {
        interface_id: u32,
        address: Ipv4Address,
        netmask: u8,
        dhcp_status: DhcpMode,
    },
    GetIpV4StaticAddress {
        interface_id: u32,
        address: Ipv4Address,
        netmask: u8,
    },
    GetIpV4DefaultRoute {
        interface_id: u32,
        address: Ipv4Route,
    },
    GetDnsIpV4NameServer {
        name_server_index: u8,
        address: Ipv4Address,
    },
    GetDnsHostName(
        #[cfg(feature = "alloc")] String,
        #[cfg(not(feature = "alloc"))] String<63>,
    ),
    GetDnsDomainName(
        #[cfg(feature = "alloc")] String,
        #[cfg(not(feature = "alloc"))] String<32>,
    ),
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
    pub fn encode(&self) -> EncodedParameterData {
        #[cfg(feature = "alloc")]
        let mut buf = Vec::new();

        #[cfg(not(feature = "alloc"))]
        let mut buf = Vec::new();

        match self {
            Self::DiscMute {
                control_field,
                binding_uid,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x0e);

                buf.extend(control_field.to_be_bytes());

                if let Some(binding_uid) = binding_uid {
                    buf.extend(binding_uid.manufacturer_id.to_be_bytes());
                    buf.extend(binding_uid.device_id.to_be_bytes());
                }
            }
            Self::DiscUnMute {
                control_field,
                binding_uid,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x0e);

                buf.extend(control_field.to_be_bytes());

                if let Some(binding_uid) = binding_uid {
                    buf.extend(binding_uid.manufacturer_id.to_be_bytes());
                    buf.extend(binding_uid.device_id.to_be_bytes());
                }
            }
            Self::GetProxiedDeviceCount {
                device_count,
                list_change,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x03);

                buf.extend(device_count.to_be_bytes());

                #[cfg(feature = "alloc")]
                buf.push(*list_change as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*list_change as u8).unwrap();
            }
            Self::GetProxiedDevices(devices) => {
                #[cfg(feature = "alloc")]
                buf.reserve(devices.len() * 6);

                for device in devices {
                    buf.extend(device.manufacturer_id.to_be_bytes());
                    buf.extend(device.device_id.to_be_bytes());
                }
            }
            Self::GetCommsStatus {
                short_message,
                length_mismatch,
                checksum_fail,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(6);

                buf.extend(short_message.to_be_bytes());
                buf.extend(length_mismatch.to_be_bytes());
                buf.extend(checksum_fail.to_be_bytes());
            }
            Self::GetStatusMessages(messages) => {
                for message in messages {
                    buf.extend(u16::from(message.sub_device_id).to_be_bytes());

                    #[cfg(feature = "alloc")]
                    buf.push(message.status_type as u8);
                    #[cfg(not(feature = "alloc"))]
                    buf.push(message.status_type as u8).unwrap();

                    buf.extend(message.status_message_id.to_be_bytes());
                    buf.extend(message.data_value1.to_be_bytes());
                    buf.extend(message.data_value2.to_be_bytes());

                    if let Some(description) = &message.description {
                        buf.extend(description.bytes());
                    }
                }
            }
            Self::GetStatusIdDescription(description) => {
                #[cfg(feature = "alloc")]
                buf.reserve(description.len());

                buf.extend(description.bytes());
            }
            Self::GetSubDeviceIdStatusReportThreshold(status) => {
                #[cfg(feature = "alloc")]
                buf.reserve(1);

                #[cfg(feature = "alloc")]
                buf.push(*status as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*status as u8).unwrap();
            }
            Self::GetSupportedParameters(parameters) => {
                #[cfg(feature = "alloc")]
                buf.reserve(parameters.len() * 2);

                for parameter in parameters {
                    buf.extend(parameter.to_be_bytes());
                }
            }
            Self::GetParameterDescription(description) => {
                #[cfg(feature = "alloc")]
                buf.reserve(20 + description.description.len());

                buf.extend(description.parameter_id.to_be_bytes());

                #[cfg(feature = "alloc")]
                buf.push(description.parameter_data_length);
                #[cfg(not(feature = "alloc"))]
                buf.push(description.parameter_data_length).unwrap();

                #[cfg(feature = "alloc")]
                buf.push(description.data_type.into());
                #[cfg(not(feature = "alloc"))]
                buf.push(description.data_type.into()).unwrap();

                #[cfg(feature = "alloc")]
                buf.push(description.command_class as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(description.command_class as u8).unwrap();

                #[cfg(feature = "alloc")]
                buf.push(description.command_class as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(description.command_class as u8).unwrap();

                #[cfg(feature = "alloc")]
                buf.push(description.unit_type.into());
                #[cfg(not(feature = "alloc"))]
                buf.push(description.unit_type.into()).unwrap();

                #[cfg(feature = "alloc")]
                buf.push(description.prefix as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(description.prefix as u8).unwrap();

                buf.extend(description.raw_minimum_valid_value);
                buf.extend(description.raw_maximum_valid_value);
                buf.extend(description.raw_default_value);

                #[cfg(feature = "alloc")]
                buf.extend(description.description.bytes());
                #[cfg(not(feature = "alloc"))]
                buf.extend(description.description.bytes());
            }
            Self::GetDeviceInfo {
                protocol_version,
                model_id,
                product_category,
                software_version_id,
                footprint,
                current_personality,
                personality_count,
                start_address,
                sub_device_count,
                sensor_count,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(19 + protocol_version.len());

                buf.extend(protocol_version.bytes());

                buf.extend(model_id.to_be_bytes());
                buf.extend(u16::from(*product_category).to_be_bytes());
                buf.extend(software_version_id.to_be_bytes());
                buf.extend(footprint.to_be_bytes());

                #[cfg(feature = "alloc")]
                buf.push(*current_personality);
                #[cfg(not(feature = "alloc"))]
                buf.push(*current_personality).unwrap();

                #[cfg(feature = "alloc")]
                buf.push(*personality_count);
                #[cfg(not(feature = "alloc"))]
                buf.push(*personality_count).unwrap();

                buf.extend(start_address.to_be_bytes());
                buf.extend(sub_device_count.to_be_bytes());

                #[cfg(feature = "alloc")]
                buf.push(*sensor_count);
                #[cfg(not(feature = "alloc"))]
                buf.push(*sensor_count).unwrap();
            }
            Self::GetProductDetailIdList(details) => {
                #[cfg(feature = "alloc")]
                buf.reserve(details.len() * 2);

                for &detail in details {
                    buf.extend(u16::from(detail).to_be_bytes());
                }
            }
            Self::GetDeviceModelDescription(description) => {
                #[cfg(feature = "alloc")]
                buf.reserve(description.len());

                buf.extend(description.bytes());
            }
            Self::GetManufacturerLabel(label) => {
                #[cfg(feature = "alloc")]
                buf.reserve(label.len());

                buf.extend(label.bytes());
            }
            Self::GetDeviceLabel(label) => {
                #[cfg(feature = "alloc")]
                buf.reserve(label.len());

                buf.extend(label.bytes());
            }
            Self::GetFactoryDefaults(defaults) => {
                #[cfg(feature = "alloc")]
                buf.reserve(1);

                #[cfg(feature = "alloc")]
                buf.push(*defaults as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*defaults as u8).unwrap();
            }
            Self::GetLanguageCapabilities(languages) => {
                #[cfg(feature = "alloc")]
                buf.reserve(languages.len() * 2);

                for language in languages {
                    buf.extend(language.bytes());
                }
            }
            Self::GetLanguage(language) => {
                #[cfg(feature = "alloc")]
                buf.reserve(2);

                buf.extend(language.bytes());
            }
            Self::GetSoftwareVersionLabel(label) => {
                #[cfg(feature = "alloc")]
                buf.reserve(label.len());

                buf.extend(label.bytes());
            }
            Self::GetBootSoftwareVersionId(version_id) => {
                #[cfg(feature = "alloc")]
                buf.reserve(4);

                buf.extend(version_id.to_be_bytes());
            }
            Self::GetBootSoftwareVersionLabel(label) => {
                #[cfg(feature = "alloc")]
                buf.reserve(label.len());

                buf.extend(label.bytes());
            }
            Self::GetDmxPersonality {
                current_personality,
                personality_count,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(2);

                #[cfg(feature = "alloc")]
                buf.push(*current_personality);
                #[cfg(not(feature = "alloc"))]
                buf.push(*current_personality).unwrap();

                #[cfg(feature = "alloc")]
                buf.push(*personality_count);
                #[cfg(not(feature = "alloc"))]
                buf.push(*personality_count).unwrap();
            }
            Self::GetDmxPersonalityDescription {
                id,
                dmx_slots_required,
                description,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(3 + description.len());

                #[cfg(feature = "alloc")]
                buf.push(*id);
                #[cfg(not(feature = "alloc"))]
                buf.push(*id).unwrap();

                buf.extend(dmx_slots_required.to_be_bytes());
                buf.extend(description.bytes());
            }
            Self::GetDmxStartAddress(address) => {
                #[cfg(feature = "alloc")]
                buf.reserve(2);

                buf.extend(address.to_be_bytes());
            }
            Self::GetSlotInfo(slots) => {
                #[cfg(feature = "alloc")]
                buf.reserve(slots.len() * 5);

                for slot in slots {
                    buf.extend(slot.id.to_be_bytes());

                    #[cfg(feature = "alloc")]
                    buf.push(slot.r#type.into());
                    #[cfg(not(feature = "alloc"))]
                    buf.push(slot.r#type.into()).unwrap();

                    buf.extend(slot.label_id.to_be_bytes());
                }
            }
            Self::GetSlotDescription {
                slot_id,
                description,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(2 + description.len());

                buf.extend(slot_id.to_be_bytes());
                buf.extend(description.bytes());
            }
            Self::GetDefaultSlotValue(values) => {
                #[cfg(feature = "alloc")]
                buf.reserve(values.len() * 3);

                for slot in values {
                    buf.extend(slot.id.to_be_bytes());

                    #[cfg(feature = "alloc")]
                    buf.push(slot.value);
                    #[cfg(not(feature = "alloc"))]
                    buf.push(slot.value).unwrap();
                }
            }
            Self::GetSensorDefinition(definition) => {
                #[cfg(feature = "alloc")]
                buf.reserve(14 + definition.description.len());

                #[cfg(feature = "alloc")]
                buf.push(definition.kind.into());
                #[cfg(not(feature = "alloc"))]
                buf.push(definition.kind.into()).unwrap();

                #[cfg(feature = "alloc")]
                buf.push(definition.unit.into());
                #[cfg(not(feature = "alloc"))]
                buf.push(definition.unit.into()).unwrap();

                #[cfg(feature = "alloc")]
                buf.push(definition.prefix as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(definition.prefix as u8).unwrap();

                #[cfg(feature = "alloc")]
                buf.push(definition.prefix as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(definition.prefix as u8).unwrap();

                buf.extend(definition.range_minimum_value.to_be_bytes());
                buf.extend(definition.range_maximum_value.to_be_bytes());
                buf.extend(definition.normal_minimum_value.to_be_bytes());
                buf.extend(definition.normal_maximum_value.to_be_bytes());

                #[cfg(feature = "alloc")]
                buf.push(definition.is_lowest_highest_detected_value_supported as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(definition.is_lowest_highest_detected_value_supported as u8)
                    .unwrap();

                #[cfg(feature = "alloc")]
                buf.push(definition.is_recorded_value_supported as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(definition.is_recorded_value_supported as u8)
                    .unwrap();

                buf.extend(definition.description.bytes());
            }
            Self::GetSensorValue(sensor_value) => {
                #[cfg(feature = "alloc")]
                buf.reserve(9);

                #[cfg(feature = "alloc")]
                buf.push(sensor_value.sensor_id);
                #[cfg(not(feature = "alloc"))]
                buf.push(sensor_value.sensor_id).unwrap();

                buf.extend(sensor_value.current_value.to_be_bytes());
                buf.extend(sensor_value.lowest_detected_value.to_be_bytes());
                buf.extend(sensor_value.highest_detected_value.to_be_bytes());
                buf.extend(sensor_value.recorded_value.to_be_bytes());
            }
            Self::SetSensorValue(sensor_value) => {
                #[cfg(feature = "alloc")]
                buf.reserve(9);

                #[cfg(feature = "alloc")]
                buf.push(sensor_value.sensor_id);
                #[cfg(not(feature = "alloc"))]
                buf.push(sensor_value.sensor_id).unwrap();

                buf.extend(sensor_value.current_value.to_be_bytes());
                buf.extend(sensor_value.lowest_detected_value.to_be_bytes());
                buf.extend(sensor_value.highest_detected_value.to_be_bytes());
                buf.extend(sensor_value.recorded_value.to_be_bytes());
            }
            Self::GetDeviceHours(hours) => {
                #[cfg(feature = "alloc")]
                buf.reserve(4);

                buf.extend(hours.to_be_bytes());
            }
            Self::GetLampHours(hours) => {
                #[cfg(feature = "alloc")]
                buf.reserve(4);

                buf.extend(hours.to_be_bytes());
            }
            Self::GetLampStrikes(strikes) => {
                #[cfg(feature = "alloc")]
                buf.reserve(4);

                buf.extend(strikes.to_be_bytes());
            }
            Self::GetLampState(state) => {
                #[cfg(feature = "alloc")]
                buf.reserve(1);

                #[cfg(feature = "alloc")]
                buf.push((*state).into());
                #[cfg(not(feature = "alloc"))]
                buf.push((*state).into()).unwrap();
            }
            Self::GetLampOnMode(mode) => {
                #[cfg(feature = "alloc")]
                buf.reserve(1);

                #[cfg(feature = "alloc")]
                buf.push((*mode).into());
                #[cfg(not(feature = "alloc"))]
                buf.push((*mode).into()).unwrap();
            }
            Self::GetDevicePowerCycles(cycles) => {
                #[cfg(feature = "alloc")]
                buf.reserve(4);

                buf.extend(cycles.to_be_bytes());
            }
            Self::GetDisplayInvert(mode) => {
                #[cfg(feature = "alloc")]
                buf.reserve(1);

                #[cfg(feature = "alloc")]
                buf.push(*mode as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*mode as u8).unwrap();
            }
            Self::GetDisplayLevel(level) => {
                #[cfg(feature = "alloc")]
                buf.reserve(1);

                #[cfg(feature = "alloc")]
                buf.push(*level);
                #[cfg(not(feature = "alloc"))]
                buf.push(*level).unwrap();
            }
            Self::GetPanInvert(invert) => {
                #[cfg(feature = "alloc")]
                buf.reserve(1);

                #[cfg(feature = "alloc")]
                buf.push(*invert as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*invert as u8).unwrap();
            }
            Self::GetTiltInvert(invert) => {
                #[cfg(feature = "alloc")]
                buf.reserve(1);

                #[cfg(feature = "alloc")]
                buf.push(*invert as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*invert as u8).unwrap();
            }
            Self::GetPanTiltSwap(swap) => {
                #[cfg(feature = "alloc")]
                buf.reserve(1);

                #[cfg(feature = "alloc")]
                buf.push(*swap as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*swap as u8).unwrap();
            }
            Self::GetRealTimeClock {
                year,
                month,
                day,
                hour,
                minute,
                second,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x07);

                buf.extend((*year).to_be_bytes());

                #[cfg(feature = "alloc")]
                buf.push(*month);
                #[cfg(not(feature = "alloc"))]
                buf.push(*month).unwrap();

                #[cfg(feature = "alloc")]
                buf.push(*day);
                #[cfg(not(feature = "alloc"))]
                buf.push(*day).unwrap();

                #[cfg(feature = "alloc")]
                buf.push(*hour);
                #[cfg(not(feature = "alloc"))]
                buf.push(*hour).unwrap();

                #[cfg(feature = "alloc")]
                buf.push(*minute);
                #[cfg(not(feature = "alloc"))]
                buf.push(*minute).unwrap();

                #[cfg(feature = "alloc")]
                buf.push(*second);
                #[cfg(not(feature = "alloc"))]
                buf.push(*second).unwrap();
            }
            Self::GetIdentifyDevice(identifying) => {
                #[cfg(feature = "alloc")]
                buf.reserve(1);

                #[cfg(feature = "alloc")]
                buf.push(*identifying as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*identifying as u8).unwrap();
            }
            Self::GetPowerState(state) => {
                #[cfg(feature = "alloc")]
                buf.reserve(1);

                #[cfg(feature = "alloc")]
                buf.push(*state as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*state as u8).unwrap();
            }
            Self::GetPerformSelfTest(test) => {
                #[cfg(feature = "alloc")]
                buf.reserve(1);

                #[cfg(feature = "alloc")]
                buf.push(*test as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*test as u8).unwrap();
            }
            Self::GetSelfTestDescription {
                self_test_id,
                description,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(1 + description.len());

                #[cfg(feature = "alloc")]
                buf.push((*self_test_id).into());
                #[cfg(not(feature = "alloc"))]
                buf.push((*self_test_id).into()).unwrap();

                buf.extend(description.bytes());
            }
            Self::GetPresetPlayback { mode, level } => {
                #[cfg(feature = "alloc")]
                buf.reserve(3);

                buf.extend(u16::from(*mode).to_be_bytes());

                #[cfg(feature = "alloc")]
                buf.push(*level);
                #[cfg(not(feature = "alloc"))]
                buf.push(*level).unwrap();
            }
            Self::GetDmxBlockAddress {
                total_sub_device_footprint,
                base_dmx_address,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(4);

                buf.extend(total_sub_device_footprint.to_be_bytes());
                buf.extend(base_dmx_address.to_be_bytes());
            }
            Self::GetDmxFailMode {
                scene_id,
                loss_of_signal_delay,
                hold_time,
                level,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(7);

                buf.extend(u16::from(*scene_id).to_be_bytes());
                buf.extend(u16::from(*loss_of_signal_delay).to_be_bytes());
                buf.extend(u16::from(*hold_time).to_be_bytes());

                #[cfg(feature = "alloc")]
                buf.push(*level);
                #[cfg(not(feature = "alloc"))]
                buf.push(*level).unwrap();
            }
            Self::GetDmxStartupMode {
                scene_id,
                startup_delay,
                hold_time,
                level,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(7);

                buf.extend(u16::from(*scene_id).to_be_bytes());
                buf.extend(u16::from(*startup_delay).to_be_bytes());
                buf.extend(u16::from(*hold_time).to_be_bytes());

                #[cfg(feature = "alloc")]
                buf.push(*level);
                #[cfg(not(feature = "alloc"))]
                buf.push(*level).unwrap();
            }
            Self::GetPowerOnSelfTest(test) => {
                #[cfg(feature = "alloc")]
                buf.reserve(1);

                #[cfg(feature = "alloc")]
                buf.push(*test as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*test as u8).unwrap();
            }
            Self::GetLockState {
                lock_state_id,
                lock_state_count,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(2);

                #[cfg(feature = "alloc")]
                buf.push(*lock_state_id);
                #[cfg(not(feature = "alloc"))]
                buf.push(*lock_state_id).unwrap();

                #[cfg(feature = "alloc")]
                buf.push(*lock_state_count);
                #[cfg(not(feature = "alloc"))]
                buf.push(*lock_state_count).unwrap();
            }
            Self::GetLockStateDescription {
                lock_state_id,
                description,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(1 + description.len());

                #[cfg(feature = "alloc")]
                buf.push(*lock_state_id);
                #[cfg(not(feature = "alloc"))]
                buf.push(*lock_state_id).unwrap();

                buf.extend(description.bytes());
            }
            Self::GetLockPin(pin) => {
                #[cfg(feature = "alloc")]
                buf.reserve(4);

                buf.extend(pin.0.to_be_bytes());
            }
            Self::GetBurnIn(hours) => {
                #[cfg(feature = "alloc")]
                buf.reserve(1);

                #[cfg(feature = "alloc")]
                buf.push(*hours);
                #[cfg(not(feature = "alloc"))]
                buf.push(*hours).unwrap();
            }
            Self::GetDimmerInfo {
                minimum_level_lower_limit,
                minimum_level_upper_limit,
                maximum_level_lower_limit,
                maximum_level_upper_limit,
                number_of_supported_curves,
                levels_resolution,
                minimum_level_split_levels_supported,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(11);

                buf.extend(minimum_level_lower_limit.to_be_bytes());
                buf.extend(minimum_level_upper_limit.to_be_bytes());
                buf.extend(maximum_level_lower_limit.to_be_bytes());
                buf.extend(maximum_level_upper_limit.to_be_bytes());

                #[cfg(feature = "alloc")]
                buf.push(*number_of_supported_curves);
                #[cfg(not(feature = "alloc"))]
                buf.push(*number_of_supported_curves).unwrap();

                #[cfg(feature = "alloc")]
                buf.push(*levels_resolution);
                #[cfg(not(feature = "alloc"))]
                buf.push(*levels_resolution).unwrap();

                #[cfg(feature = "alloc")]
                buf.push(*minimum_level_split_levels_supported as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*minimum_level_split_levels_supported as u8)
                    .unwrap();
            }
            Self::GetMinimumLevel {
                minimum_level_increasing,
                minimum_level_decreasing,
                on_below_minimum,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(5);

                buf.extend(minimum_level_increasing.to_be_bytes());
                buf.extend(minimum_level_decreasing.to_be_bytes());

                #[cfg(feature = "alloc")]
                buf.push(*on_below_minimum as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*on_below_minimum as u8).unwrap();
            }
            Self::GetMaximumLevel(level) => {
                #[cfg(feature = "alloc")]
                buf.reserve(2);

                buf.extend(level.to_be_bytes());
            }
            Self::GetCurve {
                curve_id,
                curve_count,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(2);

                #[cfg(feature = "alloc")]
                buf.push(*curve_id);
                #[cfg(not(feature = "alloc"))]
                buf.push(*curve_id).unwrap();

                #[cfg(feature = "alloc")]
                buf.push(*curve_count);
                #[cfg(not(feature = "alloc"))]
                buf.push(*curve_count).unwrap();
            }
            Self::GetCurveDescription {
                curve_id,
                description,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(1 + description.len());

                #[cfg(feature = "alloc")]
                buf.push(*curve_id);
                #[cfg(not(feature = "alloc"))]
                buf.push(*curve_id).unwrap();

                buf.extend(description.bytes());
            }
            Self::GetOutputResponseTime {
                response_time_id,
                response_time_count,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(2);

                #[cfg(feature = "alloc")]
                buf.push(*response_time_id);
                #[cfg(not(feature = "alloc"))]
                buf.push(*response_time_id).unwrap();

                #[cfg(feature = "alloc")]
                buf.push(*response_time_count);
                #[cfg(not(feature = "alloc"))]
                buf.push(*response_time_count).unwrap();
            }
            Self::GetOutputResponseTimeDescription {
                response_time_id,
                description,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(1 + description.len());

                #[cfg(feature = "alloc")]
                buf.push(*response_time_id);
                #[cfg(not(feature = "alloc"))]
                buf.push(*response_time_id).unwrap();

                buf.extend(description.bytes());
            }
            Self::GetModulationFrequency {
                modulation_frequency_id,
                modulation_frequency_count,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(2);

                #[cfg(feature = "alloc")]
                buf.push(*modulation_frequency_id);
                #[cfg(not(feature = "alloc"))]
                buf.push(*modulation_frequency_id).unwrap();

                #[cfg(feature = "alloc")]
                buf.push(*modulation_frequency_count);
                #[cfg(not(feature = "alloc"))]
                buf.push(*modulation_frequency_count).unwrap();
            }
            Self::GetModulationFrequencyDescription {
                modulation_frequency_id,
                frequency,
                description,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(5 + description.len());

                #[cfg(feature = "alloc")]
                buf.push(*modulation_frequency_id);
                #[cfg(not(feature = "alloc"))]
                buf.push(*modulation_frequency_id).unwrap();

                buf.extend(frequency.to_be_bytes());
                buf.extend(description.bytes());
            }
            Self::GetPresetInfo {
                level_field_supported,
                preset_sequence_supported,
                split_times_supported,
                dmx_fail_infinite_delay_time_supported,
                dmx_fail_infinite_hold_time_supported,
                startup_infinite_hold_time_supported,
                maximum_scene_number,
                minimum_preset_fade_time_supported,
                maximum_preset_fade_time_supported,
                minimum_preset_wait_time_supported,
                maximum_preset_wait_time_supported,
                minimum_dmx_fail_delay_time_supported,
                maximum_dmx_fail_delay_time_supported,
                minimum_dmx_fail_hold_time_supported,
                maximum_dmx_fail_hold_time_supported,
                minimum_startup_delay_time_supported,
                maximum_startup_delay_time_supported,
                minimum_startup_hold_time_supported,
                maximum_startup_hold_time_supported,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(38);

                #[cfg(feature = "alloc")]
                buf.push(*level_field_supported as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*level_field_supported as u8).unwrap();

                #[cfg(feature = "alloc")]
                buf.push(*preset_sequence_supported as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*preset_sequence_supported as u8).unwrap();

                #[cfg(feature = "alloc")]
                buf.push(*split_times_supported as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*split_times_supported as u8).unwrap();

                #[cfg(feature = "alloc")]
                buf.push(*dmx_fail_infinite_delay_time_supported as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*dmx_fail_infinite_delay_time_supported as u8)
                    .unwrap();

                #[cfg(feature = "alloc")]
                buf.push(*dmx_fail_infinite_hold_time_supported as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*dmx_fail_infinite_hold_time_supported as u8)
                    .unwrap();

                #[cfg(feature = "alloc")]
                buf.push(*startup_infinite_hold_time_supported as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*startup_infinite_hold_time_supported as u8)
                    .unwrap();

                buf.extend(maximum_scene_number.to_be_bytes());
                buf.extend(minimum_preset_fade_time_supported.to_be_bytes());
                buf.extend(maximum_preset_fade_time_supported.to_be_bytes());
                buf.extend(minimum_preset_wait_time_supported.to_be_bytes());
                buf.extend(maximum_preset_wait_time_supported.to_be_bytes());

                buf.extend(u16::from(*minimum_dmx_fail_delay_time_supported).to_be_bytes());
                buf.extend(u16::from(*maximum_dmx_fail_delay_time_supported).to_be_bytes());
                buf.extend(u16::from(*minimum_dmx_fail_hold_time_supported).to_be_bytes());
                buf.extend(u16::from(*maximum_dmx_fail_hold_time_supported).to_be_bytes());
                buf.extend(u16::from(*minimum_startup_delay_time_supported).to_be_bytes());
                buf.extend(u16::from(*maximum_startup_delay_time_supported).to_be_bytes());
                buf.extend(u16::from(*minimum_startup_hold_time_supported).to_be_bytes());
                buf.extend(u16::from(*maximum_startup_hold_time_supported).to_be_bytes());
            }
            Self::GetPresetStatus {
                scene_id,
                up_fade_time,
                down_fade_time,
                wait_time,
                programmed,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(9);

                buf.extend(scene_id.to_be_bytes());
                buf.extend(up_fade_time.to_be_bytes());
                buf.extend(down_fade_time.to_be_bytes());
                buf.extend(wait_time.to_be_bytes());

                #[cfg(feature = "alloc")]
                buf.push(*programmed as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*programmed as u8).unwrap();
            }
            Self::GetPresetMergeMode(mode) => {
                #[cfg(feature = "alloc")]
                buf.reserve(1);

                #[cfg(feature = "alloc")]
                buf.push(*mode as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*mode as u8).unwrap();
            }
            Self::GetListInterfaces(interfaces) => {
                #[cfg(feature = "alloc")]
                buf.reserve(interfaces.len() * 6);

                for interface in interfaces {
                    buf.extend(interface.interface_id.to_be_bytes());
                    buf.extend(u16::from(interface.hardware_type).to_be_bytes());
                }
            }
            Self::GetInterfaceLabel {
                interface_id,
                interface_label,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(4 + interface_label.len());

                buf.extend(interface_id.to_be_bytes());
                buf.extend(interface_label.bytes());
            }
            Self::GetInterfaceHardwareAddressType1 {
                interface_id,
                hardware_address,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(10);

                buf.extend(interface_id.to_be_bytes());
                buf.extend(hardware_address.into_array());
            }
            Self::GetIpV4DhcpMode {
                interface_id,
                dhcp_mode,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(5);

                buf.extend(interface_id.to_be_bytes());

                #[cfg(feature = "alloc")]
                buf.push(*dhcp_mode as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*dhcp_mode as u8).unwrap();
            }
            Self::GetIpV4ZeroConfMode {
                interface_id,
                zero_conf_mode,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(5);

                buf.extend(interface_id.to_be_bytes());

                #[cfg(feature = "alloc")]
                buf.push(*zero_conf_mode as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*zero_conf_mode as u8).unwrap();
            }
            Self::GetIpV4CurrentAddress {
                interface_id,
                address,
                netmask,
                dhcp_status,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(13);

                buf.extend(interface_id.to_be_bytes());
                buf.extend::<[u8; 4]>((*address).into());

                #[cfg(feature = "alloc")]
                buf.push(*netmask);
                #[cfg(not(feature = "alloc"))]
                buf.push(*netmask).unwrap();

                #[cfg(feature = "alloc")]
                buf.push(*dhcp_status as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*dhcp_status as u8).unwrap();
            }
            Self::GetIpV4StaticAddress {
                interface_id,
                address,
                netmask,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(10);

                buf.extend(interface_id.to_be_bytes());
                buf.extend::<[u8; 4]>((*address).into());

                #[cfg(feature = "alloc")]
                buf.push(*netmask);
                #[cfg(not(feature = "alloc"))]
                buf.push(*netmask).unwrap();

            }
            Self::GetIpV4DefaultRoute {
                interface_id,
                address,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(6);

                buf.extend(interface_id.to_be_bytes());
                buf.extend::<[u8; 4]>((*address).into());
            }
            Self::GetDnsIpV4NameServer {
                name_server_index,
                address,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(5);

                buf.extend(name_server_index.to_be_bytes());
                buf.extend::<[u8; 4]>((*address).into());
            }
            Self::GetDnsHostName(host_name) => {
                #[cfg(feature = "alloc")]
                buf.reserve(host_name.len());

                buf.extend(host_name.bytes());
            },
            Self::GetDnsDomainName(domain_name) => {
                #[cfg(feature = "alloc")]
                buf.reserve(domain_name.len());

                buf.extend(domain_name.bytes());
            },
            Self::ManufacturerSpecific(data) => {
                #[cfg(feature = "alloc")]
                buf.reserve(data.len());

                #[cfg(feature = "alloc")]
                buf.extend(data);
                #[cfg(not(feature = "alloc"))]
                buf.extend_from_slice(data).unwrap();
            }
            Self::Unsupported(data) => {
                #[cfg(feature = "alloc")]
                buf.reserve(data.len());

                #[cfg(feature = "alloc")]
                buf.extend(data);
                #[cfg(not(feature = "alloc"))]
                buf.extend_from_slice(data).unwrap();
            }
        }

        buf
    }

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
            // E1.37-2
            (CommandClass::GetCommandResponse, ParameterId::ListInterfaces) => {
                Ok(Self::GetListInterfaces(
                    #[cfg(feature = "alloc")]
                    bytes
                        .chunks(6)
                        .map(|chunk| {
                            Ok(NetworkInterface {
                                interface_id: u32::from_be_bytes(chunk[0..=3].try_into()?),
                                hardware_type: u16::from_be_bytes(chunk[4..=5].try_into()?).into(),
                            })
                        })
                        .collect::<Result<Vec<NetworkInterface>, RdmError>>()?,
                    #[cfg(not(feature = "alloc"))]
                    bytes
                        .chunks(6)
                        .map(|chunk| {
                            Ok(NetworkInterface {
                                interface_id: u32::from_be_bytes(chunk[0..=3].try_into()?),
                                hardware_type: u16::from_be_bytes(chunk[4..=5].try_into()?).into(),
                            })
                        })
                        .collect::<Result<Vec<NetworkInterface, 38>, RdmError>>()?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::InterfaceLabel) => {
                Ok(Self::GetInterfaceLabel {
                    interface_id: u32::from_be_bytes(bytes[0..=3].try_into()?),
                    interface_label: decode_string_bytes(&bytes[4..])?,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::InterfaceHardwareAddressType1) => {
                Ok(Self::GetInterfaceHardwareAddressType1 {
                    interface_id: u32::from_be_bytes(bytes[0..=3].try_into()?),
                    hardware_address: TryInto::<[u8; 6]>::try_into(&bytes[4..=9])?.into(),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::IpV4DhcpMode) => {
                Ok(Self::GetIpV4DhcpMode {
                    interface_id: u32::from_be_bytes(bytes[0..=3].try_into()?),
                    dhcp_mode: bytes[4] == 1,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::IpV4ZeroConfMode) => {
                Ok(Self::GetIpV4ZeroConfMode {
                    interface_id: u32::from_be_bytes(bytes[0..=3].try_into()?),
                    zero_conf_mode: bytes[4] == 1,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::IpV4CurrentAddress) => {
                Ok(Self::GetIpV4CurrentAddress {
                    interface_id: u32::from_be_bytes(bytes[0..=3].try_into()?),
                    address: TryInto::<[u8; 4]>::try_into(&bytes[4..=7])?.into(),
                    netmask: bytes[8],
                    dhcp_status: DhcpMode::try_from(bytes[9])?,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::IpV4StaticAddress) => {
                Ok(Self::GetIpV4StaticAddress {
                    interface_id: u32::from_be_bytes(bytes[0..=3].try_into()?),
                    address: TryInto::<[u8; 4]>::try_into(&bytes[4..=7])?.into(),
                    netmask: bytes[8],
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::IpV4DefaultRoute) => {
                Ok(Self::GetIpV4DefaultRoute {
                    interface_id: u32::from_be_bytes(bytes[0..=3].try_into()?),
                    address: TryInto::<[u8; 4]>::try_into(&bytes[4..=7])?.into(),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::DnsIpV4NameServer) => {
                Ok(Self::GetDnsIpV4NameServer {
                    name_server_index: bytes[0],
                    address: TryInto::<[u8; 4]>::try_into(&bytes[1..=4])?.into(),
                })
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
    pub fn encode(&self) -> EncodedFrame {
        let parameter_data = self.parameter_data.encode();

        let message_length = 24 + parameter_data.len();

        #[cfg(feature = "alloc")]
        let mut buf = Vec::with_capacity(message_length + 2);
        #[cfg(not(feature = "alloc"))]
        let mut buf = Vec::new();

        #[cfg(feature = "alloc")]
        buf.push(RDM_START_CODE_BYTE);
        #[cfg(not(feature = "alloc"))]
        buf.push(RDM_START_CODE_BYTE).unwrap();

        #[cfg(feature = "alloc")]
        buf.push(RDM_SUB_START_CODE_BYTE);
        #[cfg(not(feature = "alloc"))]
        buf.push(RDM_SUB_START_CODE_BYTE).unwrap();

        #[cfg(feature = "alloc")]
        buf.push(message_length as u8);
        #[cfg(not(feature = "alloc"))]
        buf.push(message_length as u8).unwrap();

        buf.extend(self.destination_uid.manufacturer_id.to_be_bytes());
        buf.extend(self.destination_uid.device_id.to_be_bytes());
        buf.extend(self.source_uid.manufacturer_id.to_be_bytes());
        buf.extend(self.source_uid.device_id.to_be_bytes());

        #[cfg(feature = "alloc")]
        buf.push(self.transaction_number);
        #[cfg(not(feature = "alloc"))]
        buf.push(self.transaction_number).unwrap();

        #[cfg(feature = "alloc")]
        buf.push(self.response_type as u8);
        #[cfg(not(feature = "alloc"))]
        buf.push(self.response_type as u8).unwrap();

        // Message Count shall be set to 0x00 in all controller generated requests
        #[cfg(feature = "alloc")]
        buf.push(self.message_count);
        #[cfg(not(feature = "alloc"))]
        buf.push(self.message_count).unwrap();

        buf.extend(u16::from(self.sub_device_id).to_be_bytes());

        #[cfg(feature = "alloc")]
        buf.push(self.command_class as u8);
        #[cfg(not(feature = "alloc"))]
        buf.push(self.command_class as u8).unwrap();

        buf.extend(u16::from(self.parameter_id).to_be_bytes());

        #[cfg(feature = "alloc")]
        buf.push(parameter_data.len() as u8);
        #[cfg(not(feature = "alloc"))]
        buf.push(parameter_data.len() as u8).unwrap();

        buf.extend(parameter_data);
        buf.extend(bsd_16_crc(&buf[..]).to_be_bytes());

        buf
    }

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

        let parameter_data = ResponseData::decode(
            response_type,
            command_class,
            parameter_data_length,
            parameter_id,
            &bytes[24..=(24 + parameter_data_length as usize - 1)],
        )?;

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
    pub fn encode(&self) -> EncodedFrame {
        #[cfg(feature = "alloc")]
        let mut buf = Vec::with_capacity(24);
        #[cfg(not(feature = "alloc"))]
        let mut buf = Vec::new();

        buf.extend(iter::repeat(DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE).take(7));

        #[cfg(feature = "alloc")]
        buf.push(DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE);
        #[cfg(not(feature = "alloc"))]
        buf.push(DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE)
            .unwrap();

        let [manufacturer_id1, manufacturer_id0] = self.0.manufacturer_id.to_be_bytes();

        buf.extend([
            manufacturer_id1 | 0xaa,
            manufacturer_id1 | 0x55,
            manufacturer_id0 | 0xaa,
            manufacturer_id0 | 0x55,
        ]);

        let [device_id3, device_id2, device_id1, device_id0] = self.0.device_id.to_be_bytes();

        buf.extend([
            device_id3 | 0xaa,
            device_id3 | 0x55,
            device_id2 | 0xaa,
            device_id2 | 0x55,
            device_id1 | 0xaa,
            device_id1 | 0x55,
            device_id0 | 0xaa,
            device_id0 | 0x55,
        ]);

        let [checksum1, checksum0] = bsd_16_crc(&buf[8..]).to_be_bytes();

        buf.extend([
            checksum1 | 0xaa,
            checksum1 | 0x55,
            checksum0 | 0xaa,
            checksum0 | 0x55,
        ]);

        buf
    }

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
    pub fn encode(&self) -> EncodedFrame {
        match self {
            RdmResponse::RdmFrame(frame) => frame.encode(),
            RdmResponse::DiscoveryUniqueBranchFrame(frame) => frame.encode(),
        }
    }

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
    fn should_encode_valid_rdm_ack_response() {
        let encoded = RdmResponse::RdmFrame(RdmFrameResponse {
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
        }).encode();

        let expected = &[
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
        ];

        assert_eq!(encoded, expected);
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
    fn should_encode_valid_rdm_ack_manufacturer_specific_response() {
        let encoded = RdmResponse::RdmFrame(RdmFrameResponse {
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
        }).encode();

        let expected = &[
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
        ];

        assert_eq!(encoded, expected);
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
    fn should_encode_valid_rdm_ack_timer_response() {
        let encoded = RdmResponse::RdmFrame(RdmFrameResponse {
            destination_uid: DeviceUID::new(0x0102, 0x03040506),
            source_uid: DeviceUID::new(0x0605, 0x04030201),
            transaction_number: 0x00,
            response_type: ResponseType::AckTimer,
            message_count: 0x00,
            sub_device_id: SubDeviceId::RootDevice,
            command_class: CommandClass::GetCommandResponse,
            parameter_id: ParameterId::IdentifyDevice,
            parameter_data: ResponseData::EstimateResponseTime(0x0a),
        }).encode();

        let expected = &[
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
        ];

        assert_eq!(encoded, expected);
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
    fn should_encode_valid_rdm_nack_reason_response() {
        let encoded = RdmResponse::RdmFrame(RdmFrameResponse {
            destination_uid: DeviceUID::new(0x0102, 0x03040506),
            source_uid: DeviceUID::new(0x0605, 0x04030201),
            transaction_number: 0x00,
            response_type: ResponseType::NackReason,
            message_count: 0x00,
            sub_device_id: SubDeviceId::RootDevice,
            command_class: CommandClass::GetCommandResponse,
            parameter_id: ParameterId::IdentifyDevice,
            parameter_data: ResponseData::NackReason(ResponseNackReasonCode::FormatError),
        }).encode();

        let expected = &[
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
        ];

        assert_eq!(encoded, expected);
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
    fn should_encode_valid_rdm_ack_overflow_response() {
        let encoded = RdmResponse::RdmFrame(RdmFrameResponse {
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
        }).encode();

        let expected = &[
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
        ];

        assert_eq!(encoded, expected);
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

    #[test]
    fn should_encode_valid_discovery_unique_branch_response() {
        let encoded = RdmResponse::DiscoveryUniqueBranchFrame(
            DiscoveryUniqueBranchFrameResponse(DeviceUID::new(0x0102, 0x03040506)),
        ).encode();

        let expected = &[
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

        assert_eq!(encoded, expected);
    }
}
