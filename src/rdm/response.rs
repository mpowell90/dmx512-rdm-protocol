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
    DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE, DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE,
    RDM_START_CODE_BYTE, RDM_SUB_START_CODE_BYTE, RdmError,
    header::{CommandClass, DeviceUID, SubDeviceId},
    parameter::{
        ParameterId,
        e120::{
            BootSoftwareVersionLabel, DefaultSlotValue, DeviceModelDescription, DisplayInvertMode,
            DmxPersonalityDescription, LampOnMode, LampState, ManufacturerLabel,
            ParameterDescription, ParameterDescriptionLabel, PowerState, ProductDetailValue,
            SelfTestDescription, SensorDefinition, SensorDefinitionDescription, SensorValue,
            SlotDescription, SlotInfo, SoftwareVersionLabel, StatusIdDescription, StatusType,
        },
        e137_1::{MergeMode, PinCode},
    },
    utils::{RdmPadNullStr, RdmTruncateNullStr, bsd_16_crc},
};
use crate::rdm::parameter::{
    e120::{
        DiscMuteResponse, DiscUnMuteResponse, GetCommsStatusResponse, GetDeviceInfoResponse,
        GetDeviceLabelResponse, GetDmxPersonality, GetDmxPersonalityDescription,
        GetLanguageCapabilitiesResponse, GetLanguageResponse, GetPresetPlayback,
        GetProxiedDeviceCountResponse, GetProxiedDevicesResponse, GetRealTimeClock,
        GetSelfTestDescription, GetSlotDescription, GetStatusMessagesResponse,
    },
    e133::{
        GetBrokerStatusResponse, GetComponentScopeResponse, GetSearchDomainResponse,
        GetTcpCommsStatusResponse,
    },
    e137_1::{
        GetCurve, GetCurveDescription, GetDimmerInfo, GetDmxBlockAddress, GetDmxFailMode,
        GetDmxStartupMode, GetLockState, GetLockStateDescription, GetMinimumLevel,
        GetModulationFrequency, GetModulationFrequencyDescription, GetOutputResponseTime,
        GetOutputResponseTimeDescription, GetPresetInfo, GetPresetStatus,
    },
    e137_2::{
        GetDnsDomainNameResponse, GetDnsHostNameResponse, GetDnsIpV4NameServerResponse,
        GetInterfaceHardwareAddressType1Response, GetInterfaceLabelResponse,
        GetIpV4CurrentAddressResponse, GetIpV4DefaultRouteResponse, GetIpV4DhcpModeResponse,
        GetIpV4StaticAddressResponse, GetIpV4ZeroConfModeResponse, GetListInterfacesResponse,
    },
    e137_7::{
        GetBackgroundDiscoveryResponse, GetBackgroundQueuedStatusPolicyDescriptionResponse,
        GetBackgroundQueuedStatusPolicyResponse, GetBindingControlFieldsResponse,
        GetDiscoveryStateResponse, GetEndpointLabelResponse, GetEndpointListChangeResponse,
        GetEndpointListResponse, GetEndpointModeResponse, GetEndpointResponderListChangeResponse,
        GetEndpointRespondersResponse, GetEndpointTimingDescriptionResponse,
        GetEndpointTimingResponse, GetEndpointToUniverseResponse, GetIdentifyEndpointResponse,
        GetRdmTrafficEnableResponse, SetBackgroundDiscoveryResponse, SetDiscoveryStateResponse,
        SetEndpointLabelResponse, SetEndpointModeResponse, SetEndpointTimingResponse,
        SetEndpointToUniverseResponse, SetIdentifyEndpointResponse, SetRdmTrafficEnableResponse,
    },
};
use core::{convert::TryFrom, fmt::Display, result::Result};
use heapless::Vec;
use rdm_parameter_traits::{
    RdmDiscoveryResponseParameterCodec, RdmGetResponseParameterCodec, RdmSetResponseParameterCodec,
};

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
            Self::UnknownPid => {
                "The responder cannot comply with request because the message is not implemented in responder."
            }
            Self::FormatError => {
                "The responder cannot interpret request as controller data was not formatted correctly."
            }
            Self::HardwareFault => "The responder cannot comply due to an internal hardware fault.",
            Self::ProxyReject => "Proxy is not the RDM line master and cannot comply with message.",
            Self::WriteProtect => "Command normally allowed but being blocked currently.",
            Self::UnsupportedCommandClass => {
                "Not valid for Command Class attempted. May be used where GET allowed but SET is not supported."
            }
            Self::DataOutOfRange => {
                "Value for given Parameter out of allowable range or not supported."
            }
            Self::BufferFull => "Buffer or Queue space currently has no free space to store data.",
            Self::PacketSizeUnsupported => "Incoming message exceeds buffer capacity.",
            Self::SubDeviceIdOutOfRange => "Sub-Device is out of range or unknown.",
            Self::ProxyBufferFull => {
                "The proxy buffer is full and can not store any more Queued Message or Status Message responses."
            }
            Self::ActionNotSupported => {
                "The parameter data is valid but the SET operation cannot be performed with the current configuration."
            }
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
pub enum ResponseData {
    ParameterData(Option<ResponseParameterData>),
    EstimateResponseTime(u16),
    NackReason(ResponseNackReasonCode),
}

