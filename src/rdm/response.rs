//! Data types and functionality for decoding RDM responses
//!
//! ### RdmResponse
//!
//! ```ignore
//! use dmx512_rdm_protocol::rdm::{
//!     parameter::{
//!         ParameterId,
//!         e120::GetIdentifyDeviceResponse,
//!     },
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
//!     0x21, // Command Class = GetResponse
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
//!     command_class: CommandClass::GetResponse,
//!     parameter_id: ParameterId::IdentifyDevice,
//!     parameter_data: ResponseData::ParameterData(Some(
//!         ResponseParameterData::GetIdentifyDevice(
//!            GetIdentifyDeviceResponse { identify: true }
//!         ),
//!     )),
//! }));
//!
//! assert_eq!(decoded, expected);
//! ```

use super::{
    DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE, DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE,
    RDM_START_CODE_BYTE, RDM_SUB_START_CODE_BYTE, utils::bsd_16_crc,
};
use crate::rdm::{
    MIN_DISC_FRAME_LENGTH, MIN_RDM_FRAME_LENGTH,
    parameter::{
        e120::response::{
            DiscMuteResponse, DiscUnMuteResponse, DiscoveryUniqueBranchFrameResponse,
            GetBootSoftwareVersionIdResponse, GetBootSoftwareVersionLabelResponse,
            GetCommsStatusResponse, GetDefaultSlotValueResponse, GetDeviceHoursResponse,
            GetDeviceInfoResponse, GetDeviceLabelResponse, GetDeviceModelDescriptionResponse,
            GetDevicePowerCyclesResponse, GetDisplayInvertResponse, GetDisplayLevelResponse,
            GetDmxPersonalityDescriptionResponse, GetDmxPersonalityResponse,
            GetDmxStartAddressResponse, GetFactoryDefaultsResponse, GetIdentifyDeviceResponse,
            GetLampHoursResponse, GetLampOnModeResponse, GetLampStateResponse,
            GetLampStrikesResponse, GetLanguageCapabilitiesResponse, GetLanguageResponse,
            GetManufacturerLabelResponse, GetPanInvertResponse, GetPanTiltSwapResponse,
            GetParameterDescriptionResponse, GetPerformSelfTestResponse, GetPowerStateResponse,
            GetPresetPlaybackResponse, GetProductDetailIdListResponse,
            GetProxiedDeviceCountResponse, GetProxiedDevicesResponse, GetRealTimeClockResponse,
            GetSelfTestDescriptionResponse, GetSensorDefinitionResponse, GetSensorValueResponse,
            GetSlotDescriptionResponse, GetSlotInfoResponse, GetSoftwareVersionLabelResponse,
            GetStatusIdDescriptionResponse, GetStatusMessagesResponse,
            GetSubDeviceIdStatusReportThresholdResponse, GetSupportedParametersResponse,
            GetTiltInvertResponse, SetSensorValueResponse,
        },
        e133::response::{
            GetBrokerStatusResponse, GetComponentScopeResponse, GetSearchDomainResponse,
            GetTcpCommsStatusResponse,
        },
        e137_1::response::{
            GetBurnInResponse, GetCurveDescriptionResponse, GetCurveResponse,
            GetDimmerInfoResponse, GetDmxBlockAddressResponse, GetDmxFailModeResponse,
            GetDmxStartupModeResponse, GetIdentifyModeResponse, GetLockPinResponse,
            GetLockStateDescriptionResponse, GetLockStateResponse, GetMaximumLevelResponse,
            GetMinimumLevelResponse, GetModulationFrequencyDescriptionResponse,
            GetModulationFrequencyResponse, GetOutputResponseTimeDescriptionResponse,
            GetOutputResponseTimeResponse, GetPowerOnSelfTestResponse, GetPresetInfoResponse,
            GetPresetMergeModeResponse, GetPresetStatusResponse,
        },
        e137_2::response::{
            GetDnsDomainNameResponse, GetDnsHostNameResponse, GetDnsIpV4NameServerResponse,
            GetInterfaceHardwareAddressType1Response, GetInterfaceLabelResponse,
            GetIpV4CurrentAddressResponse, GetIpV4DefaultRouteResponse, GetIpV4DhcpModeResponse,
            GetIpV4StaticAddressResponse, GetIpV4ZeroConfModeResponse, GetListInterfacesResponse,
        },
        e137_7::response::{
            GetBackgroundDiscoveryResponse, GetBackgroundQueuedStatusPolicyDescriptionResponse,
            GetBackgroundQueuedStatusPolicyResponse, GetBindingControlFieldsResponse,
            GetDiscoveryStateResponse, GetEndpointLabelResponse, GetEndpointListChangeResponse,
            GetEndpointListResponse, GetEndpointModeResponse,
            GetEndpointResponderListChangeResponse, GetEndpointRespondersResponse,
            GetEndpointTimingDescriptionResponse, GetEndpointTimingResponse,
            GetEndpointToUniverseResponse, GetIdentifyEndpointResponse,
            GetRdmTrafficEnableResponse, SetBackgroundDiscoveryResponse, SetDiscoveryStateResponse,
            SetEndpointLabelResponse, SetEndpointModeResponse, SetEndpointTimingResponse,
            SetEndpointToUniverseResponse, SetIdentifyEndpointResponse,
            SetRdmTrafficEnableResponse,
        },
    },
};
use core::convert::TryFrom;
use heapless::Vec;
use rdm_core::{
    CommandClass, DeviceUID, ParameterId, ResponseResult, SubDeviceId,
    parameter_traits::RdmParameterData, response::CustomResponseParameter,
};
use rdm_core::{
    ResponseType,
    error::{ParameterCodecError, RdmError},
};

#[allow(clippy::large_enum_variant)]
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum ResponseParameter {
    // E1.20
    DiscMute(DiscMuteResponse),
    DiscUnMute(DiscUnMuteResponse),
    GetProxiedDeviceCount(ResponseResult<GetProxiedDeviceCountResponse>),
    GetProxiedDevices(ResponseResult<GetProxiedDevicesResponse>),
    GetCommsStatus(ResponseResult<GetCommsStatusResponse>),
    SetCommsStatus(ResponseResult<()>),
    GetStatusMessages(ResponseResult<GetStatusMessagesResponse>),
    GetStatusIdDescription(ResponseResult<GetStatusIdDescriptionResponse>),
    SetClearStatusId(ResponseResult<()>),
    GetSubDeviceIdStatusReportThreshold(
        ResponseResult<GetSubDeviceIdStatusReportThresholdResponse>,
    ),
    SetSubDeviceIdStatusReportThreshold(ResponseResult<()>),
    GetSupportedParameters(ResponseResult<GetSupportedParametersResponse>),
    GetParameterDescription(ResponseResult<GetParameterDescriptionResponse>),
    GetDeviceInfo(ResponseResult<GetDeviceInfoResponse>),
    GetProductDetailIdList(ResponseResult<GetProductDetailIdListResponse>),
    GetDeviceModelDescription(ResponseResult<GetDeviceModelDescriptionResponse>),
    GetManufacturerLabel(ResponseResult<GetManufacturerLabelResponse>),
    SetManufacturerLabel(ResponseResult<()>),
    GetDeviceLabel(ResponseResult<GetDeviceLabelResponse>),
    SetDeviceLabel(ResponseResult<()>),
    GetFactoryDefaults(ResponseResult<GetFactoryDefaultsResponse>),
    SetFactoryDefaults(ResponseResult<()>),
    GetLanguageCapabilities(ResponseResult<GetLanguageCapabilitiesResponse>),
    GetLanguage(ResponseResult<GetLanguageResponse>),
    SetLanguage(ResponseResult<()>),
    GetSoftwareVersionLabel(ResponseResult<GetSoftwareVersionLabelResponse>),
    GetBootSoftwareVersionId(ResponseResult<GetBootSoftwareVersionIdResponse>),
    GetBootSoftwareVersionLabel(ResponseResult<GetBootSoftwareVersionLabelResponse>),
    GetDmxPersonality(ResponseResult<GetDmxPersonalityResponse>),
    SetDmxPersonality(ResponseResult<()>),
    GetDmxPersonalityDescription(ResponseResult<GetDmxPersonalityDescriptionResponse>),
    GetDmxStartAddress(ResponseResult<GetDmxStartAddressResponse>),
    SetDmxStartAddress(ResponseResult<()>),
    GetSlotInfo(ResponseResult<GetSlotInfoResponse>),
    GetSlotDescription(ResponseResult<GetSlotDescriptionResponse>),
    GetDefaultSlotValue(ResponseResult<GetDefaultSlotValueResponse>),
    GetSensorDefinition(ResponseResult<GetSensorDefinitionResponse>),
    GetSensorValue(ResponseResult<GetSensorValueResponse>),
    SetSensorValue(ResponseResult<SetSensorValueResponse>),
    SetRecordSensors(ResponseResult<()>),
    GetDeviceHours(ResponseResult<GetDeviceHoursResponse>),
    SetDeviceHours(ResponseResult<()>),
    GetLampHours(ResponseResult<GetLampHoursResponse>),
    SetLampHours(ResponseResult<()>),
    GetLampStrikes(ResponseResult<GetLampStrikesResponse>),
    SetLampStrikes(ResponseResult<()>),
    GetLampState(ResponseResult<GetLampStateResponse>),
    SetLampState(ResponseResult<()>),
    GetLampOnMode(ResponseResult<GetLampOnModeResponse>),
    SetLampOnMode(ResponseResult<()>),
    GetDevicePowerCycles(ResponseResult<GetDevicePowerCyclesResponse>),
    SetDevicePowerCycles(ResponseResult<()>),
    GetDisplayInvert(ResponseResult<GetDisplayInvertResponse>),
    SetDisplayInvert(ResponseResult<()>),
    GetDisplayLevel(ResponseResult<GetDisplayLevelResponse>),
    SetDisplayLevel(ResponseResult<()>),
    GetPanInvert(ResponseResult<GetPanInvertResponse>),
    SetPanInvert(ResponseResult<()>),
    GetTiltInvert(ResponseResult<GetTiltInvertResponse>),
    SetTiltInvert(ResponseResult<()>),
    GetPanTiltSwap(ResponseResult<GetPanTiltSwapResponse>),
    SetPanTiltSwap(ResponseResult<()>),
    GetRealTimeClock(ResponseResult<GetRealTimeClockResponse>),
    SetRealTimeClock(ResponseResult<()>),
    GetIdentifyDevice(ResponseResult<GetIdentifyDeviceResponse>),
    SetIdentifyDevice(ResponseResult<()>),
    GetPowerState(ResponseResult<GetPowerStateResponse>),
    SetPowerState(ResponseResult<()>),
    GetPerformSelfTest(ResponseResult<GetPerformSelfTestResponse>),
    SetPerformSelfTest(ResponseResult<()>),
    GetSelfTestDescription(ResponseResult<GetSelfTestDescriptionResponse>),
    SetCapturePreset(ResponseResult<()>),
    GetPresetPlayback(ResponseResult<GetPresetPlaybackResponse>),
    SetPresetPlayback(ResponseResult<()>),
    // E1.37-1
    GetDmxBlockAddress(ResponseResult<GetDmxBlockAddressResponse>),
    SetDmxBlockAddress(ResponseResult<()>),
    GetDmxFailMode(ResponseResult<GetDmxFailModeResponse>),
    SetDmxFailMode(ResponseResult<()>),
    GetDmxStartupMode(ResponseResult<GetDmxStartupModeResponse>),
    SetDmxStartupMode(ResponseResult<()>),
    GetDimmerInfo(ResponseResult<GetDimmerInfoResponse>),
    GetMinimumLevel(ResponseResult<GetMinimumLevelResponse>),
    SetMinimumLevel(ResponseResult<()>),
    GetMaximumLevel(ResponseResult<GetMaximumLevelResponse>),
    SetMaximumLevel(ResponseResult<()>),
    GetCurve(ResponseResult<GetCurveResponse>),
    SetCurve(ResponseResult<()>),
    GetCurveDescription(ResponseResult<GetCurveDescriptionResponse>),
    GetOutputResponseTime(ResponseResult<GetOutputResponseTimeResponse>),
    SetOutputResponseTime(ResponseResult<()>),
    GetOutputResponseTimeDescription(ResponseResult<GetOutputResponseTimeDescriptionResponse>),
    GetModulationFrequency(ResponseResult<GetModulationFrequencyResponse>),
    SetModulationFrequency(ResponseResult<()>),
    GetModulationFrequencyDescription(ResponseResult<GetModulationFrequencyDescriptionResponse>),
    GetBurnIn(ResponseResult<GetBurnInResponse>),
    SetBurnIn(ResponseResult<()>),
    GetLockPin(ResponseResult<GetLockPinResponse>),
    SetLockPin(ResponseResult<()>),
    GetLockState(ResponseResult<GetLockStateResponse>),
    SetLockState(ResponseResult<()>),
    GetLockStateDescription(ResponseResult<GetLockStateDescriptionResponse>),
    GetIdentifyMode(ResponseResult<GetIdentifyModeResponse>),
    SetIdentifyMode(ResponseResult<()>),
    GetPresetInfo(ResponseResult<GetPresetInfoResponse>),
    GetPresetStatus(ResponseResult<GetPresetStatusResponse>),
    SetPresetStatus(ResponseResult<()>),
    GetPresetMergeMode(ResponseResult<GetPresetMergeModeResponse>),
    SetPresetMergeMode(ResponseResult<()>),
    GetPowerOnSelfTest(ResponseResult<GetPowerOnSelfTestResponse>),
    SetPowerOnSelfTest(ResponseResult<()>),
    // E1.37-2
    GetListInterfaces(ResponseResult<GetListInterfacesResponse>),
    GetInterfaceLabel(ResponseResult<GetInterfaceLabelResponse>),
    GetInterfaceHardwareAddressType1(ResponseResult<GetInterfaceHardwareAddressType1Response>),
    GetIpV4DhcpMode(ResponseResult<GetIpV4DhcpModeResponse>),
    SetIpV4DhcpMode(ResponseResult<()>),
    GetIpV4ZeroConfMode(ResponseResult<GetIpV4ZeroConfModeResponse>),
    SetIpV4ZeroConfMode(ResponseResult<()>),
    GetIpV4CurrentAddress(ResponseResult<GetIpV4CurrentAddressResponse>),
    GetIpV4StaticAddress(ResponseResult<GetIpV4StaticAddressResponse>),
    SetIpV4StaticAddress(ResponseResult<()>),
    SetInterfaceRenewDhcp(ResponseResult<()>),
    SetInterfaceReleaseDhcp(ResponseResult<()>),
    SetInterfaceApplyConfiguration(ResponseResult<()>),
    GetIpV4DefaultRoute(ResponseResult<GetIpV4DefaultRouteResponse>),
    SetIpV4DefaultRoute(ResponseResult<()>),
    GetDnsIpV4NameServer(ResponseResult<GetDnsIpV4NameServerResponse>),
    SetDnsIpV4NameServer(ResponseResult<()>),
    GetDnsHostName(ResponseResult<GetDnsHostNameResponse>),
    SetDnsHostName(ResponseResult<()>),
    GetDnsDomainName(ResponseResult<GetDnsDomainNameResponse>),
    SetDnsDomainName(ResponseResult<()>),
    // E1.37-7
    GetEndpointList(ResponseResult<GetEndpointListResponse>),
    GetEndpointListChange(ResponseResult<GetEndpointListChangeResponse>),
    GetIdentifyEndpoint(ResponseResult<GetIdentifyEndpointResponse>),
    SetIdentifyEndpoint(ResponseResult<SetIdentifyEndpointResponse>),
    GetEndpointToUniverse(ResponseResult<GetEndpointToUniverseResponse>),
    SetEndpointToUniverse(ResponseResult<SetEndpointToUniverseResponse>),
    GetEndpointMode(ResponseResult<GetEndpointModeResponse>),
    SetEndpointMode(ResponseResult<SetEndpointModeResponse>),
    GetEndpointLabel(ResponseResult<GetEndpointLabelResponse>),
    SetEndpointLabel(ResponseResult<SetEndpointLabelResponse>),
    GetRdmTrafficEnable(ResponseResult<GetRdmTrafficEnableResponse>),
    SetRdmTrafficEnable(ResponseResult<SetRdmTrafficEnableResponse>),
    GetDiscoveryState(ResponseResult<GetDiscoveryStateResponse>),
    SetDiscoveryState(ResponseResult<SetDiscoveryStateResponse>),
    GetBackgroundDiscovery(ResponseResult<GetBackgroundDiscoveryResponse>),
    SetBackgroundDiscovery(ResponseResult<SetBackgroundDiscoveryResponse>),
    GetEndpointTiming(ResponseResult<GetEndpointTimingResponse>),
    SetEndpointTiming(ResponseResult<SetEndpointTimingResponse>),
    GetEndpointTimingDescription(ResponseResult<GetEndpointTimingDescriptionResponse>),
    GetEndpointResponders(ResponseResult<GetEndpointRespondersResponse>),
    GetEndpointResponderListChange(ResponseResult<GetEndpointResponderListChangeResponse>),
    GetBindingControlFields(ResponseResult<GetBindingControlFieldsResponse>),
    GetBackgroundQueuedStatusPolicy(ResponseResult<GetBackgroundQueuedStatusPolicyResponse>),
    SetBackgroundQueuedStatusPolicy(ResponseResult<()>),
    GetBackgroundQueuedStatusPolicyDescription(
        ResponseResult<GetBackgroundQueuedStatusPolicyDescriptionResponse>,
    ),
    // E1.33
    GetSearchDomain(ResponseResult<GetSearchDomainResponse>),
    SetSearchDomain(ResponseResult<()>),
    GetComponentScope(ResponseResult<GetComponentScopeResponse>),
    SetComponentScope(ResponseResult<()>),
    GetTcpCommsStatus(ResponseResult<GetTcpCommsStatusResponse>),
    SetTcpCommsStatus(ResponseResult<()>),
    GetBrokerStatus(ResponseResult<GetBrokerStatusResponse>),
    SetBrokerStatus(ResponseResult<()>),
    // Manufacturer or Unsupported
    CustomParameter(CustomResponseParameter),
}

