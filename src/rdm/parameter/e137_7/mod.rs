pub mod request;
pub mod response;

use crate::impl_rdm_string;
use heapless::String;
use rdm_core::{
    error::{ParameterCodecError, RdmError},
    parameter_traits::RdmParameterData,
};

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

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        buf[0] = (*self).into();
        Ok(1)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        let discovery_state =
            DiscoveryState::try_from(buf[0]).map_err(|_| ParameterCodecError::MalformedData)?;
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

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        let value: u16 = (*self).into();
        buf[0..2].copy_from_slice(&value.to_be_bytes());
        Ok(2)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
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

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        buf[0] = *self as u8;
        Ok(1)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        let endpoint_mode =
            EndpointMode::try_from(buf[0]).map_err(|_| ParameterCodecError::MalformedData)?;
        Ok(endpoint_mode)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EndpointIdValue(pub u16);

impl RdmParameterData for EndpointIdValue {
    fn size_of(&self) -> usize {
        2
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        buf[0..2].copy_from_slice(&self.0.to_be_bytes());
        Ok(2)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
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

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        let value: u16 = (*self).into();
        buf[0..2].copy_from_slice(&value.to_be_bytes());
        Ok(2)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
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

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        buf[0..2].copy_from_slice(&self.endpoint_id.0.to_be_bytes());
        buf[2] = self.endpoint_type as u8;
        Ok(3)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        let endpoint_id = EndpointIdValue::decode_rdm_parameter_data(&buf[0..2])?;
        let endpoint_type =
            EndpointType::try_from(buf[2]).map_err(|_| ParameterCodecError::MalformedData)?;
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
