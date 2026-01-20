//! Data types and functionality for encoding RDM requests
//!
//! # RdmRequest
//!
//! ```rust
//! use dmx512_rdm_protocol::rdm::{
//!     request::{RdmRequest, RequestParameter},
//!     DeviceUID, SubDeviceId,
//! };
//!
//! fn encoded_request() {
//!     let mut encoded = [0u8; 26];
//!
//!     let bytes_written = RdmRequest::new(
//!         DeviceUID::new(0x0102, 0x03040506),
//!         DeviceUID::new(0x0605, 0x04030201),
//!         0x00,
//!         0x01,
//!         SubDeviceId::RootDevice,
//!         RequestParameter::GetIdentifyDevice,
//!     )
//!     .encode(&mut encoded);
//!
//!     let expected: &[u8; 26] = &[
//!         0xcc, // Start Code
//!         0x01, // Sub Start Code
//!         0x18, // Message Length
//!         0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // Destination UID
//!         0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // Source UID
//!         0x00, // Transaction Number
//!         0x01, // Port ID
//!         0x00, // Message Count
//!         0x00, 0x00, // Sub-Device ID = Root Device
//!         0x20, // Command Class = GetCommand
//!         0x10, 0x00, // Parameter ID = Identify Device
//!         0x00, // PDL
//!         0x01, 0x40, // Checksum
//!     ];
//!
//!     assert_eq!(&encoded, expected);
//! }
//!
//! ```
//!
//! See tests for more examples.

use super::{
    RDM_START_CODE_BYTE, RDM_SUB_START_CODE_BYTE,
    error::RdmError,
    header::{CommandClass, DeviceUID, SubDeviceId},
    parameter::{
        ParameterId,
        e120::{PresetPlaybackMode, SelfTest},
        e133::{BrokerState, Scope, SearchDomain, StaticConfigType},
        e137_1::{MergeMode, PinCode, TimeMode},
        e137_2::{DnsDomainName, DnsHostName, Ipv4Address, Ipv4Route, Ipv6Address},
        e137_7::{DiscoveryState, EndpointId, EndpointLabel, EndpointMode},
    },
    utils::{RdmPadNullStr, RdmTruncateNullStr, bsd_16_crc},
};
use crate::rdm::parameter::{
    e120::{
        GetDmxPersonalityDescriptionRequest, GetParameterDescriptionRequest,
        GetQueuedMessageRequest, GetSelfTestDescriptionRequest, GetSensorDefinitionRequest,
        GetSensorValueRequest, GetSlotDescriptionRequest, GetStatusIdDescriptionRequest,
        GetStatusMessagesRequest, SetCapturePresetRequest, SetDeviceHoursRequest,
        SetDeviceLabelRequest, SetDevicePowerCyclesRequest, SetDisplayInvertModeRequest,
        SetDisplayLevelRequest, SetDmxPersonalityRequest, SetDmxStartAddressRequest,
        SetIdentifyDeviceRequest, SetLampHoursRequest, SetLampOnModeRequest, SetLampStateRequest,
        SetLampStrikesRequest, SetLanguageRequest, SetPanInvertRequest, SetPanTiltSwapRequest,
        SetPerformSelfTestRequest, SetPowerStateRequest, SetPresetPlaybackRequest,
        SetRealTimeClockRequest, SetRecordSensorsRequest, SetResetDeviceRequest,
        SetSensorValueRequest, SetSubDeviceIdStatusReportThresholdRequest, SetTiltInvertRequest,
    },
    e133::SCOPE_MAX_LENGTH,
    e137_1::{
        GetCurveDescriptionRequest, GetModulationFrequencyDescriptionRequest,
        GetOutputResponseTimeDescriptionRequest, GetPresetStatusRequest, SetBurnInRequest,
        SetCurveRequest, SetDmxBlockAddressRequest, SetIdentifyModeRequest, SetMaximumLevelRequest,
        SetMinimumLevelRequest, SetModulationFrequencyRequest, SetOutputResponseTimeRequest,
        SetPresetStatusRequest,
    },
};
use heapless::Vec;
use rdm_parameter_traits::{RdmGetRequestParameterCodec, RdmSetRequestParameterCodec};

#[derive(Clone, Debug, PartialEq)]
pub enum RequestParameter {
    // E1.20
    DiscMute,
    DiscUnMute,
    DiscUniqueBranch {
        lower_bound_uid: DeviceUID,
        upper_bound_uid: DeviceUID,
    },
    GetCommsStatus,
    SetCommsStatus,
    GetQueuedMessage(GetQueuedMessageRequest),
    GetStatusMessages(GetStatusMessagesRequest),
    GetStatusIdDescription(GetStatusIdDescriptionRequest),
    SetClearStatusId,
    GetSubDeviceIdStatusReportThreshold,
    SetSubDeviceIdStatusReportThreshold(SetSubDeviceIdStatusReportThresholdRequest),
    GetSupportedParameters,
    GetParameterDescription(GetParameterDescriptionRequest),
    GetDeviceInfo,
    GetProductDetailIdList,
    GetDeviceModelDescription,
    GetManufacturerLabel,
    GetDeviceLabel,
    SetDeviceLabel(SetDeviceLabelRequest),
    GetFactoryDefaults,
    SetFactoryDefaults,
    GetLanguageCapabilities,
    GetLanguage,
    SetLanguage(SetLanguageRequest),
    GetSoftwareVersionLabel,
    GetBootSoftwareVersionId,
    GetBootSoftwareVersionLabel,
    GetDmxPersonality,
    SetDmxPersonality(SetDmxPersonalityRequest),
    GetDmxPersonalityDescription(GetDmxPersonalityDescriptionRequest),
    GetDmxStartAddress,
    SetDmxStartAddress(SetDmxStartAddressRequest),
    GetSlotInfo,
    GetSlotDescription(GetSlotDescriptionRequest),
    GetDefaultSlotValue,
    GetSensorDefinition(GetSensorDefinitionRequest),
    GetSensorValue(GetSensorValueRequest),
    SetSensorValue(SetSensorValueRequest),
    SetRecordSensors(SetRecordSensorsRequest),
    GetDeviceHours,
    SetDeviceHours(SetDeviceHoursRequest),
    GetLampHours,
    SetLampHours(SetLampHoursRequest),
    GetLampStrikes,
    SetLampStrikes(SetLampStrikesRequest),
    GetLampState,
    SetLampState(SetLampStateRequest),
    GetLampOnMode,
    SetLampOnMode(SetLampOnModeRequest),
    GetDevicePowerCycles,
    SetDevicePowerCycles(SetDevicePowerCyclesRequest),
    GetDisplayInvert,
    SetDisplayInvert(SetDisplayInvertModeRequest),
    GetDisplayLevel,
    SetDisplayLevel(SetDisplayLevelRequest),
    GetPanInvert,
    SetPanInvert(SetPanInvertRequest),
    GetTiltInvert,
    SetTiltInvert(SetTiltInvertRequest),
    GetPanTiltSwap,
    SetPanTiltSwap(SetPanTiltSwapRequest),
    GetRealTimeClock,
    SetRealTimeClock(SetRealTimeClockRequest),
    GetIdentifyDevice,
    SetIdentifyDevice(SetIdentifyDeviceRequest),
    SetResetDevice(SetResetDeviceRequest),
    GetPowerState,
    SetPowerState(SetPowerStateRequest),
    GetPerformSelfTest,
    SetPerformSelfTest(SetPerformSelfTestRequest),
    SetCapturePreset(SetCapturePresetRequest),
    GetSelfTestDescription(GetSelfTestDescriptionRequest),
    GetPresetPlayback,
    SetPresetPlayback(SetPresetPlaybackRequest),
    // E1.37-1
    GetDmxBlockAddress,
    SetDmxBlockAddress(SetDmxBlockAddressRequest),
    GetDmxFailMode,
    SetDmxFailMode {
        scene_id: PresetPlaybackMode,
        loss_of_signal_delay_time: TimeMode,
        hold_time: TimeMode,
        level: u8,
    },
    GetDmxStartupMode,
    SetDmxStartupMode {
        scene_id: PresetPlaybackMode,
        startup_delay: TimeMode,
        hold_time: TimeMode,
        level: u8,
    },
    GetDimmerInfo,
    GetMinimumLevel,
    SetMinimumLevel(SetMinimumLevelRequest),
    GetMaximumLevel,
    SetMaximumLevel(SetMaximumLevelRequest),
    GetCurve,
    SetCurve(SetCurveRequest),
    GetCurveDescription(GetCurveDescriptionRequest),
    GetOutputResponseTime,
    SetOutputResponseTime(SetOutputResponseTimeRequest),
    GetOutputResponseTimeDescription(GetOutputResponseTimeDescriptionRequest),
    GetModulationFrequency,
    SetModulationFrequency(SetModulationFrequencyRequest),
    GetModulationFrequencyDescription(GetModulationFrequencyDescriptionRequest),
    GetPowerOnSelfTest,
    SetPowerOnSelfTest {
        self_test_id: SelfTest,
    },
    GetLockState,
    SetLockState {
        pin_code: PinCode,
        lock_state: bool,
    },
    GetLockStateDescription,
    GetLockPin,
    SetLockPin {
        new_pin_code: PinCode,
        current_pin_code: PinCode,
    },
    GetBurnIn,
    SetBurnIn(SetBurnInRequest),
    GetIdentifyMode,
    SetIdentifyMode(SetIdentifyModeRequest),
    GetPresetInfo,
    GetPresetStatus(GetPresetStatusRequest),
    GetPresetMergeMode,
    SetPresetMergeMode {
        merge_mode: MergeMode,
    },
    SetPresetStatus(SetPresetStatusRequest),
    // E1.37-2
    GetListInterfaces,
    GetInterfaceLabel {
        interface_id: u32,
    },
    GetInterfaceHardwareAddressType1 {
        interface_id: u32,
    },
    GetIpV4DhcpMode {
        interface_id: u32,
    },
    SetIpV4DhcpMode {
        interface_id: u32,
        dhcp_mode: bool,
    },
    GetIpV4ZeroConfMode {
        interface_id: u32,
    },
    SetIpV4ZeroConfMode {
        interface_id: u32,
        zero_conf_mode: bool,
    },
    GetIpV4CurrentAddress {
        interface_id: u32,
    },
    GetIpV4StaticAddress {
        interface_id: u32,
    },
    SetIpV4StaticAddress {
        interface_id: u32,
        address: Ipv4Address,
        netmask: u8,
    },
    SetInterfaceApplyConfiguration {
        interface_id: u32,
    },
    SetInterfaceRenewDhcp {
        interface_id: u32,
    },
    SetInterfaceReleaseDhcp {
        interface_id: u32,
    },
    GetIpV4DefaultRoute,
    SetIpV4DefaultRoute {
        interface_id: u32,
        ipv4_default_route: Ipv4Route,
    },
    GetDnsIpV4NameServer {
        name_server_index: u8,
    },
    SetDnsIpV4NameServer {
        name_server_index: u8,
        name_server_address: Ipv4Address,
    },
    GetDnsHostName,
    SetDnsHostName(DnsHostName),
    GetDnsDomainName,
    SetDnsDomainName(DnsDomainName),
    // E1.37-7
    GetEndpointList,
    GetEndpointListChange,
    GetIdentifyEndpoint {
        endpoint_id: EndpointId,
    },
    SetIdentifyEndpoint {
        endpoint_id: EndpointId,
        identify: bool,
    },
    GetEndpointToUniverse {
        endpoint_id: EndpointId,
    },
    SetEndpointToUniverse {
        endpoint_id: EndpointId,
        universe: u16,
    },
    GetEndpointMode {
        endpoint_id: EndpointId,
    },
    SetEndpointMode {
        endpoint_id: EndpointId,
        mode: EndpointMode,
    },
    GetEndpointLabel {
        endpoint_id: EndpointId,
    },
    SetEndpointLabel {
        endpoint_id: EndpointId,
        label: EndpointLabel,
    },
    GetRdmTrafficEnable {
        endpoint_id: EndpointId,
    },
    SetRdmTrafficEnable {
        endpoint_id: EndpointId,
        enable: bool,
    },
    GetDiscoveryState {
        endpoint_id: EndpointId,
    },
    SetDiscoveryState {
        endpoint_id: EndpointId,
        state: DiscoveryState,
    },
    GetBackgroundDiscovery {
        endpoint_id: EndpointId,
    },
    SetBackgroundDiscovery {
        endpoint_id: EndpointId,
        enable: bool,
    },
    GetEndpointTiming {
        endpoint_id: EndpointId,
    },
    SetEndpointTiming {
        endpoint_id: EndpointId,
        setting_id: u8,
    },
    GetEndpointTimingDescription {
        setting_id: u8,
    },
    GetEndpointResponders {
        endpoint_id: EndpointId,
    },
    GetEndpointResponderListChange {
        endpoint_id: EndpointId,
    },
    GetBindingControlFields {
        endpoint_id: EndpointId,
        uid: DeviceUID,
    },
    GetBackgroundQueuedStatusPolicy,
    SetBackgroundQueuedStatusPolicy {
        policy_id: u8,
    },
    GetBackgroundQueuedStatusPolicyDescription {
        policy_id: u8,
    },
    // E1.33
    GetSearchDomain,
    SetSearchDomain(SearchDomain),
    GetComponentScope {
        scope_slot: u16,
    },
    SetComponentScope {
        scope_slot: u16,
        scope_string: Scope,
        static_config_type: StaticConfigType,
        static_broker_ipv4_address: Ipv4Address,
        static_broker_ipv6_address: Ipv6Address,
        static_broker_port: u16,
    },
    GetTcpCommsStatus,
    SetTcpCommsStatus(Scope),
    GetBrokerStatus,
    SetBrokerStatus {
        broker_state: BrokerState,
    },
    // use for unsupported standard and manufacturer specific parameters
    RawParameter {
        command_class: CommandClass,
        parameter_id: u16,
        parameter_data: Vec<u8, 231>,
    },
}

