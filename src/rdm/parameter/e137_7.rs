use super::RdmError;
use crate::rdm::{DeviceUID, utils::RdmTruncateNullStr};
use core::{ops::Deref, str::FromStr};
use heapless::{String, Vec};
use rdm_parameter_derive::{RdmGetResponseParameter, RdmSetResponseParameter};
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

pub const ENDPOINT_LABEL_MAX_LENGTH: usize = 231;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EndpointLabel(String<ENDPOINT_LABEL_MAX_LENGTH>);

impl RdmTruncateNullStr for EndpointLabel {
    type Error = RdmError;
}

impl Deref for EndpointLabel {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl FromStr for EndpointLabel {
    type Err = RdmError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > ENDPOINT_LABEL_MAX_LENGTH {
            return Err(RdmError::InvalidStringLength(
                s.len(),
                ENDPOINT_LABEL_MAX_LENGTH,
            ));
        }
        Ok(Self(
            String::<{ ENDPOINT_LABEL_MAX_LENGTH }>::from_str(s).unwrap(),
        ))
    }
}

impl RdmParameterData for EndpointLabel {
    fn size_of(&self) -> usize {
        self.0.len()
    }

    fn encode_rdm_parameter_data(
        &self,
        buf: &mut [u8],
    ) -> Result<usize, rdm_parameter_traits::ParameterCodecError> {
        let len = self.0.len();
        buf[..len].copy_from_slice(self.0.as_bytes());
        Ok(len)
    }

    fn decode_rdm_parameter_data(
        buf: &[u8],
    ) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
        let s = core::str::from_utf8(buf)
            .map_err(|_| rdm_parameter_traits::ParameterCodecError::MalformedData)?;
        let endpoint_label = EndpointLabel::from_str(s)
            .map_err(|_| rdm_parameter_traits::ParameterCodecError::MalformedData)?;
        Ok(endpoint_label)
    }
}

pub const ENDPOINT_TIMING_DESCRIPTION_MAX_LENGTH: usize = 32;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EndpointTimingDescription(String<ENDPOINT_TIMING_DESCRIPTION_MAX_LENGTH>);

impl RdmTruncateNullStr for EndpointTimingDescription {
    type Error = RdmError;
}

impl Deref for EndpointTimingDescription {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl FromStr for EndpointTimingDescription {
    type Err = RdmError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > ENDPOINT_TIMING_DESCRIPTION_MAX_LENGTH {
            return Err(RdmError::InvalidStringLength(
                s.len(),
                ENDPOINT_TIMING_DESCRIPTION_MAX_LENGTH,
            ));
        }
        Ok(Self(
            String::<{ ENDPOINT_TIMING_DESCRIPTION_MAX_LENGTH }>::from_str(s).unwrap(),
        ))
    }
}

impl RdmParameterData for EndpointTimingDescription {
    fn size_of(&self) -> usize {
        self.0.len()
    }

    fn encode_rdm_parameter_data(
        &self,
        buf: &mut [u8],
    ) -> Result<usize, rdm_parameter_traits::ParameterCodecError> {
        let len = self.0.len();
        buf[..len].copy_from_slice(self.0.as_bytes());
        Ok(len)
    }

    fn decode_rdm_parameter_data(
        buf: &[u8],
    ) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
        let s = core::str::from_utf8(buf)
            .map_err(|_| rdm_parameter_traits::ParameterCodecError::MalformedData)?;
        let endpoint_timing_description = EndpointTimingDescription::from_str(s)
            .map_err(|_| rdm_parameter_traits::ParameterCodecError::MalformedData)?;
        Ok(endpoint_timing_description)
    }
}

pub const BACKGROUND_QUEUED_STATUS_POLICY_DESCRIPTION_MAX_LENGTH: usize = 32;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BackgroundQueuedStatusPolicyDescription(
    String<BACKGROUND_QUEUED_STATUS_POLICY_DESCRIPTION_MAX_LENGTH>,
);

impl RdmTruncateNullStr for BackgroundQueuedStatusPolicyDescription {
    type Error = RdmError;
}

impl Deref for BackgroundQueuedStatusPolicyDescription {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl FromStr for BackgroundQueuedStatusPolicyDescription {
    type Err = RdmError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > BACKGROUND_QUEUED_STATUS_POLICY_DESCRIPTION_MAX_LENGTH {
            return Err(RdmError::InvalidStringLength(
                s.len(),
                BACKGROUND_QUEUED_STATUS_POLICY_DESCRIPTION_MAX_LENGTH,
            ));
        }
        Ok(Self(
            String::<{ BACKGROUND_QUEUED_STATUS_POLICY_DESCRIPTION_MAX_LENGTH }>::from_str(s)
                .unwrap(),
        ))
    }
}