impl ResponseData {
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
    DiscMute(DiscMuteResponse),
    DiscUnMute(DiscUnMuteResponse),
    GetProxiedDeviceCount(GetProxiedDeviceCountResponse),
    GetProxiedDevices(GetProxiedDevicesResponse),
    GetCommsStatus(GetCommsStatusResponse),
    GetStatusMessages(GetStatusMessagesResponse),
    GetStatusIdDescription(StatusIdDescription),
    GetSubDeviceIdStatusReportThreshold(StatusType),
    GetSupportedParameters(Vec<u16, 115>),
    GetParameterDescription(ParameterDescription),
    GetDeviceInfo(GetDeviceInfoResponse),
    GetProductDetailIdList(Vec<ProductDetailValue, 115>),
    GetDeviceModelDescription(DeviceModelDescription),
    GetManufacturerLabel(ManufacturerLabel),
    GetDeviceLabel(GetDeviceLabelResponse),
    GetFactoryDefaults(bool),
    GetLanguageCapabilities(GetLanguageCapabilitiesResponse),
    GetLanguage(GetLanguageResponse),
    GetSoftwareVersionLabel(SoftwareVersionLabel),
    GetBootSoftwareVersionId(u32),
    GetBootSoftwareVersionLabel(BootSoftwareVersionLabel),
    GetDmxPersonality(GetDmxPersonality),
    GetDmxPersonalityDescription(GetDmxPersonalityDescription),
    GetDmxStartAddress(u16),
    GetSlotInfo(Vec<SlotInfo, 46>),
    GetSlotDescription(GetSlotDescription),
    GetDefaultSlotValue(Vec<DefaultSlotValue, 77>),
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
    GetRealTimeClock(GetRealTimeClock),
    GetIdentifyDevice(bool),
    GetPowerState(PowerState),
    GetPerformSelfTest(bool),
    GetSelfTestDescription(GetSelfTestDescription),
    GetPresetPlayback(GetPresetPlayback),
    // E1.37-1
    GetDmxBlockAddress(GetDmxBlockAddress),
    GetDmxFailMode(GetDmxFailMode),
    GetDmxStartupMode(GetDmxStartupMode),
    GetPowerOnSelfTest(bool),
    GetLockState(GetLockState),
    GetLockStateDescription(GetLockStateDescription),
    GetLockPin(PinCode),
    GetBurnIn(u8),
    GetDimmerInfo(GetDimmerInfo),
    GetMinimumLevel(GetMinimumLevel),
    GetMaximumLevel(u16),
    GetCurve(GetCurve),
    GetCurveDescription(GetCurveDescription),
    GetOutputResponseTime(GetOutputResponseTime),
    GetOutputResponseTimeDescription(GetOutputResponseTimeDescription),
    GetModulationFrequency(GetModulationFrequency),
    GetModulationFrequencyDescription(GetModulationFrequencyDescription),
    GetPresetInfo(GetPresetInfo),
    GetPresetStatus(GetPresetStatus),
    GetPresetMergeMode(MergeMode),
    // E1.37-2
    GetListInterfaces(GetListInterfacesResponse),
    GetInterfaceLabel(GetInterfaceLabelResponse),
    GetInterfaceHardwareAddressType1(GetInterfaceHardwareAddressType1Response),
    GetIpV4DhcpMode(GetIpV4DhcpModeResponse),
    GetIpV4ZeroConfMode(GetIpV4ZeroConfModeResponse),
    GetIpV4CurrentAddress(GetIpV4CurrentAddressResponse),
    GetIpV4StaticAddress(GetIpV4StaticAddressResponse),
    GetIpV4DefaultRoute(GetIpV4DefaultRouteResponse),
    GetDnsIpV4NameServer(GetDnsIpV4NameServerResponse),
    GetDnsHostName(GetDnsHostNameResponse),
    GetDnsDomainName(GetDnsDomainNameResponse),
    // E1.37-7
    GetEndpointList(GetEndpointListResponse),
    GetEndpointListChange(GetEndpointListChangeResponse),
    GetIdentifyEndpoint(GetIdentifyEndpointResponse),
    SetIdentifyEndpoint(SetIdentifyEndpointResponse),
    GetEndpointToUniverse(GetEndpointToUniverseResponse),
    SetEndpointToUniverse(SetEndpointToUniverseResponse),
    GetEndpointMode(GetEndpointModeResponse),
    SetEndpointMode(SetEndpointModeResponse),
    GetEndpointLabel(GetEndpointLabelResponse),
    SetEndpointLabel(SetEndpointLabelResponse),
    GetRdmTrafficEnable(GetRdmTrafficEnableResponse),
    SetRdmTrafficEnable(SetRdmTrafficEnableResponse),
    GetDiscoveryState(GetDiscoveryStateResponse),
    SetDiscoveryState(SetDiscoveryStateResponse),
    GetBackgroundDiscovery(GetBackgroundDiscoveryResponse),
    SetBackgroundDiscovery(SetBackgroundDiscoveryResponse),
    GetEndpointTiming(GetEndpointTimingResponse),
    SetEndpointTiming(SetEndpointTimingResponse),
    GetEndpointTimingDescription(GetEndpointTimingDescriptionResponse),
    GetEndpointResponders(GetEndpointRespondersResponse),
    GetEndpointResponderListChange(GetEndpointResponderListChangeResponse),
    GetBindingControlFields(GetBindingControlFieldsResponse),
    GetBackgroundQueuedStatusPolicy(GetBackgroundQueuedStatusPolicyResponse),
    GetBackgroundQueuedStatusPolicyDescription(GetBackgroundQueuedStatusPolicyDescriptionResponse),
    // E1.33
    GetComponentScope(GetComponentScopeResponse),
    GetSearchDomain(GetSearchDomainResponse),
    GetTcpCommsStatus(GetTcpCommsStatusResponse),
    GetBrokerStatus(GetBrokerStatusResponse),
    RawParameter(Vec<u8, 231>),
}