impl ResponseParameter {
    fn response_type(&self) -> ResponseType {
        match self {
            // E1.20
            Self::DiscMute(_) => ResponseType::Ack,
            Self::DiscUnMute(_) => ResponseType::Ack,
            Self::GetProxiedDeviceCount(param) => param.response_type(),
            Self::GetProxiedDevices(param) => param.response_type(),
            Self::GetCommsStatus(param) => param.response_type(),
            Self::SetCommsStatus(param) => param.response_type(),
            Self::GetStatusMessages(param) => param.response_type(),
            Self::GetStatusIdDescription(param) => param.response_type(),
            Self::SetClearStatusId(param) => param.response_type(),
            Self::GetSubDeviceIdStatusReportThreshold(param) => param.response_type(),
            Self::SetSubDeviceIdStatusReportThreshold(param) => param.response_type(),
            Self::GetSupportedParameters(param) => param.response_type(),
            Self::GetParameterDescription(param) => param.response_type(),
            Self::GetDeviceInfo(param) => param.response_type(),
            Self::GetProductDetailIdList(param) => param.response_type(),
            Self::GetDeviceModelDescription(param) => param.response_type(),
            Self::GetManufacturerLabel(param) => param.response_type(),
            Self::SetManufacturerLabel(param) => param.response_type(),
            Self::GetDeviceLabel(param) => param.response_type(),
            Self::SetDeviceLabel(param) => param.response_type(),
            Self::GetFactoryDefaults(param) => param.response_type(),
            Self::SetFactoryDefaults(param) => param.response_type(),
            Self::GetLanguageCapabilities(param) => param.response_type(),
            Self::GetLanguage(param) => param.response_type(),
            Self::SetLanguage(param) => param.response_type(),
            Self::GetSoftwareVersionLabel(param) => param.response_type(),
            Self::GetBootSoftwareVersionId(param) => param.response_type(),
            Self::GetBootSoftwareVersionLabel(param) => param.response_type(),
            Self::GetDmxPersonality(param) => param.response_type(),
            Self::SetDmxPersonality(param) => param.response_type(),
            Self::GetDmxPersonalityDescription(param) => param.response_type(),
            Self::GetDmxStartAddress(param) => param.response_type(),
            Self::SetDmxStartAddress(param) => param.response_type(),
            Self::GetSlotInfo(param) => param.response_type(),
            Self::GetSlotDescription(param) => param.response_type(),
            Self::GetDefaultSlotValue(param) => param.response_type(),
            Self::GetSensorDefinition(param) => param.response_type(),
            Self::GetSensorValue(param) => param.response_type(),
            Self::SetSensorValue(param) => param.response_type(),
            Self::SetRecordSensors(param) => param.response_type(),
            Self::GetDeviceHours(param) => param.response_type(),
            Self::SetDeviceHours(param) => param.response_type(),
            Self::GetLampHours(param) => param.response_type(),
            Self::SetLampHours(param) => param.response_type(),
            Self::GetLampStrikes(param) => param.response_type(),
            Self::SetLampStrikes(param) => param.response_type(),
            Self::GetLampState(param) => param.response_type(),
            Self::SetLampState(param) => param.response_type(),
            Self::GetLampOnMode(param) => param.response_type(),
            Self::SetLampOnMode(param) => param.response_type(),
            Self::GetDevicePowerCycles(param) => param.response_type(),
            Self::SetDevicePowerCycles(param) => param.response_type(),
            Self::GetDisplayInvert(param) => param.response_type(),
            Self::SetDisplayInvert(param) => param.response_type(),
            Self::GetDisplayLevel(param) => param.response_type(),
            Self::SetDisplayLevel(param) => param.response_type(),
            Self::GetPanInvert(param) => param.response_type(),
            Self::SetPanInvert(param) => param.response_type(),
            Self::GetTiltInvert(param) => param.response_type(),
            Self::SetTiltInvert(param) => param.response_type(),
            Self::GetPanTiltSwap(param) => param.response_type(),
            Self::SetPanTiltSwap(param) => param.response_type(),
            Self::GetRealTimeClock(param) => param.response_type(),
            Self::SetRealTimeClock(param) => param.response_type(),
            Self::GetIdentifyDevice(param) => param.response_type(),
            Self::SetIdentifyDevice(param) => param.response_type(),
            Self::GetPowerState(param) => param.response_type(),
            Self::SetPowerState(param) => param.response_type(),
            Self::GetPerformSelfTest(param) => param.response_type(),
            Self::SetPerformSelfTest(param) => param.response_type(),
            Self::GetSelfTestDescription(param) => param.response_type(),
            Self::SetCapturePreset(param) => param.response_type(),
            Self::GetPresetPlayback(param) => param.response_type(),
            Self::SetPresetPlayback(param) => param.response_type(),
            // E1.37-1
            Self::GetDmxBlockAddress(param) => param.response_type(),
            Self::SetDmxBlockAddress(param) => param.response_type(),
            Self::GetDmxFailMode(param) => param.response_type(),
            Self::SetDmxFailMode(param) => param.response_type(),
            Self::GetDmxStartupMode(param) => param.response_type(),
            Self::SetDmxStartupMode(param) => param.response_type(),
            Self::GetDimmerInfo(param) => param.response_type(),
            Self::GetMinimumLevel(param) => param.response_type(),
            Self::SetMinimumLevel(param) => param.response_type(),
            Self::GetMaximumLevel(param) => param.response_type(),
            Self::SetMaximumLevel(param) => param.response_type(),
            Self::GetCurve(param) => param.response_type(),
            Self::SetCurve(param) => param.response_type(),
            Self::GetCurveDescription(param) => param.response_type(),
            Self::GetOutputResponseTime(param) => param.response_type(),
            Self::SetOutputResponseTime(param) => param.response_type(),
            Self::GetOutputResponseTimeDescription(param) => param.response_type(),
            Self::GetModulationFrequency(param) => param.response_type(),
            Self::SetModulationFrequency(param) => param.response_type(),
            Self::GetModulationFrequencyDescription(param) => param.response_type(),
            Self::GetBurnIn(param) => param.response_type(),
            Self::SetBurnIn(param) => param.response_type(),
            Self::GetLockPin(param) => param.response_type(),
            Self::SetLockPin(param) => param.response_type(),
            Self::GetLockState(param) => param.response_type(),
            Self::SetLockState(param) => param.response_type(),
            Self::GetLockStateDescription(param) => param.response_type(),
            Self::GetIdentifyMode(param) => param.response_type(),
            Self::SetIdentifyMode(param) => param.response_type(),
            Self::GetPresetInfo(param) => param.response_type(),
            Self::GetPresetStatus(param) => param.response_type(),
            Self::SetPresetStatus(param) => param.response_type(),
            Self::GetPresetMergeMode(param) => param.response_type(),
            Self::SetPresetMergeMode(param) => param.response_type(),
            Self::GetPowerOnSelfTest(param) => param.response_type(),
            Self::SetPowerOnSelfTest(param) => param.response_type(),
            // E1.37-2
            Self::GetListInterfaces(param) => param.response_type(),
            Self::GetInterfaceLabel(param) => param.response_type(),
            Self::GetInterfaceHardwareAddressType1(param) => param.response_type(),
            Self::GetIpV4DhcpMode(param) => param.response_type(),
            Self::SetIpV4DhcpMode(param) => param.response_type(),
            Self::GetIpV4ZeroConfMode(param) => param.response_type(),
            Self::SetIpV4ZeroConfMode(param) => param.response_type(),
            Self::GetIpV4CurrentAddress(param) => param.response_type(),
            Self::GetIpV4StaticAddress(param) => param.response_type(),
            Self::SetIpV4StaticAddress(param) => param.response_type(),
            Self::SetInterfaceRenewDhcp(param) => param.response_type(),
            Self::SetInterfaceReleaseDhcp(param) => param.response_type(),
            Self::SetInterfaceApplyConfiguration(param) => param.response_type(),
            Self::GetIpV4DefaultRoute(param) => param.response_type(),
            Self::SetIpV4DefaultRoute(param) => param.response_type(),
            Self::GetDnsIpV4NameServer(param) => param.response_type(),
            Self::SetDnsIpV4NameServer(param) => param.response_type(),
            Self::GetDnsHostName(param) => param.response_type(),
            Self::SetDnsHostName(param) => param.response_type(),
            Self::GetDnsDomainName(param) => param.response_type(),
            Self::SetDnsDomainName(param) => param.response_type(),
            // E1.37-7
            Self::GetEndpointList(param) => param.response_type(),
            Self::GetEndpointListChange(param) => param.response_type(),
            Self::GetIdentifyEndpoint(param) => param.response_type(),
            Self::SetIdentifyEndpoint(param) => param.response_type(),
            Self::GetEndpointToUniverse(param) => param.response_type(),
            Self::SetEndpointToUniverse(param) => param.response_type(),
            Self::GetEndpointMode(param) => param.response_type(),
            Self::SetEndpointMode(param) => param.response_type(),
            Self::GetEndpointLabel(param) => param.response_type(),
            Self::SetEndpointLabel(param) => param.response_type(),
            Self::GetRdmTrafficEnable(param) => param.response_type(),
            Self::SetRdmTrafficEnable(param) => param.response_type(),
            Self::GetDiscoveryState(param) => param.response_type(),
            Self::SetDiscoveryState(param) => param.response_type(),
            Self::GetBackgroundDiscovery(param) => param.response_type(),
            Self::SetBackgroundDiscovery(param) => param.response_type(),
            Self::GetEndpointTiming(param) => param.response_type(),
            Self::SetEndpointTiming(param) => param.response_type(),
            Self::GetEndpointTimingDescription(param) => param.response_type(),
            Self::GetEndpointResponders(param) => param.response_type(),
            Self::GetEndpointResponderListChange(param) => param.response_type(),
            Self::GetBindingControlFields(param) => param.response_type(),
            Self::GetBackgroundQueuedStatusPolicy(param) => param.response_type(),
            Self::SetBackgroundQueuedStatusPolicy(param) => param.response_type(),
            Self::GetBackgroundQueuedStatusPolicyDescription(param) => param.response_type(),
            // E1.33
            Self::GetSearchDomain(param) => param.response_type(),
            Self::SetSearchDomain(param) => param.response_type(),
            Self::GetComponentScope(param) => param.response_type(),
            Self::SetComponentScope(param) => param.response_type(),
            Self::GetTcpCommsStatus(param) => param.response_type(),
            Self::SetTcpCommsStatus(param) => param.response_type(),
            Self::GetBrokerStatus(param) => param.response_type(),
            Self::SetBrokerStatus(param) => param.response_type(),
            // Manufacturer or Unsupported
            Self::CustomParameter(param) => param.response.response_type(),
        }
    }

