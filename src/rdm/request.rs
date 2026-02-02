//! Data types and functionality for encoding RDM requests
//!
//! # RdmRequest
//!
//! ```ignore
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
//!         0x20, // Command Class = Get
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

use super::{RDM_START_CODE_BYTE, RDM_SUB_START_CODE_BYTE, utils::bsd_16_crc};
use crate::rdm::parameter::{
    e120::request::{
        DiscUniqueBranchRequest, GetDmxPersonalityDescriptionRequest,
        GetParameterDescriptionRequest, GetQueuedMessageRequest, GetSelfTestDescriptionRequest,
        GetSensorDefinitionRequest, GetSensorValueRequest, GetSlotDescriptionRequest,
        GetStatusIdDescriptionRequest, GetStatusMessagesRequest, SetCapturePresetRequest,
        SetDeviceHoursRequest, SetDeviceLabelRequest, SetDevicePowerCyclesRequest,
        SetDisplayInvertRequest, SetDisplayLevelRequest, SetDmxPersonalityRequest,
        SetDmxStartAddressRequest, SetIdentifyDeviceRequest, SetLampHoursRequest,
        SetLampOnModeRequest, SetLampStateRequest, SetLampStrikesRequest, SetLanguageRequest,
        SetPanInvertRequest, SetPanTiltSwapRequest, SetPerformSelfTestRequest,
        SetPowerStateRequest, SetPresetPlaybackRequest, SetRealTimeClockRequest,
        SetRecordSensorsRequest, SetResetDeviceRequest, SetSensorValueRequest,
        SetSubDeviceIdStatusReportThresholdRequest, SetTiltInvertRequest,
    },
    e133::request::{
        GetComponentScopeRequest, SetBrokerStatusRequest, SetComponentScopeRequest,
        SetSearchDomainRequest, SetTcpCommsStatusRequest,
    },
    e137_1::request::{
        GetCurveDescriptionRequest, GetModulationFrequencyDescriptionRequest,
        GetOutputResponseTimeDescriptionRequest, GetPresetStatusRequest, SetBurnInRequest,
        SetCurveRequest, SetDmxBlockAddressRequest, SetDmxFailModeRequest,
        SetDmxStartupModeRequest, SetIdentifyModeRequest, SetLockPinRequest, SetLockStateRequest,
        SetMaximumLevelRequest, SetMinimumLevelRequest, SetModulationFrequencyRequest,
        SetOutputResponseTimeRequest, SetPowerOnSelfTestRequest, SetPresetMergeModeRequest,
        SetPresetStatusRequest,
    },
    e137_2::request::{
        GetDnsIpV4NameServerRequest, GetInterfaceHardwareAddressType1Request,
        GetInterfaceLabelRequest, GetIpV4CurrentAddressRequest, GetIpV4DhcpModeRequest,
        GetIpV4StaticAddressRequest, GetIpV4ZeroConfModeRequest, SetDnsDomainNameRequest,
        SetDnsHostNameRequest, SetDnsIpv4NameServerRequest, SetInterfaceApplyConfigurationRequest,
        SetInterfaceReleaseDhcpRequest, SetInterfaceRenewDhcpRequest, SetIpV4DefaultRouteRequest,
        SetIpV4DhcpModeRequest, SetIpV4StaticAddressRequest, SetIpV4ZeroConfModeRequest,
    },
    e137_7::request::{
        GetBackgroundDiscoveryRequest, GetBackgroundQueuedStatusPolicyDescriptionRequest,
        GetBindingControlFieldsRequest, GetDiscoveryStateRequest, GetEndpointLabelRequest,
        GetEndpointModeRequest, GetEndpointResponderListChangeRequest,
        GetEndpointRespondersRequest, GetEndpointTimingDescriptionRequest,
        GetEndpointTimingRequest, GetEndpointToUniverseRequest, GetIdentifyEndpointRequest,
        GetRdmTrafficEnableRequest, SetBackgroundDiscoveryRequest,
        SetBackgroundQueuedStatusPolicyRequest, SetDiscoveryStateRequest, SetEndpointLabelRequest,
        SetEndpointModeRequest, SetEndpointTimingRequest, SetEndpointToUniverseRequest,
        SetIdentifyEndpointRequest, SetRdmTrafficEnableRequest,
    },
};
use heapless::Vec;
use rdm_core::{
    CommandClass, DeviceUID, SubDeviceId,
    error::ParameterCodecError,
    parameter_traits::{RdmParameter, RdmParameterData},
    request::CustomRequestParameter,
};
use rdm_core::{ParameterId, error::RdmError};

#[derive(Clone, Debug, PartialEq)]
pub enum RequestParameter {
    // E1.20
    DiscUniqueBranch(DiscUniqueBranchRequest),
    DiscMute,
    DiscUnMute,
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
    SetDisplayInvert(SetDisplayInvertRequest),
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
    SetDmxFailMode(SetDmxFailModeRequest),
    GetDmxStartupMode,
    SetDmxStartupMode(SetDmxStartupModeRequest),
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
    SetPowerOnSelfTest(SetPowerOnSelfTestRequest),
    GetLockState,
    SetLockState(SetLockStateRequest),
    GetLockStateDescription,
    GetLockPin,
    SetLockPin(SetLockPinRequest),
    GetBurnIn,
    SetBurnIn(SetBurnInRequest),
    GetIdentifyMode,
    SetIdentifyMode(SetIdentifyModeRequest),
    GetPresetInfo,
    GetPresetStatus(GetPresetStatusRequest),
    GetPresetMergeMode,
    SetPresetMergeMode(SetPresetMergeModeRequest),
    SetPresetStatus(SetPresetStatusRequest),
    // E1.37-2
    GetListInterfaces,
    GetInterfaceLabel(GetInterfaceLabelRequest),
    GetInterfaceHardwareAddressType1(GetInterfaceHardwareAddressType1Request),
    GetIpV4DhcpMode(GetIpV4DhcpModeRequest),
    SetIpV4DhcpMode(SetIpV4DhcpModeRequest),
    GetIpV4ZeroConfMode(GetIpV4ZeroConfModeRequest),
    SetIpV4ZeroConfMode(SetIpV4ZeroConfModeRequest),
    GetIpV4CurrentAddress(GetIpV4CurrentAddressRequest),
    GetIpV4StaticAddress(GetIpV4StaticAddressRequest),
    SetIpV4StaticAddress(SetIpV4StaticAddressRequest),
    SetInterfaceApplyConfiguration(SetInterfaceApplyConfigurationRequest),
    SetInterfaceRenewDhcp(SetInterfaceRenewDhcpRequest),
    SetInterfaceReleaseDhcp(SetInterfaceReleaseDhcpRequest),
    GetIpV4DefaultRoute,
    SetIpV4DefaultRoute(SetIpV4DefaultRouteRequest),
    GetDnsIpV4NameServer(GetDnsIpV4NameServerRequest),
    SetDnsIpV4NameServer(SetDnsIpv4NameServerRequest),
    GetDnsHostName,
    SetDnsHostName(SetDnsHostNameRequest),
    GetDnsDomainName,
    SetDnsDomainName(SetDnsDomainNameRequest),
    // E1.37-7
    GetEndpointList,
    GetEndpointListChange,
    GetIdentifyEndpoint(GetIdentifyEndpointRequest),
    SetIdentifyEndpoint(SetIdentifyEndpointRequest),
    GetEndpointToUniverse(GetEndpointToUniverseRequest),
    SetEndpointToUniverse(SetEndpointToUniverseRequest),
    GetEndpointMode(GetEndpointModeRequest),
    SetEndpointMode(SetEndpointModeRequest),
    GetEndpointLabel(GetEndpointLabelRequest),
    SetEndpointLabel(SetEndpointLabelRequest),
    GetRdmTrafficEnable(GetRdmTrafficEnableRequest),
    SetRdmTrafficEnable(SetRdmTrafficEnableRequest),
    GetDiscoveryState(GetDiscoveryStateRequest),
    SetDiscoveryState(SetDiscoveryStateRequest),
    GetBackgroundDiscovery(GetBackgroundDiscoveryRequest),
    SetBackgroundDiscovery(SetBackgroundDiscoveryRequest),
    GetEndpointTiming(GetEndpointTimingRequest),
    SetEndpointTiming(SetEndpointTimingRequest),
    GetEndpointTimingDescription(GetEndpointTimingDescriptionRequest),
    GetEndpointResponders(GetEndpointRespondersRequest),
    GetEndpointResponderListChange(GetEndpointResponderListChangeRequest),
    GetBindingControlFields(GetBindingControlFieldsRequest),
    GetBackgroundQueuedStatusPolicy,
    SetBackgroundQueuedStatusPolicy(SetBackgroundQueuedStatusPolicyRequest),
    GetBackgroundQueuedStatusPolicyDescription(GetBackgroundQueuedStatusPolicyDescriptionRequest),
    // E1.33
    GetSearchDomain,
    SetSearchDomain(SetSearchDomainRequest),
    GetComponentScope(GetComponentScopeRequest),
    SetComponentScope(SetComponentScopeRequest),
    GetTcpCommsStatus,
    SetTcpCommsStatus(SetTcpCommsStatusRequest),
    GetBrokerStatus,
    SetBrokerStatus(SetBrokerStatusRequest),
    // use for unsupported standard and manufacturer specific parameters
    CustomParameter(CustomRequestParameter),
}