impl RdmParameterData for BackgroundQueuedStatusPolicyDescription {
    fn size_of(&self) -> usize {
        self.0.len()
    }

    fn encode_rdm_parameter_data(
        &self,
        buf: &mut [u8],
    ) -> Result<usize, rdm_parameter_traits::ParameterCodecError> {
        let len = self.0.len();
        buf[..len].copy_from_slice(self.0.as_bytes());
        Ok(len)
    }

    fn decode_rdm_parameter_data(
        buf: &[u8],
    ) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
        let s = core::str::from_utf8(buf)
            .map_err(|_| rdm_parameter_traits::ParameterCodecError::MalformedData)?;
        let description = BackgroundQueuedStatusPolicyDescription::from_str(s)
            .map_err(|_| rdm_parameter_traits::ParameterCodecError::MalformedData)?;
        Ok(description)
    }
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetEndpointList {
    pub list_change_number: u32,
    pub endpoint_list: Vec<EndpointEntry, 75>,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetEndpointListChange {
    pub list_change_number: u32,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetIdentifyEndpoint {
    pub endpoint_id: EndpointIdValue,
    pub identify: bool,
}

#[derive(Clone, Debug, PartialEq, RdmSetResponseParameter)]
pub struct SetIdentifyEndpoint {
    pub endpoint_id: EndpointIdValue,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetEndpointToUniverse {
    pub endpoint_id: EndpointIdValue,
    pub universe: u16,
}

#[derive(Clone, Debug, PartialEq, RdmSetResponseParameter)]
pub struct SetEndpointToUniverse {
    pub endpoint_id: EndpointIdValue,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetEndpointMode {
    pub endpoint_id: EndpointIdValue,
    pub mode: EndpointMode,
}

#[derive(Clone, Debug, PartialEq, RdmSetResponseParameter)]
pub struct SetEndpointMode {
    pub endpoint_id: EndpointIdValue,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetEndpointLabel {
    pub endpoint_id: EndpointIdValue,
    pub label: EndpointLabel,
}

#[derive(Clone, Debug, PartialEq, RdmSetResponseParameter)]
pub struct SetEndpointLabel {
    pub endpoint_id: EndpointIdValue,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetRdmTrafficEnable {
    pub endpoint_id: EndpointIdValue,
    pub enable: bool,
}

#[derive(Clone, Debug, PartialEq, RdmSetResponseParameter)]
pub struct SetRdmTrafficEnable {
    pub endpoint_id: EndpointIdValue,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetDiscoveryState {
    pub endpoint_id: EndpointIdValue,
    pub device_count: DiscoveryCountStatus,
    pub discovery_state: DiscoveryState,
}

#[derive(Clone, Debug, PartialEq, RdmSetResponseParameter)]
pub struct SetDiscoveryState {
    pub endpoint_id: EndpointIdValue,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetBackgroundDiscovery {
    pub endpoint_id: EndpointIdValue,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, RdmSetResponseParameter)]
pub struct SetBackgroundDiscovery {
    pub endpoint_id: EndpointIdValue,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetEndpointTiming {
    pub endpoint_id: EndpointIdValue,
    pub current_setting_id: u8,
    pub setting_count: u8,
}

#[derive(Clone, Debug, PartialEq, RdmSetResponseParameter)]
pub struct SetEndpointTiming {
    pub endpoint_id: EndpointIdValue,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetEndpointTimingDescription {
    pub setting_id: u8,
    pub description: EndpointTimingDescription,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetEndpointResponders {
    pub endpoint_id: EndpointIdValue,
    pub list_change_number: u32,
    pub responders: Vec<DeviceUID, 37>,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetEndpointResponderListChange {
    pub endpoint_id: EndpointIdValue,
    pub list_change_number: u32,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetBindingControlFields {
    pub endpoint_id: EndpointIdValue,
    pub uid: DeviceUID,
    pub control_field: u16,
    pub binding_uid: DeviceUID,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetBackgroundQueuedStatusPolicy {
    pub current_policy_id: u8,
    pub policy_count: u8,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetBackgroundQueuedStatusPolicyDescription {
    pub policy_id: u8,
    pub description: BackgroundQueuedStatusPolicyDescription,
}