    fn command_class(&self) -> CommandClass {
        match self {
            // E1.20
            Self::DiscMute(_) | Self::DiscUnMute(_) => CommandClass::DiscoveryResponse,
            // E1.20
            Self::GetProxiedDeviceCount(_)
            | Self::GetProxiedDevices(_)
            | Self::GetCommsStatus(_)
            | Self::GetStatusMessages(_)
            | Self::GetStatusIdDescription(_)
            | Self::GetSubDeviceIdStatusReportThreshold(_)
            | Self::GetSupportedParameters(_)
            | Self::GetParameterDescription(_)
            | Self::GetDeviceInfo(_)
            | Self::GetProductDetailIdList(_)
            | Self::GetDeviceModelDescription(_)
            | Self::GetManufacturerLabel(_)
            | Self::GetDeviceLabel(_)
            | Self::GetFactoryDefaults(_)
            | Self::GetLanguageCapabilities(_) |
            Self::GetLanguage(_) |
            Self::GetSoftwareVersionLabel(_) |
            Self::GetBootSoftwareVersionId(_) |
            Self::GetBootSoftwareVersionLabel(_) |
            Self::GetDmxPersonality(_) |
            Self::GetDmxPersonalityDescription(_) |
            Self::GetDmxStartAddress(_) |
            Self::GetSlotInfo(_) |
            Self::GetSlotDescription(_) |
            Self::GetDefaultSlotValue(_) |
            Self::GetSensorDefinition(_) |
            Self::GetSensorValue(_) |
            Self::GetDeviceHours(_) |
            Self::GetLampHours(_) |
            Self::GetLampStrikes(_) |
            Self::GetLampState(_) |
            Self::GetLampOnMode(_) |
            Self::GetDevicePowerCycles(_) |
            Self::GetDisplayInvert(_)
            | Self::GetDisplayLevel(_)
            | Self::GetPanInvert(_)
            | Self::GetTiltInvert(_)
| Self::GetPanTiltSwap(_)
| Self::GetRealTimeClock(_)
| Self::GetIdentifyDevice(_)
            | Self::GetPowerState(_)
| Self::GetPerformSelfTest(_)
| Self::GetSelfTestDescription(_)
| Self::GetPresetPlayback(_)
            // E1.37-1
            | Self::GetDmxBlockAddress(_)|
            Self::GetDmxFailMode(_)|
            Self::GetDmxStartupMode(_)|
            Self::GetDimmerInfo(_) |
            Self::GetMinimumLevel(_) |
            Self::GetMaximumLevel(_) |
            Self::GetCurve(_) |
            Self::GetCurveDescription(_) |
            Self::GetOutputResponseTime(_) |
            Self::GetOutputResponseTimeDescription(_) |
            Self::GetModulationFrequency(_) |
            Self::GetModulationFrequencyDescription(_) |
            Self::GetBurnIn(_) |
            Self::GetLockPin(_) |
            Self::GetLockState(_) |
            Self::GetLockStateDescription(_) |
            Self::GetIdentifyMode(_) |
            Self::GetPresetInfo(_) |
            Self::GetPresetStatus(_) |
            Self::GetPresetMergeMode(_) |
            Self::GetPowerOnSelfTest(_) |
            // E1.37-2
            Self::GetListInterfaces(_)|
            Self::GetInterfaceLabel(_)|
            Self::GetInterfaceHardwareAddressType1(_) |
            Self::GetIpV4DhcpMode(_) |
            Self::GetIpV4ZeroConfMode(_) |
            Self::GetIpV4CurrentAddress(_) |
            Self::GetIpV4StaticAddress(_) |
            Self::GetIpV4DefaultRoute(_) |
            Self::GetDnsIpV4NameServer(_) |
            Self::GetDnsHostName(_) |
            Self::GetDnsDomainName(_) |
            // E1.37-7
            Self::GetEndpointList(_) |
            Self::GetEndpointListChange(_) |
            Self::GetIdentifyEndpoint(_) |
            Self::GetEndpointToUniverse(_) |
            Self::GetEndpointMode(_) |
            Self::GetEndpointLabel(_) |
            Self::GetRdmTrafficEnable(_) |
            Self::GetDiscoveryState(_) |
            Self::GetBackgroundDiscovery(_) |
            Self::GetEndpointTiming(_) |
            Self::GetEndpointTimingDescription(_) |
            Self::GetEndpointResponders(_) |
            Self::GetEndpointResponderListChange(_) |
            Self::GetBindingControlFields(_) |
            Self::GetBackgroundQueuedStatusPolicy(_) |
            Self::GetBackgroundQueuedStatusPolicyDescription(_) |
            // E1.33
            Self::GetSearchDomain(_) |
            Self::GetComponentScope(_) |
            Self::GetTcpCommsStatus(_) |
            Self::GetBrokerStatus(_)


            => CommandClass::GetResponse,

            Self::SetCommsStatus(_) |
            Self::SetClearStatusId(_) |
            Self::SetSubDeviceIdStatusReportThreshold(_)
            | Self::SetManufacturerLabel(_)  | Self::SetDeviceLabel(_)
             | Self::SetFactoryDefaults(_)
             | Self::SetLanguage(_)  | Self::SetDmxPersonality(_) | Self::SetDmxStartAddress(_) | Self::SetDeviceHours(_)
             | Self::SetSensorValue(_) | Self::SetRecordSensors(_)
             | Self::SetLampHours(_) | Self::SetLampStrikes(_)
             | Self::SetLampState(_) | Self::SetLampOnMode(_)
             | Self::SetDevicePowerCycles(_)
             | Self::SetDisplayInvert(_)
             | Self::SetDisplayLevel(_)
             | Self::SetPanInvert(_)
             | Self::SetTiltInvert(_)
            | Self::SetPanTiltSwap(_)
            | Self::SetRealTimeClock(_)
            | Self::SetIdentifyDevice(_)
            | Self::SetPowerState(_)
            | Self::SetPerformSelfTest(_)
            | Self::SetPresetPlayback(_)
            // E1.37-1
            | Self::SetDmxBlockAddress(_)
            | Self::SetDmxFailMode(_)
            | Self::SetDmxStartupMode(_)
            | Self::SetCapturePreset(_)
            | Self::SetMinimumLevel(_)
            | Self::SetMaximumLevel(_)
| Self::SetCurve(_)
 | Self::SetOutputResponseTime(_)
 | Self::SetModulationFrequency(_)
 | Self::SetBurnIn(_)
| Self::SetLockPin(_)
 | Self::SetLockState(_)
| Self::SetIdentifyMode(_)
| Self::SetPresetStatus(_)
 | Self::SetPresetMergeMode(_)
 | Self::SetPowerOnSelfTest(_)
            // E1.37-2
| Self::SetIpV4DhcpMode(_)
 | Self::SetIpV4ZeroConfMode(_)
  | Self::SetIpV4StaticAddress(_) |
            Self::SetInterfaceRenewDhcp(_) |
            Self::SetInterfaceReleaseDhcp(_) |
            Self::SetInterfaceApplyConfiguration(_) |
Self::SetIpV4DefaultRoute(_) |
Self::SetDnsIpV4NameServer(_) |
Self::SetDnsHostName(_) |
 Self::SetDnsDomainName(_) |
            Self::SetIdentifyEndpoint(_) |
            Self::SetEndpointToUniverse(_) |
            Self::SetEndpointLabel(_) |
            Self::SetEndpointMode(_) |
            Self::SetRdmTrafficEnable(_) |
            Self::SetDiscoveryState(_) |
            Self::SetBackgroundDiscovery(_) |
            Self::SetEndpointTiming(_) |
            Self::SetBackgroundQueuedStatusPolicy(_) |
            // E1.33
            Self::SetSearchDomain(_) |
            Self::SetComponentScope(_) |
            Self::SetTcpCommsStatus(_) |
            Self::SetBrokerStatus(_)

             => CommandClass::SetResponse,
            // Manufacturer or Unsupported
            Self::CustomParameter(param) => param.command_class,
        }
    }