impl RequestParameter {
    pub fn command_class(&self) -> CommandClass {
        match self {
            // E1.20
            Self::DiscUniqueBranch { .. } | Self::DiscMute | Self::DiscUnMute => {
                CommandClass::Discovery
            }
            // E1.20
            Self::GetCommsStatus
            | Self::GetQueuedMessage { .. }
            | Self::GetStatusMessages(_)
            | Self::GetStatusIdDescription(_)
            | Self::GetSubDeviceIdStatusReportThreshold
            | Self::GetSupportedParameters
            | Self::GetParameterDescription(_)
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
            | Self::GetDmxPersonalityDescription(_)
            | Self::GetDmxStartAddress
            | Self::GetSlotInfo
            | Self::GetSlotDescription(_)
            | Self::GetDefaultSlotValue
            | Self::GetSensorDefinition(_)
            | Self::GetSensorValue(_)
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
            | Self::GetSelfTestDescription(_)
            | Self::GetPresetPlayback
            // E1.37-1
            | Self::GetDmxBlockAddress
            | Self::GetDmxFailMode
            | Self::GetDmxStartupMode
            | Self::GetDimmerInfo
            | Self::GetMinimumLevel
            | Self::GetMaximumLevel
            | Self::GetCurve
            | Self::GetCurveDescription(_)
            | Self::GetOutputResponseTime
            | Self::GetOutputResponseTimeDescription(_)
            | Self::GetModulationFrequency
            | Self::GetModulationFrequencyDescription(_)
            | Self::GetPowerOnSelfTest
            | Self::GetLockState
            | Self::GetLockStateDescription
            | Self::GetLockPin
            | Self::GetBurnIn
            | Self::GetIdentifyMode
            | Self::GetPresetInfo
            | Self::GetPresetStatus(_)
            | Self::GetPresetMergeMode
            // E1.37-2
            | Self::GetListInterfaces
            | Self::GetInterfaceLabel(_)
            | Self::GetInterfaceHardwareAddressType1(_)
            | Self::GetIpV4DhcpMode(_)
            | Self::GetIpV4ZeroConfMode(_)
            | Self::GetIpV4CurrentAddress(_)
            | Self::GetIpV4StaticAddress(_)
            | Self::GetIpV4DefaultRoute
            | Self::GetDnsIpV4NameServer(_)
            | Self::GetDnsHostName
            | Self::GetDnsDomainName
            // E1.37-7
            | Self::GetEndpointList
            | Self::GetEndpointListChange
            | Self::GetIdentifyEndpoint(_)
            | Self::GetEndpointToUniverse(_)
            | Self::GetEndpointMode(_)
            | Self::GetEndpointLabel(_)
            | Self::GetRdmTrafficEnable(_)
            | Self::GetDiscoveryState(_)
            | Self::GetBackgroundDiscovery(_)
            | Self::GetEndpointTiming(_)
            | Self::GetEndpointTimingDescription(_)
            | Self::GetEndpointResponders(_)
            | Self::GetEndpointResponderListChange(_)
            | Self::GetBindingControlFields(_)
            | Self::GetBackgroundQueuedStatusPolicy
            | Self::GetBackgroundQueuedStatusPolicyDescription(_)
            // E1.33
            | Self::GetComponentScope(_)
            | Self::GetSearchDomain
            | Self::GetTcpCommsStatus
            | Self::GetBrokerStatus
            => CommandClass::Get,
            // E1.20
            Self::SetCommsStatus
            | Self::SetClearStatusId
            | Self::SetSubDeviceIdStatusReportThreshold(_)
            | Self::SetDeviceLabel(_)
            | Self::SetFactoryDefaults
            | Self::SetLanguage(_)
            | Self::SetDmxPersonality(_)
            | Self::SetDmxStartAddress(_)
            | Self::SetSensorValue(_)
            | Self::SetRecordSensors(_)
            | Self::SetDeviceHours(_)
            | Self::SetLampHours(_)
            | Self::SetLampStrikes(_)
            | Self::SetLampState(_)
            | Self::SetLampOnMode(_)
            | Self::SetDevicePowerCycles(_)
            | Self::SetDisplayInvert(_)
            | Self::SetDisplayLevel(_)
            | Self::SetPanInvert(_)
            | Self::SetTiltInvert(_)
            | Self::SetPanTiltSwap(_)
            | Self::SetRealTimeClock(_)
            | Self::SetIdentifyDevice(_)
            | Self::SetResetDevice(_)
            | Self::SetPowerState(_)
            | Self::SetPerformSelfTest(_)
            | Self::SetCapturePreset(_)
            | Self::SetPresetPlayback(_)
            // E1.37-1
            | Self::SetDmxBlockAddress(_)
            | Self::SetDmxFailMode(_)
            | Self::SetDmxStartupMode(_)
            | Self::SetMinimumLevel(_)
            | Self::SetMaximumLevel(_)
            | Self::SetCurve(_)
            | Self::SetOutputResponseTime(_)
            | Self::SetModulationFrequency(_)
            | Self::SetPowerOnSelfTest(_)
            | Self::SetLockState(_)
            | Self::SetLockPin(_)
            | Self::SetBurnIn(_)
            | Self::SetIdentifyMode(_)
            | Self::SetPresetMergeMode(_)
            | Self::SetPresetStatus(_)
            // E1.37-2
            | Self::SetIpV4DhcpMode(_)
            | Self::SetIpV4ZeroConfMode(_)
            | Self::SetIpV4StaticAddress(_)
            | Self::SetInterfaceApplyConfiguration(_)
            | Self::SetInterfaceRenewDhcp(_)
            | Self::SetInterfaceReleaseDhcp(_)
            | Self::SetIpV4DefaultRoute(_)
            | Self::SetDnsIpV4NameServer(_)
            | Self::SetDnsHostName(_)
            | Self::SetDnsDomainName(_)
            // E1.37-7
            | Self::SetIdentifyEndpoint(_)
            | Self::SetEndpointToUniverse(_)
            | Self::SetEndpointMode(_)
            | Self::SetEndpointLabel(_)
            | Self::SetRdmTrafficEnable(_)
            | Self::SetDiscoveryState(_)
            | Self::SetBackgroundDiscovery(_)
            | Self::SetEndpointTiming(_)
            | Self::SetBackgroundQueuedStatusPolicy(_)
            // E1.33
            | Self::SetComponentScope(_)
            | Self::SetSearchDomain(_)
            | Self::SetTcpCommsStatus(_)
            | Self::SetBrokerStatus(_)
            => CommandClass::Set,
            // Manufacturer or Unsupported
            Self::CustomParameter(param) => param.command_class,
        }
    }

