use super::RdmError;
use crate::{impl_rdm_string, rdm::DeviceUID};
use heapless::{String, Vec};
use rdm_parameter_derive::{
    RdmGetRequestParameter, RdmGetResponseParameter, RdmSetRequestParameter,
    RdmSetResponseParameter,
};
use rdm_parameter_traits::RdmParameterData;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DiscoveryState {
    Incomplete,
    Incremental,
    Full,
    NotActive,
    ManufacturerSpecific(u8),
}

impl TryFrom<u8> for DiscoveryState {
    type Error = RdmError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Incomplete),
            0x01 => Ok(Self::Incremental),
            0x02 => Ok(Self::Full),
            0x04 => Ok(Self::NotActive),
            value if (0x80..=0xff).contains(&value) => Ok(Self::ManufacturerSpecific(value)),
            value => Err(RdmError::InvalidDiscoveryState(value)),
        }
    }
}

impl From<DiscoveryState> for u8 {
    fn from(value: DiscoveryState) -> u8 {
        match value {
            DiscoveryState::Incomplete => 0x00,
            DiscoveryState::Incremental => 0x01,
            DiscoveryState::Full => 0x02,
            DiscoveryState::NotActive => 0x04,
            DiscoveryState::ManufacturerSpecific(value) => value,
        }
    }
}

impl RdmParameterData for DiscoveryState {
    fn size_of(&self) -> usize {
        1
    }

    fn encode_rdm_parameter_data(
        &self,
        buf: &mut [u8],
    ) -> Result<usize, rdm_parameter_traits::ParameterCodecError> {
        buf[0] = (*self).into();
        Ok(1)
    }

    fn decode_rdm_parameter_data(
        buf: &[u8],
    ) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
        let discovery_state = DiscoveryState::try_from(buf[0])
            .map_err(|_| rdm_parameter_traits::ParameterCodecError::MalformedData)?;
        Ok(discovery_state)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DiscoveryCountStatus {
    Incomplete,
    Count(u16),
    Unknown,
}

impl From<u16> for DiscoveryCountStatus {
    fn from(value: u16) -> Self {
        match value {
            0x0000 => Self::Incomplete,
            0xffff => Self::Unknown,
            value => Self::Count(value),
        }
    }
}

impl From<DiscoveryCountStatus> for u16 {
    fn from(value: DiscoveryCountStatus) -> u16 {
        match value {
            DiscoveryCountStatus::Incomplete => 0x0000,
            DiscoveryCountStatus::Unknown => 0xffff,
            DiscoveryCountStatus::Count(value) => value,
        }
    }
}

impl RdmParameterData for DiscoveryCountStatus {
    fn size_of(&self) -> usize {
        2
    }

    fn encode_rdm_parameter_data(
        &self,
        buf: &mut [u8],
    ) -> Result<usize, rdm_parameter_traits::ParameterCodecError> {
        let value: u16 = (*self).into();
        buf[0..2].copy_from_slice(&value.to_be_bytes());
        Ok(2)
    }

    fn decode_rdm_parameter_data(
        buf: &[u8],
    ) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
        let value = u16::from_be_bytes([buf[0], buf[1]]);
        Ok(DiscoveryCountStatus::from(value))
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum EndpointMode {
    Disabled = 0x00, // Does not pass any DMX512-A/RDM traffic on a local RDM Command Port or DMX512-A Data Link
    Input = 0x01, // Receives DMX512-A/RDM data on a local RDM Responder Port or DMX512-A Data Link
    Output = 0x02, // Sends DMX512-A/RDM data on a local Command Port or DMX512-A Data Link
}

impl TryFrom<u8> for EndpointMode {
    type Error = RdmError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Disabled),
            0x01 => Ok(Self::Input),
            0x02 => Ok(Self::Output),
            value => Err(RdmError::InvalidEndpointMode(value)),
        }
    }
}

impl RdmParameterData for EndpointMode {
    fn size_of(&self) -> usize {
        1
    }

    fn encode_rdm_parameter_data(
        &self,
        buf: &mut [u8],
    ) -> Result<usize, rdm_parameter_traits::ParameterCodecError> {
        buf[0] = *self as u8;
        Ok(1)
    }

    fn decode_rdm_parameter_data(
        buf: &[u8],
    ) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
        let endpoint_mode = EndpointMode::try_from(buf[0])
            .map_err(|_| rdm_parameter_traits::ParameterCodecError::MalformedData)?;
        Ok(endpoint_mode)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EndpointIdValue(pub u16);

impl RdmParameterData for EndpointIdValue {
    fn size_of(&self) -> usize {
        2
    }

    fn encode_rdm_parameter_data(
        &self,
        buf: &mut [u8],
    ) -> Result<usize, rdm_parameter_traits::ParameterCodecError> {
        buf[0..2].copy_from_slice(&self.0.to_be_bytes());
        Ok(2)
    }

