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
    parameter::{
        e120::{
            DefaultSlotValue, DeviceLabel, DisplayInvertMode, LampOnMode, LampState,
            ParameterDescription, ParameterDescriptionLabel, PowerState, PresetPlaybackMode,
            ProductCategory, ProductDetail, ProtocolVersion, SelfTest, SensorDefinition,
            SensorValue, SlotInfo, StatusMessage, StatusType,
        },
        e133::{BrokerState, ScopeString, SearchDomain, StaticConfigType},
        e137_1::{MergeMode, PinCode, PresetProgrammed, SupportedTimes, TimeMode},
        e137_2::{DhcpMode, DnsHostName, Ipv4Address, Ipv4Route, Ipv6Address, NetworkInterface},
        e137_7::{DiscoveryCountStatus, DiscoveryState, EndpointId, EndpointMode, EndpointType},
        ParameterId,
    },
    utils::{bsd_16_crc, decode_string_bytes},
    CommandClass, DeviceUID, RdmError, SubDeviceId, DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
    DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE, RDM_START_CODE_BYTE, RDM_SUB_START_CODE_BYTE,
};
use core::{fmt::Display, result::Result};
use macaddr::MacAddr6;

#[cfg(not(feature = "alloc"))]
use heapless::{String, Vec};

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u16)]
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
    EndpointNumberInvalid = 0x000c,
    InvalidEndpointMode = 0x000d,
    UnknownUid = 0x000e,
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
            0x000c => Ok(Self::EndpointNumberInvalid),
            0x000d => Ok(Self::InvalidEndpointMode),
            0x000e => Ok(Self::UnknownUid),
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
            Self::EndpointNumberInvalid => "The Endpoint Number is invalid.",
            Self::InvalidEndpointMode => "The Endpoint Mode is invalid.",
            Self::UnknownUid => "The UID is not known to the responder.",
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
pub enum ResponseData<'a> {
    ParameterData(Option<ResponseParameterData<'a>>),
    EstimateResponseTime(u16),
    NackReason(ResponseNackReasonCode),
}