    pub fn parameter_id(&self) -> ParameterId {
        match self {
            // E1.20
            Self::DiscUniqueBranch { .. } => ParameterId::DiscUniqueBranch,
            Self::DiscMute => ParameterId::DiscMute,
            Self::DiscUnMute => ParameterId::DiscUnMute,
            Self::GetCommsStatus | Self::SetCommsStatus => ParameterId::CommsStatus,
            Self::GetQueuedMessage(_) => ParameterId::QueuedMessage,
            Self::GetStatusMessages(_) => ParameterId::StatusMessages,
            Self::GetStatusIdDescription(_) => ParameterId::StatusIdDescription,
            Self::SetClearStatusId => ParameterId::ClearStatusId,
            Self::GetSubDeviceIdStatusReportThreshold
            | Self::SetSubDeviceIdStatusReportThreshold(_) => {
                ParameterId::SubDeviceIdStatusReportThreshold
            }
            Self::GetSupportedParameters => ParameterId::SupportedParameters,
            Self::GetParameterDescription(_) => ParameterId::ParameterDescription,
            Self::GetDeviceInfo => ParameterId::DeviceInfo,
            Self::GetProductDetailIdList => ParameterId::ProductDetailIdList,
            Self::GetDeviceModelDescription => ParameterId::DeviceModelDescription,
            Self::GetManufacturerLabel => ParameterId::ManufacturerLabel,
            Self::GetDeviceLabel | Self::SetDeviceLabel(_) => ParameterId::DeviceLabel,
            Self::GetFactoryDefaults | Self::SetFactoryDefaults => ParameterId::FactoryDefaults,
            Self::GetLanguageCapabilities => ParameterId::LanguageCapabilities,
            Self::GetLanguage | Self::SetLanguage(_) => ParameterId::Language,
            Self::GetSoftwareVersionLabel => ParameterId::SoftwareVersionLabel,
            Self::GetBootSoftwareVersionId => ParameterId::BootSoftwareVersionId,
            Self::GetBootSoftwareVersionLabel => ParameterId::BootSoftwareVersionLabel,
            Self::GetDmxPersonality | Self::SetDmxPersonality(_) => ParameterId::DmxPersonality,
            Self::GetDmxPersonalityDescription(_) => ParameterId::DmxPersonalityDescription,
            Self::GetDmxStartAddress | Self::SetDmxStartAddress(_) => ParameterId::DmxStartAddress,
            Self::GetSlotInfo => ParameterId::SlotInfo,
            Self::GetSlotDescription(_) => ParameterId::SlotDescription,
            Self::GetDefaultSlotValue => ParameterId::DefaultSlotValue,
            Self::GetSensorDefinition(_) => ParameterId::SensorDefinition,
            Self::GetSensorValue(_) | Self::SetSensorValue(_) => ParameterId::SensorValue,
            Self::SetRecordSensors(_) => ParameterId::RecordSensors,
            Self::GetDeviceHours | Self::SetDeviceHours(_) => ParameterId::DeviceHours,
            Self::GetLampHours | Self::SetLampHours(_) => ParameterId::LampHours,
            Self::GetLampStrikes | Self::SetLampStrikes(_) => ParameterId::LampStrikes,
            Self::GetLampState | Self::SetLampState(_) => ParameterId::LampState,
            Self::GetLampOnMode | Self::SetLampOnMode(_) => ParameterId::LampOnMode,
            Self::GetDevicePowerCycles | Self::SetDevicePowerCycles(_) => {
                ParameterId::DevicePowerCycles
            }
            Self::GetDisplayInvert | Self::SetDisplayInvert(_) => ParameterId::DisplayInvert,
            Self::GetDisplayLevel | Self::SetDisplayLevel(_) => ParameterId::DisplayLevel,
            Self::GetPanInvert | Self::SetPanInvert(_) => ParameterId::PanInvert,
            Self::GetTiltInvert | Self::SetTiltInvert(_) => ParameterId::TiltInvert,
            Self::GetPanTiltSwap | Self::SetPanTiltSwap(_) => ParameterId::PanTiltSwap,
            Self::GetRealTimeClock | Self::SetRealTimeClock(_) => ParameterId::RealTimeClock,
            Self::GetIdentifyDevice | Self::SetIdentifyDevice(_) => ParameterId::IdentifyDevice,
            Self::SetResetDevice(_) => ParameterId::ResetDevice,
            Self::GetPowerState | Self::SetPowerState(_) => ParameterId::PowerState,
            Self::GetPerformSelfTest | Self::SetPerformSelfTest(_) => ParameterId::PerformSelfTest,
            Self::SetCapturePreset(_) => ParameterId::CapturePreset,
            Self::GetSelfTestDescription(_) => ParameterId::SelfTestDescription,
            Self::GetPresetPlayback | Self::SetPresetPlayback(_) => ParameterId::PresetPlayback,
            // E1.37-1
            Self::GetDmxBlockAddress | Self::SetDmxBlockAddress(_) => ParameterId::DmxBlockAddress,
            Self::GetDmxFailMode | Self::SetDmxFailMode(_) => ParameterId::DmxFailMode,
            Self::GetDmxStartupMode | Self::SetDmxStartupMode(_) => ParameterId::DmxStartupMode,
            Self::GetDimmerInfo => ParameterId::DimmerInfo,
            Self::GetMinimumLevel | Self::SetMinimumLevel(_) => ParameterId::MinimumLevel,
            Self::GetMaximumLevel | Self::SetMaximumLevel(_) => ParameterId::MaximumLevel,
            Self::GetCurve | Self::SetCurve(_) => ParameterId::Curve,
            Self::GetCurveDescription(_) => ParameterId::CurveDescription,
            Self::GetOutputResponseTime | Self::SetOutputResponseTime(_) => {
                ParameterId::OutputResponseTime
            }
            Self::GetOutputResponseTimeDescription(_) => ParameterId::OutputResponseTimeDescription,
            Self::GetModulationFrequency | Self::SetModulationFrequency(_) => {
                ParameterId::ModulationFrequency
            }
            Self::GetModulationFrequencyDescription(_) => {
                ParameterId::ModulationFrequencyDescription
            }
            Self::GetPowerOnSelfTest | Self::SetPowerOnSelfTest(_) => ParameterId::PowerOnSelfTest,
            Self::GetLockState | Self::SetLockState(_) => ParameterId::LockState,
            Self::GetLockStateDescription => ParameterId::LockStateDescription,
            Self::GetLockPin | Self::SetLockPin(_) => ParameterId::LockPin,
            Self::GetBurnIn | Self::SetBurnIn(_) => ParameterId::BurnIn,
            Self::GetIdentifyMode | Self::SetIdentifyMode(_) => ParameterId::IdentifyMode,
            Self::GetPresetInfo => ParameterId::PresetInfo,
            Self::GetPresetStatus(_) | Self::SetPresetStatus(_) => ParameterId::PresetStatus,
            Self::GetPresetMergeMode | Self::SetPresetMergeMode(_) => ParameterId::PresetMergeMode,
            // E1.37-2
            Self::GetListInterfaces => ParameterId::ListInterfaces,
            Self::GetInterfaceLabel(_) => ParameterId::InterfaceLabel,
            Self::GetInterfaceHardwareAddressType1(_) => ParameterId::InterfaceHardwareAddressType1,
            Self::GetIpV4DhcpMode(_) | Self::SetIpV4DhcpMode(_) => ParameterId::IpV4DhcpMode,
            Self::GetIpV4ZeroConfMode(_) | Self::SetIpV4ZeroConfMode(_) => {
                ParameterId::IpV4ZeroConfMode
            }
            Self::GetIpV4CurrentAddress(_) => ParameterId::IpV4CurrentAddress,
            Self::GetIpV4StaticAddress(_) | Self::SetIpV4StaticAddress(_) => {
                ParameterId::IpV4StaticAddress
            }
            Self::SetInterfaceApplyConfiguration(_) => ParameterId::InterfaceApplyConfiguration,
            Self::SetInterfaceRenewDhcp(_) => ParameterId::InterfaceRenewDhcp,
            Self::SetInterfaceReleaseDhcp(_) => ParameterId::InterfaceReleaseDhcp,
            Self::GetIpV4DefaultRoute | Self::SetIpV4DefaultRoute(_) => {
                ParameterId::IpV4DefaultRoute
            }
            Self::GetDnsIpV4NameServer(_) | Self::SetDnsIpV4NameServer(_) => {
                ParameterId::DnsIpV4NameServer
            }
            Self::GetDnsHostName | Self::SetDnsHostName(_) => ParameterId::DnsHostName,
            Self::GetDnsDomainName | Self::SetDnsDomainName(_) => ParameterId::DnsDomainName,
            // E1.37-7
            Self::GetEndpointList => ParameterId::EndpointList,
            Self::GetEndpointListChange => ParameterId::EndpointListChange,
            Self::GetIdentifyEndpoint(_) | Self::SetIdentifyEndpoint(_) => {
                ParameterId::IdentifyEndpoint
            }
            Self::GetEndpointToUniverse(_) | Self::SetEndpointToUniverse(_) => {
                ParameterId::EndpointToUniverse
            }
            Self::GetEndpointMode(_) | Self::SetEndpointMode(_) => ParameterId::EndpointMode,
            Self::GetEndpointLabel(_) | Self::SetEndpointLabel(_) => ParameterId::EndpointLabel,
            Self::GetRdmTrafficEnable(_) | Self::SetRdmTrafficEnable(_) => {
                ParameterId::RdmTrafficEnable
            }
            Self::GetDiscoveryState(_) | Self::SetDiscoveryState(_) => ParameterId::DiscoveryState,
            Self::GetBackgroundDiscovery(_) | Self::SetBackgroundDiscovery(_) => {
                ParameterId::BackgroundDiscovery
            }
            Self::GetEndpointTiming(_) | Self::SetEndpointTiming(_) => ParameterId::EndpointTiming,
            Self::GetEndpointTimingDescription(_) => ParameterId::EndpointTimingDescription,
            Self::GetEndpointResponders(_) => ParameterId::EndpointResponders,
            Self::GetEndpointResponderListChange(_) => ParameterId::EndpointResponderListChange,
            Self::GetBindingControlFields(_) => ParameterId::BindingControlFields,
            Self::GetBackgroundQueuedStatusPolicy => ParameterId::BackgroundQueuedStatusPolicy,
            Self::SetBackgroundQueuedStatusPolicy(_) => ParameterId::BackgroundQueuedStatusPolicy,
            Self::GetBackgroundQueuedStatusPolicyDescription(_) => {
                ParameterId::BackgroundQueuedStatusPolicyDescription
            }
            // E1.33
            Self::GetComponentScope(_) | Self::SetComponentScope(_) => ParameterId::ComponentScope,
            Self::GetSearchDomain | Self::SetSearchDomain(_) => ParameterId::SearchDomain,
            Self::GetTcpCommsStatus | Self::SetTcpCommsStatus(_) => ParameterId::TcpCommsStatus,
            Self::GetBrokerStatus | Self::SetBrokerStatus(_) => ParameterId::BrokerStatus,
            // Manufacturer or Unsupported
            Self::CustomParameter(param) => param.parameter_id,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            // E1.20
            Self::DiscUniqueBranch(param) => param.size_of(),
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
            Self::GetQueuedMessage(param) => param.size_of(),
            Self::GetStatusMessages(param) => param.size_of(),
            Self::SetSubDeviceIdStatusReportThreshold(param) => param.size_of(),
            Self::SetDmxPersonality(param) => param.size_of(),
            Self::GetDmxPersonalityDescription(param) => param.size_of(),
            Self::GetSensorDefinition(param) => param.size_of(),
            Self::GetSensorValue(param) => param.size_of(),
            Self::SetSensorValue(param) => param.size_of(),
            Self::SetRecordSensors(param) => param.size_of(),
            Self::SetLampState(param) => param.size_of(),
            Self::SetLampOnMode(param) => param.size_of(),
            Self::SetDisplayInvert(param) => param.size_of(),
            Self::SetDisplayLevel(param) => param.size_of(),
            Self::SetPanInvert(param) => param.size_of(),
            Self::SetTiltInvert(param) => param.size_of(),
            Self::SetPanTiltSwap(param) => param.size_of(),
            Self::SetIdentifyDevice(param) => param.size_of(),
            Self::SetResetDevice(param) => param.size_of(),
            Self::SetPowerState(param) => param.size_of(),
            Self::SetPerformSelfTest(param) => param.size_of(),
            Self::GetSelfTestDescription(param) => param.size_of(),
            Self::SetCurve(param) => param.size_of(),
            Self::GetCurveDescription(param) => param.size_of(),
            Self::SetOutputResponseTime(param) => param.size_of(),
            Self::GetOutputResponseTimeDescription(param) => param.size_of(),
            Self::SetModulationFrequency(param) => param.size_of(),
            Self::GetModulationFrequencyDescription(param) => param.size_of(),
            Self::SetPowerOnSelfTest(param) => param.size_of(),
            Self::SetBurnIn(param) => param.size_of(),
            Self::SetIdentifyMode(param) => param.size_of(),
            Self::SetPresetMergeMode(param) => param.size_of(),
            Self::GetDnsIpV4NameServer(param) => param.size_of(),
            Self::GetEndpointTimingDescription(param) => param.size_of(),
            Self::SetBackgroundQueuedStatusPolicy(param) => param.size_of(),
            Self::GetBackgroundQueuedStatusPolicyDescription(param) => param.size_of(),
            Self::SetBrokerStatus(param) => param.size_of(),
            Self::GetStatusIdDescription(param) => param.size_of(),
            Self::GetParameterDescription(param) => param.size_of(),
            Self::SetDmxStartAddress(param) => param.size_of(),
            Self::GetSlotDescription(param) => param.size_of(),
            Self::SetDmxBlockAddress(param) => param.size_of(),
            Self::SetMaximumLevel(param) => param.size_of(),
            Self::GetPresetStatus(param) => param.size_of(),
            Self::GetIdentifyEndpoint(param) => param.size_of(),
            Self::GetEndpointToUniverse(param) => param.size_of(),
            Self::GetEndpointMode(param) => param.size_of(),
            Self::GetEndpointLabel(param) => param.size_of(),
            Self::GetRdmTrafficEnable(param) => param.size_of(),
            Self::GetDiscoveryState(param) => param.size_of(),
            Self::GetBackgroundDiscovery(param) => param.size_of(),
            Self::GetEndpointTiming(param) => param.size_of(),
            Self::GetEndpointResponders(param) => param.size_of(),
            Self::GetEndpointResponderListChange(param) => param.size_of(),
            Self::GetComponentScope(param) => param.size_of(),
            Self::SetPresetPlayback(param) => param.size_of(),
            Self::SetLockState(param) => param.size_of(),
            Self::SetEndpointMode(param) => param.size_of(),
            Self::SetRdmTrafficEnable(param) => param.size_of(),
            Self::SetDiscoveryState(param) => param.size_of(),
            Self::SetBackgroundDiscovery(param) => param.size_of(),
            Self::SetIdentifyEndpoint(param) => param.size_of(),
            Self::SetEndpointTiming(param) => param.size_of(),
            Self::SetDeviceHours(param) => param.size_of(),
            Self::SetLampHours(param) => param.size_of(),
            Self::SetLampStrikes(param) => param.size_of(),
            Self::SetDevicePowerCycles(param) => param.size_of(),
            Self::SetLockPin(param) => param.size_of(),
            Self::GetInterfaceLabel(param) => param.size_of(),
            Self::GetInterfaceHardwareAddressType1(param) => param.size_of(),
            Self::GetIpV4DhcpMode(param) => param.size_of(),
            Self::GetIpV4ZeroConfMode(param) => param.size_of(),
            Self::SetIpV4ZeroConfMode(param) => param.size_of(),
            Self::GetIpV4CurrentAddress(param) => param.size_of(),
            Self::GetIpV4StaticAddress(param) => param.size_of(),
            Self::SetInterfaceApplyConfiguration(param) => param.size_of(),
            Self::SetInterfaceRenewDhcp(param) => param.size_of(),
            Self::SetInterfaceReleaseDhcp(param) => param.size_of(),
            Self::SetEndpointToUniverse(param) => param.size_of(),
            Self::SetMinimumLevel(param) => param.size_of(),
            Self::SetIpV4DhcpMode(param) => param.size_of(),
            Self::SetDnsIpV4NameServer(param) => param.size_of(),
            Self::GetBindingControlFields(param) => param.size_of(),
            Self::SetRealTimeClock(param) => param.size_of(),
            Self::SetDmxFailMode(param) => param.size_of(),
            Self::SetDmxStartupMode(param) => param.size_of(),
            Self::SetIpV4DefaultRoute(param) => param.size_of(),
            Self::SetPresetStatus(param) => param.size_of(),
            Self::SetIpV4StaticAddress(param) => param.size_of(),
            Self::SetCapturePreset(param) => param.size_of(),
            Self::SetDeviceLabel(param) => param.size_of(),
            Self::SetLanguage(param) => param.size_of(),
            Self::SetDnsHostName(param) => param.size_of(),
            Self::SetDnsDomainName(param) => param.size_of(),
            Self::SetEndpointLabel(param) => param.size_of(),
            Self::SetSearchDomain(param) => param.size_of(),
            Self::SetTcpCommsStatus(param) => param.size_of(),
            Self::SetComponentScope(param) => param.size_of(),
            // Manufacturer or Unsupported
            Self::CustomParameter(param) => param.parameter_data_length as usize,
        }
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        match self {
            // E1.20
            Self::DiscUniqueBranch(param) => param.encode_parameter_data(buf),
            Self::DiscMute => Ok(0),
            Self::DiscUnMute => Ok(0),
            Self::GetCommsStatus => Ok(0),
            Self::SetCommsStatus => Ok(0),
            Self::GetQueuedMessage(param) => param.encode_parameter_data(buf),
            Self::GetStatusMessages(param) => param.encode_parameter_data(buf),
            Self::GetStatusIdDescription(param) => param.encode_parameter_data(buf),
            Self::SetClearStatusId => Ok(0),
            Self::GetSubDeviceIdStatusReportThreshold => Ok(0),
            Self::SetSubDeviceIdStatusReportThreshold(param) => param.encode_parameter_data(buf),
            Self::GetSupportedParameters => Ok(0),
            Self::GetParameterDescription(param) => param.encode_parameter_data(buf),
            Self::GetDeviceInfo => Ok(0),
            Self::GetProductDetailIdList => Ok(0),
            Self::GetDeviceModelDescription => Ok(0),
            Self::GetManufacturerLabel => Ok(0),
            Self::GetDeviceLabel => Ok(0),
            Self::SetDeviceLabel(param) => param.encode_parameter_data(buf),
            Self::GetFactoryDefaults => Ok(0),
            Self::SetFactoryDefaults => Ok(0),
            Self::GetLanguageCapabilities => Ok(0),
            Self::GetLanguage => Ok(0),
            Self::SetLanguage(param) => param.encode_parameter_data(buf),
            Self::GetSoftwareVersionLabel => Ok(0),
            Self::GetBootSoftwareVersionId => Ok(0),
            Self::GetBootSoftwareVersionLabel => Ok(0),
            Self::GetDmxPersonality => Ok(0),
            Self::SetDmxPersonality(param) => param.encode_parameter_data(buf),
            Self::GetDmxPersonalityDescription(param) => param.encode_parameter_data(buf),
            Self::GetDmxStartAddress => Ok(0),
            Self::SetDmxStartAddress(param) => param.encode_parameter_data(buf),
            Self::GetSlotInfo => Ok(0),
            Self::GetSlotDescription(param) => param.encode_parameter_data(buf),
            Self::GetDefaultSlotValue => Ok(0),
            Self::GetSensorDefinition(param) => param.encode_parameter_data(buf),
            Self::GetSensorValue(param) => param.encode_parameter_data(buf),
            Self::SetSensorValue(param) => param.encode_parameter_data(buf),
            Self::SetRecordSensors(param) => param.encode_parameter_data(buf),
            Self::GetDeviceHours => Ok(0),
            Self::SetDeviceHours(param) => param.encode_parameter_data(buf),
            Self::GetLampHours => Ok(0),
            Self::SetLampHours(param) => param.encode_parameter_data(buf),
            Self::GetLampStrikes => Ok(0),
            Self::SetLampStrikes(param) => param.encode_parameter_data(buf),
            Self::GetLampState => Ok(0),
            Self::SetLampState(param) => param.encode_parameter_data(buf),
            Self::GetLampOnMode => Ok(0),
            Self::SetLampOnMode(param) => param.encode_parameter_data(buf),
            Self::GetDevicePowerCycles => Ok(0),
            Self::SetDevicePowerCycles(param) => param.encode_parameter_data(buf),
            Self::GetDisplayInvert => Ok(0),
            Self::SetDisplayInvert(param) => param.encode_parameter_data(buf),
            Self::GetDisplayLevel => Ok(0),
            Self::SetDisplayLevel(param) => param.encode_parameter_data(buf),
            Self::GetPanInvert => Ok(0),
            Self::SetPanInvert(param) => param.encode_parameter_data(buf),
            Self::GetTiltInvert => Ok(0),
            Self::SetTiltInvert(param) => param.encode_parameter_data(buf),
            Self::GetPanTiltSwap => Ok(0),
            Self::SetPanTiltSwap(param) => param.encode_parameter_data(buf),
            Self::GetRealTimeClock => Ok(0),
            Self::SetRealTimeClock(param) => param.encode_parameter_data(buf),
            Self::GetIdentifyDevice => Ok(0),
            Self::SetIdentifyDevice(param) => param.encode_parameter_data(buf),
            Self::SetResetDevice(param) => param.encode_parameter_data(buf),
            Self::GetPowerState => Ok(0),
            Self::SetPowerState(param) => param.encode_parameter_data(buf),
            Self::GetPerformSelfTest => Ok(0),
            Self::SetPerformSelfTest(param) => param.encode_parameter_data(buf),
            Self::SetCapturePreset(param) => param.encode_parameter_data(buf),
            Self::GetSelfTestDescription(param) => param.encode_parameter_data(buf),
            Self::GetPresetPlayback => Ok(0),
            Self::SetPresetPlayback(param) => param.encode_parameter_data(buf),
            // E1.37-1
            Self::GetDmxBlockAddress => Ok(0),
            Self::SetDmxBlockAddress(param) => param.encode_parameter_data(buf),
            Self::GetDmxFailMode => Ok(0),
            Self::SetDmxFailMode(param) => param.encode_parameter_data(buf),
            Self::GetDmxStartupMode => Ok(0),
            Self::SetDmxStartupMode(param) => param.encode_parameter_data(buf),
            Self::GetDimmerInfo => Ok(0),
            Self::GetMinimumLevel => Ok(0),
            Self::SetMinimumLevel(param) => param.encode_parameter_data(buf),
            Self::GetMaximumLevel => Ok(0),
            Self::SetMaximumLevel(param) => param.encode_parameter_data(buf),
            Self::GetCurve => Ok(0),
            Self::SetCurve(param) => param.encode_parameter_data(buf),
            Self::GetCurveDescription(param) => param.encode_parameter_data(buf),
            Self::GetOutputResponseTime => Ok(0),
            Self::SetOutputResponseTime(param) => param.encode_parameter_data(buf),
            Self::GetOutputResponseTimeDescription(param) => param.encode_parameter_data(buf),
            Self::GetModulationFrequency => Ok(0),
            Self::SetModulationFrequency(param) => param.encode_parameter_data(buf),
            Self::GetModulationFrequencyDescription(param) => param.encode_parameter_data(buf),
            Self::GetPowerOnSelfTest => Ok(0),
            Self::SetPowerOnSelfTest(param) => param.encode_parameter_data(buf),
            Self::GetLockState => Ok(0),
            Self::SetLockState(param) => param.encode_parameter_data(buf),
            Self::GetLockStateDescription => Ok(0),
            Self::GetLockPin => Ok(0),
            Self::SetLockPin(param) => param.encode_parameter_data(buf),
            Self::GetBurnIn => Ok(0),
            Self::SetBurnIn(param) => param.encode_parameter_data(buf),
            Self::GetIdentifyMode => Ok(0),
            Self::SetIdentifyMode(param) => param.encode_parameter_data(buf),
            Self::GetPresetInfo => Ok(0),
            Self::GetPresetStatus(param) => param.encode_parameter_data(buf),
            Self::SetPresetStatus(param) => param.encode_parameter_data(buf),
            Self::GetPresetMergeMode => Ok(0),
            Self::SetPresetMergeMode(param) => param.encode_parameter_data(buf),
            // E1.37-2
            Self::GetListInterfaces => Ok(0),
            Self::GetInterfaceLabel(param) => param.encode_parameter_data(buf),
            Self::GetInterfaceHardwareAddressType1(param) => param.encode_parameter_data(buf),
            Self::GetIpV4DhcpMode(param) => param.encode_parameter_data(buf),
            Self::SetIpV4DhcpMode(param) => param.encode_parameter_data(buf),
            Self::GetIpV4ZeroConfMode(param) => param.encode_parameter_data(buf),
            Self::SetIpV4ZeroConfMode(param) => param.encode_parameter_data(buf),
            Self::GetIpV4CurrentAddress(param) => param.encode_parameter_data(buf),
            Self::GetIpV4StaticAddress(param) => param.encode_parameter_data(buf),
            Self::SetIpV4StaticAddress(param) => param.encode_parameter_data(buf),
            Self::SetInterfaceApplyConfiguration(param) => param.encode_parameter_data(buf),
            Self::SetInterfaceRenewDhcp(param) => param.encode_parameter_data(buf),
            Self::SetInterfaceReleaseDhcp(param) => param.encode_parameter_data(buf),
            Self::GetIpV4DefaultRoute => Ok(0),
            Self::SetIpV4DefaultRoute(param) => param.encode_parameter_data(buf),
            Self::GetDnsIpV4NameServer(param) => param.encode_parameter_data(buf),
            Self::SetDnsIpV4NameServer(param) => param.encode_parameter_data(buf),
            Self::GetDnsHostName => Ok(0),
            Self::SetDnsHostName(param) => param.encode_parameter_data(buf),
            Self::GetDnsDomainName => Ok(0),
            Self::SetDnsDomainName(param) => param.encode_parameter_data(buf),
            // E1.37-7
            Self::GetEndpointList => Ok(0),
            Self::GetEndpointListChange => Ok(0),
            Self::GetIdentifyEndpoint(param) => param.encode_parameter_data(buf),
            Self::SetIdentifyEndpoint(param) => param.encode_parameter_data(buf),
            Self::GetEndpointToUniverse(param) => param.encode_parameter_data(buf),
            Self::SetEndpointToUniverse(param) => param.encode_parameter_data(buf),
            Self::GetEndpointMode(param) => param.encode_parameter_data(buf),
            Self::SetEndpointMode(param) => param.encode_parameter_data(buf),
            Self::GetEndpointLabel(param) => param.encode_parameter_data(buf),
            Self::SetEndpointLabel(param) => param.encode_parameter_data(buf),
            Self::GetRdmTrafficEnable(param) => param.encode_parameter_data(buf),
            Self::SetRdmTrafficEnable(param) => param.encode_parameter_data(buf),
            Self::GetDiscoveryState(param) => param.encode_parameter_data(buf),
            Self::SetDiscoveryState(param) => param.encode_parameter_data(buf),
            Self::GetBackgroundDiscovery(param) => param.encode_parameter_data(buf),
            Self::SetBackgroundDiscovery(param) => param.encode_parameter_data(buf),
            Self::GetEndpointTiming(param) => param.encode_parameter_data(buf),
            Self::SetEndpointTiming(param) => param.encode_parameter_data(buf),
            Self::GetEndpointTimingDescription(param) => param.encode_parameter_data(buf),
            Self::GetEndpointResponders(param) => param.encode_parameter_data(buf),
            Self::GetEndpointResponderListChange(param) => param.encode_parameter_data(buf),
            Self::GetBindingControlFields(param) => param.encode_parameter_data(buf),
            Self::GetBackgroundQueuedStatusPolicy => Ok(0),
            Self::SetBackgroundQueuedStatusPolicy(param) => param.encode_parameter_data(buf),
            Self::GetBackgroundQueuedStatusPolicyDescription(param) => {
                param.encode_parameter_data(buf)
            }
            // E1.33
            Self::GetComponentScope(param) => param.encode_parameter_data(buf),
            Self::SetComponentScope(param) => param.encode_parameter_data(buf),
            Self::GetSearchDomain => Ok(0),
            Self::SetSearchDomain(param) => param.encode_parameter_data(buf),
            Self::GetTcpCommsStatus => Ok(0),
            Self::SetTcpCommsStatus(param) => param.encode_parameter_data(buf),
            Self::GetBrokerStatus => Ok(0),
            Self::SetBrokerStatus(param) => param.encode_parameter_data(buf),
            // Manufacturer or Unsupported
            Self::CustomParameter(param) => {
                buf[0..param.parameter_data_length as usize].copy_from_slice(
                    &param.parameter_data[0..param.parameter_data_length as usize],
                );
                Ok(param.parameter_data_length as usize)
            }
        }
    }