    fn decode_rdm_parameter_data(
        buf: &[u8],
    ) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
        let endpoint_id_value = EndpointIdValue(u16::from_be_bytes([buf[0], buf[1]]));
        Ok(endpoint_id_value)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum EndpointId {
    Null,
    Device(u16),
    Reserved(u16),
    Broadcast,
}

impl From<u16> for EndpointId {
    fn from(value: u16) -> Self {
        match value {
            0 => EndpointId::Null,
            0xffff => EndpointId::Broadcast,
            value if (0xfa00..=0xfffe).contains(&value) => EndpointId::Reserved(value),
            value => EndpointId::Device(value),
        }
    }
}

impl From<EndpointId> for u16 {
    fn from(value: EndpointId) -> Self {
        match value {
            EndpointId::Broadcast => 0xffff,
            EndpointId::Null => 0,
            EndpointId::Device(value) => value,
            EndpointId::Reserved(value) => value,
        }
    }
}

impl From<EndpointIdValue> for EndpointId {
    fn from(value: EndpointIdValue) -> Self {
        value.0.into()
    }
}

impl From<EndpointId> for EndpointIdValue {
    fn from(value: EndpointId) -> Self {
        Self(value.into())
    }
}

impl RdmParameterData for EndpointId {
    fn size_of(&self) -> usize {
        2
    }

    fn encode_rdm_parameter_data(
        &self,
        buf: &mut [u8],
    ) -> Result<usize, rdm_parameter_traits::ParameterCodecError> {
        let value: u16 = (*self).into();
        buf[0..2].copy_from_slice(&value.to_be_bytes());
        Ok(2)
    }