impl RequestParameter {
    pub fn command_class(&self) -> CommandClass {
        match self {
            Self::DiscMute | Self::DiscUnMute | Self::DiscUniqueBranch { .. } => {
                CommandClass::DiscoveryCommand
            }
            // E1.20
            Self::GetCommsStatus
            | Self::GetQueuedMessage { .. }
            | Self::GetStatusMessages { .. }
            | Self::GetStatusIdDescription { .. }
            | Self::GetSubDeviceIdStatusReportThreshold
            | Self::GetSupportedParameters
            | Self::GetParameterDescription { .. }
            | Self::GetDeviceInfo
            | Self::GetProductDetailIdList
            | Self::GetDeviceModelDescription
            | Self::GetManufacturerLabel
            | Self::GetDeviceLabel
            | Self::GetFactoryDefaults
            | Self::GetLanguageCapabilities
            | Self::GetLanguage
            | Self::GetSoftwareVersionLabel
            | Self::GetBootSoftwareVersionId
            | Self::GetBootSoftwareVersionLabel
            | Self::GetDmxPersonality
            | Self::GetDmxPersonalityDescription { .. }
            | Self::GetDmxStartAddress
            | Self::GetSlotInfo
            | Self::GetSlotDescription { .. }
            | Self::GetDefaultSlotValue
            | Self::GetSensorDefinition { .. }
            | Self::GetSensorValue { .. }
            | Self::GetDeviceHours
            | Self::GetLampHours
            | Self::GetLampStrikes
            | Self::GetLampState
            | Self::GetLampOnMode
            | Self::GetDevicePowerCycles
            | Self::GetDisplayInvert
            | Self::GetDisplayLevel
            | Self::GetPanInvert
            | Self::GetTiltInvert
            | Self::GetPanTiltSwap
            | Self::GetRealTimeClock
            | Self::GetIdentifyDevice
            | Self::GetPowerState
            | Self::GetPerformSelfTest
            | Self::GetSelfTestDescription { .. }
            | Self::GetPresetPlayback
            // E1.37-1
            | Self::GetDmxBlockAddress
            | Self::GetDmxFailMode
            | Self::GetDmxStartupMode
            | Self::GetDimmerInfo
            | Self::GetMinimumLevel
            | Self::GetMaximumLevel
            | Self::GetCurve
            | Self::GetCurveDescription { .. }
            | Self::GetOutputResponseTime
            | Self::GetOutputResponseTimeDescription { .. }
            | Self::GetModulationFrequency
            | Self::GetModulationFrequencyDescription { .. }
            | Self::GetPowerOnSelfTest
            | Self::GetLockState
            | Self::GetLockStateDescription
            | Self::GetLockPin
            | Self::GetBurnIn
            | Self::GetIdentifyMode
            | Self::GetPresetInfo
            | Self::GetPresetStatus { .. }
            | Self::GetPresetMergeMode
            // E1.37-2
            | Self::GetListInterfaces
            | Self::GetInterfaceLabel { .. }
            | Self::GetInterfaceHardwareAddressType1 { .. }
            | Self::GetIpV4DhcpMode { .. }
            | Self::GetIpV4ZeroConfMode { .. }
            | Self::GetIpV4CurrentAddress { .. }
            | Self::GetIpV4StaticAddress { .. }
            | Self::GetIpV4DefaultRoute
            | Self::GetDnsIpV4NameServer { .. }
            | Self::GetDnsHostName
            | Self::GetDnsDomainName
            // E1.37-7
            | Self::GetEndpointList
            | Self::GetEndpointListChange
            | Self::GetIdentifyEndpoint { .. }
            | Self::GetEndpointToUniverse { .. }
            | Self::GetEndpointMode { .. }
            | Self::GetEndpointLabel { .. }
            | Self::GetRdmTrafficEnable { .. }
            | Self::GetDiscoveryState { .. }
            | Self::GetBackgroundDiscovery { .. }
            | Self::GetEndpointTiming { .. }
            | Self::GetEndpointTimingDescription { .. }
            | Self::GetEndpointResponders { .. }
            | Self::GetEndpointResponderListChange { .. }
            | Self::GetBindingControlFields { .. }
            | Self::GetBackgroundQueuedStatusPolicy
            | Self::GetBackgroundQueuedStatusPolicyDescription { .. }
            // E1.33
            | Self::GetComponentScope { .. }
            | Self::GetSearchDomain
            | Self::GetTcpCommsStatus
            | Self::GetBrokerStatus
            => CommandClass::GetCommand,
            // E1.20
            Self::SetCommsStatus
            | Self::SetClearStatusId
            | Self::SetSubDeviceIdStatusReportThreshold { .. }
            | Self::SetDeviceLabel { .. }
            | Self::SetFactoryDefaults
            | Self::SetLanguage { .. }
            | Self::SetDmxPersonality { .. }
            | Self::SetDmxStartAddress { .. }
            | Self::SetSensorValue { .. }
            | Self::SetRecordSensors { .. }
            | Self::SetDeviceHours { .. }
            | Self::SetLampHours { .. }
            | Self::SetLampStrikes { .. }
            | Self::SetLampState { .. }
            | Self::SetLampOnMode { .. }
            | Self::SetDevicePowerCycles { .. }
            | Self::SetDisplayInvert { .. }
            | Self::SetDisplayLevel { .. }
            | Self::SetPanInvert { .. }
            | Self::SetTiltInvert { .. }
            | Self::SetPanTiltSwap { .. }
            | Self::SetRealTimeClock { .. }
            | Self::SetIdentifyDevice { .. }
            | Self::SetResetDevice { .. }
            | Self::SetPowerState { .. }
            | Self::SetPerformSelfTest { .. }
            | Self::SetCapturePreset { .. }
            | Self::SetPresetPlayback { .. }
            // E1.37-1
            | Self::SetDmxBlockAddress { .. }
            | Self::SetDmxFailMode { .. }
            | Self::SetDmxStartupMode { .. }
            | Self::SetMinimumLevel { .. }
            | Self::SetMaximumLevel { .. }
            | Self::SetCurve { .. }
            | Self::SetOutputResponseTime { .. }
            | Self::SetModulationFrequency { .. }
            | Self::SetPowerOnSelfTest { .. }
            | Self::SetLockState { .. }
            | Self::SetLockPin { .. }
            | Self::SetBurnIn { .. }
            | Self::SetIdentifyMode { .. }
            | Self::SetPresetMergeMode { .. }
            | Self::SetPresetStatus { .. }
            // E1.37-2
            | Self::SetIpV4DhcpMode { .. }
            | Self::SetIpV4ZeroConfMode { .. }
            | Self::SetIpV4StaticAddress { .. }
            | Self::SetInterfaceApplyConfiguration { .. }
            | Self::SetInterfaceRenewDhcp { .. }
            | Self::SetInterfaceReleaseDhcp { .. }
            | Self::SetIpV4DefaultRoute { .. }
            | Self::SetDnsIpV4NameServer { .. }
            | Self::SetDnsHostName { .. }
            | Self::SetDnsDomainName { .. }
            // E1.37-7
            | Self::SetIdentifyEndpoint { .. }
            | Self::SetEndpointToUniverse { .. }
            | Self::SetEndpointMode { .. }
            | Self::SetEndpointLabel { .. }
            | Self::SetRdmTrafficEnable { .. }
            | Self::SetDiscoveryState { .. }
            | Self::SetBackgroundDiscovery { .. }
            | Self::SetEndpointTiming { .. }
            | Self::SetBackgroundQueuedStatusPolicy { .. }
            // E1.33
            | Self::SetComponentScope { .. }
            | Self::SetSearchDomain(..)
            | Self::SetTcpCommsStatus { .. }
            | Self::SetBrokerStatus { .. }
            => CommandClass::SetCommand,
            Self::RawParameter { command_class, .. } => *command_class,
        }
    }

