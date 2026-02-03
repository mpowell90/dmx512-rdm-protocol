pub mod request;
pub mod response;

use crate::impl_rdm_string;
use core::time::Duration;
use heapless::String;
use rdm_core::{
    error::{ParameterCodecError, RdmError},
    parameter_traits::RdmParameterData,
};

pub const E133_TCP_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15);
pub const E133_HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(45);

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StaticConfigType {
    NoStaticConfig = 0x00,
    StaticConfigIpv4 = 0x01,
    StaticConfigIpv6 = 0x02,
}

impl TryFrom<u8> for StaticConfigType {
    type Error = RdmError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::NoStaticConfig),
            0x01 => Ok(Self::StaticConfigIpv4),
            0x02 => Ok(Self::StaticConfigIpv6),
            value => Err(RdmError::InvalidStaticConfigType(value)),
        }
    }
}

impl RdmParameterData for StaticConfigType {
    fn size_of(&self) -> usize {
        1
    }

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        buf[0] = *self as u8;
        Ok(1)
    }

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        let static_config_type =
            StaticConfigType::try_from(buf[0]).map_err(|_| ParameterCodecError::MalformedData)?;
        Ok(static_config_type)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BrokerState {
    Disabled = 0x00,
    Active = 0x01,
    Standby = 0x02,
}

impl TryFrom<u8> for BrokerState {
    type Error = RdmError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Disabled),
            0x01 => Ok(Self::Active),
            0x02 => Ok(Self::Standby),
            value => Err(RdmError::InvalidBrokerState(value)),
        }
    }
}

impl RdmParameterData for BrokerState {
    fn size_of(&self) -> usize {
        1
    }

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        buf[0] = *self as u8;
        Ok(1)
    }

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        let broker_state =
            BrokerState::try_from(buf[0]).map_err(|_| ParameterCodecError::MalformedData)?;
        Ok(broker_state)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Hash)]
pub struct SearchDomain(String<{ SearchDomain::MAX_LENGTH }>);

impl_rdm_string!(SearchDomain, 231);

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Scope(String<{ Scope::MAX_LENGTH }>);

impl_rdm_string!(Scope, 63);