impl<'a> ResponseData<'a> {
    pub fn size(&self) -> usize {
        match self {
            Self::ParameterData(Some(data)) => data.size(),
            Self::ParameterData(None) => 0,
            Self::EstimateResponseTime(_) => 2,
            Self::NackReason(_) => 2,
        }
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, RdmError> {
        let bytes_encoded = match self {
            Self::ParameterData(Some(data)) => data.encode(buf)?,
            Self::ParameterData(None) => 0,
            Self::EstimateResponseTime(time) => {
                buf[0..2].copy_from_slice(&time.to_be_bytes());
                2
            }
            Self::NackReason(reason) => {
                buf[0..2].copy_from_slice(&(*reason as u16).to_be_bytes());
                2
            }
        };

        Ok(bytes_encoded)
    }

    pub fn decode(
        response_type: ResponseType,
        command_class: CommandClass,
        parameter_data_length: u8,
        parameter_id: ParameterId,
        bytes: &'a [u8],
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
pub enum ResponseParameterData<'a> {
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
    GetParameterDescription(ParameterDescription<'a>),
    GetDeviceInfo {
        protocol_version: ProtocolVersion,
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
    GetDeviceLabel(DeviceLabel<'a>),
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
    GetDnsHostName(DnsHostName<'a>),
    GetDnsDomainName(
        #[cfg(feature = "alloc")] String,
        #[cfg(not(feature = "alloc"))] String<32>,
    ),
    // E1.37-7
    GetEndpointList {
        list_change_number: u32,
        #[cfg(feature = "alloc")]
        endpoint_list: Vec<(EndpointId, EndpointType)>,
        #[cfg(not(feature = "alloc"))]
        endpoint_list: Vec<(EndpointId, EndpointType), 75>,
    },
    GetEndpointListChange {
        list_change_number: u32,
    },
    GetIdentifyEndpoint {
        endpoint_id: EndpointId,
        identify: bool,
    },
    SetIdentifyEndpoint {
        endpoint_id: EndpointId,
    },
    GetEndpointToUniverse {
        endpoint_id: EndpointId,
        universe: u16,
    },
    SetEndpointToUniverse {
        endpoint_id: EndpointId,
    },
    GetEndpointMode {
        endpoint_id: EndpointId,
        mode: EndpointMode,
    },
    SetEndpointMode {
        endpoint_id: EndpointId,
    },
    GetEndpointLabel {
        endpoint_id: EndpointId,
        #[cfg(feature = "alloc")]
        label: String,
        #[cfg(not(feature = "alloc"))]
        label: String<32>,
    },
    SetEndpointLabel {
        endpoint_id: EndpointId,
    },
    GetRdmTrafficEnable {
        endpoint_id: EndpointId,
        enable: bool,
    },
    SetRdmTrafficEnable {
        endpoint_id: EndpointId,
    },
    GetDiscoveryState {
        endpoint_id: EndpointId,
        device_count: DiscoveryCountStatus,
        discovery_state: DiscoveryState,
    },
    SetDiscoveryState {
        endpoint_id: EndpointId,
    },
    GetBackgroundDiscovery {
        endpoint_id: EndpointId,
        enabled: bool,
    },
    SetBackgroundDiscovery {
        endpoint_id: EndpointId,
    },
    GetEndpointTiming {
        endpoint_id: EndpointId,
        current_setting_id: u8,
        setting_count: u8,
    },
    SetEndpointTiming {
        endpoint_id: EndpointId,
    },
    GetEndpointTimingDescription {
        setting_id: u8,
        #[cfg(feature = "alloc")]
        description: String,
        #[cfg(not(feature = "alloc"))]
        description: String<32>,
    },
    GetEndpointResponders {
        endpoint_id: EndpointId,
        list_change_number: u32,
        #[cfg(feature = "alloc")]
        responders: Vec<DeviceUID>,
        #[cfg(not(feature = "alloc"))]
        responders: Vec<DeviceUID, 37>,
    },
    GetEndpointResponderListChange {
        endpoint_id: EndpointId,
        list_change_number: u32,
    },
    GetBindingControlFields {
        endpoint_id: EndpointId,
        uid: DeviceUID,
        control_field: u16,
        binding_uid: DeviceUID,
    },
    GetBackgroundQueuedStatusPolicy {
        current_policy_id: u8,
        policy_count: u8,
    },
    GetBackgroundQueuedStatusPolicyDescription {
        policy_id: u8,
        #[cfg(feature = "alloc")]
        description: String,
        #[cfg(not(feature = "alloc"))]
        description: String<32>,
    },
    // E1.33
    GetComponentScope {
        scope_slot: u16,
        scope_string: ScopeString<'a>,
        static_config_type: StaticConfigType,
        static_ipv4_address: Ipv4Address,
        static_ipv6_address: Ipv6Address,
        static_port: u16,
    },
    GetSearchDomain(SearchDomain<'a>),
    GetTcpCommsStatus {
        scope_string: ScopeString<'a>,
        broker_ipv4_address: Ipv4Address,
        broker_ipv6_address: Ipv6Address,
        broker_port: u16,
        unhealthy_tcp_events: u16,
    },
    GetBrokerStatus {
        is_allowing_set_commands: bool,
        broker_state: BrokerState,
    },
    RawParameter(&'a [u8]),
}

impl<'a> ResponseParameterData<'a> {
    pub fn size(&self) -> usize {
        match self {
            ResponseParameterData::DiscMute { binding_uid, .. }
            | ResponseParameterData::DiscUnMute { binding_uid, .. } => {
                if binding_uid.is_some() {
                    8
                } else {
                    2
                }
            }
            ResponseParameterData::GetProxiedDeviceCount { .. } => 3,
            ResponseParameterData::GetProxiedDevices(device_uids) => device_uids.len() * 6,
            ResponseParameterData::GetCommsStatus { .. } => 6,
            ResponseParameterData::GetStatusMessages(status_messages) => {
                let mut bytes_encoded = 0;

                // TODO use iterators instead of for loop
                for message in status_messages {
                    if let Some(description) = &message.description {
                        bytes_encoded += 9 + description.len();
                    } else {
                        bytes_encoded += 9_usize;
                    };
                }

                bytes_encoded
            }
            ResponseParameterData::GetStatusIdDescription(description) => description.len(),
            ResponseParameterData::GetSubDeviceIdStatusReportThreshold(_) => 1,
            ResponseParameterData::GetSupportedParameters(parameters) => parameters.len() * 2,
            ResponseParameterData::GetParameterDescription(description) => {
                20 + description.description.len()
            }
            ResponseParameterData::GetDeviceInfo { .. } => 19,
            ResponseParameterData::GetProductDetailIdList(details) => details.len() * 2,
            ResponseParameterData::GetDeviceModelDescription(description) => description.len(),
            ResponseParameterData::GetManufacturerLabel(label) => label.len(),
            ResponseParameterData::GetDeviceLabel(label) => label.len(),
            ResponseParameterData::GetFactoryDefaults(_) => 1,
            ResponseParameterData::GetLanguageCapabilities(languages) => languages.len() * 2,
            ResponseParameterData::GetLanguage(_) => 2,
            ResponseParameterData::GetSoftwareVersionLabel(label) => label.len(),
            ResponseParameterData::GetBootSoftwareVersionId(_) => 4,
            ResponseParameterData::GetBootSoftwareVersionLabel(label) => label.len(),
            ResponseParameterData::GetDmxPersonality { .. } => 2,
            ResponseParameterData::GetDmxPersonalityDescription { description, .. } => {
                3 + description.len()
            }
            ResponseParameterData::GetDmxStartAddress(_) => 2,
            ResponseParameterData::GetSlotInfo(slots) => slots.len() * 5,
            ResponseParameterData::GetSlotDescription { description, .. } => 2 + description.len(),
            ResponseParameterData::GetDefaultSlotValue(values) => values.len() * 3,
            ResponseParameterData::GetSensorDefinition(definition) => {
                14 + definition.description.len()
            }
            ResponseParameterData::GetSensorValue(_) => 9,
            ResponseParameterData::SetSensorValue(_) => 9,
            ResponseParameterData::GetDeviceHours(_) => 4,
            ResponseParameterData::GetLampHours(_) => 4,
            ResponseParameterData::GetLampStrikes(_) => 4,
            ResponseParameterData::GetLampState(_) => 1,
            ResponseParameterData::GetLampOnMode(_) => 1,
            ResponseParameterData::GetDevicePowerCycles(_) => 4,
            ResponseParameterData::GetDisplayInvert(_) => 1,
            ResponseParameterData::GetDisplayLevel(_) => 1,
            ResponseParameterData::GetPanInvert(_) => 1,
            ResponseParameterData::GetTiltInvert(_) => 1,
            ResponseParameterData::GetPanTiltSwap(_) => 1,
            ResponseParameterData::GetRealTimeClock { .. } => 7,
            ResponseParameterData::GetIdentifyDevice(_) => 1,
            ResponseParameterData::GetPowerState(_) => 1,
            ResponseParameterData::GetPerformSelfTest(_) => 1,
            ResponseParameterData::GetSelfTestDescription { description, .. } => {
                1 + description.len()
            }
            ResponseParameterData::GetPresetPlayback { .. } => 3,
            ResponseParameterData::GetDmxBlockAddress { .. } => 4,
            ResponseParameterData::GetDmxFailMode { .. } => 7,
            ResponseParameterData::GetDmxStartupMode { .. } => 7,
            ResponseParameterData::GetPowerOnSelfTest(_) => 1,
            ResponseParameterData::GetLockState { .. } => 2,
            ResponseParameterData::GetLockStateDescription { description, .. } => {
                1 + description.len()
            }
            ResponseParameterData::GetLockPin(_) => 2,
            ResponseParameterData::GetBurnIn(_) => 1,
            ResponseParameterData::GetDimmerInfo { .. } => 11,
            ResponseParameterData::GetMinimumLevel { .. } => 5,
            ResponseParameterData::GetMaximumLevel(_) => 2,
            ResponseParameterData::GetCurve { .. } => 2,
            ResponseParameterData::GetCurveDescription { description, .. } => 1 + description.len(),
            ResponseParameterData::GetOutputResponseTime { .. } => 2,
            ResponseParameterData::GetOutputResponseTimeDescription { description, .. } => {
                1 + description.len()
            }
            ResponseParameterData::GetModulationFrequency { .. } => 2,
            ResponseParameterData::GetModulationFrequencyDescription { description, .. } => {
                5 + description.len()
            }
            ResponseParameterData::GetPresetInfo { .. } => 32,
            ResponseParameterData::GetPresetStatus { .. } => 9,
            ResponseParameterData::GetPresetMergeMode(_) => 1,
            ResponseParameterData::GetListInterfaces(interfaces) => interfaces.len() * 6,
            ResponseParameterData::GetInterfaceLabel {
                interface_label, ..
            } => 4 + interface_label.len(),
            ResponseParameterData::GetInterfaceHardwareAddressType1 { .. } => 10,
            ResponseParameterData::GetIpV4DhcpMode { .. } => 5,
            ResponseParameterData::GetIpV4ZeroConfMode { .. } => 5,
            ResponseParameterData::GetIpV4CurrentAddress { .. } => 10,
            ResponseParameterData::GetIpV4StaticAddress { .. } => 9,
            ResponseParameterData::GetIpV4DefaultRoute { .. } => 8,
            ResponseParameterData::GetDnsIpV4NameServer { .. } => 5,
            ResponseParameterData::GetDnsHostName(host_name) => host_name.len(),
            ResponseParameterData::GetDnsDomainName(domain_name) => domain_name.len(),
            ResponseParameterData::GetEndpointList { endpoint_list, .. } => {
                4 + (endpoint_list.len() * 3)
            }
            ResponseParameterData::GetEndpointListChange { .. } => 4,
            ResponseParameterData::GetIdentifyEndpoint { .. } => 3,
            ResponseParameterData::SetIdentifyEndpoint { .. } => 2,
            ResponseParameterData::GetEndpointToUniverse { .. } => 4,
            ResponseParameterData::SetEndpointToUniverse { .. } => 2,
            ResponseParameterData::GetEndpointMode { .. } => 3,
            ResponseParameterData::SetEndpointMode { .. } => 2,
            ResponseParameterData::GetEndpointLabel { label, .. } => 2 + label.len(),
            ResponseParameterData::SetEndpointLabel { .. } => 2,
            ResponseParameterData::GetRdmTrafficEnable { .. } => 3,
            ResponseParameterData::SetRdmTrafficEnable { .. } => 2,
            ResponseParameterData::GetDiscoveryState { .. } => 5,
            ResponseParameterData::SetDiscoveryState { .. } => 2,
            ResponseParameterData::GetBackgroundDiscovery { .. } => 3,
            ResponseParameterData::SetBackgroundDiscovery { .. } => 2,
            ResponseParameterData::GetEndpointTiming { .. } => 4,
            ResponseParameterData::SetEndpointTiming { .. } => 2,
            ResponseParameterData::GetEndpointTimingDescription { description, .. } => {
                1 + description.len()
            }
            ResponseParameterData::GetEndpointResponders { responders, .. } => {
                6 + (responders.len() * 6)
            }
            ResponseParameterData::GetEndpointResponderListChange { .. } => 6,
            ResponseParameterData::GetBindingControlFields { .. } => 16,
            ResponseParameterData::GetBackgroundQueuedStatusPolicy { .. } => 2,
            ResponseParameterData::GetBackgroundQueuedStatusPolicyDescription {
                description,
                ..
            } => 2 + description.len(),
            ResponseParameterData::GetComponentScope { .. } => 25 + ScopeString::MAX_LENGTH,
            ResponseParameterData::GetSearchDomain(search_domain) => search_domain.len(),
            ResponseParameterData::GetTcpCommsStatus { .. } => 24 + ScopeString::MAX_LENGTH,
            ResponseParameterData::GetBrokerStatus { .. } => 2,
            ResponseParameterData::RawParameter(data) => data.len(),
        }
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, RdmError> {
        match self {
            Self::DiscMute {
                control_field,
                binding_uid,
            } => {
                buf[0..2].copy_from_slice(&control_field.to_be_bytes());

                if let Some(binding_uid) = binding_uid {
                    buf[2..8].copy_from_slice(&<[u8; 6]>::from(*binding_uid));
                }
            }
            Self::DiscUnMute {
                control_field,
                binding_uid,
            } => {
                buf[0..2].copy_from_slice(&control_field.to_be_bytes());

                if let Some(binding_uid) = binding_uid {
                    buf[2..8].copy_from_slice(&<[u8; 6]>::from(*binding_uid));
                }
            }
            Self::GetProxiedDeviceCount {
                device_count,
                list_change,
            } => {
                buf[0..2].copy_from_slice(&device_count.to_be_bytes());
                buf[2] = *list_change as u8;
            }
            Self::GetProxiedDevices(devices) => {
                for (idx, device) in devices.iter().enumerate() {
                    buf[idx * 6..(idx + 1) * 6].copy_from_slice(&<[u8; 6]>::from(*device));
                }
            }
            Self::GetCommsStatus {
                short_message,
                length_mismatch,
                checksum_fail,
            } => {
                buf[0..2].copy_from_slice(&short_message.to_be_bytes());
                buf[2..4].copy_from_slice(&length_mismatch.to_be_bytes());
                buf[4..6].copy_from_slice(&checksum_fail.to_be_bytes());
            }
            Self::GetStatusMessages(messages) => {
                let mut bytes_encoded = 0;

                for message in messages {
                    buf[bytes_encoded..bytes_encoded + 2]
                        .copy_from_slice(&u16::from(message.sub_device_id).to_be_bytes());

                    buf[bytes_encoded + 2] = message.status_message_id as u8;

                    buf[bytes_encoded + 3..bytes_encoded + 5]
                        .copy_from_slice(&message.status_message_id.to_be_bytes());
                    buf[bytes_encoded + 5..bytes_encoded + 7]
                        .copy_from_slice(&message.data_value1.to_be_bytes());
                    buf[bytes_encoded + 7..bytes_encoded + 9]
                        .copy_from_slice(&message.data_value2.to_be_bytes());

                    if let Some(description) = &message.description {
                        buf[bytes_encoded + 9..bytes_encoded + 9 + description.len()]
                            .copy_from_slice(description.as_bytes());
                        bytes_encoded += 9 + description.len();
                    } else {
                        bytes_encoded += 9_usize;
                    };
                }
            }
            Self::GetStatusIdDescription(description) => {
                buf[0..description.len()].copy_from_slice(description.as_bytes());
            }
            Self::GetSubDeviceIdStatusReportThreshold(status) => {
                buf[0] = *status as u8;
            }
            Self::GetSupportedParameters(parameters) => {
                for (idx, parameter) in parameters.iter().enumerate() {
                    buf[idx * 2..(idx + 2) * 2].copy_from_slice(&parameter.to_be_bytes());
                }
            }
            Self::GetParameterDescription(description) => {
                buf[0..2].copy_from_slice(&description.parameter_id.to_be_bytes());
                buf[2] = description.parameter_data_length;
                buf[3] = description.data_type.into();
                buf[4] = description.command_class as u8;
                buf[6] = description.unit_type.into();
                buf[7] = description.prefix as u8;
                buf[8..12].copy_from_slice(&description.raw_minimum_valid_value);
                buf[12..16].copy_from_slice(&description.raw_maximum_valid_value);
                buf[16..20].copy_from_slice(&description.raw_default_value);
                description
                    .description
                    .encode(&mut buf[20..20 + description.description.len()])?;
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
                buf[0..2].copy_from_slice(&u16::from(*protocol_version).to_be_bytes());
                buf[2..4].copy_from_slice(&model_id.to_be_bytes());
                buf[4..6].copy_from_slice(&u16::from(*product_category).to_be_bytes());
                buf[6..10].copy_from_slice(&software_version_id.to_be_bytes());
                buf[10..12].copy_from_slice(&footprint.to_be_bytes());
                buf[12] = *current_personality;
                buf[13] = *personality_count;
                buf[14..16].copy_from_slice(&start_address.to_be_bytes());
                buf[16..18].copy_from_slice(&sub_device_count.to_be_bytes());
                buf[19] = *sensor_count;
            }
            Self::GetProductDetailIdList(details) => {
                for (idx, detail) in details.iter().enumerate() {
                    buf[idx * 2..(idx + 2) * 2].copy_from_slice(&u16::from(*detail).to_be_bytes());
                }
            }
            Self::GetDeviceModelDescription(description) => {
                buf[0..description.len()].copy_from_slice(description.as_bytes());
            }
            Self::GetManufacturerLabel(label) => {
                buf[0..label.len()].copy_from_slice(label.as_bytes());
            }
            Self::GetDeviceLabel(device_label) => {
                device_label.encode(buf)?;
            }
            Self::GetFactoryDefaults(defaults) => {
                buf[0] = *defaults as u8;
            }
            Self::GetLanguageCapabilities(languages) => {
                for (idx, language) in languages.iter().enumerate() {
                    buf[idx * 2..(idx + 2) * 2].copy_from_slice(language.as_bytes());
                }
            }
            Self::GetLanguage(language) => {
                buf[0..2].copy_from_slice(language.as_bytes());
            }
            Self::GetSoftwareVersionLabel(label) => {
                buf[0..label.len()].copy_from_slice(label.as_bytes());
            }
            Self::GetBootSoftwareVersionId(version_id) => {
                buf[0..4].copy_from_slice(&version_id.to_be_bytes());
            }
            Self::GetBootSoftwareVersionLabel(label) => {
                buf[0..label.len()].copy_from_slice(label.as_bytes());
            }
            Self::GetDmxPersonality {
                current_personality,
                personality_count,
            } => {
                buf[0] = *current_personality;
                buf[1] = *personality_count;
            }
            Self::GetDmxPersonalityDescription {
                id,
                dmx_slots_required,
                description,
            } => {
                buf[0] = *id;
                buf[1..3].copy_from_slice(&dmx_slots_required.to_be_bytes());
                buf[3..3 + description.len()].copy_from_slice(description.as_bytes());
            }
            Self::GetDmxStartAddress(address) => {
                buf[0..2].copy_from_slice(&address.to_be_bytes());
            }
            Self::GetSlotInfo(slots) => {
                for (idx, slot) in slots.iter().enumerate() {
                    buf[idx * 5..(idx + 2) * 5].copy_from_slice(&slot.id.to_be_bytes());
                    buf[(idx + 2) * 5] = slot.r#type.into();
                    buf[(idx + 3) * 5..(idx + 5) * 5].copy_from_slice(&slot.label_id.to_be_bytes());
                }
            }
            Self::GetSlotDescription {
                slot_id,
                description,
            } => {
                buf[0..2].copy_from_slice(&slot_id.to_be_bytes());
                buf[2..2 + description.len()].copy_from_slice(description.as_bytes());
            }
            Self::GetDefaultSlotValue(values) => {
                for (idx, slot) in values.iter().enumerate() {
                    buf[idx * 3..(idx + 2) * 3].copy_from_slice(&slot.id.to_be_bytes());
                    buf[(idx + 2) * 3] = slot.value;
                }
            }
            Self::GetSensorDefinition(definition) => {
                buf[0] = definition.id;
                buf[1] = definition.kind.into();
                buf[2] = definition.unit.into();
                buf[3] = definition.prefix as u8;
                buf[4..6].copy_from_slice(&definition.range_minimum_value.to_be_bytes());
                buf[6..8].copy_from_slice(&definition.range_maximum_value.to_be_bytes());
                buf[8..10].copy_from_slice(&definition.normal_minimum_value.to_be_bytes());
                buf[10..12].copy_from_slice(&definition.normal_maximum_value.to_be_bytes());
                buf[12] = definition.is_lowest_highest_detected_value_supported as u8;
                buf[13] = definition.is_recorded_value_supported as u8;
                buf[14..14 + definition.description.len()]
                    .copy_from_slice(definition.description.as_bytes());
            }
            Self::GetSensorValue(sensor_value) => {
                buf[0] = sensor_value.sensor_id;
                buf[1..3].copy_from_slice(&sensor_value.current_value.to_be_bytes());
                buf[3..5].copy_from_slice(&sensor_value.lowest_detected_value.to_be_bytes());
                buf[5..7].copy_from_slice(&sensor_value.highest_detected_value.to_be_bytes());
                buf[7..9].copy_from_slice(&sensor_value.recorded_value.to_be_bytes());
            }
            Self::SetSensorValue(sensor_value) => {
                buf[0] = sensor_value.sensor_id;
                buf[1..3].copy_from_slice(&sensor_value.current_value.to_be_bytes());
                buf[3..5].copy_from_slice(&sensor_value.lowest_detected_value.to_be_bytes());
                buf[5..7].copy_from_slice(&sensor_value.highest_detected_value.to_be_bytes());
                buf[7..9].copy_from_slice(&sensor_value.recorded_value.to_be_bytes());
            }
            Self::GetDeviceHours(hours) => {
                buf[0..4].copy_from_slice(&hours.to_be_bytes());
            }
            Self::GetLampHours(hours) => {
                buf[0..4].copy_from_slice(&hours.to_be_bytes());
            }
            Self::GetLampStrikes(strikes) => {
                buf[0..4].copy_from_slice(&strikes.to_be_bytes());
            }
            Self::GetLampState(state) => {
                buf[0] = (*state).into();
            }
            Self::GetLampOnMode(mode) => {
                buf[0] = (*mode).into();
            }
            Self::GetDevicePowerCycles(cycles) => {
                buf[0..4].copy_from_slice(&cycles.to_be_bytes());
            }
            Self::GetDisplayInvert(mode) => {
                buf[0] = *mode as u8;
            }
            Self::GetDisplayLevel(level) => {
                buf[0] = *level;
            }
            Self::GetPanInvert(invert) => {
                buf[0] = *invert as u8;
            }
            Self::GetTiltInvert(invert) => {
                buf[0] = *invert as u8;
            }
            Self::GetPanTiltSwap(swap) => {
                buf[0] = *swap as u8;
            }
            Self::GetRealTimeClock {
                year,
                month,
                day,
                hour,
                minute,
                second,
            } => {
                buf[0..2].copy_from_slice(&year.to_be_bytes());
                buf[2] = *month;
                buf[3] = *day;
                buf[4] = *hour;
                buf[5] = *minute;
                buf[6] = *second;
            }
            Self::GetIdentifyDevice(identifying) => {
                buf[0] = *identifying as u8;
            }
            Self::GetPowerState(state) => {
                buf[0] = *state as u8;
            }
            Self::GetPerformSelfTest(test) => {
                buf[0] = *test as u8;
            }
            Self::GetSelfTestDescription {
                self_test_id,
                description,
            } => {
                buf[0] = (*self_test_id).into();
                buf[1..1 + description.len()].copy_from_slice(description.as_bytes());
            }
            Self::GetPresetPlayback { mode, level } => {
                buf[0..2].copy_from_slice(&u16::from(*mode).to_be_bytes());
                buf[0] = *level;
            }
            Self::GetDmxBlockAddress {
                total_sub_device_footprint,
                base_dmx_address,
            } => {
                buf[0..2].copy_from_slice(&total_sub_device_footprint.to_be_bytes());
                buf[2..4].copy_from_slice(&base_dmx_address.to_be_bytes());
            }
            Self::GetDmxFailMode {
                scene_id,
                loss_of_signal_delay,
                hold_time,
                level,
            } => {
                buf[0..2].copy_from_slice(&u16::from(*scene_id).to_be_bytes());
                buf[2..4].copy_from_slice(&u16::from(*loss_of_signal_delay).to_be_bytes());
                buf[4..6].copy_from_slice(&u16::from(*hold_time).to_be_bytes());
                buf[6] = *level;
            }
            Self::GetDmxStartupMode {
                scene_id,
                startup_delay,
                hold_time,
                level,
            } => {
                buf[0..2].copy_from_slice(&u16::from(*scene_id).to_be_bytes());
                buf[2..4].copy_from_slice(&u16::from(*startup_delay).to_be_bytes());
                buf[4..6].copy_from_slice(&u16::from(*hold_time).to_be_bytes());
                buf[6] = *level;
            }
            Self::GetPowerOnSelfTest(test) => {
                buf[0] = *test as u8;
            }
            Self::GetLockState {
                lock_state_id,
                lock_state_count,
            } => {
                buf[0] = *lock_state_id;
                buf[1] = *lock_state_count;
            }
            Self::GetLockStateDescription {
                lock_state_id,
                description,
            } => {
                buf[0] = *lock_state_id;
                buf[1..1 + description.len()].copy_from_slice(description.as_bytes());
            }
            Self::GetLockPin(pin) => {
                buf[0..2].copy_from_slice(&pin.0.to_be_bytes());
            }
            Self::GetBurnIn(hours) => {
                buf[0] = *hours;
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
                buf[0..2].copy_from_slice(&minimum_level_lower_limit.to_be_bytes());
                buf[2..4].copy_from_slice(&minimum_level_upper_limit.to_be_bytes());
                buf[4..6].copy_from_slice(&maximum_level_lower_limit.to_be_bytes());
                buf[6..8].copy_from_slice(&maximum_level_upper_limit.to_be_bytes());
                buf[8] = *number_of_supported_curves;
                buf[9] = *levels_resolution;
                buf[10] = *minimum_level_split_levels_supported as u8;
            }
            Self::GetMinimumLevel {
                minimum_level_increasing,
                minimum_level_decreasing,
                on_below_minimum,
            } => {
                buf[0..2].copy_from_slice(&minimum_level_increasing.to_be_bytes());
                buf[2..4].copy_from_slice(&minimum_level_decreasing.to_be_bytes());
                buf[4] = *on_below_minimum as u8;
            }
            Self::GetMaximumLevel(level) => {
                buf[0..2].copy_from_slice(&level.to_be_bytes());
            }
            Self::GetCurve {
                curve_id,
                curve_count,
            } => {
                buf[0] = *curve_id;
                buf[1] = *curve_count;
            }
            Self::GetCurveDescription {
                curve_id,
                description,
            } => {
                buf[0] = *curve_id;
                buf[1..1 + description.len()].copy_from_slice(description.as_bytes());
            }
            Self::GetOutputResponseTime {
                response_time_id,
                response_time_count,
            } => {
                buf[0] = *response_time_id;
                buf[1] = *response_time_count;
            }
            Self::GetOutputResponseTimeDescription {
                response_time_id,
                description,
            } => {
                buf[0] = *response_time_id;
                buf[1..1 + description.len()].copy_from_slice(description.as_bytes());
            }
            Self::GetModulationFrequency {
                modulation_frequency_id,
                modulation_frequency_count,
            } => {
                buf[0] = *modulation_frequency_id;
                buf[1] = *modulation_frequency_count;
            }
            Self::GetModulationFrequencyDescription {
                modulation_frequency_id,
                frequency,
                description,
            } => {
                buf[0] = *modulation_frequency_id;
                buf[1..5].copy_from_slice(&frequency.to_be_bytes());
                buf[5..5 + description.len()].copy_from_slice(description.as_bytes());
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
                buf[0] = *level_field_supported as u8;
                buf[1] = *preset_sequence_supported as u8;
                buf[2] = *split_times_supported as u8;
                buf[3] = *dmx_fail_infinite_delay_time_supported as u8;
                buf[4] = *dmx_fail_infinite_hold_time_supported as u8;
                buf[5] = *startup_infinite_hold_time_supported as u8;
                buf[6..8].copy_from_slice(&maximum_scene_number.to_be_bytes());
                buf[8..10].copy_from_slice(&minimum_preset_fade_time_supported.to_be_bytes());
                buf[10..12].copy_from_slice(&maximum_preset_fade_time_supported.to_be_bytes());
                buf[12..14].copy_from_slice(&minimum_preset_wait_time_supported.to_be_bytes());
                buf[14..16].copy_from_slice(&maximum_preset_wait_time_supported.to_be_bytes());
                buf[16..18].copy_from_slice(
                    &u16::from(*minimum_dmx_fail_delay_time_supported).to_be_bytes(),
                );
                buf[18..20].copy_from_slice(
                    &u16::from(*maximum_dmx_fail_delay_time_supported).to_be_bytes(),
                );
                buf[20..22].copy_from_slice(
                    &u16::from(*minimum_dmx_fail_hold_time_supported).to_be_bytes(),
                );
                buf[22..24].copy_from_slice(
                    &u16::from(*maximum_dmx_fail_hold_time_supported).to_be_bytes(),
                );
                buf[24..26].copy_from_slice(
                    &u16::from(*minimum_startup_delay_time_supported).to_be_bytes(),
                );
                buf[26..28].copy_from_slice(
                    &u16::from(*maximum_startup_delay_time_supported).to_be_bytes(),
                );
                buf[28..30].copy_from_slice(
                    &u16::from(*minimum_startup_hold_time_supported).to_be_bytes(),
                );
                buf[30..32].copy_from_slice(
                    &u16::from(*maximum_startup_hold_time_supported).to_be_bytes(),
                );
            }
            Self::GetPresetStatus {
                scene_id,
                up_fade_time,
                down_fade_time,
                wait_time,
                programmed,
            } => {
                buf[0..2].copy_from_slice(&scene_id.to_be_bytes());
                buf[2..4].copy_from_slice(&up_fade_time.to_be_bytes());
                buf[4..6].copy_from_slice(&down_fade_time.to_be_bytes());
                buf[6..8].copy_from_slice(&wait_time.to_be_bytes());
                buf[8] = *programmed as u8;
            }
            Self::GetPresetMergeMode(mode) => {
                buf[0] = *mode as u8;
            }
            Self::GetListInterfaces(interfaces) => {
                for (idx, interface) in interfaces.iter().enumerate() {
                    buf[idx * 6..(idx + 4) * 6]
                        .copy_from_slice(&interface.interface_id.to_be_bytes());
                    buf[(idx + 4) * 6..(idx + 6) * 6]
                        .copy_from_slice(&u16::from(interface.hardware_type).to_be_bytes());
                }
            }
            Self::GetInterfaceLabel {
                interface_id,
                interface_label,
            } => {
                buf[0..4].copy_from_slice(&interface_id.to_be_bytes());
                buf[4..4 + interface_label.len()].copy_from_slice(interface_label.as_bytes());
            }
            Self::GetInterfaceHardwareAddressType1 {
                interface_id,
                hardware_address,
            } => {
                buf[0..4].copy_from_slice(&interface_id.to_be_bytes());
                buf[4..10].copy_from_slice(&hardware_address.into_array());
            }
            Self::GetIpV4DhcpMode {
                interface_id,
                dhcp_mode,
            } => {
                buf[0..4].copy_from_slice(&interface_id.to_be_bytes());
                buf[4] = *dhcp_mode as u8;
            }
            Self::GetIpV4ZeroConfMode {
                interface_id,
                zero_conf_mode,
            } => {
                buf[0..4].copy_from_slice(&interface_id.to_be_bytes());
                buf[4] = *zero_conf_mode as u8;
            }
            Self::GetIpV4CurrentAddress {
                interface_id,
                address,
                netmask,
                dhcp_status,
            } => {
                buf[0..4].copy_from_slice(&interface_id.to_be_bytes());
                buf[4..8].copy_from_slice(&u32::from(*address).to_be_bytes());
                buf[8] = *netmask;
                buf[9] = *dhcp_status as u8;
            }
            Self::GetIpV4StaticAddress {
                interface_id,
                address,
                netmask,
            } => {
                buf[0..4].copy_from_slice(&interface_id.to_be_bytes());
                buf[4..8].copy_from_slice(&u32::from(*address).to_be_bytes());
                buf[8] = *netmask;
            }
            Self::GetIpV4DefaultRoute {
                interface_id,
                address,
            } => {
                buf[0..4].copy_from_slice(&interface_id.to_be_bytes());
                buf[4..8].copy_from_slice(&u32::from(*address).to_be_bytes());
            }
            Self::GetDnsIpV4NameServer {
                name_server_index,
                address,
            } => {
                buf[0] = *name_server_index;
                buf[1..5].copy_from_slice(&u32::from(*address).to_be_bytes());
            }
            Self::GetDnsHostName(dns_hostname) => {
                dns_hostname.encode(buf)?;
            }
            Self::GetDnsDomainName(domain_name) => {
                buf[0..domain_name.len()].copy_from_slice(domain_name.as_bytes());
            }
            // E1.37-7
            Self::GetEndpointList {
                list_change_number,
                endpoint_list,
            } => {
                buf[0..4].copy_from_slice(&list_change_number.to_be_bytes());

                for (idx, (endpoint_id, endpoint_type)) in endpoint_list.iter().enumerate() {
                    buf[(idx + 1) * 3..(idx + 3) * 3]
                        .copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
                    buf[(idx + 3) * 3] = *endpoint_type as u8;
                }
            }
            Self::GetEndpointListChange { list_change_number } => {
                buf[0..4].copy_from_slice(&list_change_number.to_be_bytes());
            }
            Self::GetIdentifyEndpoint {
                endpoint_id,
                identify,
            } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
                buf[2] = *identify as u8;
            }
            Self::SetIdentifyEndpoint { endpoint_id } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
            }
            Self::GetEndpointToUniverse {
                endpoint_id,
                universe,
            } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
                buf[2..4].copy_from_slice(&universe.to_be_bytes());
            }
            Self::SetEndpointToUniverse { endpoint_id } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
            }
            Self::GetEndpointMode { endpoint_id, mode } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
                buf[2] = *mode as u8;
            }
            Self::SetEndpointMode { endpoint_id } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
            }
            Self::GetEndpointLabel { endpoint_id, label } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
                buf[2..2 + label.len()].copy_from_slice(label.as_bytes());
            }
            Self::SetEndpointLabel { endpoint_id } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
            }
            Self::GetRdmTrafficEnable {
                endpoint_id,
                enable,
            } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
                buf[2] = *enable as u8;
            }
            Self::SetRdmTrafficEnable { endpoint_id } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
            }
            Self::GetDiscoveryState {
                endpoint_id,
                device_count,
                discovery_state,
            } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
                buf[2..4].copy_from_slice(&u16::from(*device_count).to_be_bytes());
                buf[4] = (*discovery_state).into();
            }
            Self::SetDiscoveryState { endpoint_id } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
            }
            Self::GetBackgroundDiscovery {
                endpoint_id,
                enabled,
            } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
                buf[2] = *enabled as u8;
            }
            Self::SetBackgroundDiscovery { endpoint_id } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
            }
            Self::GetEndpointTiming {
                endpoint_id,
                current_setting_id,
                setting_count,
            } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
                buf[2] = *current_setting_id;
                buf[3] = *setting_count;
            }
            Self::SetEndpointTiming { endpoint_id } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
            }
            Self::GetEndpointTimingDescription {
                setting_id,
                description,
            } => {
                buf[0] = *setting_id;
                buf[1..1 + description.len()].copy_from_slice(description.as_bytes());
            }
            Self::GetEndpointResponders {
                endpoint_id,
                list_change_number,
                responders,
            } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
                buf[2..6].copy_from_slice(&list_change_number.to_be_bytes());

                for (idx, responder) in responders.iter().enumerate() {
                    buf[(idx + 6) * 6..(idx + 12) * 6]
                        .copy_from_slice(&<[u8; 6]>::from(*responder));
                }
            }
            Self::GetEndpointResponderListChange {
                endpoint_id,
                list_change_number,
            } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
                buf[2..6].copy_from_slice(&list_change_number.to_be_bytes());
            }
            Self::GetBindingControlFields {
                endpoint_id,
                uid,
                control_field,
                binding_uid,
            } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
                buf[2..8].copy_from_slice(&<[u8; 6]>::from(*uid));
                buf[8..10].copy_from_slice(&control_field.to_be_bytes());
                buf[10..16].copy_from_slice(&<[u8; 6]>::from(*binding_uid));
            }
            Self::GetBackgroundQueuedStatusPolicy {
                current_policy_id,
                policy_count,
            } => {
                buf[0] = *current_policy_id;
                buf[1] = *policy_count;
            }
            Self::GetBackgroundQueuedStatusPolicyDescription {
                policy_id,
                description,
            } => {
                buf[0] = *policy_id;
                buf[1..1 + description.len()].copy_from_slice(description.as_bytes());
            }
            // E1.33
            Self::GetComponentScope {
                scope_slot,
                scope_string,
                static_config_type,
                static_ipv4_address,
                static_ipv6_address,
                static_port,
            } => {
                buf[0..2].copy_from_slice(&scope_slot.to_be_bytes());
                scope_string.encode(&mut buf[2..2 + ScopeString::MAX_LENGTH])?;
                buf[2 + ScopeString::MAX_LENGTH] = *static_config_type as u8;
                buf[3 + ScopeString::MAX_LENGTH..7 + ScopeString::MAX_LENGTH]
                    .copy_from_slice(&u32::from(*static_ipv4_address).to_be_bytes());
                buf[7 + ScopeString::MAX_LENGTH..23 + ScopeString::MAX_LENGTH]
                    .copy_from_slice(&<[u8; 16]>::from(*static_ipv6_address));
                buf[23 + ScopeString::MAX_LENGTH..25 + ScopeString::MAX_LENGTH]
                    .copy_from_slice(&static_port.to_be_bytes());
            }
            Self::GetSearchDomain(search_domain) => {
                search_domain.encode(buf)?;
            }
            Self::GetTcpCommsStatus {
                scope_string,
                broker_ipv4_address,
                broker_ipv6_address,
                broker_port,
                unhealthy_tcp_events,
            } => {
                scope_string.encode(&mut buf[0..ScopeString::MAX_LENGTH])?;
                buf[ScopeString::MAX_LENGTH..ScopeString::MAX_LENGTH + 4]
                    .copy_from_slice(&u32::from(*broker_ipv4_address).to_be_bytes());
                buf[ScopeString::MAX_LENGTH + 4..ScopeString::MAX_LENGTH + 20]
                    .copy_from_slice(&<[u8; 16]>::from(*broker_ipv6_address));
                buf[ScopeString::MAX_LENGTH + 20..ScopeString::MAX_LENGTH + 22]
                    .copy_from_slice(&broker_port.to_be_bytes());
                buf[ScopeString::MAX_LENGTH + 22..ScopeString::MAX_LENGTH + 24]
                    .copy_from_slice(&unhealthy_tcp_events.to_be_bytes());
            }
            Self::GetBrokerStatus {
                is_allowing_set_commands,
                broker_state,
            } => {
                buf[0] = *is_allowing_set_commands as u8;
                buf[1] = *broker_state as u8;
            }
            Self::RawParameter(data) => {
                buf[0..data.len()].copy_from_slice(data);
            }
        };

        Ok(self.size())
    }

    pub fn decode(
        command_class: CommandClass,
        parameter_id: ParameterId,
        bytes: &'a [u8],
    ) -> Result<Self, RdmError> {
        match (command_class, parameter_id) {
            (CommandClass::DiscoveryCommandResponse, ParameterId::DiscMute) => {
                let binding_uid = if bytes.len() > 2 {
                    Some(DeviceUID::from(<[u8; 6]>::try_from(&bytes[2..=7])?))
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
                    Some(DeviceUID::from(<[u8; 6]>::try_from(&bytes[2..=7])?))
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
                        .map(|chunk| Ok(DeviceUID::from(<[u8; 6]>::try_from(&chunk[0..=5])?)))
                        .collect::<Result<Vec<DeviceUID>, RdmError>>()?,
                    #[cfg(not(feature = "alloc"))]
                    bytes
                        .chunks(6)
                        .map(|chunk| Ok(DeviceUID::from(<[u8; 6]>::try_from(&chunk[0..=5])?)))
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
                    description: ParameterDescriptionLabel::decode(&bytes[20..])?,
                }))
            }
            (CommandClass::GetCommandResponse, ParameterId::DeviceInfo) => {
                Ok(Self::GetDeviceInfo {
                    protocol_version: ProtocolVersion::new(bytes[0], bytes[1]),
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
                Ok(Self::GetDeviceLabel(DeviceLabel::decode(bytes)?))
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
                    hardware_address: <[u8; 6]>::try_from(&bytes[4..=9])?.into(),
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
                    address: <[u8; 4]>::try_from(&bytes[4..=7])?.into(),
                    netmask: bytes[8],
                    dhcp_status: DhcpMode::try_from(bytes[9])?,
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::IpV4StaticAddress) => {
                Ok(Self::GetIpV4StaticAddress {
                    interface_id: u32::from_be_bytes(bytes[0..=3].try_into()?),
                    address: <[u8; 4]>::try_from(&bytes[4..=7])?.into(),
                    netmask: bytes[8],
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::IpV4DefaultRoute) => {
                Ok(Self::GetIpV4DefaultRoute {
                    interface_id: u32::from_be_bytes(bytes[0..=3].try_into()?),
                    address: <[u8; 4]>::try_from(&bytes[4..=7])?.into(),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::DnsIpV4NameServer) => {
                Ok(Self::GetDnsIpV4NameServer {
                    name_server_index: bytes[0],
                    address: <[u8; 4]>::try_from(&bytes[1..=4])?.into(),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::DnsHostName) => {
                Ok(Self::GetDnsHostName(DnsHostName::decode(&bytes[0..])?))
            }
            // E1.37-7
            (CommandClass::GetCommand, ParameterId::EndpointList) => Ok(Self::GetEndpointList {
                list_change_number: u32::from_be_bytes(bytes[0..=3].try_into()?),
                #[cfg(feature = "alloc")]
                endpoint_list: bytes[4..]
                    .chunks(3)
                    .map(|chunk| {
                        Ok((
                            u16::from_be_bytes(chunk[0..=1].try_into()?).into(),
                            chunk[1].try_into()?,
                        ))
                    })
                    .collect::<Result<Vec<(EndpointId, EndpointType)>, RdmError>>()?,
                #[cfg(not(feature = "alloc"))]
                endpoint_list: bytes[4..]
                    .chunks(6)
                    .map(|chunk| {
                        Ok((
                            u16::from_be_bytes(chunk[0..=1].try_into()?).into(),
                            chunk[1].try_into()?,
                        ))
                    })
                    .collect::<Result<Vec<(EndpointId, EndpointType), 75>, RdmError>>()?,
            }),
            (CommandClass::GetCommand, ParameterId::EndpointListChange) => {
                Ok(Self::GetEndpointListChange {
                    list_change_number: u32::from_be_bytes(bytes[0..=3].try_into()?),
                })
            }
            (CommandClass::GetCommand, ParameterId::IdentifyEndpoint) => {
                Ok(Self::GetIdentifyEndpoint {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                    identify: bytes[2] == 1,
                })
            }
            (CommandClass::SetCommand, ParameterId::IdentifyEndpoint) => {
                Ok(Self::SetIdentifyEndpoint {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                })
            }
            (CommandClass::GetCommand, ParameterId::EndpointToUniverse) => {
                Ok(Self::GetEndpointToUniverse {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                    universe: u16::from_be_bytes(bytes[2..=3].try_into()?),
                })
            }
            (CommandClass::SetCommand, ParameterId::EndpointToUniverse) => {
                Ok(Self::SetEndpointToUniverse {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                })
            }
            (CommandClass::GetCommand, ParameterId::EndpointMode) => Ok(Self::GetEndpointMode {
                endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                mode: bytes[2].try_into()?,
            }),
            (CommandClass::SetCommand, ParameterId::EndpointMode) => Ok(Self::SetEndpointMode {
                endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
            }),
            (CommandClass::GetCommand, ParameterId::EndpointLabel) => Ok(Self::GetEndpointLabel {
                endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                label: decode_string_bytes(&bytes[2..])?,
            }),
            (CommandClass::SetCommand, ParameterId::EndpointLabel) => Ok(Self::SetEndpointLabel {
                endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
            }),
            (CommandClass::GetCommand, ParameterId::RdmTrafficEnable) => {
                Ok(Self::GetRdmTrafficEnable {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                    enable: bytes[2] == 1,
                })
            }
            (CommandClass::SetCommand, ParameterId::RdmTrafficEnable) => {
                Ok(Self::SetRdmTrafficEnable {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                })
            }
            (CommandClass::GetCommand, ParameterId::DiscoveryState) => {
                Ok(Self::GetDiscoveryState {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                    device_count: u16::from_be_bytes(bytes[2..=3].try_into()?).into(),
                    discovery_state: bytes[4].try_into()?,
                })
            }
            (CommandClass::SetCommand, ParameterId::DiscoveryState) => {
                Ok(Self::SetDiscoveryState {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                })
            }
            (CommandClass::GetCommand, ParameterId::BackgroundDiscovery) => {
                Ok(Self::GetBackgroundDiscovery {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                    enabled: bytes[2] == 1,
                })
            }
            (CommandClass::SetCommand, ParameterId::BackgroundDiscovery) => {
                Ok(Self::SetBackgroundDiscovery {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                })
            }
            (CommandClass::GetCommand, ParameterId::EndpointTiming) => {
                Ok(Self::GetEndpointTiming {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                    current_setting_id: bytes[2],
                    setting_count: bytes[3],
                })
            }
            (CommandClass::SetCommand, ParameterId::EndpointTiming) => {
                Ok(Self::SetEndpointTiming {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                })
            }
            (CommandClass::GetCommand, ParameterId::EndpointTimingDescription) => {
                Ok(Self::GetEndpointTimingDescription {
                    setting_id: bytes[0],
                    description: decode_string_bytes(&bytes[1..])?,
                })
            }
            (CommandClass::GetCommand, ParameterId::EndpointResponders) => {
                Ok(Self::GetEndpointResponders {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                    list_change_number: u32::from_be_bytes(bytes[2..=5].try_into()?),
                    #[cfg(feature = "alloc")]
                    responders: bytes[6..]
                        .chunks(6)
                        .map(|chunk| {
                            Ok(DeviceUID::new(
                                u16::from_be_bytes(chunk[0..=1].try_into()?),
                                u32::from_be_bytes(chunk[2..=5].try_into()?),
                            ))
                        })
                        .collect::<Result<Vec<DeviceUID>, RdmError>>()?,
                    #[cfg(not(feature = "alloc"))]
                    responders: bytes[6..]
                        .chunks(6)
                        .map(|chunk| {
                            Ok(DeviceUID::new(
                                u16::from_be_bytes(chunk[0..=1].try_into()?),
                                u32::from_be_bytes(chunk[2..=5].try_into()?),
                            ))
                        })
                        .collect::<Result<Vec<DeviceUID, 37>, RdmError>>()?,
                })
            }
            (CommandClass::GetCommand, ParameterId::EndpointResponderListChange) => {
                Ok(Self::GetEndpointResponderListChange {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                    list_change_number: u32::from_be_bytes(bytes[2..=5].try_into()?),
                })
            }
            (CommandClass::GetCommand, ParameterId::BindingControlFields) => {
                Ok(Self::GetBindingControlFields {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                    uid: DeviceUID::new(
                        u16::from_be_bytes(bytes[2..=3].try_into()?),
                        u32::from_be_bytes(bytes[4..=7].try_into()?),
                    ),
                    control_field: u16::from_be_bytes(bytes[8..=9].try_into()?),
                    binding_uid: DeviceUID::new(
                        u16::from_be_bytes(bytes[10..=11].try_into()?),
                        u32::from_be_bytes(bytes[12..=15].try_into()?),
                    ),
                })
            }
            (CommandClass::GetCommand, ParameterId::BackgroundQueuedStatusPolicy) => {
                Ok(Self::GetBackgroundQueuedStatusPolicy {
                    current_policy_id: bytes[0],
                    policy_count: bytes[1],
                })
            }
            (CommandClass::GetCommand, ParameterId::BackgroundQueuedStatusPolicyDescription) => {
                Ok(Self::GetBackgroundQueuedStatusPolicyDescription {
                    policy_id: bytes[0],
                    description: decode_string_bytes(&bytes[1..])?,
                })
            }
            // E1.33
            (CommandClass::GetCommandResponse, ParameterId::ComponentScope) => {
                Ok(Self::GetComponentScope {
                    scope_slot: u16::from_be_bytes(bytes[0..2].try_into()?),
                    scope_string: ScopeString::decode(&bytes[2..2 + ScopeString::MAX_LENGTH])?,
                    static_config_type: bytes[3 + ScopeString::MAX_LENGTH].try_into()?,
                    static_ipv4_address: <[u8; 4]>::try_from(
                        &bytes[4 + ScopeString::MAX_LENGTH..8 + ScopeString::MAX_LENGTH],
                    )?
                    .into(),
                    static_ipv6_address: <[u8; 16]>::try_from(
                        &bytes[8 + ScopeString::MAX_LENGTH..24 + ScopeString::MAX_LENGTH],
                    )?
                    .into(),
                    static_port: u16::from_be_bytes(
                        bytes[24 + ScopeString::MAX_LENGTH..26 + ScopeString::MAX_LENGTH]
                            .try_into()?,
                    ),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::SearchDomain) => {
                Ok(Self::GetSearchDomain(SearchDomain::decode(bytes)?))
            }
            (CommandClass::GetCommandResponse, ParameterId::TcpCommsStatus) => {
                Ok(Self::GetTcpCommsStatus {
                    scope_string: ScopeString::decode(&bytes[0..ScopeString::MAX_LENGTH])?,
                    broker_ipv4_address: <[u8; 4]>::try_from(
                        &bytes[ScopeString::MAX_LENGTH..ScopeString::MAX_LENGTH + 4],
                    )?
                    .into(),
                    broker_ipv6_address: <[u8; 16]>::try_from(
                        &bytes[ScopeString::MAX_LENGTH + 4..ScopeString::MAX_LENGTH + 20],
                    )?
                    .into(),
                    broker_port: u16::from_be_bytes(
                        bytes[ScopeString::MAX_LENGTH + 20..ScopeString::MAX_LENGTH + 22]
                            .try_into()?,
                    ),
                    unhealthy_tcp_events: u16::from_be_bytes(
                        bytes[ScopeString::MAX_LENGTH + 22..ScopeString::MAX_LENGTH + 24]
                            .try_into()?,
                    ),
                })
            }
            (CommandClass::GetCommandResponse, ParameterId::BrokerStatus) => {
                Ok(Self::GetBrokerStatus {
                    is_allowing_set_commands: bytes[0] == 1,
                    broker_state: bytes[1].try_into()?,
                })
            }
            (_, _) => Ok(Self::RawParameter(bytes)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RdmFrameResponse<'a> {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub response_type: ResponseType,
    pub message_count: u8,
    pub sub_device_id: SubDeviceId,
    pub command_class: CommandClass,
    pub parameter_id: ParameterId,
    pub parameter_data: ResponseData<'a>,
}

impl<'a> RdmFrameResponse<'a> {
    pub fn size(&self) -> usize {
        24 + self.parameter_data.size() + 2
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, RdmError> {
        // let parameter_data = self.parameter_data.encode();

        // let message_length = 24 + parameter_data.len();

        buf[0] = RDM_START_CODE_BYTE;
        buf[1] = RDM_SUB_START_CODE_BYTE;

        let parameter_data_length = self.parameter_data.encode(&mut buf[24..])?;
        let message_length = 24 + parameter_data_length;

        buf[2] = message_length as u8;
        buf[3..9].copy_from_slice(&<[u8; 6]>::from(self.destination_uid));
        buf[9..15].copy_from_slice(&<[u8; 6]>::from(self.source_uid));
        buf[15] = self.transaction_number;
        buf[16] = self.response_type as u8;
        buf[17] = self.message_count;
        buf[18..20].copy_from_slice(&u16::from(self.sub_device_id).to_be_bytes());
        buf[20] = self.command_class as u8;
        buf[21..23].copy_from_slice(&u16::from(self.parameter_id).to_be_bytes());
        buf[23] = parameter_data_length as u8;

        let mut crc = 0_u16;

        for byte in &buf[0..message_length] {
            crc = crc.overflowing_add(*byte as u16).0;
        }

        buf[message_length..message_length + 2].copy_from_slice(&crc.to_be_bytes());

        Ok(message_length + 2)
    }

    pub fn decode(bytes: &'a [u8]) -> Result<Self, RdmError> {
        let message_length = bytes[2];

        if message_length < 24 {
            return Err(RdmError::InvalidMessageLength(message_length));
        }

        if bytes.len() < message_length as usize + 2 {
            return Err(RdmError::InvalidMessageLength(message_length));
        }

        let packet_checksum = u16::from_be_bytes(
            bytes[message_length as usize..=message_length as usize + 1].try_into()?,
        );

        let decoded_checksum = bsd_16_crc(&bytes[..message_length as usize]);

        if decoded_checksum != packet_checksum {
            return Err(RdmError::InvalidChecksum(decoded_checksum, packet_checksum));
        }

        let destination_uid = DeviceUID::from(<[u8; 6]>::try_from(&bytes[3..=8])?);

        let source_uid = DeviceUID::from(<[u8; 6]>::try_from(&bytes[9..=14])?);

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

impl<'a> TryFrom<&'a [u8]> for RdmFrameResponse<'a> {
    type Error = RdmError;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        RdmFrameResponse::decode(bytes)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DiscoveryUniqueBranchFrameResponse(pub DeviceUID);

impl DiscoveryUniqueBranchFrameResponse {
    pub fn size(&self) -> usize {
        24 // Fixed size for Discovery Unique Branch Frame
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, RdmError> {
        buf[0..7].copy_from_slice(&[
            DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
            DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
            DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
            DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
            DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
            DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
            DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
        ]);

        buf[7] = DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE;

        let [manufacturer_id1, manufacturer_id0] = self.0.manufacturer_id.to_be_bytes();

        buf[8..12].copy_from_slice(&[
            manufacturer_id1 | 0xaa,
            manufacturer_id1 | 0x55,
            manufacturer_id0 | 0xaa,
            manufacturer_id0 | 0x55,
        ]);

        let [device_id3, device_id2, device_id1, device_id0] = self.0.device_id.to_be_bytes();

        buf[12..20].copy_from_slice(&[
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

        buf[20..24].copy_from_slice(&[
            checksum1 | 0xaa,
            checksum1 | 0x55,
            checksum0 | 0xaa,
            checksum0 | 0x55,
        ]);

        Ok(24)
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
pub enum RdmResponse<'a> {
    RdmFrame(RdmFrameResponse<'a>),
    DiscoveryUniqueBranchFrame(DiscoveryUniqueBranchFrameResponse),
}

impl<'a> RdmResponse<'a> {
    pub fn size(&self) -> usize {
        match self {
            RdmResponse::RdmFrame(frame) => frame.size(),
            RdmResponse::DiscoveryUniqueBranchFrame(frame) => frame.size(),
        }
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, RdmError> {
        match self {
            RdmResponse::RdmFrame(frame) => frame.encode(buf),
            RdmResponse::DiscoveryUniqueBranchFrame(frame) => frame.encode(buf),
        }
    }

    pub fn decode(bytes: &'a [u8]) -> Result<Self, RdmError> {
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

impl<'a> TryFrom<&'a [u8]> for RdmResponse<'a> {
    type Error = RdmError;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
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
        let mut encoded = [0u8; 256];

        let bytes_encoded = RdmResponse::RdmFrame(RdmFrameResponse {
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
        })
        .encode(&mut encoded)
        .unwrap();

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

        assert_eq!(&encoded[0..bytes_encoded], expected);
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
            parameter_id: ParameterId::RawParameterId(0x8080),
            parameter_data: ResponseData::ParameterData(Some(ResponseParameterData::RawParameter(
                &[0x04, 0x03, 0x02, 0x01],
            ))),
        }));

        assert_eq!(decoded, expected);
    }

    #[test]
    fn should_encode_valid_rdm_ack_manufacturer_specific_response() {
        let mut encoded = [0u8; 256];

        let bytes_encoded = RdmResponse::RdmFrame(RdmFrameResponse {
            destination_uid: DeviceUID::new(0x0102, 0x03040506),
            source_uid: DeviceUID::new(0x0605, 0x04030201),
            transaction_number: 0x00,
            response_type: ResponseType::Ack,
            message_count: 0x00,
            sub_device_id: SubDeviceId::RootDevice,
            command_class: CommandClass::SetCommandResponse,
            parameter_id: ParameterId::RawParameterId(0x8080),
            parameter_data: ResponseData::ParameterData(Some(ResponseParameterData::RawParameter(
                &[0x04, 0x03, 0x02, 0x01],
            ))),
        })
        .encode(&mut encoded)
        .unwrap();

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

        assert_eq!(&encoded[0..bytes_encoded], expected);
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
        let mut encoded = [0u8; 256];

        let bytes_encoded = RdmResponse::RdmFrame(RdmFrameResponse {
            destination_uid: DeviceUID::new(0x0102, 0x03040506),
            source_uid: DeviceUID::new(0x0605, 0x04030201),
            transaction_number: 0x00,
            response_type: ResponseType::AckTimer,
            message_count: 0x00,
            sub_device_id: SubDeviceId::RootDevice,
            command_class: CommandClass::GetCommandResponse,
            parameter_id: ParameterId::IdentifyDevice,
            parameter_data: ResponseData::EstimateResponseTime(0x0a),
        })
        .encode(&mut encoded)
        .unwrap();

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

        assert_eq!(&encoded[0..bytes_encoded], expected);
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
        let mut encoded = [0u8; 256];

        let bytes_encoded = RdmResponse::RdmFrame(RdmFrameResponse {
            destination_uid: DeviceUID::new(0x0102, 0x03040506),
            source_uid: DeviceUID::new(0x0605, 0x04030201),
            transaction_number: 0x00,
            response_type: ResponseType::NackReason,
            message_count: 0x00,
            sub_device_id: SubDeviceId::RootDevice,
            command_class: CommandClass::GetCommandResponse,
            parameter_id: ParameterId::IdentifyDevice,
            parameter_data: ResponseData::NackReason(ResponseNackReasonCode::FormatError),
        })
        .encode(&mut encoded)
        .unwrap();

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

        assert_eq!(&encoded[0..bytes_encoded], expected);
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
        let mut encoded = [0u8; 256];

        let bytes_encoded = RdmResponse::RdmFrame(RdmFrameResponse {
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
        })
        .encode(&mut encoded)
        .unwrap();

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

        assert_eq!(&encoded[0..bytes_encoded], expected);
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
        let mut encoded = [0u8; 256];

        let bytes_encoded = RdmResponse::DiscoveryUniqueBranchFrame(
            DiscoveryUniqueBranchFrameResponse(DeviceUID::new(0x0102, 0x03040506)),
        )
        .encode(&mut encoded)
        .unwrap();

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

        assert_eq!(&encoded[0..bytes_encoded], expected);
    }
}