    pub fn parameter_id(&self) -> ParameterId {
        match self {
            // E1.20
            Self::DiscMute => ParameterId::DiscMute,
            Self::DiscUnMute => ParameterId::DiscUnMute,
            Self::DiscUniqueBranch { .. } => ParameterId::DiscUniqueBranch,
            Self::GetCommsStatus | Self::SetCommsStatus => ParameterId::CommsStatus,
            Self::GetQueuedMessage { .. } => ParameterId::QueuedMessage,
            Self::GetStatusMessages { .. } => ParameterId::StatusMessages,
            Self::GetStatusIdDescription { .. } => ParameterId::StatusIdDescription,
            Self::SetClearStatusId => ParameterId::ClearStatusId,
            Self::GetSubDeviceIdStatusReportThreshold
            | Self::SetSubDeviceIdStatusReportThreshold { .. } => {
                ParameterId::SubDeviceIdStatusReportThreshold
            }
            Self::GetSupportedParameters => ParameterId::SupportedParameters,
            Self::GetParameterDescription { .. } => ParameterId::ParameterDescription,
            Self::GetDeviceInfo => ParameterId::DeviceInfo,
            Self::GetProductDetailIdList => ParameterId::ProductDetailIdList,
            Self::GetDeviceModelDescription => ParameterId::DeviceModelDescription,
            Self::GetManufacturerLabel => ParameterId::ManufacturerLabel,
            Self::GetDeviceLabel | Self::SetDeviceLabel { .. } => ParameterId::DeviceLabel,
            Self::GetFactoryDefaults | Self::SetFactoryDefaults => ParameterId::FactoryDefaults,
            Self::GetLanguageCapabilities => ParameterId::LanguageCapabilities,
            Self::GetLanguage | Self::SetLanguage { .. } => ParameterId::Language,
            Self::GetSoftwareVersionLabel => ParameterId::SoftwareVersionLabel,
            Self::GetBootSoftwareVersionId => ParameterId::BootSoftwareVersionId,
            Self::GetBootSoftwareVersionLabel => ParameterId::BootSoftwareVersionLabel,
            Self::GetDmxPersonality | Self::SetDmxPersonality { .. } => ParameterId::DmxPersonality,
            Self::GetDmxPersonalityDescription { .. } => ParameterId::DmxPersonalityDescription,
            Self::GetDmxStartAddress | Self::SetDmxStartAddress { .. } => {
                ParameterId::DmxStartAddress
            }
            Self::GetSlotInfo => ParameterId::SlotInfo,
            Self::GetSlotDescription { .. } => ParameterId::SlotDescription,
            Self::GetDefaultSlotValue => ParameterId::DefaultSlotValue,
            Self::GetSensorDefinition { .. } => ParameterId::SensorDefinition,
            Self::GetSensorValue { .. } | Self::SetSensorValue { .. } => ParameterId::SensorValue,
            Self::SetRecordSensors { .. } => ParameterId::RecordSensors,
            Self::GetDeviceHours | Self::SetDeviceHours { .. } => ParameterId::DeviceHours,
            Self::GetLampHours | Self::SetLampHours { .. } => ParameterId::LampHours,
            Self::GetLampStrikes | Self::SetLampStrikes { .. } => ParameterId::LampStrikes,
            Self::GetLampState | Self::SetLampState { .. } => ParameterId::LampState,
            Self::GetLampOnMode | Self::SetLampOnMode { .. } => ParameterId::LampOnMode,
            Self::GetDevicePowerCycles | Self::SetDevicePowerCycles { .. } => {
                ParameterId::DevicePowerCycles
            }
            Self::GetDisplayInvert | Self::SetDisplayInvert { .. } => ParameterId::DisplayInvert,
            Self::GetDisplayLevel | Self::SetDisplayLevel { .. } => ParameterId::DisplayLevel,
            Self::GetPanInvert | Self::SetPanInvert { .. } => ParameterId::PanInvert,
            Self::GetTiltInvert | Self::SetTiltInvert { .. } => ParameterId::TiltInvert,
            Self::GetPanTiltSwap | Self::SetPanTiltSwap { .. } => ParameterId::PanTiltSwap,
            Self::GetRealTimeClock | Self::SetRealTimeClock { .. } => ParameterId::RealTimeClock,
            Self::GetIdentifyDevice | Self::SetIdentifyDevice { .. } => ParameterId::IdentifyDevice,
            Self::SetResetDevice { .. } => ParameterId::ResetDevice,
            Self::GetPowerState | Self::SetPowerState { .. } => ParameterId::PowerState,
            Self::GetPerformSelfTest | Self::SetPerformSelfTest { .. } => {
                ParameterId::PerformSelfTest
            }
            Self::SetCapturePreset { .. } => ParameterId::CapturePreset,
            Self::GetSelfTestDescription { .. } => ParameterId::SelfTestDescription,
            Self::GetPresetPlayback | Self::SetPresetPlayback { .. } => ParameterId::PresetPlayback,
            // E1.37-1
            Self::GetDmxBlockAddress | Self::SetDmxBlockAddress { .. } => {
                ParameterId::DmxBlockAddress
            }
            Self::GetDmxFailMode | Self::SetDmxFailMode { .. } => ParameterId::DmxFailMode,
            Self::GetDmxStartupMode | Self::SetDmxStartupMode { .. } => ParameterId::DmxStartupMode,
            Self::GetDimmerInfo => ParameterId::DimmerInfo,
            Self::GetMinimumLevel | Self::SetMinimumLevel { .. } => ParameterId::MinimumLevel,
            Self::GetMaximumLevel | Self::SetMaximumLevel { .. } => ParameterId::MaximumLevel,
            Self::GetCurve | Self::SetCurve { .. } => ParameterId::Curve,
            Self::GetCurveDescription { .. } => ParameterId::CurveDescription,
            Self::GetOutputResponseTime | Self::SetOutputResponseTime { .. } => {
                ParameterId::OutputResponseTime
            }
            Self::GetOutputResponseTimeDescription { .. } => {
                ParameterId::OutputResponseTimeDescription
            }
            Self::GetModulationFrequency | Self::SetModulationFrequency { .. } => {
                ParameterId::ModulationFrequency
            }
            Self::GetModulationFrequencyDescription { .. } => {
                ParameterId::ModulationFrequencyDescription
            }
            Self::GetPowerOnSelfTest | Self::SetPowerOnSelfTest { .. } => {
                ParameterId::PowerOnSelfTest
            }
            Self::GetLockState | Self::SetLockState { .. } => ParameterId::LockState,
            Self::GetLockStateDescription => ParameterId::LockStateDescription,
            Self::GetLockPin | Self::SetLockPin { .. } => ParameterId::LockPin,
            Self::GetBurnIn | Self::SetBurnIn { .. } => ParameterId::BurnIn,
            Self::GetIdentifyMode | Self::SetIdentifyMode { .. } => ParameterId::IdentifyMode,
            Self::GetPresetInfo => ParameterId::PresetInfo,
            Self::GetPresetStatus { .. } | Self::SetPresetStatus { .. } => {
                ParameterId::PresetStatus
            }
            Self::GetPresetMergeMode | Self::SetPresetMergeMode { .. } => {
                ParameterId::PresetMergeMode
            }
            // E1.37-2
            Self::GetListInterfaces => ParameterId::ListInterfaces,
            Self::GetInterfaceLabel { .. } => ParameterId::InterfaceLabel,
            Self::GetInterfaceHardwareAddressType1 { .. } => {
                ParameterId::InterfaceHardwareAddressType1
            }
            Self::GetIpV4DhcpMode { .. } | Self::SetIpV4DhcpMode { .. } => {
                ParameterId::IpV4DhcpMode
            }
            Self::GetIpV4ZeroConfMode { .. } | Self::SetIpV4ZeroConfMode { .. } => {
                ParameterId::IpV4ZeroConfMode
            }
            Self::GetIpV4CurrentAddress { .. } => ParameterId::IpV4CurrentAddress,
            Self::GetIpV4StaticAddress { .. } | Self::SetIpV4StaticAddress { .. } => {
                ParameterId::IpV4StaticAddress
            }
            Self::SetInterfaceApplyConfiguration { .. } => ParameterId::InterfaceApplyConfiguration,
            Self::SetInterfaceRenewDhcp { .. } => ParameterId::InterfaceRenewDhcp,
            Self::SetInterfaceReleaseDhcp { .. } => ParameterId::InterfaceReleaseDhcp,
            Self::GetIpV4DefaultRoute | Self::SetIpV4DefaultRoute { .. } => {
                ParameterId::IpV4DefaultRoute
            }
            Self::GetDnsIpV4NameServer { .. } | Self::SetDnsIpV4NameServer { .. } => {
                ParameterId::DnsIpV4NameServer
            }
            Self::GetDnsHostName | Self::SetDnsHostName { .. } => ParameterId::DnsHostName,
            Self::GetDnsDomainName | Self::SetDnsDomainName { .. } => ParameterId::DnsDomainName,
            // E1.37-7
            Self::GetEndpointList => ParameterId::EndpointList,
            Self::GetEndpointListChange => ParameterId::EndpointListChange,
            Self::GetIdentifyEndpoint { .. } | Self::SetIdentifyEndpoint { .. } => {
                ParameterId::IdentifyEndpoint
            }
            Self::GetEndpointToUniverse { .. } | Self::SetEndpointToUniverse { .. } => {
                ParameterId::EndpointToUniverse
            }
            Self::GetEndpointMode { .. } | Self::SetEndpointMode { .. } => {
                ParameterId::EndpointMode
            }
            Self::GetEndpointLabel { .. } | Self::SetEndpointLabel { .. } => {
                ParameterId::EndpointLabel
            }
            Self::GetRdmTrafficEnable { .. } | Self::SetRdmTrafficEnable { .. } => {
                ParameterId::RdmTrafficEnable
            }
            Self::GetDiscoveryState { .. } | Self::SetDiscoveryState { .. } => {
                ParameterId::DiscoveryState
            }
            Self::GetBackgroundDiscovery { .. } | Self::SetBackgroundDiscovery { .. } => {
                ParameterId::BackgroundDiscovery
            }
            Self::GetEndpointTiming { .. } | Self::SetEndpointTiming { .. } => {
                ParameterId::EndpointTiming
            }
            Self::GetEndpointTimingDescription { .. } => ParameterId::EndpointTimingDescription,
            Self::GetEndpointResponders { .. } => ParameterId::EndpointResponders,
            Self::GetEndpointResponderListChange { .. } => ParameterId::EndpointResponderListChange,
            Self::GetBindingControlFields { .. } => ParameterId::BindingControlFields,
            Self::GetBackgroundQueuedStatusPolicy => ParameterId::BackgroundQueuedStatusPolicy,
            Self::SetBackgroundQueuedStatusPolicy { .. } => {
                ParameterId::BackgroundQueuedStatusPolicy
            }
            Self::GetBackgroundQueuedStatusPolicyDescription { .. } => {
                ParameterId::BackgroundQueuedStatusPolicyDescription
            }
            // E1.33
            Self::GetComponentScope { .. } | Self::SetComponentScope { .. } => {
                ParameterId::ComponentScope
            }
            Self::GetSearchDomain | Self::SetSearchDomain(..) => ParameterId::SearchDomain,
            Self::GetTcpCommsStatus | Self::SetTcpCommsStatus { .. } => ParameterId::TcpCommsStatus,
            Self::GetBrokerStatus | Self::SetBrokerStatus { .. } => ParameterId::BrokerStatus,
            Self::RawParameter { parameter_id, .. } => ParameterId::RawParameterId(*parameter_id),
        }
    }