    fn parameter_id(&self) -> ParameterId {
        match self {
            // E1.20
            Self::DiscMute(_) => ParameterId::DiscMute,
            Self::DiscUnMute(_) => ParameterId::DiscUnMute,
            Self::GetProxiedDeviceCount(_) => ParameterId::ProxiedDeviceCount,
            Self::GetProxiedDevices(_) => ParameterId::ProxiedDevices,
            Self::GetCommsStatus(_) | Self::SetCommsStatus(_) => ParameterId::CommsStatus,
            Self::GetStatusMessages(_) => ParameterId::StatusMessages,
            Self::GetStatusIdDescription(_) => ParameterId::StatusIdDescription,
            Self::SetClearStatusId(_) => ParameterId::ClearStatusId,
            Self::GetSubDeviceIdStatusReportThreshold(_)
            | Self::SetSubDeviceIdStatusReportThreshold(_) => {
                ParameterId::SubDeviceIdStatusReportThreshold
            }
            Self::GetSupportedParameters(_) => ParameterId::SupportedParameters,
            Self::GetParameterDescription(_) => ParameterId::ParameterDescription,
            Self::GetDeviceInfo(_) => ParameterId::DeviceInfo,
            Self::GetProductDetailIdList(_) => ParameterId::ProductDetailIdList,
            Self::GetDeviceModelDescription(_) => ParameterId::DeviceModelDescription,
            Self::GetManufacturerLabel(_) | Self::SetManufacturerLabel(_) => {
                ParameterId::ManufacturerLabel
            }
            Self::GetDeviceLabel(_) | Self::SetDeviceLabel(_) => ParameterId::DeviceLabel,
            Self::GetFactoryDefaults(_) | Self::SetFactoryDefaults(_) => {
                ParameterId::FactoryDefaults
            }
            Self::GetLanguageCapabilities(_) => ParameterId::LanguageCapabilities,
            Self::GetLanguage(_) | Self::SetLanguage(_) => ParameterId::Language,
            Self::GetSoftwareVersionLabel(_) => ParameterId::SoftwareVersionLabel,
            Self::GetBootSoftwareVersionId(_) => ParameterId::BootSoftwareVersionId,
            Self::GetBootSoftwareVersionLabel(_) => ParameterId::BootSoftwareVersionLabel,
            Self::GetDmxPersonality(_) | Self::SetDmxPersonality(_) => ParameterId::DmxPersonality,
            Self::GetDmxPersonalityDescription(_) => ParameterId::DmxPersonalityDescription,
            Self::GetDmxStartAddress(_) | Self::SetDmxStartAddress(_) => {
                ParameterId::DmxStartAddress
            }
            Self::GetSlotInfo(_) => ParameterId::SlotInfo,
            Self::GetSlotDescription(_) => ParameterId::SlotDescription,
            Self::GetDefaultSlotValue(_) => ParameterId::DefaultSlotValue,
            Self::GetSensorDefinition(_) => ParameterId::SensorDefinition,
            Self::GetSensorValue(_) | Self::SetSensorValue(_) => ParameterId::SensorValue,
            Self::SetRecordSensors(_) => ParameterId::RecordSensors,
            Self::GetDeviceHours(_) | Self::SetDeviceHours(_) => ParameterId::DeviceHours,
            Self::GetLampHours(_) | Self::SetLampHours(_) => ParameterId::LampHours,
            Self::GetLampStrikes(_) | Self::SetLampStrikes(_) => ParameterId::LampStrikes,
            Self::GetLampState(_) | Self::SetLampState(_) => ParameterId::LampState,
            Self::GetLampOnMode(_) | Self::SetLampOnMode(_) => ParameterId::LampOnMode,
            Self::GetDevicePowerCycles(_) | Self::SetDevicePowerCycles(_) => {
                ParameterId::DevicePowerCycles
            }
            Self::GetDisplayInvert(_) | Self::SetDisplayInvert(_) => ParameterId::DisplayInvert,
            Self::GetDisplayLevel(_) | Self::SetDisplayLevel(_) => ParameterId::DisplayLevel,
            Self::GetPanInvert(_) | Self::SetPanInvert(_) => ParameterId::PanInvert,
            Self::GetTiltInvert(_) | Self::SetTiltInvert(_) => ParameterId::TiltInvert,
            Self::GetPanTiltSwap(_) | Self::SetPanTiltSwap(_) => ParameterId::PanTiltSwap,
            Self::GetRealTimeClock(_) | Self::SetRealTimeClock(_) => ParameterId::RealTimeClock,
            Self::GetIdentifyDevice(_) | Self::SetIdentifyDevice(_) => ParameterId::IdentifyDevice,
            Self::GetPowerState(_) | Self::SetPowerState(_) => ParameterId::PowerState,
            Self::GetPerformSelfTest(_) | Self::SetPerformSelfTest(_) => {
                ParameterId::PerformSelfTest
            }
            Self::GetSelfTestDescription(_) => ParameterId::SelfTestDescription,
            Self::SetCapturePreset(_) => ParameterId::CapturePreset,
            Self::GetPresetPlayback(_) | Self::SetPresetPlayback(_) => ParameterId::PresetPlayback,
            // E1.37-1
            Self::GetDmxBlockAddress(_) | Self::SetDmxBlockAddress(_) => {
                ParameterId::DmxBlockAddress
            }
            Self::GetDmxFailMode(_) | Self::SetDmxFailMode(_) => ParameterId::DmxFailMode,
            Self::GetDmxStartupMode(_) | Self::SetDmxStartupMode(_) => ParameterId::DmxStartupMode,
            Self::GetDimmerInfo(_) => ParameterId::DimmerInfo,
            Self::GetMinimumLevel(_) | Self::SetMinimumLevel(_) => ParameterId::MinimumLevel,
            Self::GetMaximumLevel(_) | Self::SetMaximumLevel(_) => ParameterId::MaximumLevel,
            Self::GetCurve(_) | Self::SetCurve(_) => ParameterId::Curve,
            Self::GetCurveDescription(_) => ParameterId::CurveDescription,
            Self::GetOutputResponseTime(_) | Self::SetOutputResponseTime(_) => {
                ParameterId::OutputResponseTime
            }
            Self::GetOutputResponseTimeDescription(_) => ParameterId::OutputResponseTimeDescription,
            Self::GetModulationFrequency(_) | Self::SetModulationFrequency(_) => {
                ParameterId::ModulationFrequency
            }
            Self::GetModulationFrequencyDescription(_) => {
                ParameterId::ModulationFrequencyDescription
            }
            Self::GetBurnIn(_) | Self::SetBurnIn(_) => ParameterId::BurnIn,
            Self::GetLockPin(_) | Self::SetLockPin(_) => ParameterId::LockPin,
            Self::GetLockState(_) | Self::SetLockState(_) => ParameterId::LockState,
            Self::GetLockStateDescription(_) => ParameterId::LockStateDescription,
            Self::GetIdentifyMode(_) | Self::SetIdentifyMode(_) => ParameterId::IdentifyMode,
            Self::GetPresetInfo(_) => ParameterId::PresetInfo,
            Self::GetPresetStatus(_) | Self::SetPresetStatus(_) => ParameterId::PresetStatus,
            Self::GetPresetMergeMode(_) | Self::SetPresetMergeMode(_) => {
                ParameterId::PresetMergeMode
            }
            Self::GetPowerOnSelfTest(_) | Self::SetPowerOnSelfTest(_) => {
                ParameterId::PowerOnSelfTest
            }
            // E1.37-2
            Self::GetListInterfaces(_) => ParameterId::ListInterfaces,
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
            Self::SetInterfaceRenewDhcp(_) => ParameterId::InterfaceRenewDhcp,
            Self::SetInterfaceReleaseDhcp(_) => ParameterId::InterfaceReleaseDhcp,
            Self::SetInterfaceApplyConfiguration(_) => ParameterId::InterfaceApplyConfiguration,
            Self::GetIpV4DefaultRoute(_) | Self::SetIpV4DefaultRoute(_) => {
                ParameterId::IpV4DefaultRoute
            }
            Self::GetDnsIpV4NameServer(_) | Self::SetDnsIpV4NameServer(_) => {
                ParameterId::DnsIpV4NameServer
            }
            Self::GetDnsHostName(_) | Self::SetDnsHostName(_) => ParameterId::DnsHostName,
            Self::GetDnsDomainName(_) | Self::SetDnsDomainName(_) => ParameterId::DnsDomainName,
            // E1.37-7
            Self::GetEndpointList(_) => ParameterId::EndpointList,
            Self::GetEndpointListChange(_) => ParameterId::EndpointListChange,
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
            Self::GetBackgroundQueuedStatusPolicy(_) | Self::SetBackgroundQueuedStatusPolicy(_) => {
                ParameterId::BackgroundQueuedStatusPolicy
            }
            Self::GetBackgroundQueuedStatusPolicyDescription(_) => {
                ParameterId::BackgroundQueuedStatusPolicyDescription
            }
            // E1.33
            Self::GetSearchDomain(_) | Self::SetSearchDomain(_) => ParameterId::SearchDomain,
            Self::GetComponentScope(_) | Self::SetComponentScope(_) => ParameterId::ComponentScope,
            Self::GetTcpCommsStatus(_) | Self::SetTcpCommsStatus(_) => ParameterId::TcpCommsStatus,
            Self::GetBrokerStatus(_) | Self::SetBrokerStatus(_) => ParameterId::BrokerStatus,
            // Manufacturer or Unsupported
            Self::CustomParameter(param) => param.parameter_id,
        }
    }