    pub fn decode(
        command_class: CommandClass,
        parameter_id: ParameterId,
        buf: &[u8],
    ) -> Result<Self, ParameterCodecError> {
        match (command_class, parameter_id) {
            // E1.20
            (CommandClass::Discovery, ParameterId::DiscUniqueBranch) => Ok(Self::DiscUniqueBranch(
                DiscUniqueBranchRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Discovery, ParameterId::DiscMute) => Ok(Self::DiscMute),
            (CommandClass::Discovery, ParameterId::DiscUnMute) => Ok(Self::DiscUnMute),
            (CommandClass::Get, ParameterId::CommsStatus) => Ok(Self::GetCommsStatus),
            (CommandClass::Set, ParameterId::CommsStatus) => Ok(Self::SetCommsStatus),
            (CommandClass::Get, ParameterId::QueuedMessage) => Ok(Self::GetQueuedMessage(
                GetQueuedMessageRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::StatusMessages) => Ok(Self::GetStatusMessages(
                GetStatusMessagesRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::StatusIdDescription) => {
                Ok(Self::GetStatusIdDescription(
                    GetStatusIdDescriptionRequest::decode_parameter_data(buf)?,
                ))
            }
            (CommandClass::Set, ParameterId::ClearStatusId) => Ok(Self::SetClearStatusId),
            (CommandClass::Get, ParameterId::SubDeviceIdStatusReportThreshold) => {
                Ok(Self::GetSubDeviceIdStatusReportThreshold)
            }
            (CommandClass::Set, ParameterId::SubDeviceIdStatusReportThreshold) => {
                Ok(Self::SetSubDeviceIdStatusReportThreshold(
                    SetSubDeviceIdStatusReportThresholdRequest::decode_parameter_data(buf)?,
                ))
            }
            (CommandClass::Get, ParameterId::SupportedParameters) => {
                Ok(Self::GetSupportedParameters)
            }
            (CommandClass::Get, ParameterId::ParameterDescription) => {
                Ok(Self::GetParameterDescription(
                    GetParameterDescriptionRequest::decode_parameter_data(buf)?,
                ))
            }
            (CommandClass::Get, ParameterId::DeviceInfo) => Ok(Self::GetDeviceInfo),
            (CommandClass::Get, ParameterId::ProductDetailIdList) => {
                Ok(Self::GetProductDetailIdList)
            }
            (CommandClass::Get, ParameterId::DeviceModelDescription) => {
                Ok(Self::GetDeviceModelDescription)
            }
            (CommandClass::Get, ParameterId::ManufacturerLabel) => Ok(Self::GetManufacturerLabel),
            (CommandClass::Get, ParameterId::DeviceLabel) => Ok(Self::GetDeviceLabel),
            (CommandClass::Set, ParameterId::DeviceLabel) => Ok(Self::SetDeviceLabel(
                SetDeviceLabelRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::FactoryDefaults) => Ok(Self::GetFactoryDefaults),
            (CommandClass::Set, ParameterId::FactoryDefaults) => Ok(Self::SetFactoryDefaults),
            (CommandClass::Get, ParameterId::LanguageCapabilities) => {
                Ok(Self::GetLanguageCapabilities)
            }
            (CommandClass::Get, ParameterId::Language) => Ok(Self::GetLanguage),
            (CommandClass::Set, ParameterId::Language) => Ok(Self::SetLanguage(
                SetLanguageRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::SoftwareVersionLabel) => {
                Ok(Self::GetSoftwareVersionLabel)
            }
            (CommandClass::Get, ParameterId::BootSoftwareVersionId) => {
                Ok(Self::GetBootSoftwareVersionId)
            }
            (CommandClass::Get, ParameterId::BootSoftwareVersionLabel) => {
                Ok(Self::GetBootSoftwareVersionLabel)
            }
            (CommandClass::Get, ParameterId::DmxPersonality) => Ok(Self::GetDmxPersonality),
            (CommandClass::Set, ParameterId::DmxPersonality) => Ok(Self::SetDmxPersonality(
                SetDmxPersonalityRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::DmxPersonalityDescription) => {
                Ok(Self::GetDmxPersonalityDescription(
                    GetDmxPersonalityDescriptionRequest::decode_parameter_data(buf)?,
                ))
            }
            (CommandClass::Get, ParameterId::DmxStartAddress) => Ok(Self::GetDmxStartAddress),
            (CommandClass::Set, ParameterId::DmxStartAddress) => Ok(Self::SetDmxStartAddress(
                SetDmxStartAddressRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::SlotInfo) => Ok(Self::GetSlotInfo),
            (CommandClass::Get, ParameterId::SlotDescription) => Ok(Self::GetSlotDescription(
                GetSlotDescriptionRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::DefaultSlotValue) => Ok(Self::GetDefaultSlotValue),
            (CommandClass::Get, ParameterId::SensorDefinition) => Ok(Self::GetSensorDefinition(
                GetSensorDefinitionRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::SensorValue) => Ok(Self::GetSensorValue(
                GetSensorValueRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Set, ParameterId::SensorValue) => Ok(Self::SetSensorValue(
                SetSensorValueRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Set, ParameterId::RecordSensors) => Ok(Self::SetRecordSensors(
                SetRecordSensorsRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::DeviceHours) => Ok(Self::GetDeviceHours),
            (CommandClass::Set, ParameterId::DeviceHours) => Ok(Self::SetDeviceHours(
                SetDeviceHoursRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::LampHours) => Ok(Self::GetLampHours),
            (CommandClass::Set, ParameterId::LampHours) => Ok(Self::SetLampHours(
                SetLampHoursRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::LampStrikes) => Ok(Self::GetLampStrikes),
            (CommandClass::Set, ParameterId::LampStrikes) => Ok(Self::SetLampStrikes(
                SetLampStrikesRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::LampState) => Ok(Self::GetLampState),
            (CommandClass::Set, ParameterId::LampState) => Ok(Self::SetLampState(
                SetLampStateRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::LampOnMode) => Ok(Self::GetLampOnMode),
            (CommandClass::Set, ParameterId::LampOnMode) => Ok(Self::SetLampOnMode(
                SetLampOnModeRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::DevicePowerCycles) => Ok(Self::GetDevicePowerCycles),
            (CommandClass::Set, ParameterId::DevicePowerCycles) => Ok(Self::SetDevicePowerCycles(
                SetDevicePowerCyclesRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::DisplayInvert) => Ok(Self::GetDisplayInvert),
            (CommandClass::Set, ParameterId::DisplayInvert) => Ok(Self::SetDisplayInvert(
                SetDisplayInvertRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::DisplayLevel) => Ok(Self::GetDisplayLevel),
            (CommandClass::Set, ParameterId::DisplayLevel) => Ok(Self::SetDisplayLevel(
                SetDisplayLevelRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::PanInvert) => Ok(Self::GetPanInvert),
            (CommandClass::Set, ParameterId::PanInvert) => Ok(Self::SetPanInvert(
                SetPanInvertRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::TiltInvert) => Ok(Self::GetTiltInvert),
            (CommandClass::Set, ParameterId::TiltInvert) => Ok(Self::SetTiltInvert(
                SetTiltInvertRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::PanTiltSwap) => Ok(Self::GetPanTiltSwap),
            (CommandClass::Set, ParameterId::PanTiltSwap) => Ok(Self::SetPanTiltSwap(
                SetPanTiltSwapRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::RealTimeClock) => Ok(Self::GetRealTimeClock),
            (CommandClass::Set, ParameterId::RealTimeClock) => Ok(Self::SetRealTimeClock(
                SetRealTimeClockRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::IdentifyDevice) => Ok(Self::GetIdentifyDevice),
            (CommandClass::Set, ParameterId::IdentifyDevice) => Ok(Self::SetIdentifyDevice(
                SetIdentifyDeviceRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Set, ParameterId::ResetDevice) => Ok(Self::SetResetDevice(
                SetResetDeviceRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::PowerState) => Ok(Self::GetPowerState),
            (CommandClass::Set, ParameterId::PowerState) => Ok(Self::SetPowerState(
                SetPowerStateRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::PerformSelfTest) => Ok(Self::GetPerformSelfTest),
            (CommandClass::Set, ParameterId::PerformSelfTest) => Ok(Self::SetPerformSelfTest(
                SetPerformSelfTestRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Set, ParameterId::CapturePreset) => Ok(Self::SetCapturePreset(
                SetCapturePresetRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::SelfTestDescription) => {
                Ok(Self::GetSelfTestDescription(
                    GetSelfTestDescriptionRequest::decode_parameter_data(buf)?,
                ))
            }
            (CommandClass::Get, ParameterId::PresetPlayback) => Ok(Self::GetPresetPlayback),
            (CommandClass::Set, ParameterId::PresetPlayback) => Ok(Self::SetPresetPlayback(
                SetPresetPlaybackRequest::decode_parameter_data(buf)?,
            )),
            // E1.37-1
            (CommandClass::Get, ParameterId::DmxBlockAddress) => Ok(Self::GetDmxBlockAddress),
            (CommandClass::Set, ParameterId::DmxBlockAddress) => Ok(Self::SetDmxBlockAddress(
                SetDmxBlockAddressRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::DmxFailMode) => Ok(Self::GetDmxFailMode),
            (CommandClass::Set, ParameterId::DmxFailMode) => Ok(Self::SetDmxFailMode(
                SetDmxFailModeRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::DmxStartupMode) => Ok(Self::GetDmxStartupMode),
            (CommandClass::Set, ParameterId::DmxStartupMode) => Ok(Self::SetDmxStartupMode(
                SetDmxStartupModeRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::DimmerInfo) => Ok(Self::GetDimmerInfo),
            (CommandClass::Get, ParameterId::MinimumLevel) => Ok(Self::GetMinimumLevel),
            (CommandClass::Set, ParameterId::MinimumLevel) => Ok(Self::SetMinimumLevel(
                SetMinimumLevelRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::MaximumLevel) => Ok(Self::GetMaximumLevel),
            (CommandClass::Set, ParameterId::MaximumLevel) => Ok(Self::SetMaximumLevel(
                SetMaximumLevelRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::Curve) => Ok(Self::GetCurve),
            (CommandClass::Set, ParameterId::Curve) => {
                Ok(Self::SetCurve(SetCurveRequest::decode_parameter_data(buf)?))
            }
            (CommandClass::Get, ParameterId::CurveDescription) => Ok(Self::GetCurveDescription(
                GetCurveDescriptionRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::OutputResponseTime) => Ok(Self::GetOutputResponseTime),
            (CommandClass::Set, ParameterId::OutputResponseTime) => {
                Ok(Self::SetOutputResponseTime(
                    SetOutputResponseTimeRequest::decode_parameter_data(buf)?,
                ))
            }
            (CommandClass::Get, ParameterId::OutputResponseTimeDescription) => {
                Ok(Self::GetOutputResponseTimeDescription(
                    GetOutputResponseTimeDescriptionRequest::decode_parameter_data(buf)?,
                ))
            }
            (CommandClass::Get, ParameterId::ModulationFrequency) => {
                Ok(Self::GetModulationFrequency)
            }
            (CommandClass::Set, ParameterId::ModulationFrequency) => {
                Ok(Self::SetModulationFrequency(
                    SetModulationFrequencyRequest::decode_parameter_data(buf)?,
                ))
            }
            (CommandClass::Get, ParameterId::ModulationFrequencyDescription) => {
                Ok(Self::GetModulationFrequencyDescription(
                    GetModulationFrequencyDescriptionRequest::decode_parameter_data(buf)?,
                ))
            }
            (CommandClass::Get, ParameterId::PowerOnSelfTest) => Ok(Self::GetPowerOnSelfTest),
            (CommandClass::Set, ParameterId::PowerOnSelfTest) => Ok(Self::SetPowerOnSelfTest(
                SetPowerOnSelfTestRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::LockState) => Ok(Self::GetLockState),
            (CommandClass::Set, ParameterId::LockState) => Ok(Self::SetLockState(
                SetLockStateRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::LockStateDescription) => {
                Ok(Self::GetLockStateDescription)
            }
            (CommandClass::Get, ParameterId::LockPin) => Ok(Self::GetLockPin),
            (CommandClass::Set, ParameterId::LockPin) => Ok(Self::SetLockPin(
                SetLockPinRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::BurnIn) => Ok(Self::GetBurnIn),
            (CommandClass::Set, ParameterId::BurnIn) => Ok(Self::SetBurnIn(
                SetBurnInRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::IdentifyMode) => Ok(Self::GetIdentifyMode),
            (CommandClass::Set, ParameterId::IdentifyMode) => Ok(Self::SetIdentifyMode(
                SetIdentifyModeRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::PresetInfo) => Ok(Self::GetPresetInfo),
            (CommandClass::Get, ParameterId::PresetStatus) => Ok(Self::GetPresetStatus(
                GetPresetStatusRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Set, ParameterId::PresetStatus) => Ok(Self::SetPresetStatus(
                SetPresetStatusRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::PresetMergeMode) => Ok(Self::GetPresetMergeMode),
            (CommandClass::Set, ParameterId::PresetMergeMode) => Ok(Self::SetPresetMergeMode(
                SetPresetMergeModeRequest::decode_parameter_data(buf)?,
            )),
            // E1.37-2
            (CommandClass::Get, ParameterId::ListInterfaces) => Ok(Self::GetListInterfaces),
            (CommandClass::Get, ParameterId::InterfaceLabel) => Ok(Self::GetInterfaceLabel(
                GetInterfaceLabelRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::InterfaceHardwareAddressType1) => {
                Ok(Self::GetInterfaceHardwareAddressType1(
                    GetInterfaceHardwareAddressType1Request::decode_parameter_data(buf)?,
                ))
            }
            (CommandClass::Get, ParameterId::IpV4DhcpMode) => Ok(Self::GetIpV4DhcpMode(
                GetIpV4DhcpModeRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Set, ParameterId::IpV4DhcpMode) => Ok(Self::SetIpV4DhcpMode(
                SetIpV4DhcpModeRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::IpV4ZeroConfMode) => Ok(Self::GetIpV4ZeroConfMode(
                GetIpV4ZeroConfModeRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Set, ParameterId::IpV4ZeroConfMode) => Ok(Self::SetIpV4ZeroConfMode(
                SetIpV4ZeroConfModeRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::IpV4CurrentAddress) => {
                Ok(Self::GetIpV4CurrentAddress(
                    GetIpV4CurrentAddressRequest::decode_parameter_data(buf)?,
                ))
            }
            (CommandClass::Get, ParameterId::IpV4StaticAddress) => Ok(Self::GetIpV4StaticAddress(
                GetIpV4StaticAddressRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Set, ParameterId::IpV4StaticAddress) => Ok(Self::SetIpV4StaticAddress(
                SetIpV4StaticAddressRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Set, ParameterId::InterfaceApplyConfiguration) => {
                Ok(Self::SetInterfaceApplyConfiguration(
                    SetInterfaceApplyConfigurationRequest::decode_parameter_data(buf)?,
                ))
            }
            (CommandClass::Set, ParameterId::InterfaceRenewDhcp) => {
                Ok(Self::SetInterfaceRenewDhcp(
                    SetInterfaceRenewDhcpRequest::decode_parameter_data(buf)?,
                ))
            }
            (CommandClass::Set, ParameterId::InterfaceReleaseDhcp) => {
                Ok(Self::SetInterfaceReleaseDhcp(
                    SetInterfaceReleaseDhcpRequest::decode_parameter_data(buf)?,
                ))
            }
            (CommandClass::Get, ParameterId::IpV4DefaultRoute) => Ok(Self::GetIpV4DefaultRoute),
            (CommandClass::Set, ParameterId::IpV4DefaultRoute) => Ok(Self::SetIpV4DefaultRoute(
                SetIpV4DefaultRouteRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::DnsIpV4NameServer) => Ok(Self::GetDnsIpV4NameServer(
                GetDnsIpV4NameServerRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Set, ParameterId::DnsIpV4NameServer) => Ok(Self::SetDnsIpV4NameServer(
                SetDnsIpv4NameServerRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::DnsHostName) => Ok(Self::GetDnsHostName),
            (CommandClass::Set, ParameterId::DnsHostName) => Ok(Self::SetDnsHostName(
                SetDnsHostNameRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::DnsDomainName) => Ok(Self::GetDnsDomainName),
            (CommandClass::Set, ParameterId::DnsDomainName) => Ok(Self::SetDnsDomainName(
                SetDnsDomainNameRequest::decode_parameter_data(buf)?,
            )),
            // E1.37-7
            (CommandClass::Get, ParameterId::EndpointList) => Ok(Self::GetEndpointList),
            (CommandClass::Get, ParameterId::EndpointListChange) => Ok(Self::GetEndpointListChange),
            (CommandClass::Get, ParameterId::IdentifyEndpoint) => Ok(Self::GetIdentifyEndpoint(
                GetIdentifyEndpointRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Set, ParameterId::IdentifyEndpoint) => Ok(Self::SetIdentifyEndpoint(
                SetIdentifyEndpointRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::EndpointToUniverse) => {
                Ok(Self::GetEndpointToUniverse(
                    GetEndpointToUniverseRequest::decode_parameter_data(buf)?,
                ))
            }
            (CommandClass::Set, ParameterId::EndpointToUniverse) => {
                Ok(Self::SetEndpointToUniverse(
                    SetEndpointToUniverseRequest::decode_parameter_data(buf)?,
                ))
            }
            (CommandClass::Get, ParameterId::EndpointMode) => Ok(Self::GetEndpointMode(
                GetEndpointModeRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Set, ParameterId::EndpointMode) => Ok(Self::SetEndpointMode(
                SetEndpointModeRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::EndpointLabel) => Ok(Self::GetEndpointLabel(
                GetEndpointLabelRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Set, ParameterId::EndpointLabel) => Ok(Self::SetEndpointLabel(
                SetEndpointLabelRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::RdmTrafficEnable) => Ok(Self::GetRdmTrafficEnable(
                GetRdmTrafficEnableRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Set, ParameterId::RdmTrafficEnable) => Ok(Self::SetRdmTrafficEnable(
                SetRdmTrafficEnableRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::DiscoveryState) => Ok(Self::GetDiscoveryState(
                GetDiscoveryStateRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Set, ParameterId::DiscoveryState) => Ok(Self::SetDiscoveryState(
                SetDiscoveryStateRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::BackgroundDiscovery) => {
                Ok(Self::GetBackgroundDiscovery(
                    GetBackgroundDiscoveryRequest::decode_parameter_data(buf)?,
                ))
            }
            (CommandClass::Set, ParameterId::BackgroundDiscovery) => {
                Ok(Self::SetBackgroundDiscovery(
                    SetBackgroundDiscoveryRequest::decode_parameter_data(buf)?,
                ))
            }
            (CommandClass::Get, ParameterId::EndpointTiming) => Ok(Self::GetEndpointTiming(
                GetEndpointTimingRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Set, ParameterId::EndpointTiming) => Ok(Self::SetEndpointTiming(
                SetEndpointTimingRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::EndpointTimingDescription) => {
                Ok(Self::GetEndpointTimingDescription(
                    GetEndpointTimingDescriptionRequest::decode_parameter_data(buf)?,
                ))
            }
            (CommandClass::Get, ParameterId::EndpointResponders) => {
                Ok(Self::GetEndpointResponders(
                    GetEndpointRespondersRequest::decode_parameter_data(buf)?,
                ))
            }
            (CommandClass::Get, ParameterId::EndpointResponderListChange) => {
                Ok(Self::GetEndpointResponderListChange(
                    GetEndpointResponderListChangeRequest::decode_parameter_data(buf)?,
                ))
            }
            (CommandClass::Get, ParameterId::BindingControlFields) => {
                Ok(Self::GetBindingControlFields(
                    GetBindingControlFieldsRequest::decode_parameter_data(buf)?,
                ))
            }
            (CommandClass::Get, ParameterId::BackgroundQueuedStatusPolicy) => {
                Ok(Self::GetBackgroundQueuedStatusPolicy)
            }
            (CommandClass::Set, ParameterId::BackgroundQueuedStatusPolicy) => {
                Ok(Self::SetBackgroundQueuedStatusPolicy(
                    SetBackgroundQueuedStatusPolicyRequest::decode_parameter_data(buf)?,
                ))
            }
            (CommandClass::Get, ParameterId::BackgroundQueuedStatusPolicyDescription) => {
                Ok(Self::GetBackgroundQueuedStatusPolicyDescription(
                    GetBackgroundQueuedStatusPolicyDescriptionRequest::decode_parameter_data(buf)?,
                ))
            }
            // E1.33
            (CommandClass::Get, ParameterId::ComponentScope) => Ok(Self::GetComponentScope(
                GetComponentScopeRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Set, ParameterId::ComponentScope) => Ok(Self::SetComponentScope(
                SetComponentScopeRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::SearchDomain) => Ok(Self::GetSearchDomain),
            (CommandClass::Set, ParameterId::SearchDomain) => Ok(Self::SetSearchDomain(
                SetSearchDomainRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::TcpCommsStatus) => Ok(Self::GetTcpCommsStatus),
            (CommandClass::Set, ParameterId::TcpCommsStatus) => Ok(Self::SetTcpCommsStatus(
                SetTcpCommsStatusRequest::decode_parameter_data(buf)?,
            )),
            (CommandClass::Get, ParameterId::BrokerStatus) => Ok(Self::GetBrokerStatus),
            (CommandClass::Set, ParameterId::BrokerStatus) => Ok(Self::SetBrokerStatus(
                SetBrokerStatusRequest::decode_parameter_data(buf)?,
            )),
            // Manufacturer or Unsupported
            (command_class, parameter_id) => Ok(Self::CustomParameter(CustomRequestParameter {
                command_class,
                parameter_id,
                parameter_data_length: buf.len() as u8,
                parameter_data: Vec::from_slice(buf).unwrap(),
            })),
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
        Self {
            destination_uid,
            source_uid,
            transaction_number,
            port_id,
            sub_device_id,
            parameter,
        }
    }

    pub fn new_custom_parameter(
        destination_uid: DeviceUID,
        source_uid: DeviceUID,
        transaction_number: u8,
        port_id: u8,
        sub_device_id: SubDeviceId,
        parameter: impl RdmParameter,
    ) -> Result<Self, RdmError> {
        let mut parameter_data = [0u8; 231];

        let bytes_written = parameter.encode_parameter_data(&mut parameter_data)?;

        Ok(Self {
            destination_uid,
            source_uid,
            transaction_number,
            port_id,
            sub_device_id,
            parameter: RequestParameter::CustomParameter(CustomRequestParameter {
                command_class: parameter.command_class(),
                parameter_id: parameter.parameter_id(),
                parameter_data_length: bytes_written as u8,
                parameter_data: Vec::from_slice(&parameter_data[0..bytes_written]).unwrap(),
            }),
        })
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
    use rdm_derive::rdm_request_parameter;

    use super::*;

    #[test]
    fn should_encode_discovery_unique_branch_request() {
        let mut buf = [0u8; 38];

        let frame = RdmRequest::new(
            DeviceUID::new(0x0102, 0x03040506),
            DeviceUID::new(0x0605, 0x04030201),
            0x00,
            0x01,
            SubDeviceId::Id(0x01),
            RequestParameter::DiscUniqueBranch(DiscUniqueBranchRequest {
                lower_bound_uid: DeviceUID::new(0x0000, 0x00000000),
                upper_bound_uid: DeviceUID::new(0xffff, 0xffffffff),
            }),
        );

        let bytes_written = frame.encode(&mut buf).unwrap();

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

        assert_eq!(&buf[0..bytes_written], expected);

        let decoded = RdmRequest::decode(&buf[0..bytes_written]).unwrap();

        assert_eq!(decoded, frame);
    }

    #[test]
    fn should_encode_decode_valid_request() {
        let mut buf = [0u8; 26];

        let frame = RdmRequest::new(
            DeviceUID::new(0x0102, 0x03040506),
            DeviceUID::new(0x0605, 0x04030201),
            0x00,
            0x01,
            SubDeviceId::RootDevice,
            RequestParameter::GetIdentifyDevice,
        );

        let bytes_encoded = frame.encode(&mut buf).unwrap();

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
            0x20, // Command Class = Get
            0x10, 0x00, // Parameter ID = Identify Device
            0x00, // PDL
            0x01, 0x40, // Checksum
        ];

        assert_eq!(&buf[0..bytes_encoded], expected);

        let decoded = RdmRequest::decode(&buf[0..bytes_encoded]).unwrap();

        assert_eq!(decoded, frame);
    }

    #[test]
    fn should_encode_decode_manufacturer_specific_request() {
        let mut buf = [0u8; 30];

        let frame = RdmRequest::new(
            DeviceUID::new(0x0102, 0x03040506),
            DeviceUID::new(0x0605, 0x04030201),
            0x00,
            0x01,
            SubDeviceId::RootDevice,
            RequestParameter::CustomParameter(CustomRequestParameter {
                command_class: CommandClass::Set,
                parameter_data_length: 4,
                parameter_id: ParameterId::Custom(0x8080),
                parameter_data: Vec::from_slice(&[0x01, 0x02, 0x03, 0x04]).unwrap(),
            }),
        );

        let bytes_encoded = frame.encode(&mut buf).unwrap();

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
            0x30, // Command Class = Set
            0x80, 0x80, // Parameter ID = Identify Device
            0x04, // PDL
            0x01, 0x02, 0x03, 0x04, // Parameter Data
            0x02, 0x52, // Checksum
        ];

        assert_eq!(&buf[0..bytes_encoded], expected);

        let decoded = RdmRequest::decode(&buf[0..bytes_encoded]).unwrap();

        assert_eq!(decoded, frame);
    }

    #[test]
    fn should_encode_decode_derived_manufacturer_specific_request() {
        #[derive(Clone, Debug, PartialEq)]
        #[rdm_request_parameter(pid = ParameterId::Custom(0x8080), command_class = CommandClass::Set)]
        struct CustomDerivedParameter {
            pub some_field: u32,
        }

        let mut buf = [0u8; 30];

        let frame = RdmRequest::new_custom_parameter(
            DeviceUID::new(0x0102, 0x03040506),
            DeviceUID::new(0x0605, 0x04030201),
            0x00,
            0x01,
            SubDeviceId::RootDevice,
            CustomDerivedParameter {
                some_field: 0x01020304,
            },
        )
        .unwrap();

        let bytes_encoded = frame.encode(&mut buf).unwrap();

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
            0x30, // Command Class = Set
            0x80, 0x80, // Parameter ID = Manufacturer Specific
            0x04, // PDL
            0x01, 0x02, 0x03, 0x04, // Parameter Data
            0x02, 0x52, // Checksum
        ];

        assert_eq!(&buf[0..bytes_encoded], expected);

        let decoded = RdmRequest::decode(&buf[0..bytes_encoded]).unwrap();

        assert_eq!(decoded, frame);
    }
}