    fn decode_rdm_parameter_data(
        buf: &[u8],
    ) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
        let value = u16::from_be_bytes([buf[0], buf[1]]);
        Ok(EndpointId::from(value))
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum EndpointType {
    Virtual = 0x00,
    Physical = 0x01,
}

impl TryFrom<u8> for EndpointType {
    type Error = RdmError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Virtual),
            0x01 => Ok(Self::Physical),
            value => Err(RdmError::InvalidEndpointType(value)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct EndpointEntry {
    pub endpoint_id: EndpointIdValue,
    pub endpoint_type: EndpointType,
}

impl RdmParameterData for EndpointEntry {
    fn size_of(&self) -> usize {
        3
    }

    fn encode_rdm_parameter_data(
        &self,
        buf: &mut [u8],
    ) -> Result<usize, rdm_parameter_traits::ParameterCodecError> {
        buf[0..2].copy_from_slice(&self.endpoint_id.0.to_be_bytes());
        buf[2] = self.endpoint_type as u8;
        Ok(3)
    }

    fn decode_rdm_parameter_data(
        buf: &[u8],
    ) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
        let endpoint_id = EndpointIdValue::decode_rdm_parameter_data(&buf[0..2])?;
        let endpoint_type = EndpointType::try_from(buf[2])
            .map_err(|_| rdm_parameter_traits::ParameterCodecError::MalformedData)?;
        Ok(EndpointEntry {
            endpoint_id,
            endpoint_type,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EndpointLabel(String<{ EndpointLabel::MAX_LENGTH }>);

impl_rdm_string!(EndpointLabel, 231);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EndpointTimingDescription(String<{ EndpointTimingDescription::MAX_LENGTH }>);

impl_rdm_string!(EndpointTimingDescription, 32);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BackgroundQueuedStatusPolicyDescription(
    String<{ BackgroundQueuedStatusPolicyDescription::MAX_LENGTH }>,
);

impl_rdm_string!(BackgroundQueuedStatusPolicyDescription, 32);

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetEndpointListResponse {
    pub list_change_number: u32,
    pub endpoint_list: Vec<EndpointEntry, 75>,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetEndpointListChangeResponse {
    pub list_change_number: u32,
}

#[derive(Clone, Debug, PartialEq, RdmGetRequestParameter)]
pub struct GetIdentifyEndpointRequest {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetIdentifyEndpointResponse {
    pub endpoint_id: EndpointIdValue,
    pub identify: bool,
}

#[derive(Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetIdentifyEndpointRequest {
    pub endpoint_id: EndpointId,
    pub identify: bool,
}

#[derive(Clone, Debug, PartialEq, RdmSetResponseParameter)]
pub struct SetIdentifyEndpointResponse {
    pub endpoint_id: EndpointIdValue,
}

#[derive(Clone, Debug, PartialEq, RdmGetRequestParameter)]
pub struct GetEndpointToUniverseRequest {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetEndpointToUniverseResponse {
    pub endpoint_id: EndpointIdValue,
    pub universe: u16,
}

#[derive(Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetEndpointToUniverseRequest {
    pub endpoint_id: EndpointId,
    pub universe: u16,
}

#[derive(Clone, Debug, PartialEq, RdmSetResponseParameter)]
pub struct SetEndpointToUniverseResponse {
    pub endpoint_id: EndpointIdValue,
}

#[derive(Clone, Debug, PartialEq, RdmGetRequestParameter)]
pub struct GetEndpointModeRequest {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetEndpointModeResponse {
    pub endpoint_id: EndpointIdValue,
    pub mode: EndpointMode,
}

#[derive(Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetEndpointModeRequest {
    pub endpoint_id: EndpointId,
    pub mode: EndpointMode,
}

#[derive(Clone, Debug, PartialEq, RdmSetResponseParameter)]
pub struct SetEndpointModeResponse {
    pub endpoint_id: EndpointIdValue,
}

#[derive(Clone, Debug, PartialEq, RdmGetRequestParameter)]
pub struct GetEndpointLabelRequest {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetEndpointLabelResponse {
    pub endpoint_id: EndpointIdValue,
    pub label: EndpointLabel,
}

#[derive(Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetEndpointLabelRequest {
    pub endpoint_id: EndpointId,
    pub label: EndpointLabel,
}

#[derive(Clone, Debug, PartialEq, RdmSetResponseParameter)]
pub struct SetEndpointLabelResponse {
    pub endpoint_id: EndpointIdValue,
}

#[derive(Clone, Debug, PartialEq, RdmGetRequestParameter)]
pub struct GetRdmTrafficEnableRequest {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetRdmTrafficEnableResponse {
    pub endpoint_id: EndpointIdValue,
    pub enable: bool,
}

#[derive(Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetRdmTrafficEnableRequest {
    pub endpoint_id: EndpointId,
    pub enable: bool,
}

#[derive(Clone, Debug, PartialEq, RdmSetResponseParameter)]
pub struct SetRdmTrafficEnableResponse {
    pub endpoint_id: EndpointIdValue,
}

#[derive(Clone, Debug, PartialEq, RdmGetRequestParameter)]
pub struct GetDiscoveryStateRequest {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetDiscoveryStateResponse {
    pub endpoint_id: EndpointIdValue,
    pub device_count: DiscoveryCountStatus,
    pub discovery_state: DiscoveryState,
}

#[derive(Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetDiscoveryStateRequest {
    pub endpoint_id: EndpointId,
    pub discovery_state: DiscoveryState,
}

#[derive(Clone, Debug, PartialEq, RdmSetResponseParameter)]
pub struct SetDiscoveryStateResponse {
    pub endpoint_id: EndpointIdValue,
}

#[derive(Clone, Debug, PartialEq, RdmGetRequestParameter)]
pub struct GetBackgroundDiscoveryRequest {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetBackgroundDiscoveryResponse {
    pub endpoint_id: EndpointIdValue,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetBackgroundDiscoveryRequest {
    pub endpoint_id: EndpointId,
    pub enable: bool,
}

#[derive(Clone, Debug, PartialEq, RdmSetResponseParameter)]
pub struct SetBackgroundDiscoveryResponse {
    pub endpoint_id: EndpointIdValue,
}

#[derive(Clone, Debug, PartialEq, RdmGetRequestParameter)]
pub struct GetEndpointTimingRequest {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetEndpointTimingResponse {
    pub endpoint_id: EndpointIdValue,
    pub current_setting_id: u8,
    pub setting_count: u8,
}

#[derive(Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetEndpointTimingRequest {
    pub endpoint_id: EndpointId,
    pub setting_id: u8,
}

#[derive(Clone, Debug, PartialEq, RdmSetResponseParameter)]
pub struct SetEndpointTimingResponse {
    pub endpoint_id: EndpointIdValue,
}

#[derive(Clone, Debug, PartialEq, RdmGetRequestParameter)]
pub struct GetEndpointTimingDescriptionRequest {
    pub setting_id: u8,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetEndpointTimingDescriptionResponse {
    pub setting_id: u8,
    pub description: EndpointTimingDescription,
}

#[derive(Clone, Debug, PartialEq, RdmGetRequestParameter)]
pub struct GetEndpointRespondersRequest {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetEndpointRespondersResponse {
    pub endpoint_id: EndpointIdValue,
    pub list_change_number: u32,
    pub responders: Vec<DeviceUID, 37>,
}

#[derive(Clone, Debug, PartialEq, RdmGetRequestParameter)]
pub struct GetEndpointResponderListChangeRequest {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetEndpointResponderListChangeResponse {
    pub endpoint_id: EndpointIdValue,
    pub list_change_number: u32,
}

#[derive(Clone, Debug, PartialEq, RdmGetRequestParameter)]
pub struct GetBindingControlFieldsRequest {
    pub endpoint_id: EndpointId,
    pub uid: DeviceUID,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetBindingControlFieldsResponse {
    pub endpoint_id: EndpointIdValue,
    pub uid: DeviceUID,
    pub control_field: u16,
    pub binding_uid: DeviceUID,
}

#[derive(Clone, Debug, PartialEq, RdmGetRequestParameter)]
pub struct GetBackgroundQueuedStatusPolicyRequest {
    pub policy_id: u8,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetBackgroundQueuedStatusPolicyResponse {
    pub current_policy_id: u8,
    pub policy_count: u8,
}

#[derive(Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetBackgroundQueuedStatusPolicyRequest {
    pub policy_id: u8,
}

#[derive(Clone, Debug, PartialEq, RdmGetRequestParameter)]
pub struct GetBackgroundQueuedStatusPolicyDescriptionRequest {
    pub policy_id: u8,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetBackgroundQueuedStatusPolicyDescriptionResponse {
    pub policy_id: u8,
    pub description: BackgroundQueuedStatusPolicyDescription,
}