    fn size_of(&self) -> usize {
        match self {
            // E1.20
            Self::DiscMute(param) => param.size_of(),
            Self::DiscUnMute(param) => param.size_of(),
            Self::GetProxiedDeviceCount(param) => param.size_of(),
            Self::GetProxiedDevices(param) => param.size_of(),
            Self::GetCommsStatus(param) => param.size_of(),
            Self::SetCommsStatus(param) => param.size_of(),
            Self::GetStatusMessages(param) => param.size_of(),
            Self::GetStatusIdDescription(param) => param.size_of(),
            Self::SetClearStatusId(param) => param.size_of(),
            Self::GetSubDeviceIdStatusReportThreshold(param) => param.size_of(),
            Self::SetSubDeviceIdStatusReportThreshold(param) => param.size_of(),
            Self::GetSupportedParameters(param) => param.size_of(),
            Self::GetParameterDescription(param) => param.size_of(),
            Self::GetDeviceInfo(param) => param.size_of(),
            Self::GetProductDetailIdList(param) => param.size_of(),
            Self::GetDeviceModelDescription(param) => param.size_of(),
            Self::GetManufacturerLabel(param) => param.size_of(),
            Self::SetManufacturerLabel(param) => param.size_of(),
            Self::GetDeviceLabel(param) => param.size_of(),
            Self::SetDeviceLabel(param) => param.size_of(),
            Self::GetFactoryDefaults(param) => param.size_of(),
            Self::SetFactoryDefaults(param) => param.size_of(),
            Self::GetLanguageCapabilities(param) => param.size_of(),
            Self::GetLanguage(param) => param.size_of(),
            Self::SetLanguage(param) => param.size_of(),
            Self::GetSoftwareVersionLabel(param) => param.size_of(),
            Self::GetBootSoftwareVersionId(param) => param.size_of(),
            Self::GetBootSoftwareVersionLabel(param) => param.size_of(),
            Self::GetDmxPersonality(param) => param.size_of(),
            Self::SetDmxPersonality(param) => param.size_of(),
            Self::GetDmxPersonalityDescription(param) => param.size_of(),
            Self::GetDmxStartAddress(param) => param.size_of(),
            Self::SetDmxStartAddress(param) => param.size_of(),
            Self::GetSlotInfo(param) => param.size_of(),
            Self::GetSlotDescription(param) => param.size_of(),
            Self::GetDefaultSlotValue(param) => param.size_of(),
            Self::GetSensorDefinition(param) => param.size_of(),
            Self::GetSensorValue(param) => param.size_of(),
            Self::SetSensorValue(param) => param.size_of(),
            Self::SetRecordSensors(param) => param.size_of(),
            Self::GetDeviceHours(param) => param.size_of(),
            Self::SetDeviceHours(param) => param.size_of(),
            Self::GetLampHours(param) => param.size_of(),
            Self::SetLampHours(param) => param.size_of(),
            Self::GetLampStrikes(param) => param.size_of(),
            Self::SetLampStrikes(param) => param.size_of(),
            Self::GetLampState(param) => param.size_of(),
            Self::SetLampState(param) => param.size_of(),
            Self::GetLampOnMode(param) => param.size_of(),
            Self::SetLampOnMode(param) => param.size_of(),
            Self::GetDevicePowerCycles(param) => param.size_of(),
            Self::SetDevicePowerCycles(param) => param.size_of(),
            Self::GetDisplayInvert(param) => param.size_of(),
            Self::SetDisplayInvert(param) => param.size_of(),
            Self::GetDisplayLevel(param) => param.size_of(),
            Self::SetDisplayLevel(param) => param.size_of(),
            Self::GetPanInvert(param) => param.size_of(),
            Self::SetPanInvert(param) => param.size_of(),
            Self::GetTiltInvert(param) => param.size_of(),
            Self::SetTiltInvert(param) => param.size_of(),
            Self::GetPanTiltSwap(param) => param.size_of(),
            Self::SetPanTiltSwap(param) => param.size_of(),
            Self::GetRealTimeClock(param) => param.size_of(),
            Self::SetRealTimeClock(param) => param.size_of(),
            Self::GetIdentifyDevice(param) => param.size_of(),
            Self::SetIdentifyDevice(param) => param.size_of(),
            Self::GetPowerState(param) => param.size_of(),
            Self::SetPowerState(param) => param.size_of(),
            Self::GetPerformSelfTest(param) => param.size_of(),
            Self::SetPerformSelfTest(param) => param.size_of(),
            Self::GetSelfTestDescription(param) => param.size_of(),
            Self::SetCapturePreset(param) => param.size_of(),
            Self::GetPresetPlayback(param) => param.size_of(),
            Self::SetPresetPlayback(param) => param.size_of(),
            // E1.37-1
            Self::GetDmxBlockAddress(param) => param.size_of(),
            Self::SetDmxBlockAddress(param) => param.size_of(),
            Self::GetDmxFailMode(param) => param.size_of(),
            Self::SetDmxFailMode(param) => param.size_of(),
            Self::GetDmxStartupMode(param) => param.size_of(),
            Self::SetDmxStartupMode(param) => param.size_of(),
            Self::GetDimmerInfo(param) => param.size_of(),
            Self::GetMinimumLevel(param) => param.size_of(),
            Self::SetMinimumLevel(param) => param.size_of(),
            Self::GetMaximumLevel(param) => param.size_of(),
            Self::SetMaximumLevel(param) => param.size_of(),
            Self::GetCurve(param) => param.size_of(),
            Self::SetCurve(param) => param.size_of(),
            Self::GetCurveDescription(param) => param.size_of(),
            Self::GetOutputResponseTime(param) => param.size_of(),
            Self::SetOutputResponseTime(param) => param.size_of(),
            Self::GetOutputResponseTimeDescription(param) => param.size_of(),
            Self::GetModulationFrequency(param) => param.size_of(),
            Self::SetModulationFrequency(param) => param.size_of(),
            Self::GetModulationFrequencyDescription(param) => param.size_of(),
            Self::GetBurnIn(param) => param.size_of(),
            Self::SetBurnIn(param) => param.size_of(),
            Self::GetLockPin(param) => param.size_of(),
            Self::SetLockPin(param) => param.size_of(),
            Self::GetLockState(param) => param.size_of(),
            Self::SetLockState(param) => param.size_of(),
            Self::GetLockStateDescription(param) => param.size_of(),
            Self::GetIdentifyMode(param) => param.size_of(),
            Self::SetIdentifyMode(param) => param.size_of(),
            Self::GetPresetInfo(param) => param.size_of(),
            Self::GetPresetStatus(param) => param.size_of(),
            Self::GetPresetMergeMode(param) => param.size_of(),
            Self::SetPresetMergeMode(param) => param.size_of(),
            Self::SetPresetStatus(param) => param.size_of(),
            Self::GetPowerOnSelfTest(param) => param.size_of(),
            Self::SetPowerOnSelfTest(param) => param.size_of(),
            // E1.37-2
            Self::GetListInterfaces(param) => param.size_of(),
            Self::GetInterfaceLabel(param) => param.size_of(),
            Self::GetInterfaceHardwareAddressType1(param) => param.size_of(),
            Self::GetIpV4DhcpMode(param) => param.size_of(),
            Self::SetIpV4DhcpMode(param) => param.size_of(),
            Self::GetIpV4ZeroConfMode(param) => param.size_of(),
            Self::SetIpV4ZeroConfMode(param) => param.size_of(),
            Self::GetIpV4CurrentAddress(param) => param.size_of(),
            Self::GetIpV4StaticAddress(param) => param.size_of(),
            Self::SetIpV4StaticAddress(param) => param.size_of(),
            Self::SetInterfaceRenewDhcp(param) => param.size_of(),
            Self::SetInterfaceReleaseDhcp(param) => param.size_of(),
            Self::SetInterfaceApplyConfiguration(param) => param.size_of(),
            Self::GetIpV4DefaultRoute(param) => param.size_of(),
            Self::SetIpV4DefaultRoute(param) => param.size_of(),
            Self::GetDnsIpV4NameServer(param) => param.size_of(),
            Self::SetDnsIpV4NameServer(param) => param.size_of(),
            Self::GetDnsHostName(param) => param.size_of(),
            Self::SetDnsHostName(param) => param.size_of(),
            Self::GetDnsDomainName(param) => param.size_of(),
            Self::SetDnsDomainName(param) => param.size_of(),
            // E1.37-7
            Self::GetEndpointList(param) => param.size_of(),
            Self::GetEndpointListChange(param) => param.size_of(),
            Self::GetIdentifyEndpoint(param) => param.size_of(),
            Self::SetIdentifyEndpoint(param) => param.size_of(),
            Self::GetEndpointToUniverse(param) => param.size_of(),
            Self::SetEndpointToUniverse(param) => param.size_of(),
            Self::GetEndpointMode(param) => param.size_of(),
            Self::SetEndpointMode(param) => param.size_of(),
            Self::GetEndpointLabel(param) => param.size_of(),
            Self::SetEndpointLabel(param) => param.size_of(),
            Self::GetRdmTrafficEnable(param) => param.size_of(),
            Self::SetRdmTrafficEnable(param) => param.size_of(),
            Self::GetDiscoveryState(param) => param.size_of(),
            Self::SetDiscoveryState(param) => param.size_of(),
            Self::GetBackgroundDiscovery(param) => param.size_of(),
            Self::SetBackgroundDiscovery(param) => param.size_of(),
            Self::GetEndpointTiming(param) => param.size_of(),
            Self::SetEndpointTiming(param) => param.size_of(),
            Self::GetEndpointTimingDescription(param) => param.size_of(),
            Self::GetEndpointResponders(param) => param.size_of(),
            Self::GetEndpointResponderListChange(param) => param.size_of(),
            Self::GetBindingControlFields(param) => param.size_of(),
            Self::GetBackgroundQueuedStatusPolicy(param) => param.size_of(),
            Self::SetBackgroundQueuedStatusPolicy(param) => param.size_of(),
            Self::GetBackgroundQueuedStatusPolicyDescription(param) => param.size_of(),
            // E1.33
            Self::GetComponentScope(param) => param.size_of(),
            Self::SetComponentScope(param) => param.size_of(),
            Self::GetSearchDomain(param) => param.size_of(),
            Self::SetSearchDomain(param) => param.size_of(),
            Self::GetTcpCommsStatus(param) => param.size_of(),
            Self::SetTcpCommsStatus(param) => param.size_of(),
            Self::GetBrokerStatus(param) => param.size_of(),
            Self::SetBrokerStatus(param) => param.size_of(),
            // Manufacturer or Unsupported
            Self::CustomParameter(param) => param.response.size_of(),
        }
    }

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        match self {
            // E1.20
            Self::DiscMute(param) => param.encode_parameter_data(buf),
            Self::DiscUnMute(param) => param.encode_parameter_data(buf),
            Self::GetProxiedDeviceCount(param) => param.encode(buf),
            Self::GetProxiedDevices(param) => param.encode(buf),
            Self::GetCommsStatus(param) => param.encode(buf),
            Self::SetCommsStatus(param) => param.encode(buf),
            Self::GetStatusMessages(param) => param.encode(buf),
            Self::GetStatusIdDescription(param) => param.encode(buf),
            Self::SetClearStatusId(param) => param.encode(buf),
            Self::GetSubDeviceIdStatusReportThreshold(param) => param.encode(buf),
            Self::SetSubDeviceIdStatusReportThreshold(param) => param.encode(buf),
            Self::GetSupportedParameters(param) => param.encode(buf),
            Self::GetParameterDescription(param) => param.encode(buf),
            Self::GetDeviceInfo(param) => param.encode(buf),
            Self::GetProductDetailIdList(param) => param.encode(buf),
            Self::GetDeviceModelDescription(param) => param.encode(buf),
            Self::GetManufacturerLabel(param) => param.encode(buf),
            Self::SetManufacturerLabel(param) => param.encode(buf),
            Self::GetDeviceLabel(param) => param.encode(buf),
            Self::SetDeviceLabel(param) => param.encode(buf),
            Self::GetFactoryDefaults(param) => param.encode(buf),
            Self::SetFactoryDefaults(param) => param.encode(buf),
            Self::GetLanguageCapabilities(param) => param.encode(buf),
            Self::GetLanguage(param) => param.encode(buf),
            Self::SetLanguage(param) => param.encode(buf),
            Self::GetSoftwareVersionLabel(param) => param.encode(buf),
            Self::GetBootSoftwareVersionId(param) => param.encode(buf),
            Self::GetBootSoftwareVersionLabel(param) => param.encode(buf),
            Self::GetDmxPersonality(param) => param.encode(buf),
            Self::SetDmxPersonality(param) => param.encode(buf),
            Self::GetDmxPersonalityDescription(param) => param.encode(buf),
            Self::GetDmxStartAddress(param) => param.encode(buf),
            Self::SetDmxStartAddress(param) => param.encode(buf),
            Self::GetSlotInfo(param) => param.encode(buf),
            Self::GetSlotDescription(param) => param.encode(buf),
            Self::GetDefaultSlotValue(param) => param.encode(buf),
            Self::GetSensorDefinition(param) => param.encode(buf),
            Self::GetSensorValue(param) => param.encode(buf),
            Self::SetSensorValue(param) => param.encode(buf),
            Self::SetRecordSensors(param) => param.encode(buf),
            Self::GetDeviceHours(param) => param.encode(buf),
            Self::SetDeviceHours(param) => param.encode(buf),
            Self::GetLampHours(param) => param.encode(buf),
            Self::SetLampHours(param) => param.encode(buf),
            Self::GetLampStrikes(param) => param.encode(buf),
            Self::SetLampStrikes(param) => param.encode(buf),
            Self::GetLampState(param) => param.encode(buf),
            Self::SetLampState(param) => param.encode(buf),
            Self::GetLampOnMode(param) => param.encode(buf),
            Self::SetLampOnMode(param) => param.encode(buf),
            Self::GetDevicePowerCycles(param) => param.encode(buf),
            Self::SetDevicePowerCycles(param) => param.encode(buf),
            Self::GetDisplayInvert(param) => param.encode(buf),
            Self::SetDisplayInvert(param) => param.encode(buf),
            Self::GetDisplayLevel(param) => param.encode(buf),
            Self::SetDisplayLevel(param) => param.encode(buf),
            Self::GetPanInvert(param) => param.encode(buf),
            Self::SetPanInvert(param) => param.encode(buf),
            Self::GetTiltInvert(param) => param.encode(buf),
            Self::SetTiltInvert(param) => param.encode(buf),
            Self::GetPanTiltSwap(param) => param.encode(buf),
            Self::SetPanTiltSwap(param) => param.encode(buf),
            Self::GetRealTimeClock(param) => param.encode(buf),
            Self::SetRealTimeClock(param) => param.encode(buf),
            Self::GetIdentifyDevice(param) => param.encode(buf),
            Self::SetIdentifyDevice(param) => param.encode(buf),
            Self::GetPowerState(param) => param.encode(buf),
            Self::SetPowerState(param) => param.encode(buf),
            Self::GetPerformSelfTest(param) => param.encode(buf),
            Self::SetPerformSelfTest(param) => param.encode(buf),
            Self::GetSelfTestDescription(param) => param.encode(buf),
            Self::SetCapturePreset(param) => param.encode(buf),
            Self::GetPresetPlayback(param) => param.encode(buf),
            Self::SetPresetPlayback(param) => param.encode(buf),
            // E1.37-1
            Self::GetDmxBlockAddress(param) => param.encode(buf),
            Self::SetDmxBlockAddress(param) => param.encode(buf),
            Self::GetDmxFailMode(param) => param.encode(buf),
            Self::SetDmxFailMode(param) => param.encode(buf),
            Self::GetDmxStartupMode(param) => param.encode(buf),
            Self::SetDmxStartupMode(param) => param.encode(buf),
            Self::GetDimmerInfo(param) => param.encode(buf),
            Self::GetMinimumLevel(param) => param.encode(buf),
            Self::SetMinimumLevel(param) => param.encode(buf),
            Self::GetMaximumLevel(param) => param.encode(buf),
            Self::SetMaximumLevel(param) => param.encode(buf),
            Self::GetCurve(param) => param.encode(buf),
            Self::SetCurve(param) => param.encode(buf),
            Self::GetCurveDescription(param) => param.encode(buf),
            Self::GetOutputResponseTime(param) => param.encode(buf),
            Self::SetOutputResponseTime(param) => param.encode(buf),
            Self::GetOutputResponseTimeDescription(param) => param.encode(buf),
            Self::GetModulationFrequency(param) => param.encode(buf),
            Self::SetModulationFrequency(param) => param.encode(buf),
            Self::GetModulationFrequencyDescription(param) => param.encode(buf),
            Self::GetBurnIn(param) => param.encode(buf),
            Self::SetBurnIn(param) => param.encode(buf),
            Self::GetLockPin(param) => param.encode(buf),
            Self::SetLockPin(param) => param.encode(buf),
            Self::GetLockState(param) => param.encode(buf),
            Self::SetLockState(param) => param.encode(buf),
            Self::GetLockStateDescription(param) => param.encode(buf),
            Self::GetIdentifyMode(param) => param.encode(buf),
            Self::SetIdentifyMode(param) => param.encode(buf),
            Self::GetPresetInfo(param) => param.encode(buf),
            Self::GetPresetStatus(param) => param.encode(buf),
            Self::SetPresetStatus(param) => param.encode(buf),
            Self::GetPresetMergeMode(param) => param.encode(buf),
            Self::SetPresetMergeMode(param) => param.encode(buf),
            Self::GetPowerOnSelfTest(param) => param.encode(buf),
            Self::SetPowerOnSelfTest(param) => param.encode(buf),
            // E1.37-2
            Self::GetListInterfaces(param) => param.encode(buf),
            Self::GetInterfaceLabel(param) => param.encode(buf),
            Self::GetInterfaceHardwareAddressType1(param) => param.encode(buf),
            Self::GetIpV4DhcpMode(param) => param.encode(buf),
            Self::SetIpV4DhcpMode(param) => param.encode(buf),
            Self::GetIpV4ZeroConfMode(param) => param.encode(buf),
            Self::SetIpV4ZeroConfMode(param) => param.encode(buf),
            Self::GetIpV4CurrentAddress(param) => param.encode(buf),
            Self::GetIpV4StaticAddress(param) => param.encode(buf),
            Self::SetIpV4StaticAddress(param) => param.encode(buf),
            Self::SetInterfaceRenewDhcp(param) => param.encode(buf),
            Self::SetInterfaceReleaseDhcp(param) => param.encode(buf),
            Self::SetInterfaceApplyConfiguration(param) => param.encode(buf),
            Self::GetIpV4DefaultRoute(param) => param.encode(buf),
            Self::SetIpV4DefaultRoute(param) => param.encode(buf),
            Self::GetDnsIpV4NameServer(param) => param.encode(buf),
            Self::SetDnsIpV4NameServer(param) => param.encode(buf),
            Self::GetDnsHostName(param) => param.encode(buf),
            Self::SetDnsHostName(param) => param.encode(buf),
            Self::GetDnsDomainName(param) => param.encode(buf),
            Self::SetDnsDomainName(param) => param.encode(buf),
            // E1.37-7
            Self::GetEndpointList(param) => param.encode(buf),
            Self::GetEndpointListChange(param) => param.encode(buf),
            Self::GetIdentifyEndpoint(param) => param.encode(buf),
            Self::SetIdentifyEndpoint(param) => param.encode(buf),
            Self::GetEndpointToUniverse(param) => param.encode(buf),
            Self::SetEndpointToUniverse(param) => param.encode(buf),
            Self::GetEndpointMode(param) => param.encode(buf),
            Self::SetEndpointMode(param) => param.encode(buf),
            Self::GetEndpointLabel(param) => param.encode(buf),
            Self::SetEndpointLabel(param) => param.encode(buf),
            Self::GetRdmTrafficEnable(param) => param.encode(buf),
            Self::SetRdmTrafficEnable(param) => param.encode(buf),
            Self::GetDiscoveryState(param) => param.encode(buf),
            Self::SetDiscoveryState(param) => param.encode(buf),
            Self::GetBackgroundDiscovery(param) => param.encode(buf),
            Self::SetBackgroundDiscovery(param) => param.encode(buf),
            Self::GetEndpointTiming(param) => param.encode(buf),
            Self::SetEndpointTiming(param) => param.encode(buf),
            Self::GetEndpointTimingDescription(param) => param.encode(buf),
            Self::GetEndpointResponders(param) => param.encode(buf),
            Self::GetEndpointResponderListChange(param) => param.encode(buf),
            Self::GetBindingControlFields(param) => param.encode(buf),
            Self::GetBackgroundQueuedStatusPolicy(param) => param.encode(buf),
            Self::SetBackgroundQueuedStatusPolicy(param) => param.encode(buf),
            Self::GetBackgroundQueuedStatusPolicyDescription(param) => param.encode(buf),
            // E1.33
            Self::GetComponentScope(param) => param.encode(buf),
            Self::SetComponentScope(param) => param.encode(buf),
            Self::GetSearchDomain(param) => param.encode(buf),
            Self::SetSearchDomain(param) => param.encode(buf),
            Self::GetTcpCommsStatus(param) => param.encode(buf),
            Self::SetTcpCommsStatus(param) => param.encode(buf),
            Self::GetBrokerStatus(param) => param.encode(buf),
            Self::SetBrokerStatus(param) => param.encode(buf),
            // Manufacturer or Unsupported
            Self::CustomParameter(param) => param.response.encode(buf),
        }
    }

    fn decode_parameter_data(
        response_type: ResponseType,
        command_class: CommandClass,
        parameter_id: ParameterId,
        buf: &[u8],
    ) -> Result<Self, ParameterCodecError> {
        match (command_class, parameter_id) {
            // E1.20
            (CommandClass::DiscoveryResponse, ParameterId::DiscMute) => Ok(Self::DiscMute(
                DiscMuteResponse::decode_parameter_data(buf)?,
            )),
            (CommandClass::DiscoveryResponse, ParameterId::DiscUnMute) => Ok(Self::DiscUnMute(
                DiscUnMuteResponse::decode_parameter_data(buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::ProxiedDeviceCount) => {
                Ok(Self::GetProxiedDeviceCount(ResponseResult::<
                    GetProxiedDeviceCountResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::ProxiedDevices) => {
                Ok(Self::GetProxiedDevices(ResponseResult::<
                    GetProxiedDevicesResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::CommsStatus) => {
                Ok(Self::GetCommsStatus(ResponseResult::<
                    GetCommsStatusResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::CommsStatus) => Ok(Self::SetCommsStatus(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::StatusMessages) => {
                Ok(Self::GetStatusMessages(ResponseResult::<
                    GetStatusMessagesResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::StatusIdDescription) => {
                Ok(Self::GetStatusIdDescription(ResponseResult::<
                    GetStatusIdDescriptionResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::ClearStatusId) => Ok(Self::SetClearStatusId(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::SubDeviceIdStatusReportThreshold) => {
                Ok(Self::GetSubDeviceIdStatusReportThreshold(ResponseResult::<
                    GetSubDeviceIdStatusReportThresholdResponse,
                >::decode(
                    response_type,
                    buf,
                )?))
            }
            (CommandClass::SetResponse, ParameterId::SubDeviceIdStatusReportThreshold) => {
                Ok(Self::SetSubDeviceIdStatusReportThreshold(ResponseResult::<
                    (),
                >::decode(
                    response_type,
                    buf,
                )?))
            }
            (CommandClass::GetResponse, ParameterId::SupportedParameters) => {
                Ok(Self::GetSupportedParameters(ResponseResult::<
                    GetSupportedParametersResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::ParameterDescription) => {
                Ok(Self::GetParameterDescription(ResponseResult::<
                    GetParameterDescriptionResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::DeviceInfo) => Ok(Self::GetDeviceInfo(
                ResponseResult::<GetDeviceInfoResponse>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::ProductDetailIdList) => {
                Ok(Self::GetProductDetailIdList(ResponseResult::<
                    GetProductDetailIdListResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::DeviceModelDescription) => {
                Ok(Self::GetDeviceModelDescription(ResponseResult::<
                    GetDeviceModelDescriptionResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::ManufacturerLabel) => {
                Ok(Self::GetManufacturerLabel(ResponseResult::<
                    GetManufacturerLabelResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::ManufacturerLabel) => Ok(
                Self::SetManufacturerLabel(ResponseResult::<()>::decode(response_type, buf)?),
            ),
            (CommandClass::GetResponse, ParameterId::DeviceLabel) => {
                Ok(Self::GetDeviceLabel(ResponseResult::<
                    GetDeviceLabelResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::DeviceLabel) => Ok(Self::SetDeviceLabel(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::FactoryDefaults) => {
                Ok(Self::GetFactoryDefaults(ResponseResult::<
                    GetFactoryDefaultsResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::FactoryDefaults) => Ok(
                Self::SetFactoryDefaults(ResponseResult::<()>::decode(response_type, buf)?),
            ),
            (CommandClass::GetResponse, ParameterId::LanguageCapabilities) => {
                Ok(Self::GetLanguageCapabilities(ResponseResult::<
                    GetLanguageCapabilitiesResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::Language) => Ok(Self::GetLanguage(
                ResponseResult::<GetLanguageResponse>::decode(response_type, buf)?,
            )),
            (CommandClass::SetResponse, ParameterId::Language) => Ok(Self::SetLanguage(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::SoftwareVersionLabel) => {
                Ok(Self::GetSoftwareVersionLabel(ResponseResult::<
                    GetSoftwareVersionLabelResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::BootSoftwareVersionId) => {
                Ok(Self::GetBootSoftwareVersionId(ResponseResult::<
                    GetBootSoftwareVersionIdResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::BootSoftwareVersionLabel) => {
                Ok(Self::GetBootSoftwareVersionLabel(ResponseResult::<
                    GetBootSoftwareVersionLabelResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::DmxPersonality) => {
                Ok(Self::GetDmxPersonality(ResponseResult::<
                    GetDmxPersonalityResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::DmxPersonality) => Ok(
                Self::SetDmxPersonality(ResponseResult::<()>::decode(response_type, buf)?),
            ),
            (CommandClass::GetResponse, ParameterId::DmxPersonalityDescription) => {
                Ok(Self::GetDmxPersonalityDescription(ResponseResult::<
                    GetDmxPersonalityDescriptionResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::DmxStartAddress) => {
                Ok(Self::GetDmxStartAddress(ResponseResult::<
                    GetDmxStartAddressResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::DmxStartAddress) => Ok(
                Self::SetDmxStartAddress(ResponseResult::<()>::decode(response_type, buf)?),
            ),
            (CommandClass::GetResponse, ParameterId::SlotInfo) => Ok(Self::GetSlotInfo(
                ResponseResult::<GetSlotInfoResponse>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::SlotDescription) => {
                Ok(Self::GetSlotDescription(ResponseResult::<
                    GetSlotDescriptionResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::DefaultSlotValue) => {
                Ok(Self::GetDefaultSlotValue(ResponseResult::<
                    GetDefaultSlotValueResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::SensorDefinition) => {
                Ok(Self::GetSensorDefinition(ResponseResult::<
                    GetSensorDefinitionResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::SensorValue) => {
                Ok(Self::GetSensorValue(ResponseResult::<
                    GetSensorValueResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::SensorValue) => {
                Ok(Self::SetSensorValue(ResponseResult::<
                    SetSensorValueResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::RecordSensors) => Ok(Self::SetRecordSensors(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::DeviceHours) => {
                Ok(Self::GetDeviceHours(ResponseResult::<
                    GetDeviceHoursResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::DeviceHours) => Ok(Self::SetDeviceHours(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::LampHours) => Ok(Self::GetLampHours(
                ResponseResult::<GetLampHoursResponse>::decode(response_type, buf)?,
            )),
            (CommandClass::SetResponse, ParameterId::LampHours) => Ok(Self::SetLampHours(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::LampStrikes) => {
                Ok(Self::GetLampStrikes(ResponseResult::<
                    GetLampStrikesResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::LampStrikes) => Ok(Self::SetLampStrikes(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::LampState) => Ok(Self::GetLampState(
                ResponseResult::<GetLampStateResponse>::decode(response_type, buf)?,
            )),
            (CommandClass::SetResponse, ParameterId::LampState) => Ok(Self::SetLampState(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::LampOnMode) => Ok(Self::GetLampOnMode(
                ResponseResult::<GetLampOnModeResponse>::decode(response_type, buf)?,
            )),
            (CommandClass::SetResponse, ParameterId::LampOnMode) => Ok(Self::SetLampOnMode(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::DevicePowerCycles) => {
                Ok(Self::GetDevicePowerCycles(ResponseResult::<
                    GetDevicePowerCyclesResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::DevicePowerCycles) => Ok(
                Self::SetDevicePowerCycles(ResponseResult::<()>::decode(response_type, buf)?),
            ),
            (CommandClass::GetResponse, ParameterId::DisplayInvert) => {
                Ok(Self::GetDisplayInvert(ResponseResult::<
                    GetDisplayInvertResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::DisplayInvert) => Ok(Self::SetDisplayInvert(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::DisplayLevel) => {
                Ok(Self::GetDisplayLevel(ResponseResult::<
                    GetDisplayLevelResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::DisplayLevel) => Ok(Self::SetDisplayLevel(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::PanInvert) => Ok(Self::GetPanInvert(
                ResponseResult::<GetPanInvertResponse>::decode(response_type, buf)?,
            )),
            (CommandClass::SetResponse, ParameterId::PanInvert) => Ok(Self::SetPanInvert(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::TiltInvert) => Ok(Self::GetTiltInvert(
                ResponseResult::<GetTiltInvertResponse>::decode(response_type, buf)?,
            )),
            (CommandClass::SetResponse, ParameterId::TiltInvert) => Ok(Self::SetTiltInvert(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::PanTiltSwap) => {
                Ok(Self::GetPanTiltSwap(ResponseResult::<
                    GetPanTiltSwapResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::PanTiltSwap) => Ok(Self::SetPanTiltSwap(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::RealTimeClock) => {
                Ok(Self::GetRealTimeClock(ResponseResult::<
                    GetRealTimeClockResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::RealTimeClock) => Ok(Self::SetRealTimeClock(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::IdentifyDevice) => {
                Ok(Self::GetIdentifyDevice(ResponseResult::<
                    GetIdentifyDeviceResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::IdentifyDevice) => Ok(
                Self::SetIdentifyDevice(ResponseResult::<()>::decode(response_type, buf)?),
            ),
            (CommandClass::GetResponse, ParameterId::PowerState) => Ok(Self::GetPowerState(
                ResponseResult::<GetPowerStateResponse>::decode(response_type, buf)?,
            )),
            (CommandClass::SetResponse, ParameterId::PowerState) => Ok(Self::SetPowerState(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::PerformSelfTest) => {
                Ok(Self::GetPerformSelfTest(ResponseResult::<
                    GetPerformSelfTestResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::PerformSelfTest) => Ok(
                Self::SetPerformSelfTest(ResponseResult::<()>::decode(response_type, buf)?),
            ),
            (CommandClass::GetResponse, ParameterId::SelfTestDescription) => {
                Ok(Self::GetSelfTestDescription(ResponseResult::<
                    GetSelfTestDescriptionResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::CapturePreset) => Ok(Self::SetCapturePreset(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::PresetPlayback) => {
                Ok(Self::GetPresetPlayback(ResponseResult::<
                    GetPresetPlaybackResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::PresetPlayback) => Ok(
                Self::SetPresetPlayback(ResponseResult::<()>::decode(response_type, buf)?),
            ),
            // E1.37-1
            (CommandClass::GetResponse, ParameterId::DmxBlockAddress) => {
                Ok(Self::GetDmxBlockAddress(ResponseResult::<
                    GetDmxBlockAddressResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::DmxBlockAddress) => Ok(
                Self::SetDmxBlockAddress(ResponseResult::<()>::decode(response_type, buf)?),
            ),
            (CommandClass::GetResponse, ParameterId::DmxFailMode) => {
                Ok(Self::GetDmxFailMode(ResponseResult::<
                    GetDmxFailModeResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::DmxFailMode) => Ok(Self::SetDmxFailMode(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::DmxStartupMode) => {
                Ok(Self::GetDmxStartupMode(ResponseResult::<
                    GetDmxStartupModeResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::DmxStartupMode) => Ok(
                Self::SetDmxStartupMode(ResponseResult::<()>::decode(response_type, buf)?),
            ),
            (CommandClass::GetResponse, ParameterId::DimmerInfo) => Ok(Self::GetDimmerInfo(
                ResponseResult::<GetDimmerInfoResponse>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::MinimumLevel) => {
                Ok(Self::GetMinimumLevel(ResponseResult::<
                    GetMinimumLevelResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::MinimumLevel) => Ok(Self::SetMinimumLevel(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::MaximumLevel) => {
                Ok(Self::GetMaximumLevel(ResponseResult::<
                    GetMaximumLevelResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::MaximumLevel) => Ok(Self::SetMaximumLevel(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::Curve) => Ok(Self::GetCurve(
                ResponseResult::<GetCurveResponse>::decode(response_type, buf)?,
            )),
            (CommandClass::SetResponse, ParameterId::Curve) => Ok(Self::SetCurve(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::CurveDescription) => {
                Ok(Self::GetCurveDescription(ResponseResult::<
                    GetCurveDescriptionResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::OutputResponseTime) => {
                Ok(Self::GetOutputResponseTime(ResponseResult::<
                    GetOutputResponseTimeResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::OutputResponseTime) => Ok(
                Self::SetOutputResponseTime(ResponseResult::<()>::decode(response_type, buf)?),
            ),
            (CommandClass::GetResponse, ParameterId::OutputResponseTimeDescription) => {
                Ok(Self::GetOutputResponseTimeDescription(ResponseResult::<
                    GetOutputResponseTimeDescriptionResponse,
                >::decode(
                    response_type,
                    buf,
                )?))
            }
            (CommandClass::GetResponse, ParameterId::ModulationFrequency) => {
                Ok(Self::GetModulationFrequency(ResponseResult::<
                    GetModulationFrequencyResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::ModulationFrequency) => Ok(
                Self::SetModulationFrequency(ResponseResult::<()>::decode(response_type, buf)?),
            ),
            (CommandClass::GetResponse, ParameterId::ModulationFrequencyDescription) => {
                Ok(Self::GetModulationFrequencyDescription(ResponseResult::<
                    GetModulationFrequencyDescriptionResponse,
                >::decode(
                    response_type,
                    buf,
                )?))
            }
            (CommandClass::GetResponse, ParameterId::BurnIn) => Ok(Self::GetBurnIn(
                ResponseResult::<GetBurnInResponse>::decode(response_type, buf)?,
            )),
            (CommandClass::SetResponse, ParameterId::BurnIn) => Ok(Self::SetBurnIn(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::LockPin) => Ok(Self::GetLockPin(
                ResponseResult::<GetLockPinResponse>::decode(response_type, buf)?,
            )),
            (CommandClass::SetResponse, ParameterId::LockPin) => Ok(Self::SetLockPin(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::LockState) => Ok(Self::GetLockState(
                ResponseResult::<GetLockStateResponse>::decode(response_type, buf)?,
            )),
            (CommandClass::SetResponse, ParameterId::LockState) => Ok(Self::SetLockState(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::LockStateDescription) => {
                Ok(Self::GetLockStateDescription(ResponseResult::<
                    GetLockStateDescriptionResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::IdentifyMode) => {
                Ok(Self::GetIdentifyMode(ResponseResult::<
                    GetIdentifyModeResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::IdentifyMode) => Ok(Self::SetIdentifyMode(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::PresetInfo) => Ok(Self::GetPresetInfo(
                ResponseResult::<GetPresetInfoResponse>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::PresetStatus) => {
                Ok(Self::GetPresetStatus(ResponseResult::<
                    GetPresetStatusResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::PresetStatus) => Ok(Self::SetPresetStatus(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::PresetMergeMode) => {
                Ok(Self::GetPresetMergeMode(ResponseResult::<
                    GetPresetMergeModeResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::PresetMergeMode) => Ok(
                Self::SetPresetMergeMode(ResponseResult::<()>::decode(response_type, buf)?),
            ),
            (CommandClass::GetResponse, ParameterId::PowerOnSelfTest) => {
                Ok(Self::GetPowerOnSelfTest(ResponseResult::<
                    GetPowerOnSelfTestResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::PowerOnSelfTest) => Ok(
                Self::SetPowerOnSelfTest(ResponseResult::<()>::decode(response_type, buf)?),
            ),
            // E1.37-2
            (CommandClass::GetResponse, ParameterId::ListInterfaces) => {
                Ok(Self::GetListInterfaces(ResponseResult::<
                    GetListInterfacesResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::InterfaceLabel) => {
                Ok(Self::GetInterfaceLabel(ResponseResult::<
                    GetInterfaceLabelResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::InterfaceHardwareAddressType1) => {
                Ok(Self::GetInterfaceHardwareAddressType1(ResponseResult::<
                    GetInterfaceHardwareAddressType1Response,
                >::decode(
                    response_type,
                    buf,
                )?))
            }
            (CommandClass::GetResponse, ParameterId::IpV4DhcpMode) => {
                Ok(Self::GetIpV4DhcpMode(ResponseResult::<
                    GetIpV4DhcpModeResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::IpV4DhcpMode) => Ok(Self::SetIpV4DhcpMode(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::IpV4ZeroConfMode) => {
                Ok(Self::GetIpV4ZeroConfMode(ResponseResult::<
                    GetIpV4ZeroConfModeResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::IpV4ZeroConfMode) => Ok(
                Self::SetIpV4ZeroConfMode(ResponseResult::<()>::decode(response_type, buf)?),
            ),
            (CommandClass::GetResponse, ParameterId::IpV4CurrentAddress) => {
                Ok(Self::GetIpV4CurrentAddress(ResponseResult::<
                    GetIpV4CurrentAddressResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::IpV4StaticAddress) => {
                Ok(Self::GetIpV4StaticAddress(ResponseResult::<
                    GetIpV4StaticAddressResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::IpV4StaticAddress) => Ok(
                Self::SetIpV4StaticAddress(ResponseResult::<()>::decode(response_type, buf)?),
            ),
            (CommandClass::GetResponse, ParameterId::IpV4DefaultRoute) => {
                Ok(Self::GetIpV4DefaultRoute(ResponseResult::<
                    GetIpV4DefaultRouteResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::IpV4DefaultRoute) => Ok(
                Self::SetIpV4DefaultRoute(ResponseResult::<()>::decode(response_type, buf)?),
            ),
            (CommandClass::GetResponse, ParameterId::DnsIpV4NameServer) => {
                Ok(Self::GetDnsIpV4NameServer(ResponseResult::<
                    GetDnsIpV4NameServerResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::DnsIpV4NameServer) => Ok(
                Self::SetDnsIpV4NameServer(ResponseResult::<()>::decode(response_type, buf)?),
            ),
            (CommandClass::GetResponse, ParameterId::DnsHostName) => {
                Ok(Self::GetDnsHostName(ResponseResult::<
                    GetDnsHostNameResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::DnsHostName) => Ok(Self::SetDnsHostName(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::DnsDomainName) => {
                Ok(Self::GetDnsDomainName(ResponseResult::<
                    GetDnsDomainNameResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::DnsDomainName) => Ok(Self::SetDnsDomainName(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            // E1.37-7
            (CommandClass::GetResponse, ParameterId::EndpointList) => {
                Ok(Self::GetEndpointList(ResponseResult::<
                    GetEndpointListResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::EndpointListChange) => {
                Ok(Self::GetEndpointListChange(ResponseResult::<
                    GetEndpointListChangeResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::IdentifyEndpoint) => {
                Ok(Self::GetIdentifyEndpoint(ResponseResult::<
                    GetIdentifyEndpointResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::IdentifyEndpoint) => {
                Ok(Self::SetIdentifyEndpoint(ResponseResult::<
                    SetIdentifyEndpointResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::EndpointToUniverse) => {
                Ok(Self::GetEndpointToUniverse(ResponseResult::<
                    GetEndpointToUniverseResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::EndpointToUniverse) => {
                Ok(Self::SetEndpointToUniverse(ResponseResult::<
                    SetEndpointToUniverseResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::EndpointMode) => {
                Ok(Self::GetEndpointMode(ResponseResult::<
                    GetEndpointModeResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::EndpointMode) => {
                Ok(Self::SetEndpointMode(ResponseResult::<
                    SetEndpointModeResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::EndpointLabel) => {
                Ok(Self::GetEndpointLabel(ResponseResult::<
                    GetEndpointLabelResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::EndpointLabel) => {
                Ok(Self::SetEndpointLabel(ResponseResult::<
                    SetEndpointLabelResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::RdmTrafficEnable) => {
                Ok(Self::GetRdmTrafficEnable(ResponseResult::<
                    GetRdmTrafficEnableResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::RdmTrafficEnable) => {
                Ok(Self::SetRdmTrafficEnable(ResponseResult::<
                    SetRdmTrafficEnableResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::DiscoveryState) => {
                Ok(Self::GetDiscoveryState(ResponseResult::<
                    GetDiscoveryStateResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::DiscoveryState) => {
                Ok(Self::SetDiscoveryState(ResponseResult::<
                    SetDiscoveryStateResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::BackgroundDiscovery) => {
                Ok(Self::GetBackgroundDiscovery(ResponseResult::<
                    GetBackgroundDiscoveryResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::BackgroundDiscovery) => {
                Ok(Self::SetBackgroundDiscovery(ResponseResult::<
                    SetBackgroundDiscoveryResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::EndpointTiming) => {
                Ok(Self::GetEndpointTiming(ResponseResult::<
                    GetEndpointTimingResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::EndpointTiming) => {
                Ok(Self::SetEndpointTiming(ResponseResult::<
                    SetEndpointTimingResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::EndpointTimingDescription) => {
                Ok(Self::GetEndpointTimingDescription(ResponseResult::<
                    GetEndpointTimingDescriptionResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::EndpointResponders) => {
                Ok(Self::GetEndpointResponders(ResponseResult::<
                    GetEndpointRespondersResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::EndpointResponderListChange) => {
                Ok(Self::GetEndpointResponderListChange(ResponseResult::<
                    GetEndpointResponderListChangeResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::BindingControlFields) => {
                Ok(Self::GetBindingControlFields(ResponseResult::<
                    GetBindingControlFieldsResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::GetResponse, ParameterId::BackgroundQueuedStatusPolicy) => {
                Ok(Self::GetBackgroundQueuedStatusPolicy(ResponseResult::<
                    GetBackgroundQueuedStatusPolicyResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::BackgroundQueuedStatusPolicy) => {
                Ok(Self::SetBackgroundQueuedStatusPolicy(
                    ResponseResult::<()>::decode(response_type, buf)?,
                ))
            }
            (CommandClass::GetResponse, ParameterId::BackgroundQueuedStatusPolicyDescription) => {
                Ok(Self::GetBackgroundQueuedStatusPolicyDescription(
                    ResponseResult::<GetBackgroundQueuedStatusPolicyDescriptionResponse>::decode(
                        response_type,
                        buf,
                    )?,
                ))
            }
            // E1.33
            (CommandClass::GetResponse, ParameterId::SearchDomain) => {
                Ok(Self::GetSearchDomain(ResponseResult::<
                    GetSearchDomainResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::SearchDomain) => Ok(Self::SetSearchDomain(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            (CommandClass::GetResponse, ParameterId::ComponentScope) => {
                Ok(Self::GetComponentScope(ResponseResult::<
                    GetComponentScopeResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::ComponentScope) => Ok(
                Self::SetComponentScope(ResponseResult::<()>::decode(response_type, buf)?),
            ),
            (CommandClass::GetResponse, ParameterId::TcpCommsStatus) => {
                Ok(Self::GetTcpCommsStatus(ResponseResult::<
                    GetTcpCommsStatusResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::TcpCommsStatus) => Ok(
                Self::SetTcpCommsStatus(ResponseResult::<()>::decode(response_type, buf)?),
            ),
            (CommandClass::GetResponse, ParameterId::BrokerStatus) => {
                Ok(Self::GetBrokerStatus(ResponseResult::<
                    GetBrokerStatusResponse,
                >::decode(
                    response_type, buf
                )?))
            }
            (CommandClass::SetResponse, ParameterId::BrokerStatus) => Ok(Self::SetBrokerStatus(
                ResponseResult::<()>::decode(response_type, buf)?,
            )),
            // Manufacturer or Unsupported
            (command_class, parameter_id) => Ok(Self::CustomParameter(CustomResponseParameter {
                command_class,
                parameter_id,
                parameter_data_length: buf.len() as u8,
                response: ResponseResult::<Vec<u8, 231>>::decode(response_type, buf)?,
            })),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RdmFrameResponse {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub message_count: u8,
    pub sub_device_id: SubDeviceId,
    pub parameter: ResponseParameter,
}

impl RdmFrameResponse {
    pub fn size(&self) -> usize {
        24 + self.parameter.size_of() + 2
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, RdmError> {
        buf[0] = RDM_START_CODE_BYTE;
        buf[1] = RDM_SUB_START_CODE_BYTE;

        let parameter_data_length = self.parameter.encode_parameter_data(&mut buf[24..])?;
        let message_length = 24 + parameter_data_length;

        buf[2] = message_length as u8;
        buf[3..9].copy_from_slice(&<[u8; 6]>::from(self.destination_uid));
        buf[9..15].copy_from_slice(&<[u8; 6]>::from(self.source_uid));
        buf[15] = self.transaction_number;
        buf[16] = self.parameter.response_type() as u8;
        buf[17] = self.message_count;
        buf[18..20].copy_from_slice(&u16::from(self.sub_device_id).to_be_bytes());
        buf[20] = self.parameter.command_class().into();
        buf[21..23].copy_from_slice(&u16::from(self.parameter.parameter_id()).to_be_bytes());
        buf[23] = parameter_data_length as u8;

        let mut crc = 0_u16;

        for byte in &buf[0..message_length] {
            crc = crc.overflowing_add(*byte as u16).0;
        }

        buf[message_length..message_length + 2].copy_from_slice(&crc.to_be_bytes());

        Ok(message_length + 2)
    }

    pub fn decode(buf: &[u8]) -> Result<Self, RdmError> {
        let message_length = buf[2];

        if message_length < 24 {
            return Err(RdmError::InvalidMessageLength(message_length));
        }

        if buf.len() < message_length as usize + 2 {
            return Err(RdmError::InvalidMessageLength(message_length));
        }

        let packet_checksum = u16::from_be_bytes(
            buf[message_length as usize..=message_length as usize + 1].try_into()?,
        );

        let decoded_checksum = bsd_16_crc(&buf[..message_length as usize]);

        if decoded_checksum != packet_checksum {
            return Err(RdmError::InvalidChecksum(decoded_checksum, packet_checksum));
        }

        let destination_uid = DeviceUID::from(<[u8; 6]>::try_from(&buf[3..9])?);
        let source_uid = DeviceUID::from(<[u8; 6]>::try_from(&buf[9..15])?);
        let transaction_number = buf[15];
        let response_type = ResponseType::try_from(buf[16])?;
        let message_count = buf[17];
        let sub_device_id = u16::from_be_bytes(buf[18..20].try_into()?).into();
        let command_class = CommandClass::try_from(buf[20])?;
        let parameter_id = u16::from_be_bytes(buf[21..23].try_into()?).into();
        let parameter_data_length = buf[23];

        if parameter_data_length > 231 {
            return Err(RdmError::InvalidParameterDataLength(parameter_data_length));
        }

        Ok(Self {
            destination_uid,
            source_uid,
            transaction_number,
            message_count,
            sub_device_id,
            parameter: ResponseParameter::decode_parameter_data(
                response_type,
                command_class,
                parameter_id,
                &buf[24..(24 + parameter_data_length as usize)],
            )?,
        })
    }
}

impl TryFrom<&[u8]> for RdmFrameResponse {
    type Error = RdmError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        RdmFrameResponse::decode(bytes)
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum RdmResponse {
    RdmFrame(RdmFrameResponse),
    DiscoveryUniqueBranchFrame(DiscoveryUniqueBranchFrameResponse),
}

impl RdmResponse {
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

    pub fn decode(bytes: &[u8]) -> Result<Self, RdmError> {
        match bytes {
            [RDM_START_CODE_BYTE, RDM_SUB_START_CODE_BYTE, ..] => {
                if bytes.len() < MIN_RDM_FRAME_LENGTH {
                    return Err(RdmError::InvalidFrameLength(bytes.len() as u8));
                }
                RdmFrameResponse::decode(bytes).map(RdmResponse::RdmFrame)
            }
            [
                DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE
                | DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE,
                ..,
            ] => {
                if bytes.len() < MIN_DISC_FRAME_LENGTH {
                    return Err(RdmError::InvalidFrameLength(bytes.len() as u8));
                }
                DiscoveryUniqueBranchFrameResponse::decode(bytes)
                    .map(RdmResponse::DiscoveryUniqueBranchFrame)
            }
            _ => Err(RdmError::InvalidStartCode),
        }
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
    use core::time::Duration;
    use rdm_core::NackReasonCode;

    #[test]
    fn should_encode_decode_valid_rdm_ack_response() {
        let mut buf = [0u8; 256];

        let frame = RdmResponse::RdmFrame(RdmFrameResponse {
            destination_uid: DeviceUID::new(0x0102, 0x03040506),
            source_uid: DeviceUID::new(0x0605, 0x04030201),
            transaction_number: 0x00,
            message_count: 0x00,
            sub_device_id: SubDeviceId::RootDevice,
            parameter: ResponseParameter::GetIdentifyDevice(ResponseResult::Ack(
                GetIdentifyDeviceResponse { identify: true },
            )),
        });

        let bytes_encoded = frame.encode(&mut buf).unwrap();

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
            0x21, // Command Class = GetResponse
            0x10, 0x00, // Parameter ID = Identify Device
            0x01, // PDL
            0x01, // Identifying = true
            0x01, 0x43, // Checksum
        ];

        assert_eq!(&buf[0..bytes_encoded], expected);

        let decoded = RdmResponse::decode(&buf[0..bytes_encoded]).unwrap();

        assert_eq!(decoded, frame);
    }

    #[test]
    fn should_encode_decode_valid_rdm_ack_manufacturer_specific_response() {
        let mut buf = [0u8; 256];

        let frame = RdmResponse::RdmFrame(RdmFrameResponse {
            destination_uid: DeviceUID::new(0x0102, 0x03040506),
            source_uid: DeviceUID::new(0x0605, 0x04030201),
            transaction_number: 0x00,
            message_count: 0x00,
            sub_device_id: SubDeviceId::RootDevice,
            parameter: ResponseParameter::CustomParameter(CustomResponseParameter {
                command_class: CommandClass::SetResponse,
                parameter_id: ParameterId::Custom(0x8080),
                parameter_data_length: 4,
                response: ResponseResult::Ack(Vec::from_slice(&[0x04, 0x03, 0x02, 0x01]).unwrap()),
            }),
        });

        let bytes_encoded = frame.encode(&mut buf).unwrap();

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
            0x31, // Command Class = SetResponse
            0x80, 0x80, // Parameter ID = Identify Device
            0x04, // PDL
            0x04, 0x03, 0x02, 0x01, // Arbitrary manufacturer specific data
            0x02, 0x52, // Checksum
        ];

        assert_eq!(&buf[0..bytes_encoded], expected);

        let decoded = RdmResponse::decode(&buf[0..bytes_encoded]).unwrap();

        assert_eq!(decoded, frame);
    }

    #[test]
    fn should_encode_decode_valid_rdm_ack_timer_response() {
        let mut buf = [0u8; 256];

        let frame = RdmResponse::RdmFrame(RdmFrameResponse {
            destination_uid: DeviceUID::new(0x0102, 0x03040506),
            source_uid: DeviceUID::new(0x0605, 0x04030201),
            transaction_number: 0x00,
            message_count: 0x00,
            sub_device_id: SubDeviceId::RootDevice,
            parameter: ResponseParameter::GetIdentifyDevice(ResponseResult::AckTimer(
                Duration::from_secs(1),
            )),
        });

        let bytes_encoded = frame.encode(&mut buf).unwrap();
        assert_eq!(bytes_encoded, frame.size());

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
            0x21, // Command Class = GetResponse
            0x10, 0x00, // Parameter ID = Identify Device
            0x02, // PDL
            0x00, 0x0a, // Estimated Response Time = 10x 100ms = 1 second
            0x01, 0x4f, // Checksum
        ];

        assert_eq!(&buf[0..bytes_encoded], expected);

        let decoded = RdmResponse::decode(&buf[0..bytes_encoded]).unwrap();

        assert_eq!(decoded, frame);
    }

    #[test]
    fn should_encode_decode_valid_rdm_nack_reason_response() {
        let mut buf = [0u8; 256];

        let frame = RdmResponse::RdmFrame(RdmFrameResponse {
            destination_uid: DeviceUID::new(0x0102, 0x03040506),
            source_uid: DeviceUID::new(0x0605, 0x04030201),
            transaction_number: 0x00,
            message_count: 0x00,
            sub_device_id: SubDeviceId::RootDevice,
            parameter: ResponseParameter::GetIdentifyDevice(ResponseResult::Nack(
                NackReasonCode::FormatError,
            )),
        });

        let bytes_encoded = frame.encode(&mut buf).unwrap();
        assert_eq!(bytes_encoded, frame.size());

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
            0x21, // Command Class = GetResponse
            0x10, 0x00, // Parameter ID = Identify Device
            0x02, // PDL
            0x00, 0x01, // Nack Reason = FormatError
            0x01, 0x47, // Checksum
        ];

        assert_eq!(&buf[0..bytes_encoded], expected);

        let decoded = RdmResponse::decode(&buf[0..bytes_encoded]).unwrap();

        assert_eq!(decoded, frame);
    }

    #[test]
    fn should_encode_decode_valid_rdm_ack_overflow_response() {
        let mut buf = [0u8; 256];

        let frame = RdmResponse::RdmFrame(RdmFrameResponse {
            destination_uid: DeviceUID::new(0x0102, 0x03040506),
            source_uid: DeviceUID::new(0x0605, 0x04030201),
            transaction_number: 0x00,
            message_count: 0x00,
            sub_device_id: SubDeviceId::RootDevice,
            parameter: ResponseParameter::GetIdentifyDevice(ResponseResult::AckOverflow(
                GetIdentifyDeviceResponse { identify: true },
            )),
        });

        let bytes_encoded = frame.encode(&mut buf).unwrap();
        assert_eq!(bytes_encoded, frame.size());

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
            0x21, // Command Class = GetResponse
            0x10, 0x00, // Parameter ID = Identify Device
            0x01, // PDL
            0x01, // Identifying = true
            0x01, 0x46, // Checksum
        ];

        assert_eq!(&buf[0..bytes_encoded], expected);

        let decoded = RdmResponse::decode(&buf[0..bytes_encoded]).unwrap();

        assert_eq!(decoded, frame);
    }

    #[test]
    fn should_encode_decode_valid_discovery_unique_branch_response() {
        let mut buf = [0u8; 256];

        let frame = RdmResponse::DiscoveryUniqueBranchFrame(DiscoveryUniqueBranchFrameResponse {
            device_uid: DeviceUID::new(0x0102, 0x03040506),
        });

        let bytes_encoded = frame.encode(&mut buf).unwrap();
        assert_eq!(bytes_encoded, frame.size());

        // includes preamble bytes
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

        assert_eq!(&buf[0..bytes_encoded], expected);

        let decoded = RdmResponse::decode(&buf[0..bytes_encoded]).unwrap();

        assert_eq!(decoded, frame);

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
        ])
        .unwrap();

        assert_eq!(decoded, frame);
    }
}