    pub fn size(&self) -> usize {
        match self {
            // E1.20
            Self::DiscMute
            | Self::DiscUnMute
            | Self::GetCommsStatus
            | Self::SetCommsStatus
            | Self::SetClearStatusId
            | Self::GetSubDeviceIdStatusReportThreshold
            | Self::GetSupportedParameters
            | Self::GetDeviceInfo
            | Self::GetProductDetailIdList
            | Self::GetDeviceModelDescription
            | Self::GetManufacturerLabel
            | Self::GetDeviceLabel
            | Self::GetFactoryDefaults
            | Self::SetFactoryDefaults
            | Self::GetLanguageCapabilities
            | Self::GetLanguage
            | Self::GetSoftwareVersionLabel
            | Self::GetBootSoftwareVersionId
            | Self::GetBootSoftwareVersionLabel
            | Self::GetDmxPersonality
            | Self::GetDmxStartAddress
            | Self::GetSlotInfo
            | Self::GetDefaultSlotValue
            | Self::GetDeviceHours
            | Self::GetLampHours
            | Self::GetLampStrikes
            | Self::GetLampState
            | Self::GetLampOnMode
            | Self::GetDevicePowerCycles
            | Self::GetDisplayInvert
            | Self::GetDisplayLevel
            | Self::GetPanInvert
            | Self::GetTiltInvert
            | Self::GetPanTiltSwap
            | Self::GetRealTimeClock
            | Self::GetIdentifyDevice
            | Self::GetPowerState
            | Self::GetPerformSelfTest
            | Self::GetPresetPlayback
            | Self::GetDmxBlockAddress
            | Self::GetDmxFailMode
            | Self::GetDmxStartupMode
            | Self::GetDimmerInfo
            | Self::GetMinimumLevel
            | Self::GetMaximumLevel
            | Self::GetCurve
            | Self::GetOutputResponseTime
            | Self::GetModulationFrequency
            | Self::GetPowerOnSelfTest
            | Self::GetLockState
            | Self::GetLockStateDescription
            | Self::GetLockPin
            | Self::GetBurnIn
            | Self::GetIdentifyMode
            | Self::GetPresetInfo
            | Self::GetPresetMergeMode
            | Self::GetListInterfaces
            | Self::GetIpV4DefaultRoute
            | Self::GetDnsHostName
            | Self::GetDnsDomainName
            | Self::GetEndpointList
            | Self::GetEndpointListChange
            | Self::GetBackgroundQueuedStatusPolicy
            | Self::GetSearchDomain
            | Self::GetTcpCommsStatus
            | Self::GetBrokerStatus => 0,
            Self::GetQueuedMessage { .. }
            | Self::GetStatusMessages { .. }
            | Self::SetSubDeviceIdStatusReportThreshold { .. }
            | Self::SetDmxPersonality { .. }
            | Self::GetDmxPersonalityDescription { .. }
            | Self::GetSensorDefinition { .. }
            | Self::GetSensorValue { .. }
            | Self::SetSensorValue { .. }
            | Self::SetRecordSensors { .. }
            | Self::SetLampState { .. }
            | Self::SetLampOnMode { .. }
            | Self::SetDisplayInvert { .. }
            | Self::SetDisplayLevel { .. }
            | Self::SetPanInvert { .. }
            | Self::SetTiltInvert { .. }
            | Self::SetPanTiltSwap { .. }
            | Self::SetIdentifyDevice { .. }
            | Self::SetResetDevice { .. }
            | Self::SetPowerState { .. }
            | Self::SetPerformSelfTest { .. }
            | Self::GetSelfTestDescription { .. }
            | Self::SetCurve { .. }
            | Self::GetCurveDescription { .. }
            | Self::SetOutputResponseTime { .. }
            | Self::GetOutputResponseTimeDescription { .. }
            | Self::SetModulationFrequency { .. }
            | Self::GetModulationFrequencyDescription { .. }
            | Self::SetPowerOnSelfTest { .. }
            | Self::SetBurnIn { .. }
            | Self::SetIdentifyMode { .. }
            | Self::SetPresetMergeMode { .. }
            | Self::GetDnsIpV4NameServer { .. }
            | Self::GetEndpointTimingDescription { .. }
            | Self::SetBackgroundQueuedStatusPolicy { .. }
            | Self::GetBackgroundQueuedStatusPolicyDescription { .. }
            | Self::SetBrokerStatus { .. } => 1,
            Self::GetStatusIdDescription { .. }
            | Self::GetParameterDescription { .. }
            | Self::SetDmxStartAddress { .. }
            | Self::GetSlotDescription { .. }
            | Self::SetDmxBlockAddress { .. }
            | Self::SetMaximumLevel { .. }
            | Self::GetPresetStatus { .. }
            | Self::GetIdentifyEndpoint { .. }
            | Self::GetEndpointToUniverse { .. }
            | Self::GetEndpointMode { .. }
            | Self::GetEndpointLabel { .. }
            | Self::GetRdmTrafficEnable { .. }
            | Self::GetDiscoveryState { .. }
            | Self::GetBackgroundDiscovery { .. }
            | Self::GetEndpointTiming { .. }
            | Self::GetEndpointResponders { .. }
            | Self::GetEndpointResponderListChange { .. }
            | Self::GetComponentScope { .. } => 2,
            Self::SetPresetPlayback { .. }
            | Self::SetLockState { .. }
            | Self::SetEndpointMode { .. }
            | Self::SetRdmTrafficEnable { .. }
            | Self::SetDiscoveryState { .. }
            | Self::SetBackgroundDiscovery { .. }
            | Self::SetIdentifyEndpoint { .. }
            | Self::SetEndpointTiming { .. } => 3,
            Self::SetDeviceHours { .. }
            | Self::SetLampHours { .. }
            | Self::SetLampStrikes { .. }
            | Self::SetDevicePowerCycles { .. }
            | Self::SetLockPin { .. }
            | Self::GetInterfaceLabel { .. }
            | Self::GetInterfaceHardwareAddressType1 { .. }
            | Self::GetIpV4DhcpMode { .. }
            | Self::GetIpV4ZeroConfMode { .. }
            | Self::SetIpV4ZeroConfMode { .. }
            | Self::GetIpV4CurrentAddress { .. }
            | Self::GetIpV4StaticAddress { .. }
            | Self::SetInterfaceApplyConfiguration { .. }
            | Self::SetInterfaceRenewDhcp { .. }
            | Self::SetInterfaceReleaseDhcp { .. }
            | Self::SetEndpointToUniverse { .. } => 4,
            Self::SetMinimumLevel { .. }
            | Self::SetIpV4DhcpMode { .. }
            | Self::SetDnsIpV4NameServer { .. } => 5,
            Self::GetBindingControlFields { .. } => 6,
            Self::SetRealTimeClock { .. }
            | Self::SetDmxFailMode { .. }
            | Self::SetDmxStartupMode { .. } => 7,
            Self::SetIpV4DefaultRoute { .. } => 8,
            Self::SetPresetStatus { .. } | Self::SetIpV4StaticAddress { .. } => 9,
            Self::DiscUniqueBranch { .. } => 12,
            Self::SetCapturePreset(SetCapturePresetRequest { fade_times, .. }) => {
                if fade_times.is_some() { 8 } else { 2 }
            }
            Self::SetDeviceLabel(SetDeviceLabelRequest { device_label }) => device_label.len(),
            Self::SetLanguage(_) => 2,
            Self::SetDnsHostName(dns_hostname) => dns_hostname.len(),
            Self::SetDnsDomainName(domain_name) => domain_name.len(),
            Self::SetEndpointLabel { label, .. } => 2 + label.len(),
            Self::SetSearchDomain(search_domain) => search_domain.len(),
            Self::SetTcpCommsStatus(_) => SCOPE_MAX_LENGTH,
            Self::SetComponentScope { .. } => 25 + SCOPE_MAX_LENGTH,
            Self::RawParameter { parameter_data, .. } => parameter_data.len(),
        }
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, RdmError> {
        match self {
            // E1.20
            Self::DiscMute => {}
            Self::DiscUnMute => {}
            Self::DiscUniqueBranch {
                lower_bound_uid,
                upper_bound_uid,
            } => {
                buf[0..6].copy_from_slice(&<[u8; 6]>::from(*lower_bound_uid));
                buf[6..12].copy_from_slice(&<[u8; 6]>::from(*upper_bound_uid));
            }
            Self::GetCommsStatus => {}
            Self::SetCommsStatus => {}
            Self::GetQueuedMessage(param) => {
                param
                    .get_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetStatusMessages(param) => {
                param
                    .get_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetStatusIdDescription(param) => {
                param
                    .get_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::SetClearStatusId => {}
            Self::GetSubDeviceIdStatusReportThreshold => {}
            Self::SetSubDeviceIdStatusReportThreshold(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetSupportedParameters => {}
            Self::GetParameterDescription(param) => {
                param
                    .get_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetDeviceInfo => {}
            Self::GetProductDetailIdList => {}
            Self::GetDeviceModelDescription => {}
            Self::GetManufacturerLabel => {}
            Self::GetDeviceLabel => {}
            Self::SetDeviceLabel(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetFactoryDefaults => {}
            Self::SetFactoryDefaults => {}
            Self::GetLanguageCapabilities => {}
            Self::GetLanguage => {}
            Self::SetLanguage(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetSoftwareVersionLabel => {}
            Self::GetBootSoftwareVersionId => {}
            Self::GetBootSoftwareVersionLabel => {}
            Self::GetDmxPersonality => {}
            Self::SetDmxPersonality(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetDmxPersonalityDescription(param) => {
                param
                    .get_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetDmxStartAddress => {}
            Self::SetDmxStartAddress(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetSlotInfo => {}
            Self::GetSlotDescription(param) => {
                param
                    .get_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetDefaultSlotValue => {}
            Self::GetSensorDefinition(param) => {
                param
                    .get_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetSensorValue(param) => {
                param
                    .get_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::SetSensorValue(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::SetRecordSensors(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetDeviceHours => {}
            Self::SetDeviceHours(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetLampHours => {}
            Self::SetLampHours(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetLampStrikes => {}
            Self::SetLampStrikes(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetLampState => {}
            Self::SetLampState(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetLampOnMode => {}
            Self::SetLampOnMode(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetDevicePowerCycles => {}
            Self::SetDevicePowerCycles(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetDisplayInvert => {}
            Self::SetDisplayInvert(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetDisplayLevel => {}
            Self::SetDisplayLevel(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetPanInvert => {}
            Self::SetPanInvert(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetTiltInvert => {}
            Self::SetTiltInvert(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetPanTiltSwap => {}
            Self::SetPanTiltSwap(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetRealTimeClock => {}
            Self::SetRealTimeClock(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetIdentifyDevice => {}
            Self::SetIdentifyDevice(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::SetResetDevice(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetPowerState => {}
            Self::SetPowerState(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetPerformSelfTest => {}
            Self::SetPerformSelfTest(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::SetCapturePreset(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetSelfTestDescription(param) => {
                param
                    .get_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetPresetPlayback => {}
            Self::SetPresetPlayback(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            // E1.37-1
            Self::GetDmxBlockAddress => {}
            Self::SetDmxBlockAddress(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetDmxFailMode => {}
            Self::SetDmxFailMode {
                scene_id,
                loss_of_signal_delay_time,
                hold_time,
                level,
            } => {
                buf[0..2].copy_from_slice(&u16::from(*scene_id).to_be_bytes());
                buf[2..4].copy_from_slice(&u16::from(*loss_of_signal_delay_time).to_be_bytes());
                buf[4..6].copy_from_slice(&u16::from(*hold_time).to_be_bytes());
                buf[6] = *level;
            }
            Self::GetDmxStartupMode => {}
            Self::SetDmxStartupMode {
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
            Self::GetDimmerInfo => {}
            Self::GetMinimumLevel => {}
            Self::SetMinimumLevel(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetMaximumLevel => {}
            Self::SetMaximumLevel(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetCurve => {}
            Self::SetCurve(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetCurveDescription(param) => {
                param
                    .get_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetOutputResponseTime => {}
            Self::SetOutputResponseTime(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetOutputResponseTimeDescription(param) => {
                param
                    .get_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetModulationFrequency => {}
            Self::SetModulationFrequency(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetModulationFrequencyDescription(param) => {
                param
                    .get_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetPowerOnSelfTest => {}
            Self::SetPowerOnSelfTest { self_test_id } => {
                buf[0] = (*self_test_id).into();
            }
            Self::GetLockState => {}
            Self::SetLockState {
                pin_code,
                lock_state,
            } => {
                buf[0..2].copy_from_slice(&pin_code.0.to_be_bytes());
                buf[2] = *lock_state as u8;
            }
            Self::GetLockStateDescription => {}
            Self::GetLockPin => {}
            Self::SetLockPin {
                new_pin_code,
                current_pin_code,
            } => {
                buf[0..2].copy_from_slice(&new_pin_code.0.to_be_bytes());
                buf[2..4].copy_from_slice(&current_pin_code.0.to_be_bytes());
            }
            Self::GetBurnIn => {}
            Self::SetBurnIn(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetIdentifyMode => {}
            Self::SetIdentifyMode(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetPresetInfo => {}
            Self::GetPresetStatus(param) => {
                param
                    .get_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::SetPresetStatus(param) => {
                param
                    .set_request_encode_data(buf)
                    .map_err(RdmError::ParameterCodecError)?;
            }
            Self::GetPresetMergeMode => {}
            Self::SetPresetMergeMode { merge_mode } => {
                buf[0] = *merge_mode as u8;
            }
            // E1.37-2
            Self::GetListInterfaces => {}
            Self::GetInterfaceLabel { interface_id } => {
                buf[0..4].copy_from_slice(&interface_id.to_be_bytes());
            }
            Self::GetInterfaceHardwareAddressType1 { interface_id } => {
                buf[0..4].copy_from_slice(&interface_id.to_be_bytes());
            }
            Self::GetIpV4DhcpMode { interface_id } => {
                buf[0..4].copy_from_slice(&interface_id.to_be_bytes());
            }
            Self::SetIpV4DhcpMode {
                interface_id,
                dhcp_mode,
            } => {
                buf[0..4].copy_from_slice(&interface_id.to_be_bytes());
                buf[4] = *dhcp_mode as u8;
            }
            Self::GetIpV4ZeroConfMode { interface_id } => {
                buf[0..4].copy_from_slice(&interface_id.to_be_bytes());
            }
            Self::SetIpV4ZeroConfMode {
                interface_id,
                zero_conf_mode,
            } => {
                buf[0..4].copy_from_slice(&interface_id.to_be_bytes());
                buf[4] = *zero_conf_mode as u8;
            }
            Self::GetIpV4CurrentAddress { interface_id } => {
                buf[0..4].copy_from_slice(&interface_id.to_be_bytes());
            }
            Self::GetIpV4StaticAddress { interface_id } => {
                buf[0..4].copy_from_slice(&interface_id.to_be_bytes());
            }
            Self::SetIpV4StaticAddress {
                interface_id,
                address,
                netmask,
            } => {
                buf[0..4].copy_from_slice(&interface_id.to_be_bytes());
                buf[4..8].copy_from_slice(&<[u8; 4]>::from(*address));
                buf[8] = *netmask;
            }
            Self::SetInterfaceApplyConfiguration { interface_id } => {
                buf[0..4].copy_from_slice(&interface_id.to_be_bytes());
            }
            Self::SetInterfaceRenewDhcp { interface_id } => {
                buf[0..4].copy_from_slice(&interface_id.to_be_bytes());
            }
            Self::SetInterfaceReleaseDhcp { interface_id } => {
                buf[0..4].copy_from_slice(&interface_id.to_be_bytes());
            }
            Self::GetIpV4DefaultRoute => {}
            Self::SetIpV4DefaultRoute {
                interface_id,
                ipv4_default_route,
            } => {
                buf[0..4].copy_from_slice(&interface_id.to_be_bytes());
                buf[4..8].copy_from_slice(&<[u8; 4]>::from(*ipv4_default_route));
            }
            Self::GetDnsIpV4NameServer { name_server_index } => {
                buf[0] = *name_server_index;
            }
            Self::SetDnsIpV4NameServer {
                name_server_index,
                name_server_address,
            } => {
                buf[0] = *name_server_index;
                buf[1..5].copy_from_slice(&<[u8; 4]>::from(*name_server_address));
            }
            Self::GetDnsHostName => {}
            Self::SetDnsHostName(dns_hostname) => {
                dns_hostname.encode(buf)?;
            }
            Self::GetDnsDomainName => {}
            Self::SetDnsDomainName(domain_name) => {
                domain_name.encode(buf)?;
            }
            // E1.37-7
            Self::GetEndpointList => {}
            Self::GetEndpointListChange => {}
            Self::GetIdentifyEndpoint { endpoint_id } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
            }
            Self::SetIdentifyEndpoint {
                endpoint_id,
                identify,
            } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
                buf[2] = *identify as u8;
            }
            Self::GetEndpointToUniverse { endpoint_id } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
            }
            Self::SetEndpointToUniverse {
                endpoint_id,
                universe,
            } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
                buf[2..4].copy_from_slice(&universe.to_be_bytes());
            }
            Self::GetEndpointMode { endpoint_id } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
            }
            Self::SetEndpointMode { endpoint_id, mode } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
                buf[2] = *mode as u8;
            }
            Self::GetEndpointLabel { endpoint_id } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
            }
            Self::SetEndpointLabel { endpoint_id, label } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
                label.encode(&mut buf[2..2 + label.len()])?;
            }
            Self::GetRdmTrafficEnable { endpoint_id } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
            }
            Self::SetRdmTrafficEnable {
                endpoint_id,
                enable,
            } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
                buf[2] = *enable as u8;
            }
            Self::GetDiscoveryState { endpoint_id } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
            }
            Self::SetDiscoveryState { endpoint_id, state } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
                buf[2] = (*state).into();
            }
            Self::GetBackgroundDiscovery { endpoint_id } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
            }
            Self::SetBackgroundDiscovery {
                endpoint_id,
                enable,
            } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
                buf[2] = *enable as u8;
            }
            Self::GetEndpointTiming { endpoint_id } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
            }
            Self::SetEndpointTiming {
                endpoint_id,
                setting_id,
            } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
                buf[2] = *setting_id;
            }
            Self::GetEndpointTimingDescription { setting_id } => {
                buf[0] = *setting_id;
            }
            Self::GetEndpointResponders { endpoint_id } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
            }
            Self::GetEndpointResponderListChange { endpoint_id } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
            }
            Self::GetBindingControlFields { endpoint_id, uid } => {
                buf[0..2].copy_from_slice(&u16::from(*endpoint_id).to_be_bytes());
                buf[2..6].copy_from_slice(&<[u8; 6]>::from(*uid));
            }
            Self::GetBackgroundQueuedStatusPolicy => {}
            Self::SetBackgroundQueuedStatusPolicy { policy_id } => {
                buf[0] = *policy_id;
            }
            Self::GetBackgroundQueuedStatusPolicyDescription { policy_id } => {
                buf[0] = *policy_id;
            }
            // E1.33
            Self::GetComponentScope { scope_slot } => {
                buf[0..2].copy_from_slice(&scope_slot.to_be_bytes());
            }
            Self::SetComponentScope {
                scope_slot,
                scope_string,
                static_config_type,
                static_broker_ipv4_address,
                static_broker_ipv6_address,
                static_broker_port,
            } => {
                buf[0..2].copy_from_slice(&scope_slot.to_be_bytes());
                scope_string.encode(&mut buf[2..2 + SCOPE_MAX_LENGTH])?;
                buf[2 + SCOPE_MAX_LENGTH] = *static_config_type as u8;
                buf[3 + SCOPE_MAX_LENGTH..7 + SCOPE_MAX_LENGTH]
                    .copy_from_slice(&<[u8; 4]>::from(*static_broker_ipv4_address));
                buf[7 + SCOPE_MAX_LENGTH..23 + SCOPE_MAX_LENGTH]
                    .copy_from_slice(&<[u8; 16]>::from(*static_broker_ipv6_address));
                buf[23 + SCOPE_MAX_LENGTH..25 + SCOPE_MAX_LENGTH]
                    .copy_from_slice(&(*static_broker_port).to_be_bytes());
            }
            Self::GetSearchDomain => {}
            Self::SetSearchDomain(search_domain) => {
                search_domain.encode(buf)?;
            }
            Self::GetTcpCommsStatus => {}
            Self::SetTcpCommsStatus(scope_string) => {
                scope_string.encode(buf)?;
            }
            Self::GetBrokerStatus => {}
            Self::SetBrokerStatus { broker_state } => {
                buf[0] = *broker_state as u8;
            }
            Self::RawParameter { parameter_data, .. } => {
                buf[0..parameter_data.len()].copy_from_slice(parameter_data);
            }
        };

        Ok(self.size())
    }

    pub fn decode(
        command_class: CommandClass,
        parameter_id: ParameterId,
        bytes: &[u8],
    ) -> Result<Self, RdmError> {
        match (command_class, parameter_id) {
            // E1.20
            (CommandClass::DiscoveryCommand, ParameterId::DiscMute) => Ok(Self::DiscMute),
            (CommandClass::DiscoveryCommand, ParameterId::DiscUnMute) => Ok(Self::DiscUnMute),
            (CommandClass::DiscoveryCommand, ParameterId::DiscUniqueBranch) => {
                if bytes.len() < 12 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }

                let lower_bound_uid = DeviceUID::from(<[u8; 6]>::try_from(&bytes[0..=5])?);
                let upper_bound_uid = DeviceUID::from(<[u8; 6]>::try_from(&bytes[6..=11])?);

                Ok(Self::DiscUniqueBranch {
                    lower_bound_uid,
                    upper_bound_uid,
                })
            }
            (CommandClass::GetCommand, ParameterId::CommsStatus) => Ok(Self::GetCommsStatus),
            (CommandClass::SetCommand, ParameterId::CommsStatus) => Ok(Self::SetCommsStatus),
            (CommandClass::GetCommand, ParameterId::QueuedMessage) => Ok(Self::GetQueuedMessage(
                GetQueuedMessageRequest::get_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::StatusMessages) => Ok(Self::GetStatusMessages(
                GetStatusMessagesRequest::get_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::StatusIdDescription) => {
                Ok(Self::GetStatusIdDescription(
                    GetStatusIdDescriptionRequest::get_request_decode_data(bytes)?,
                ))
            }
            (CommandClass::SetCommand, ParameterId::ClearStatusId) => Ok(Self::SetClearStatusId),
            (CommandClass::GetCommand, ParameterId::SubDeviceIdStatusReportThreshold) => {
                Ok(Self::GetSubDeviceIdStatusReportThreshold)
            }
            (CommandClass::SetCommand, ParameterId::SubDeviceIdStatusReportThreshold) => {
                Ok(Self::SetSubDeviceIdStatusReportThreshold(
                    SetSubDeviceIdStatusReportThresholdRequest::set_request_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommand, ParameterId::SupportedParameters) => {
                Ok(Self::GetSupportedParameters)
            }
            (CommandClass::GetCommand, ParameterId::ParameterDescription) => {
                Ok(Self::GetParameterDescription(
                    GetParameterDescriptionRequest::get_request_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommand, ParameterId::DeviceInfo) => Ok(Self::GetDeviceInfo),
            (CommandClass::GetCommand, ParameterId::ProductDetailIdList) => {
                Ok(Self::GetProductDetailIdList)
            }
            (CommandClass::GetCommand, ParameterId::DeviceModelDescription) => {
                Ok(Self::GetDeviceModelDescription)
            }
            (CommandClass::GetCommand, ParameterId::ManufacturerLabel) => {
                Ok(Self::GetManufacturerLabel)
            }
            (CommandClass::GetCommand, ParameterId::DeviceLabel) => Ok(Self::GetDeviceLabel),
            (CommandClass::SetCommand, ParameterId::DeviceLabel) => Ok(Self::SetDeviceLabel(
                SetDeviceLabelRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::FactoryDefaults) => {
                Ok(Self::GetFactoryDefaults)
            }
            (CommandClass::SetCommand, ParameterId::FactoryDefaults) => {
                Ok(Self::SetFactoryDefaults)
            }
            (CommandClass::GetCommand, ParameterId::LanguageCapabilities) => {
                Ok(Self::GetLanguageCapabilities)
            }
            (CommandClass::GetCommand, ParameterId::Language) => Ok(Self::GetLanguage),
            (CommandClass::SetCommand, ParameterId::Language) => Ok(Self::SetLanguage(
                SetLanguageRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::SoftwareVersionLabel) => {
                Ok(Self::GetSoftwareVersionLabel)
            }
            (CommandClass::GetCommand, ParameterId::BootSoftwareVersionId) => {
                Ok(Self::GetBootSoftwareVersionId)
            }
            (CommandClass::GetCommand, ParameterId::BootSoftwareVersionLabel) => {
                Ok(Self::GetBootSoftwareVersionLabel)
            }
            (CommandClass::GetCommand, ParameterId::DmxPersonality) => Ok(Self::GetDmxPersonality),
            (CommandClass::SetCommand, ParameterId::DmxPersonality) => Ok(Self::SetDmxPersonality(
                SetDmxPersonalityRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::DmxPersonalityDescription) => {
                Ok(Self::GetDmxPersonalityDescription(
                    GetDmxPersonalityDescriptionRequest::get_request_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommand, ParameterId::DmxStartAddress) => {
                Ok(Self::GetDmxStartAddress)
            }
            (CommandClass::SetCommand, ParameterId::DmxStartAddress) => {
                Ok(Self::SetDmxStartAddress(
                    SetDmxStartAddressRequest::set_request_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommand, ParameterId::SlotInfo) => Ok(Self::GetSlotInfo),
            (CommandClass::GetCommand, ParameterId::SlotDescription) => {
                Ok(Self::GetSlotDescription(
                    GetSlotDescriptionRequest::get_request_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommand, ParameterId::DefaultSlotValue) => {
                Ok(Self::GetDefaultSlotValue)
            }
            (CommandClass::GetCommand, ParameterId::SensorDefinition) => {
                Ok(Self::GetSensorDefinition(
                    GetSensorDefinitionRequest::get_request_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommand, ParameterId::SensorValue) => Ok(Self::GetSensorValue(
                GetSensorValueRequest::get_request_decode_data(bytes)?,
            )),
            (CommandClass::SetCommand, ParameterId::SensorValue) => Ok(Self::SetSensorValue(
                SetSensorValueRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::SetCommand, ParameterId::RecordSensors) => Ok(Self::SetRecordSensors(
                SetRecordSensorsRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::DeviceHours) => Ok(Self::GetDeviceHours),
            (CommandClass::SetCommand, ParameterId::DeviceHours) => Ok(Self::SetDeviceHours(
                SetDeviceHoursRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::LampHours) => Ok(Self::GetLampHours),
            (CommandClass::SetCommand, ParameterId::LampHours) => Ok(Self::SetLampHours(
                SetLampHoursRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::LampStrikes) => Ok(Self::GetLampStrikes),
            (CommandClass::SetCommand, ParameterId::LampStrikes) => Ok(Self::SetLampStrikes(
                SetLampStrikesRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::LampState) => Ok(Self::GetLampState),
            (CommandClass::SetCommand, ParameterId::LampState) => Ok(Self::SetLampState(
                SetLampStateRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::LampOnMode) => Ok(Self::GetLampOnMode),
            (CommandClass::SetCommand, ParameterId::LampOnMode) => Ok(Self::SetLampOnMode(
                SetLampOnModeRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::DevicePowerCycles) => {
                Ok(Self::GetDevicePowerCycles)
            }
            (CommandClass::SetCommand, ParameterId::DevicePowerCycles) => {
                Ok(Self::SetDevicePowerCycles(
                    SetDevicePowerCyclesRequest::set_request_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommand, ParameterId::DisplayInvert) => Ok(Self::GetDisplayInvert),
            (CommandClass::SetCommand, ParameterId::DisplayInvert) => Ok(Self::SetDisplayInvert(
                SetDisplayInvertModeRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::DisplayLevel) => Ok(Self::GetDisplayLevel),
            (CommandClass::SetCommand, ParameterId::DisplayLevel) => Ok(Self::SetDisplayLevel(
                SetDisplayLevelRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::PanInvert) => Ok(Self::GetPanInvert),
            (CommandClass::SetCommand, ParameterId::PanInvert) => Ok(Self::SetPanInvert(
                SetPanInvertRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::TiltInvert) => Ok(Self::GetTiltInvert),
            (CommandClass::SetCommand, ParameterId::TiltInvert) => Ok(Self::SetTiltInvert(
                SetTiltInvertRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::PanTiltSwap) => Ok(Self::GetPanTiltSwap),
            (CommandClass::SetCommand, ParameterId::PanTiltSwap) => Ok(Self::SetPanTiltSwap(
                SetPanTiltSwapRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::RealTimeClock) => Ok(Self::GetRealTimeClock),
            (CommandClass::SetCommand, ParameterId::RealTimeClock) => Ok(Self::SetRealTimeClock(
                SetRealTimeClockRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::IdentifyDevice) => Ok(Self::GetIdentifyDevice),
            (CommandClass::SetCommand, ParameterId::IdentifyDevice) => Ok(Self::SetIdentifyDevice(
                SetIdentifyDeviceRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::SetCommand, ParameterId::ResetDevice) => Ok(Self::SetResetDevice(
                SetResetDeviceRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::PowerState) => Ok(Self::GetPowerState),
            (CommandClass::SetCommand, ParameterId::PowerState) => Ok(Self::SetPowerState(
                SetPowerStateRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::PerformSelfTest) => {
                Ok(Self::GetPerformSelfTest)
            }
            (CommandClass::SetCommand, ParameterId::PerformSelfTest) => {
                Ok(Self::SetPerformSelfTest(
                    SetPerformSelfTestRequest::set_request_decode_data(bytes)?,
                ))
            }
            (CommandClass::SetCommand, ParameterId::CapturePreset) => Ok(Self::SetCapturePreset(
                SetCapturePresetRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::SelfTestDescription) => {
                Ok(Self::GetSelfTestDescription(
                    GetSelfTestDescriptionRequest::get_request_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommand, ParameterId::PresetPlayback) => Ok(Self::GetPresetPlayback),
            (CommandClass::SetCommand, ParameterId::PresetPlayback) => Ok(Self::SetPresetPlayback(
                SetPresetPlaybackRequest::set_request_decode_data(bytes)?,
            )),
            // E1.37-1
            (CommandClass::GetCommand, ParameterId::DmxBlockAddress) => {
                Ok(Self::GetDmxBlockAddress)
            }
            (CommandClass::SetCommand, ParameterId::DmxBlockAddress) => {
                Ok(Self::SetDmxBlockAddress(
                    SetDmxBlockAddressRequest::set_request_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommand, ParameterId::DmxFailMode) => Ok(Self::GetDmxFailMode),
            (CommandClass::SetCommand, ParameterId::DmxFailMode) => {
                if bytes.len() < 7 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetDmxFailMode {
                    scene_id: u16::from_be_bytes([bytes[0], bytes[1]]).into(),
                    loss_of_signal_delay_time: u16::from_be_bytes([bytes[2], bytes[3]]).into(),
                    hold_time: u16::from_be_bytes([bytes[4], bytes[5]]).into(),
                    level: bytes[6],
                })
            }
            (CommandClass::GetCommand, ParameterId::DmxStartupMode) => Ok(Self::GetDmxStartupMode),
            (CommandClass::SetCommand, ParameterId::DmxStartupMode) => {
                if bytes.len() < 7 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetDmxStartupMode {
                    scene_id: u16::from_be_bytes([bytes[0], bytes[1]]).into(),
                    startup_delay: u16::from_be_bytes([bytes[2], bytes[3]]).into(),
                    hold_time: u16::from_be_bytes([bytes[4], bytes[5]]).into(),
                    level: bytes[6],
                })
            }
            (CommandClass::GetCommand, ParameterId::DimmerInfo) => Ok(Self::GetDimmerInfo),
            (CommandClass::GetCommand, ParameterId::MinimumLevel) => Ok(Self::GetMinimumLevel),
            (CommandClass::SetCommand, ParameterId::MinimumLevel) => Ok(Self::SetMinimumLevel(
                SetMinimumLevelRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::MaximumLevel) => Ok(Self::GetMaximumLevel),
            (CommandClass::SetCommand, ParameterId::MaximumLevel) => Ok(Self::SetMaximumLevel(
                SetMaximumLevelRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::Curve) => Ok(Self::GetCurve),
            (CommandClass::SetCommand, ParameterId::Curve) => Ok(Self::SetCurve(
                SetCurveRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::CurveDescription) => {
                Ok(Self::GetCurveDescription(
                    GetCurveDescriptionRequest::get_request_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommand, ParameterId::OutputResponseTime) => {
                Ok(Self::GetOutputResponseTime)
            }
            (CommandClass::SetCommand, ParameterId::OutputResponseTime) => {
                Ok(Self::SetOutputResponseTime(
                    SetOutputResponseTimeRequest::set_request_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommand, ParameterId::OutputResponseTimeDescription) => {
                Ok(Self::GetOutputResponseTimeDescription(
                    GetOutputResponseTimeDescriptionRequest::get_request_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommand, ParameterId::ModulationFrequency) => {
                Ok(Self::GetModulationFrequency)
            }
            (CommandClass::SetCommand, ParameterId::ModulationFrequency) => {
                Ok(Self::SetModulationFrequency(
                    SetModulationFrequencyRequest::set_request_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommand, ParameterId::ModulationFrequencyDescription) => {
                Ok(Self::GetModulationFrequencyDescription(
                    GetModulationFrequencyDescriptionRequest::get_request_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommand, ParameterId::PowerOnSelfTest) => {
                Ok(Self::GetPowerOnSelfTest)
            }
            (CommandClass::SetCommand, ParameterId::PowerOnSelfTest) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetPowerOnSelfTest {
                    self_test_id: bytes[0].into(),
                })
            }
            (CommandClass::GetCommand, ParameterId::LockState) => Ok(Self::GetLockState),
            (CommandClass::SetCommand, ParameterId::LockState) => {
                if bytes.len() < 5 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetLockState {
                    pin_code: u16::from_be_bytes([bytes[0], bytes[1]]).try_into()?,
                    lock_state: bytes[4] != 0,
                })
            }
            (CommandClass::GetCommand, ParameterId::LockStateDescription) => {
                Ok(Self::GetLockStateDescription)
            }
            (CommandClass::GetCommand, ParameterId::LockPin) => Ok(Self::GetLockPin),
            (CommandClass::SetCommand, ParameterId::LockPin) => {
                if bytes.len() < 8 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetLockPin {
                    new_pin_code: u16::from_be_bytes([bytes[0], bytes[1]]).try_into()?,
                    current_pin_code: u16::from_be_bytes([bytes[2], bytes[3]]).try_into()?,
                })
            }
            (CommandClass::GetCommand, ParameterId::BurnIn) => Ok(Self::GetBurnIn),
            (CommandClass::SetCommand, ParameterId::BurnIn) => Ok(Self::SetBurnIn(
                SetBurnInRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::IdentifyMode) => Ok(Self::GetIdentifyMode),
            (CommandClass::SetCommand, ParameterId::IdentifyMode) => Ok(Self::SetIdentifyMode(
                SetIdentifyModeRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::PresetInfo) => Ok(Self::GetPresetInfo),
            (CommandClass::GetCommand, ParameterId::PresetStatus) => Ok(Self::GetPresetStatus(
                GetPresetStatusRequest::get_request_decode_data(bytes)?,
            )),
            (CommandClass::SetCommand, ParameterId::PresetStatus) => Ok(Self::SetPresetStatus(
                SetPresetStatusRequest::set_request_decode_data(bytes)?,
            )),
            (CommandClass::GetCommand, ParameterId::PresetMergeMode) => {
                Ok(Self::GetPresetMergeMode)
            }
            (CommandClass::SetCommand, ParameterId::PresetMergeMode) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetPresetMergeMode {
                    merge_mode: bytes[0].try_into()?,
                })
            }
            // E1.37-2
            (CommandClass::GetCommand, ParameterId::ListInterfaces) => Ok(Self::GetListInterfaces),
            (CommandClass::GetCommand, ParameterId::InterfaceLabel) => {
                if bytes.len() < 4 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetInterfaceLabel {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                })
            }
            (CommandClass::GetCommand, ParameterId::InterfaceHardwareAddressType1) => {
                if bytes.len() < 4 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetInterfaceHardwareAddressType1 {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                })
            }
            (CommandClass::GetCommand, ParameterId::IpV4DhcpMode) => {
                if bytes.len() < 4 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetIpV4DhcpMode {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                })
            }
            (CommandClass::SetCommand, ParameterId::IpV4DhcpMode) => {
                if bytes.len() < 5 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetIpV4DhcpMode {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                    dhcp_mode: bytes[4] != 0,
                })
            }
            (CommandClass::GetCommand, ParameterId::IpV4ZeroConfMode) => {
                if bytes.len() < 4 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetIpV4ZeroConfMode {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                })
            }
            (CommandClass::SetCommand, ParameterId::IpV4ZeroConfMode) => {
                if bytes.len() < 5 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetIpV4ZeroConfMode {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                    zero_conf_mode: bytes[4] != 0,
                })
            }
            (CommandClass::GetCommand, ParameterId::IpV4CurrentAddress) => {
                if bytes.len() < 4 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetIpV4CurrentAddress {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                })
            }
            (CommandClass::GetCommand, ParameterId::IpV4StaticAddress) => {
                if bytes.len() < 4 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetIpV4StaticAddress {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                })
            }
            (CommandClass::SetCommand, ParameterId::IpV4StaticAddress) => {
                if bytes.len() < 9 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetIpV4StaticAddress {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                    address: Ipv4Address::from([bytes[4], bytes[5], bytes[6], bytes[7]]),
                    netmask: bytes[8],
                })
            }
            (CommandClass::SetCommand, ParameterId::InterfaceApplyConfiguration) => {
                if bytes.len() < 4 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetInterfaceApplyConfiguration {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                })
            }
            (CommandClass::SetCommand, ParameterId::InterfaceRenewDhcp) => {
                if bytes.len() < 4 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetInterfaceRenewDhcp {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                })
            }
            (CommandClass::SetCommand, ParameterId::InterfaceReleaseDhcp) => {
                if bytes.len() < 4 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetInterfaceReleaseDhcp {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                })
            }
            (CommandClass::GetCommand, ParameterId::IpV4DefaultRoute) => {
                Ok(Self::GetIpV4DefaultRoute)
            }
            (CommandClass::SetCommand, ParameterId::IpV4DefaultRoute) => {
                if bytes.len() < 8 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetIpV4DefaultRoute {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                    ipv4_default_route: Ipv4Route::from([bytes[4], bytes[5], bytes[6], bytes[7]]),
                })
            }
            (CommandClass::GetCommand, ParameterId::DnsIpV4NameServer) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetDnsIpV4NameServer {
                    name_server_index: bytes[0],
                })
            }
            (CommandClass::SetCommand, ParameterId::DnsIpV4NameServer) => {
                if bytes.len() < 5 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetDnsIpV4NameServer {
                    name_server_index: bytes[0],
                    name_server_address: Ipv4Address::from([
                        bytes[1], bytes[2], bytes[3], bytes[4],
                    ]),
                })
            }
            (CommandClass::GetCommand, ParameterId::DnsHostName) => Ok(Self::GetDnsHostName),
            (CommandClass::SetCommand, ParameterId::DnsHostName) => {
                Ok(Self::SetDnsHostName(DnsHostName::decode(bytes)?))
            }
            (CommandClass::GetCommand, ParameterId::DnsDomainName) => Ok(Self::GetDnsDomainName),
            (CommandClass::SetCommand, ParameterId::DnsDomainName) => {
                Ok(Self::SetDnsDomainName(DnsDomainName::decode(bytes)?))
            }
            // E1.37-7
            (CommandClass::GetCommand, ParameterId::EndpointList) => Ok(Self::GetEndpointList),
            (CommandClass::GetCommand, ParameterId::EndpointListChange) => {
                Ok(Self::GetEndpointListChange)
            }
            (CommandClass::GetCommand, ParameterId::IdentifyEndpoint) => {
                Ok(Self::GetIdentifyEndpoint {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                })
            }
            (CommandClass::SetCommand, ParameterId::IdentifyEndpoint) => {
                Ok(Self::SetIdentifyEndpoint {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                    identify: bytes[2] != 0,
                })
            }
            (CommandClass::GetCommand, ParameterId::EndpointToUniverse) => {
                Ok(Self::GetEndpointToUniverse {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                })
            }
            (CommandClass::SetCommand, ParameterId::EndpointToUniverse) => {
                Ok(Self::SetEndpointToUniverse {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                    universe: u16::from_be_bytes(bytes[2..=3].try_into()?),
                })
            }
            (CommandClass::GetCommand, ParameterId::EndpointMode) => Ok(Self::GetEndpointMode {
                endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
            }),
            (CommandClass::SetCommand, ParameterId::EndpointMode) => Ok(Self::SetEndpointMode {
                endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                mode: bytes[2].try_into()?,
            }),
            (CommandClass::GetCommand, ParameterId::EndpointLabel) => Ok(Self::GetEndpointLabel {
                endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
            }),
            (CommandClass::SetCommand, ParameterId::EndpointLabel) => Ok(Self::SetEndpointLabel {
                endpoint_id: u16::from_be_bytes(bytes[0..2].try_into()?).into(),
                label: EndpointLabel::decode(&bytes[2..])?,
            }),
            (CommandClass::GetCommand, ParameterId::RdmTrafficEnable) => {
                Ok(Self::GetRdmTrafficEnable {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                })
            }
            (CommandClass::SetCommand, ParameterId::RdmTrafficEnable) => {
                Ok(Self::SetRdmTrafficEnable {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                    enable: bytes[2] == 1,
                })
            }
            (CommandClass::GetCommand, ParameterId::DiscoveryState) => {
                Ok(Self::GetDiscoveryState {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                })
            }
            (CommandClass::SetCommand, ParameterId::DiscoveryState) => {
                Ok(Self::SetDiscoveryState {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                    state: bytes[2].try_into()?,
                })
            }
            (CommandClass::GetCommand, ParameterId::BackgroundDiscovery) => {
                Ok(Self::GetBackgroundDiscovery {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                })
            }
            (CommandClass::SetCommand, ParameterId::BackgroundDiscovery) => {
                Ok(Self::SetBackgroundDiscovery {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                    enable: bytes[2] == 1,
                })
            }
            (CommandClass::GetCommand, ParameterId::EndpointTiming) => {
                Ok(Self::GetEndpointTiming {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                })
            }
            (CommandClass::SetCommand, ParameterId::EndpointTiming) => {
                Ok(Self::SetEndpointTiming {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                    setting_id: bytes[2],
                })
            }
            (CommandClass::GetCommand, ParameterId::EndpointTimingDescription) => {
                Ok(Self::GetEndpointTimingDescription {
                    setting_id: bytes[0],
                })
            }
            (CommandClass::GetCommand, ParameterId::EndpointResponders) => {
                Ok(Self::GetEndpointResponders {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                })
            }
            (CommandClass::GetCommand, ParameterId::EndpointResponderListChange) => {
                Ok(Self::GetEndpointResponderListChange {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                })
            }
            (CommandClass::GetCommand, ParameterId::BindingControlFields) => {
                Ok(Self::GetBindingControlFields {
                    endpoint_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                    uid: DeviceUID::new(
                        u16::from_be_bytes(bytes[2..=3].try_into()?),
                        u32::from_be_bytes(bytes[4..=7].try_into()?),
                    ),
                })
            }
            (CommandClass::GetCommand, ParameterId::BackgroundQueuedStatusPolicy) => {
                Ok(Self::GetBackgroundQueuedStatusPolicy)
            }
            (CommandClass::SetCommand, ParameterId::BackgroundQueuedStatusPolicy) => {
                Ok(Self::SetBackgroundQueuedStatusPolicy {
                    policy_id: bytes[0],
                })
            }
            (CommandClass::GetCommand, ParameterId::BackgroundQueuedStatusPolicyDescription) => {
                Ok(Self::GetBackgroundQueuedStatusPolicyDescription {
                    policy_id: bytes[0],
                })
            }
            // E1.33
            (CommandClass::GetCommand, ParameterId::ComponentScope) => {
                Ok(Self::GetComponentScope {
                    scope_slot: u16::from_be_bytes([bytes[0], bytes[1]]),
                })
            }
            (CommandClass::SetCommand, ParameterId::ComponentScope) => {
                Ok(Self::SetComponentScope {
                    scope_slot: u16::from_be_bytes([bytes[0], bytes[1]]),
                    scope_string: Scope::decode(&bytes[2..66])?,
                    static_config_type: bytes[66].try_into()?,
                    static_broker_ipv4_address: Ipv4Address::from([
                        bytes[67], bytes[68], bytes[69], bytes[70],
                    ]),
                    static_broker_ipv6_address: Ipv6Address::from([
                        bytes[71], bytes[72], bytes[73], bytes[74], bytes[75], bytes[76],
                        bytes[77], bytes[78], bytes[79], bytes[80], bytes[81], bytes[82],
                        bytes[83], bytes[84], bytes[85], bytes[86],
                    ]),
                    static_broker_port: u16::from_be_bytes([bytes[87], bytes[88]]),
                })
            }
            (CommandClass::GetCommand, ParameterId::SearchDomain) => Ok(Self::GetSearchDomain),
            (CommandClass::SetCommand, ParameterId::SearchDomain) => {
                Ok(Self::SetSearchDomain(SearchDomain::decode(bytes)?))
            }
            (CommandClass::GetCommand, ParameterId::TcpCommsStatus) => Ok(Self::GetTcpCommsStatus),
            (CommandClass::SetCommand, ParameterId::TcpCommsStatus) => {
                Ok(Self::SetTcpCommsStatus(Scope::decode(bytes)?))
            }
            (CommandClass::GetCommand, ParameterId::BrokerStatus) => Ok(Self::GetBrokerStatus),
            (CommandClass::SetCommand, ParameterId::BrokerStatus) => {
                if bytes.len() < 2 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetBrokerStatus {
                    broker_state: bytes[0].try_into()?,
                })
            }
            (command_class, parameter_id) => Ok(Self::RawParameter {
                command_class,
                parameter_id: parameter_id.into(),
                parameter_data: Vec::from_slice(bytes).unwrap(),
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RdmRequest {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub port_id: u8,
    pub sub_device_id: SubDeviceId,
    pub parameter: RequestParameter,
}

impl RdmRequest {
    pub fn new(
        destination_uid: DeviceUID,
        source_uid: DeviceUID,
        transaction_number: u8,
        port_id: u8,
        sub_device_id: SubDeviceId,
        parameter: RequestParameter,
    ) -> Self {
        RdmRequest {
            destination_uid,
            source_uid,
            transaction_number,
            port_id,
            sub_device_id,
            parameter,
        }
    }

    pub fn command_class(&self) -> CommandClass {
        self.parameter.command_class()
    }

    pub fn parameter_id(&self) -> ParameterId {
        self.parameter.parameter_id()
    }

    pub fn size(&self) -> usize {
        24 + self.parameter.size() + 2 // 24 bytes header + parameter data + 2 bytes CRC
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, RdmError> {
        buf[0] = RDM_START_CODE_BYTE;
        buf[1] = RDM_SUB_START_CODE_BYTE;

        let parameter_data_length = self.parameter.encode(&mut buf[24..])?;
        let message_length = 24 + parameter_data_length;

        buf[2] = message_length as u8;
        buf[3..9].copy_from_slice(&<[u8; 6]>::from(self.destination_uid));
        buf[9..15].copy_from_slice(&<[u8; 6]>::from(self.source_uid));
        buf[15] = self.transaction_number;
        buf[16] = self.port_id;
        buf[17] = 0x00; // Message Count shall be set to 0x00 in all controller generated requests
        buf[18..20].copy_from_slice(&u16::from(self.sub_device_id).to_be_bytes());
        buf[20] = self.parameter.command_class() as u8;
        buf[21..23].copy_from_slice(&u16::from(self.parameter.parameter_id()).to_be_bytes());
        buf[23] = parameter_data_length as u8;

        let crc = bsd_16_crc(&buf[0..message_length]);

        buf[message_length..message_length + 2].copy_from_slice(&crc.to_be_bytes());

        Ok(message_length + 2)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, RdmError> {
        if bytes.len() < 24 {
            return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
        }

        let destination_uid = DeviceUID::from(<[u8; 6]>::try_from(&bytes[3..=8])?);
        let source_uid = DeviceUID::from(<[u8; 6]>::try_from(&bytes[9..=14])?);

        let transaction_number = bytes[15];
        let port_id = bytes[16];
        let sub_device_id = u16::from_be_bytes([bytes[18], bytes[19]]).into();
        let command_class = bytes[20].try_into()?;
        let parameter_id = u16::from_be_bytes([bytes[21], bytes[22]]).into();
        let parameter_data_length = bytes[23] as usize;

        if bytes.len() < 24 + parameter_data_length {
            return Err(RdmError::InvalidFrameLength(bytes.len() as u8));
        }

        let parameter_data = &bytes[24..24 + parameter_data_length];

        let parameter = RequestParameter::decode(command_class, parameter_id, parameter_data)?;

        Ok(Self::new(
            destination_uid,
            source_uid,
            transaction_number,
            port_id,
            sub_device_id,
            parameter,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_encode_discovery_unique_branch_request() {
        let mut encoded = [0u8; 38];

        let bytes_written = RdmRequest::new(
            DeviceUID::new(0x0102, 0x03040506),
            DeviceUID::new(0x0605, 0x04030201),
            0x00,
            0x01,
            SubDeviceId::Id(0x01),
            RequestParameter::DiscUniqueBranch {
                lower_bound_uid: DeviceUID::new(0x0000, 0x00000000),
                upper_bound_uid: DeviceUID::new(0xffff, 0xffffffff),
            },
        )
        .encode(&mut encoded)
        .unwrap();

        assert_eq!(bytes_written, 38);

        let expected: &[u8; 38] = &[
            0xcc, // Start Code
            0x01, // Sub Start Code
            0x24, // Message Length
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // Destination UID
            0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // Source UID
            0x00, // Transaction Number
            0x01, // Port ID
            0x00, // Message Count
            0x00, 0x01, // Sub-Device ID
            0x10, // Command Class
            0x00, 0x01, // Parameter ID
            0x0c, // PDL
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Parameter Data - Lower Bound UID
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // Parameter Data - Upper Bound UID
            0x07, 0x34, // Checksum
        ];

        assert_eq!(&encoded, expected);
    }

    #[test]
    fn should_decode_discovery_unique_branch_request() {
        let decoded = RdmRequest::decode(&[
            0xcc, // Start Code
            0x01, // Sub Start Code
            0x24, // Message Length
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // Destination UID
            0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // Source UID
            0x00, // Transaction Number
            0x01, // Port ID
            0x00, // Message Count
            0x00, 0x01, // Sub-Device ID
            0x10, // Command Class
            0x00, 0x01, // Parameter ID
            0x0c, // PDL
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Parameter Data - Lower Bound UID
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // Parameter Data - Upper Bound UID
            0x07, 0x34, // Checksum
        ])
        .unwrap();

        let expected = RdmRequest::new(
            DeviceUID::new(0x0102, 0x03040506),
            DeviceUID::new(0x0605, 0x04030201),
            0x00,
            0x01,
            SubDeviceId::Id(0x01),
            RequestParameter::DiscUniqueBranch {
                lower_bound_uid: DeviceUID::new(0x0000, 0x00000000),
                upper_bound_uid: DeviceUID::new(0xffff, 0xffffffff),
            },
        );

        assert_eq!(decoded, expected);
    }

    #[test]
    fn should_encode_valid_rdm_request() {
        let mut encoded = [0u8; 26];

        let bytes_encoded = RdmRequest::new(
            DeviceUID::new(0x0102, 0x03040506),
            DeviceUID::new(0x0605, 0x04030201),
            0x00,
            0x01,
            SubDeviceId::RootDevice,
            RequestParameter::GetIdentifyDevice,
        )
        .encode(&mut encoded)
        .unwrap();

        let expected: &[u8; 26] = &[
            0xcc, // Start Code
            0x01, // Sub Start Code
            0x18, // Message Length
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // Destination UID
            0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // Source UID
            0x00, // Transaction Number
            0x01, // Port ID
            0x00, // Message Count
            0x00, 0x00, // Sub-Device ID = Root Device
            0x20, // Command Class = GetCommand
            0x10, 0x00, // Parameter ID = Identify Device
            0x00, // PDL
            0x01, 0x40, // Checksum
        ];

        assert_eq!(&encoded[0..bytes_encoded], expected);
    }

    #[test]
    fn should_decode_valid_rdm_request() {
        let decoded = RdmRequest::decode(&[
            0xcc, // Start Code
            0x01, // Sub Start Code
            0x18, // Message Length
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // Destination UID
            0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // Source UID
            0x00, // Transaction Number
            0x01, // Port ID
            0x00, // Message Count
            0x00, 0x00, // Sub-Device ID = Root Device
            0x20, // Command Class = GetCommand
            0x10, 0x00, // Parameter ID = Identify Device
            0x00, // PDL
            0x01, 0x40, // Checksum
        ])
        .unwrap();

        let expected = RdmRequest::new(
            DeviceUID::new(0x0102, 0x03040506),
            DeviceUID::new(0x0605, 0x04030201),
            0x00,
            0x01,
            SubDeviceId::RootDevice,
            RequestParameter::GetIdentifyDevice,
        );

        assert_eq!(decoded, expected);
    }

    #[test]
    fn should_encode_manufacturer_specific_rdm_request() {
        let mut encoded = [0u8; 30];

        let bytes_encoded = RdmRequest::new(
            DeviceUID::new(0x0102, 0x03040506),
            DeviceUID::new(0x0605, 0x04030201),
            0x00,
            0x01,
            SubDeviceId::RootDevice,
            RequestParameter::RawParameter {
                command_class: CommandClass::SetCommand,
                parameter_id: 0x8080,
                parameter_data: Vec::from_slice(&[0x01, 0x02, 0x03, 0x04]).unwrap(),
            },
        )
        .encode(&mut encoded)
        .unwrap();

        let expected: &[u8; 30] = &[
            0xcc, // Start Code
            0x01, // Sub Start Code
            0x1c, // Message Length
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // Destination UID
            0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // Source UID
            0x00, // Transaction Number
            0x01, // Port ID
            0x00, // Message Count
            0x00, 0x00, // Sub-Device ID = Root Device
            0x30, // Command Class = SetCommand
            0x80, 0x80, // Parameter ID = Identify Device
            0x04, // PDL
            0x01, 0x02, 0x03, 0x04, // Parameter Data
            0x02, 0x52, // Checksum
        ];

        assert_eq!(&encoded[0..bytes_encoded], expected);
    }

    #[test]
    fn should_decode_manufacturer_specific_rdm_request() {
        let decoded = RdmRequest::decode(&[
            0xcc, // Start Code
            0x01, // Sub Start Code
            0x1c, // Message Length
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // Destination UID
            0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // Source UID
            0x00, // Transaction Number
            0x01, // Port ID
            0x00, // Message Count
            0x00, 0x00, // Sub-Device ID = Root Device
            0x30, // Command Class = SetCommand
            0x80, 0x80, // Parameter ID = Identify Device
            0x04, // PDL
            0x01, 0x02, 0x03, 0x04, // Parameter Data
            0x02, 0x52, // Checksum
        ])
        .unwrap();

        let expected = RdmRequest::new(
            DeviceUID::new(0x0102, 0x03040506),
            DeviceUID::new(0x0605, 0x04030201),
            0x00,
            0x01,
            SubDeviceId::RootDevice,
            RequestParameter::RawParameter {
                command_class: CommandClass::SetCommand,
                parameter_id: 0x8080,
                parameter_data: Vec::from_slice(&[0x01, 0x02, 0x03, 0x04]).unwrap(),
            },
        );

        assert_eq!(decoded, expected);
    }
}