impl ResponseParameterData {
    pub fn size(&self) -> usize {
        match self {
            ResponseParameterData::DiscMute(DiscMuteResponse { binding_uid, .. })
            | ResponseParameterData::DiscUnMute(DiscUnMuteResponse { binding_uid, .. }) => {
                if binding_uid.is_some() {
                    8
                } else {
                    2
                }
            }
            ResponseParameterData::GetProxiedDeviceCount(_) => 3,
            ResponseParameterData::GetProxiedDevices(GetProxiedDevicesResponse { device_uids }) => {
                device_uids.len() * 6
            }
            ResponseParameterData::GetCommsStatus(_) => 6,
            ResponseParameterData::GetStatusMessages(param) => param.size_of(),
            ResponseParameterData::GetStatusIdDescription(description) => description.len(),
            ResponseParameterData::GetSubDeviceIdStatusReportThreshold(_) => 1,
            ResponseParameterData::GetSupportedParameters(parameters) => parameters.len() * 2,
            ResponseParameterData::GetParameterDescription(description) => {
                20 + description.description.len()
            }
            ResponseParameterData::GetDeviceInfo(_) => 19,
            ResponseParameterData::GetProductDetailIdList(details) => details.len() * 2,
            ResponseParameterData::GetDeviceModelDescription(description) => description.len(),
            ResponseParameterData::GetManufacturerLabel(label) => label.len(),
            ResponseParameterData::GetDeviceLabel(param) => param.size_of(),
            ResponseParameterData::GetFactoryDefaults(_) => 1,
            ResponseParameterData::GetLanguageCapabilities(param) => param.size_of(),
            ResponseParameterData::GetLanguage(_) => 2,
            ResponseParameterData::GetSoftwareVersionLabel(label) => label.len(),
            ResponseParameterData::GetBootSoftwareVersionId(_) => 4,
            ResponseParameterData::GetBootSoftwareVersionLabel(label) => label.len(),
            ResponseParameterData::GetDmxPersonality(_) => 2,
            ResponseParameterData::GetDmxPersonalityDescription(GetDmxPersonalityDescription {
                description,
                ..
            }) => 3 + description.len(),
            ResponseParameterData::GetDmxStartAddress(_) => 2,
            ResponseParameterData::GetSlotInfo(slots) => slots.len() * 5,
            ResponseParameterData::GetSlotDescription(GetSlotDescription {
                description, ..
            }) => 2 + description.len(),
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
            ResponseParameterData::GetRealTimeClock(_) => 7,
            ResponseParameterData::GetIdentifyDevice(_) => 1,
            ResponseParameterData::GetPowerState(_) => 1,
            ResponseParameterData::GetPerformSelfTest(_) => 1,
            ResponseParameterData::GetSelfTestDescription(GetSelfTestDescription {
                description,
                ..
            }) => 1 + description.len(),
            ResponseParameterData::GetPresetPlayback(_) => 3,
            ResponseParameterData::GetDmxBlockAddress(_) => 4,
            ResponseParameterData::GetDmxFailMode(_) => 7,
            ResponseParameterData::GetDmxStartupMode(_) => 7,
            ResponseParameterData::GetPowerOnSelfTest(_) => 1,
            ResponseParameterData::GetLockState(_) => 2,
            ResponseParameterData::GetLockStateDescription(GetLockStateDescription {
                description,
                ..
            }) => 1 + description.len(),
            ResponseParameterData::GetLockPin(_) => 2,
            ResponseParameterData::GetBurnIn(_) => 1,
            ResponseParameterData::GetDimmerInfo(_) => 11,
            ResponseParameterData::GetMinimumLevel(_) => 5,
            ResponseParameterData::GetMaximumLevel(_) => 2,
            ResponseParameterData::GetCurve(_) => 2,
            ResponseParameterData::GetCurveDescription(GetCurveDescription {
                description, ..
            }) => 1 + description.len(),
            ResponseParameterData::GetOutputResponseTime(_) => 2,
            ResponseParameterData::GetOutputResponseTimeDescription(
                GetOutputResponseTimeDescription { description, .. },
            ) => 1 + description.len(),
            ResponseParameterData::GetModulationFrequency(_) => 2,
            ResponseParameterData::GetModulationFrequencyDescription(
                GetModulationFrequencyDescription { description, .. },
            ) => 5 + description.len(),
            ResponseParameterData::GetPresetInfo(_) => 32,
            ResponseParameterData::GetPresetStatus(_) => 9,
            ResponseParameterData::GetPresetMergeMode(_) => 1,
            ResponseParameterData::GetListInterfaces(param) => param.size_of(),
            ResponseParameterData::GetInterfaceLabel(param) => param.size_of(),
            ResponseParameterData::GetInterfaceHardwareAddressType1(_) => 10,
            ResponseParameterData::GetIpV4DhcpMode(_) => 5,
            ResponseParameterData::GetIpV4ZeroConfMode(_) => 5,
            ResponseParameterData::GetIpV4CurrentAddress(_) => 10,
            ResponseParameterData::GetIpV4StaticAddress(_) => 9,
            ResponseParameterData::GetIpV4DefaultRoute(_) => 8,
            ResponseParameterData::GetDnsIpV4NameServer(_) => 5,
            ResponseParameterData::GetDnsHostName(param) => param.size_of(),
            ResponseParameterData::GetDnsDomainName(param) => param.size_of(),
            ResponseParameterData::GetEndpointList(param) => param.size_of(),
            ResponseParameterData::GetEndpointListChange(param) => param.size_of(),
            ResponseParameterData::GetIdentifyEndpoint(param) => param.size_of(),
            ResponseParameterData::SetIdentifyEndpoint(param) => param.size_of(),
            ResponseParameterData::GetEndpointToUniverse(param) => param.size_of(),
            ResponseParameterData::SetEndpointToUniverse(param) => param.size_of(),
            ResponseParameterData::GetEndpointMode(param) => param.size_of(),
            ResponseParameterData::SetEndpointMode(param) => param.size_of(),
            ResponseParameterData::GetEndpointLabel(param) => param.size_of(),
            ResponseParameterData::SetEndpointLabel(param) => param.size_of(),
            ResponseParameterData::GetRdmTrafficEnable(param) => param.size_of(),
            ResponseParameterData::SetRdmTrafficEnable(param) => param.size_of(),
            ResponseParameterData::GetDiscoveryState(param) => param.size_of(),
            ResponseParameterData::SetDiscoveryState(param) => param.size_of(),
            ResponseParameterData::GetBackgroundDiscovery(param) => param.size_of(),
            ResponseParameterData::SetBackgroundDiscovery(param) => param.size_of(),
            ResponseParameterData::GetEndpointTiming(param) => param.size_of(),
            ResponseParameterData::SetEndpointTiming(param) => param.size_of(),
            ResponseParameterData::GetEndpointTimingDescription(param) => param.size_of(),
            ResponseParameterData::GetEndpointResponders(param) => param.size_of(),
            ResponseParameterData::GetEndpointResponderListChange(param) => param.size_of(),
            ResponseParameterData::GetBindingControlFields(param) => param.size_of(),
            ResponseParameterData::GetBackgroundQueuedStatusPolicy(param) => param.size_of(),
            ResponseParameterData::GetBackgroundQueuedStatusPolicyDescription(param) => {
                param.size_of()
            }
            ResponseParameterData::GetComponentScope(param) => param.size_of(),
            ResponseParameterData::GetSearchDomain(param) => param.size_of(),
            ResponseParameterData::GetTcpCommsStatus(param) => param.size_of(),
            ResponseParameterData::GetBrokerStatus(param) => param.size_of(),
            ResponseParameterData::RawParameter(param) => param.len(),
        }
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, RdmError> {
        match self {
            Self::DiscMute(param) => {
                param.discovery_response_encode_data(buf)?;
            }
            Self::DiscUnMute(param) => {
                param.discovery_response_encode_data(buf)?;
            }
            Self::GetProxiedDeviceCount(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetProxiedDevices(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetCommsStatus(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetStatusMessages(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetStatusIdDescription(description) => {
                description.encode(buf)?;
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
            Self::GetDeviceInfo(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetProductDetailIdList(details) => {
                for (idx, detail) in details.iter().enumerate() {
                    buf[idx * 2..(idx + 2) * 2].copy_from_slice(&detail.0.to_be_bytes());
                }
            }
            Self::GetDeviceModelDescription(description) => {
                description.encode(buf)?;
            }
            Self::GetManufacturerLabel(label) => {
                label.encode(buf)?;
            }
            Self::GetDeviceLabel(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetFactoryDefaults(defaults) => {
                buf[0] = *defaults as u8;
            }
            Self::GetLanguageCapabilities(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetLanguage(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetSoftwareVersionLabel(label) => {
                label.encode(buf)?;
            }
            Self::GetBootSoftwareVersionId(version_id) => {
                buf[0..4].copy_from_slice(&version_id.to_be_bytes());
            }
            Self::GetBootSoftwareVersionLabel(label) => {
                label.encode(buf)?;
            }
            Self::GetDmxPersonality(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetDmxPersonalityDescription(param) => {
                param.get_response_encode_data(buf)?;
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
            Self::GetSlotDescription(param) => {
                param.get_response_encode_data(buf)?;
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
            Self::GetRealTimeClock(GetRealTimeClock {
                year,
                month,
                day,
                hour,
                minute,
                second,
            }) => {
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
            Self::GetSelfTestDescription(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetPresetPlayback(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetDmxBlockAddress(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetDmxFailMode(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetDmxStartupMode(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetPowerOnSelfTest(test) => {
                buf[0] = *test as u8;
            }
            Self::GetLockState(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetLockStateDescription(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetLockPin(pin) => {
                buf[0..2].copy_from_slice(&pin.0.to_be_bytes());
            }
            Self::GetBurnIn(hours) => {
                buf[0] = *hours;
            }
            Self::GetDimmerInfo(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetMinimumLevel(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetMaximumLevel(level) => {
                buf[0..2].copy_from_slice(&level.to_be_bytes());
            }
            Self::GetCurve(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetCurveDescription(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetOutputResponseTime(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetOutputResponseTimeDescription(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetModulationFrequency(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetModulationFrequencyDescription(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetPresetInfo(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetPresetStatus(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetPresetMergeMode(mode) => {
                buf[0] = *mode as u8;
            }
            Self::GetListInterfaces(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetInterfaceLabel(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetInterfaceHardwareAddressType1(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetIpV4DhcpMode(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetIpV4ZeroConfMode(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetIpV4CurrentAddress(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetIpV4StaticAddress(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetIpV4DefaultRoute(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetDnsIpV4NameServer(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetDnsHostName(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetDnsDomainName(param) => {
                param.get_response_encode_data(buf)?;
            }
            // E1.37-7
            Self::GetEndpointList(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetEndpointListChange(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetIdentifyEndpoint(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::SetIdentifyEndpoint(param) => {
                param.set_response_encode_data(buf)?;
            }
            Self::GetEndpointToUniverse(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::SetEndpointToUniverse(param) => {
                param.set_response_encode_data(buf)?;
            }
            Self::GetEndpointMode(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::SetEndpointMode(param) => {
                param.set_response_encode_data(buf)?;
            }
            Self::GetEndpointLabel(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::SetEndpointLabel(param) => {
                param.set_response_encode_data(buf)?;
            }
            Self::GetRdmTrafficEnable(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::SetRdmTrafficEnable(param) => {
                param.set_response_encode_data(buf)?;
            }
            Self::GetDiscoveryState(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::SetDiscoveryState(param) => {
                param.set_response_encode_data(buf)?;
            }
            Self::GetBackgroundDiscovery(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::SetBackgroundDiscovery(param) => {
                param.set_response_encode_data(buf)?;
            }
            Self::GetEndpointTiming(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::SetEndpointTiming(param) => {
                param.set_response_encode_data(buf)?;
            }
            Self::GetEndpointTimingDescription(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetEndpointResponders(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetEndpointResponderListChange(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetBindingControlFields(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetBackgroundQueuedStatusPolicy(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetBackgroundQueuedStatusPolicyDescription(param) => {
                param.get_response_encode_data(buf)?;
            }
            // E1.33
            Self::GetComponentScope(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetSearchDomain(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetTcpCommsStatus(param) => {
                param.get_response_encode_data(buf)?;
            }
            Self::GetBrokerStatus(param) => {
                param.get_response_encode_data(buf)?;
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
        bytes: &[u8],
    ) -> Result<Self, RdmError> {
        match (command_class, parameter_id) {
            (CommandClass::DiscoveryCommandResponse, ParameterId::DiscMute) => Ok(Self::DiscMute(
                DiscMuteResponse::discovery_response_decode_data(bytes)?,
            )),
            (CommandClass::DiscoveryCommandResponse, ParameterId::DiscUnMute) => Ok(
                Self::DiscUnMute(DiscUnMuteResponse::discovery_response_decode_data(bytes)?),
            ),
            (CommandClass::GetCommandResponse, ParameterId::ProxiedDeviceCount) => {
                Ok(Self::GetProxiedDeviceCount(
                    GetProxiedDeviceCountResponse::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::ProxiedDevices) => {
                Ok(Self::GetProxiedDevices(
                    GetProxiedDevicesResponse::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::CommsStatus) => Ok(
                Self::GetCommsStatus(GetCommsStatusResponse::get_response_decode_data(bytes)?),
            ),
            (CommandClass::GetCommandResponse, ParameterId::StatusMessages) => {
                Ok(Self::GetStatusMessages(
                    GetStatusMessagesResponse::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::StatusIdDescription) => Ok(
                Self::GetStatusIdDescription(StatusIdDescription::decode(bytes)?),
            ),
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
            (CommandClass::GetCommandResponse, ParameterId::DeviceInfo) => Ok(Self::GetDeviceInfo(
                GetDeviceInfoResponse::get_response_decode_data(bytes)?,
            )),
            (CommandClass::GetCommandResponse, ParameterId::ProductDetailIdList) => {
                Ok(Self::GetProductDetailIdList(
                    bytes
                        .chunks(2)
                        .map(|chunk| Ok(ProductDetailValue(u16::from_be_bytes(chunk.try_into()?))))
                        .collect::<Result<Vec<ProductDetailValue, 115>, RdmError>>()?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::DeviceModelDescription) => Ok(
                Self::GetDeviceModelDescription(DeviceModelDescription::decode(bytes)?),
            ),
            (CommandClass::GetCommandResponse, ParameterId::ManufacturerLabel) => Ok(
                Self::GetManufacturerLabel(ManufacturerLabel::decode(bytes)?),
            ),
            (CommandClass::GetCommandResponse, ParameterId::DeviceLabel) => Ok(
                Self::GetDeviceLabel(GetDeviceLabelResponse::get_response_decode_data(bytes)?),
            ),
            (CommandClass::GetCommandResponse, ParameterId::FactoryDefaults) => {
                Ok(Self::GetFactoryDefaults(bytes[0] == 1))
            }
            (CommandClass::GetCommandResponse, ParameterId::LanguageCapabilities) => {
                Ok(Self::GetLanguageCapabilities(
                    GetLanguageCapabilitiesResponse::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::Language) => Ok(Self::GetLanguage(
                GetLanguageResponse::get_response_decode_data(bytes)?,
            )),
            (CommandClass::GetCommandResponse, ParameterId::SoftwareVersionLabel) => Ok(
                Self::GetSoftwareVersionLabel(SoftwareVersionLabel::decode(bytes)?),
            ),
            (CommandClass::GetCommandResponse, ParameterId::BootSoftwareVersionId) => Ok(
                Self::GetBootSoftwareVersionId(u32::from_be_bytes(bytes.try_into()?)),
            ),
            (CommandClass::GetCommandResponse, ParameterId::BootSoftwareVersionLabel) => Ok(
                Self::GetBootSoftwareVersionLabel(BootSoftwareVersionLabel::decode(bytes)?),
            ),
            (CommandClass::GetCommandResponse, ParameterId::DmxPersonality) => {
                Ok(Self::GetDmxPersonality(GetDmxPersonality {
                    current_personality: bytes[0],
                    personality_count: bytes[1],
                }))
            }
            (CommandClass::GetCommandResponse, ParameterId::DmxPersonalityDescription) => Ok(
                Self::GetDmxPersonalityDescription(GetDmxPersonalityDescription {
                    id: bytes[0],
                    dmx_slots_required: u16::from_be_bytes(bytes[1..=2].try_into()?),
                    description: DmxPersonalityDescription::decode(&bytes[3..])?,
                }),
            ),
            (CommandClass::GetCommandResponse, ParameterId::DmxStartAddress) => Ok(
                Self::GetDmxStartAddress(u16::from_be_bytes(bytes[0..=1].try_into()?)),
            ),
            (CommandClass::GetCommandResponse, ParameterId::SlotInfo) => Ok(Self::GetSlotInfo(
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
                Ok(Self::GetSlotDescription(GetSlotDescription {
                    slot_id: u16::from_be_bytes(bytes[0..=1].try_into()?),
                    description: SlotDescription::decode(&bytes[2..])?,
                }))
            }
            (CommandClass::GetCommandResponse, ParameterId::DefaultSlotValue) => {
                Ok(Self::GetDefaultSlotValue(
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
                    description: SensorDefinitionDescription::decode(&bytes[13..])?,
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
                Ok(Self::GetRealTimeClock(GetRealTimeClock {
                    year: u16::from_be_bytes(bytes[0..=1].try_into()?),
                    month: bytes[2],
                    day: bytes[3],
                    hour: bytes[4],
                    minute: bytes[5],
                    second: bytes[6],
                }))
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
                Ok(Self::GetSelfTestDescription(GetSelfTestDescription {
                    self_test_id: bytes[0].into(),
                    description: SelfTestDescription::decode(&bytes[1..])?,
                }))
            }
            (CommandClass::GetCommandResponse, ParameterId::PresetPlayback) => {
                Ok(Self::GetPresetPlayback(GetPresetPlayback {
                    mode: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                    level: bytes[2],
                }))
            }
            // E1.37-1
            (CommandClass::GetCommandResponse, ParameterId::DmxBlockAddress) => {
                Ok(Self::GetDmxBlockAddress(GetDmxBlockAddress {
                    total_sub_device_footprint: u16::from_be_bytes(bytes[0..=1].try_into()?),
                    base_dmx_address: u16::from_be_bytes(bytes[2..=3].try_into()?),
                }))
            }
            (CommandClass::GetCommandResponse, ParameterId::DmxFailMode) => {
                Ok(Self::GetDmxFailMode(GetDmxFailMode {
                    scene_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                    loss_of_signal_delay: u16::from_be_bytes(bytes[2..=3].try_into()?).into(),
                    hold_time: u16::from_be_bytes(bytes[4..=5].try_into()?).into(),
                    level: bytes[6],
                }))
            }
            (CommandClass::GetCommandResponse, ParameterId::DmxStartupMode) => {
                Ok(Self::GetDmxStartupMode(GetDmxStartupMode {
                    scene_id: u16::from_be_bytes(bytes[0..=1].try_into()?).into(),
                    startup_delay: u16::from_be_bytes(bytes[2..=3].try_into()?).into(),
                    hold_time: u16::from_be_bytes(bytes[4..=5].try_into()?).into(),
                    level: bytes[6],
                }))
            }
            (CommandClass::GetCommandResponse, ParameterId::PowerOnSelfTest) => {
                Ok(Self::GetPowerOnSelfTest(bytes[0] == 1))
            }
            (CommandClass::GetCommandResponse, ParameterId::LockState) => Ok(Self::GetLockState(
                GetLockState::get_response_decode_data(bytes)?,
            )),
            (CommandClass::GetCommandResponse, ParameterId::LockStateDescription) => {
                Ok(Self::GetLockStateDescription(
                    GetLockStateDescription::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::LockPin) => Ok(Self::GetLockPin(
                PinCode::try_from(u16::from_be_bytes(bytes[0..=1].try_into()?))?,
            )),
            (CommandClass::GetCommandResponse, ParameterId::BurnIn) => {
                Ok(Self::GetBurnIn(bytes[0]))
            }
            (CommandClass::GetCommandResponse, ParameterId::DimmerInfo) => Ok(Self::GetDimmerInfo(
                GetDimmerInfo::get_response_decode_data(bytes)?,
            )),
            (CommandClass::GetCommandResponse, ParameterId::MinimumLevel) => Ok(
                Self::GetMinimumLevel(GetMinimumLevel::get_response_decode_data(bytes)?),
            ),
            (CommandClass::GetCommandResponse, ParameterId::MaximumLevel) => Ok(
                Self::GetMaximumLevel(u16::from_be_bytes(bytes[0..=1].try_into()?)),
            ),
            (CommandClass::GetCommandResponse, ParameterId::Curve) => {
                Ok(Self::GetCurve(GetCurve::get_response_decode_data(bytes)?))
            }
            (CommandClass::GetCommandResponse, ParameterId::CurveDescription) => Ok(
                Self::GetCurveDescription(GetCurveDescription::get_response_decode_data(bytes)?),
            ),
            (CommandClass::GetCommandResponse, ParameterId::OutputResponseTime) => {
                Ok(Self::GetOutputResponseTime(
                    GetOutputResponseTime::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::OutputResponseTimeDescription) => {
                Ok(Self::GetOutputResponseTimeDescription(
                    GetOutputResponseTimeDescription::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::ModulationFrequency) => {
                Ok(Self::GetModulationFrequency(
                    GetModulationFrequency::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::ModulationFrequencyDescription) => {
                Ok(Self::GetModulationFrequencyDescription(
                    GetModulationFrequencyDescription::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::PresetInfo) => Ok(Self::GetPresetInfo(
                GetPresetInfo::get_response_decode_data(bytes)?,
            )),
            (CommandClass::GetCommandResponse, ParameterId::PresetStatus) => Ok(
                Self::GetPresetStatus(GetPresetStatus::get_response_decode_data(bytes)?),
            ),
            (CommandClass::GetCommandResponse, ParameterId::PresetMergeMode) => {
                Ok(Self::GetPresetMergeMode(MergeMode::try_from(bytes[0])?))
            }
            // E1.37-2
            (CommandClass::GetCommandResponse, ParameterId::ListInterfaces) => {
                Ok(Self::GetListInterfaces(
                    GetListInterfacesResponse::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::InterfaceLabel) => {
                Ok(Self::GetInterfaceLabel(
                    GetInterfaceLabelResponse::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::InterfaceHardwareAddressType1) => {
                Ok(Self::GetInterfaceHardwareAddressType1(
                    GetInterfaceHardwareAddressType1Response::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::IpV4DhcpMode) => Ok(
                Self::GetIpV4DhcpMode(GetIpV4DhcpModeResponse::get_response_decode_data(bytes)?),
            ),
            (CommandClass::GetCommandResponse, ParameterId::IpV4ZeroConfMode) => {
                Ok(Self::GetIpV4ZeroConfMode(
                    GetIpV4ZeroConfModeResponse::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::IpV4CurrentAddress) => {
                Ok(Self::GetIpV4CurrentAddress(
                    GetIpV4CurrentAddressResponse::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::IpV4StaticAddress) => {
                Ok(Self::GetIpV4StaticAddress(
                    GetIpV4StaticAddressResponse::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::IpV4DefaultRoute) => {
                Ok(Self::GetIpV4DefaultRoute(
                    GetIpV4DefaultRouteResponse::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::DnsIpV4NameServer) => {
                Ok(Self::GetDnsIpV4NameServer(
                    GetDnsIpV4NameServerResponse::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::DnsHostName) => Ok(
                Self::GetDnsHostName(GetDnsHostNameResponse::get_response_decode_data(bytes)?),
            ),
            (CommandClass::GetCommandResponse, ParameterId::DnsDomainName) => Ok(
                Self::GetDnsDomainName(GetDnsDomainNameResponse::get_response_decode_data(bytes)?),
            ),
            // E1.37-7
            (CommandClass::GetCommandResponse, ParameterId::EndpointList) => Ok(
                Self::GetEndpointList(GetEndpointListResponse::get_response_decode_data(bytes)?),
            ),
            (CommandClass::GetCommandResponse, ParameterId::EndpointListChange) => {
                Ok(Self::GetEndpointListChange(
                    GetEndpointListChangeResponse::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::IdentifyEndpoint) => {
                Ok(Self::GetIdentifyEndpoint(
                    GetIdentifyEndpointResponse::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::SetCommandResponse, ParameterId::IdentifyEndpoint) => {
                Ok(Self::SetIdentifyEndpoint(
                    SetIdentifyEndpointResponse::set_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::EndpointToUniverse) => {
                Ok(Self::GetEndpointToUniverse(
                    GetEndpointToUniverseResponse::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::SetCommandResponse, ParameterId::EndpointToUniverse) => {
                Ok(Self::SetEndpointToUniverse(
                    SetEndpointToUniverseResponse::set_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::EndpointMode) => Ok(
                Self::GetEndpointMode(GetEndpointModeResponse::get_response_decode_data(bytes)?),
            ),
            (CommandClass::SetCommandResponse, ParameterId::EndpointMode) => Ok(
                Self::SetEndpointMode(SetEndpointModeResponse::set_response_decode_data(bytes)?),
            ),
            (CommandClass::GetCommandResponse, ParameterId::EndpointLabel) => Ok(
                Self::GetEndpointLabel(GetEndpointLabelResponse::get_response_decode_data(bytes)?),
            ),
            (CommandClass::SetCommandResponse, ParameterId::EndpointLabel) => Ok(
                Self::SetEndpointLabel(SetEndpointLabelResponse::set_response_decode_data(bytes)?),
            ),
            (CommandClass::GetCommandResponse, ParameterId::RdmTrafficEnable) => {
                Ok(Self::GetRdmTrafficEnable(
                    GetRdmTrafficEnableResponse::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::SetCommandResponse, ParameterId::RdmTrafficEnable) => {
                Ok(Self::SetRdmTrafficEnable(
                    SetRdmTrafficEnableResponse::set_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::DiscoveryState) => {
                Ok(Self::GetDiscoveryState(
                    GetDiscoveryStateResponse::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::SetCommandResponse, ParameterId::DiscoveryState) => {
                Ok(Self::SetDiscoveryState(
                    SetDiscoveryStateResponse::set_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::BackgroundDiscovery) => {
                Ok(Self::GetBackgroundDiscovery(
                    GetBackgroundDiscoveryResponse::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::SetCommandResponse, ParameterId::BackgroundDiscovery) => {
                Ok(Self::SetBackgroundDiscovery(
                    SetBackgroundDiscoveryResponse::set_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::EndpointTiming) => {
                Ok(Self::GetEndpointTiming(
                    GetEndpointTimingResponse::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::SetCommandResponse, ParameterId::EndpointTiming) => {
                Ok(Self::SetEndpointTiming(
                    SetEndpointTimingResponse::set_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::EndpointTimingDescription) => {
                Ok(Self::GetEndpointTimingDescription(
                    GetEndpointTimingDescriptionResponse::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::EndpointResponders) => {
                Ok(Self::GetEndpointResponders(
                    GetEndpointRespondersResponse::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::EndpointResponderListChange) => {
                Ok(Self::GetEndpointResponderListChange(
                    GetEndpointResponderListChangeResponse::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::BindingControlFields) => {
                Ok(Self::GetBindingControlFields(
                    GetBindingControlFieldsResponse::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::BackgroundQueuedStatusPolicy) => {
                Ok(Self::GetBackgroundQueuedStatusPolicy(
                    GetBackgroundQueuedStatusPolicyResponse::get_response_decode_data(bytes)?,
                ))
            }
            (
                CommandClass::GetCommandResponse,
                ParameterId::BackgroundQueuedStatusPolicyDescription,
            ) => Ok(Self::GetBackgroundQueuedStatusPolicyDescription(
                GetBackgroundQueuedStatusPolicyDescriptionResponse::get_response_decode_data(
                    bytes,
                )?,
            )),
            // E1.33
            (CommandClass::GetCommandResponse, ParameterId::ComponentScope) => {
                Ok(Self::GetComponentScope(
                    GetComponentScopeResponse::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::SearchDomain) => Ok(
                Self::GetSearchDomain(GetSearchDomainResponse::get_response_decode_data(bytes)?),
            ),
            (CommandClass::GetCommandResponse, ParameterId::TcpCommsStatus) => {
                Ok(Self::GetTcpCommsStatus(
                    GetTcpCommsStatusResponse::get_response_decode_data(bytes)?,
                ))
            }
            (CommandClass::GetCommandResponse, ParameterId::BrokerStatus) => Ok(
                Self::GetBrokerStatus(GetBrokerStatusResponse::get_response_decode_data(bytes)?),
            ),
            (_, _) => Ok(Self::RawParameter(Vec::from_slice(bytes).unwrap())),
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

    pub fn decode(bytes: &[u8]) -> Result<Self, RdmError> {
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

impl TryFrom<&[u8]> for RdmFrameResponse {
    type Error = RdmError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
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
                Vec::from_slice(&[0x04, 0x03, 0x02, 0x01]).unwrap(),
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
                Vec::from_slice(&[0x04, 0x03, 0x02, 0x01]).unwrap(),
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
