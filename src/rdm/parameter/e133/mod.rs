pub mod request;
pub mod response;

use crate::{
    impl_rdm_string,
    rdm::core::{
        error::{ParameterDataError, RdmError},
        parameter_traits::RdmParameterData,
    },
};
use core::time::Duration;
use heapless::String;

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

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterDataError> {
        buf[0] = *self as u8;
        Ok(1)
    }

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterDataError> {
        let static_config_type =
            StaticConfigType::try_from(buf[0]).map_err(|_| ParameterDataError::MalformedData)?;
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

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterDataError> {
        buf[0] = *self as u8;
        Ok(1)
    }

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterDataError> {
        let broker_state =
            BrokerState::try_from(buf[0]).map_err(|_| ParameterDataError::MalformedData)?;
        Ok(broker_state)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Hash)]
pub struct SearchDomain(String<{ SearchDomain::MAX_LENGTH }>);

impl_rdm_string!(SearchDomain, 231);

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Scope(String<{ Scope::MAX_LENGTH }>);

impl_rdm_string!(Scope, 63);

#[cfg(test)]
mod tests {
    use super::*;
    use core::str::FromStr;

    #[test]
    fn should_encode_decode_static_config_type() {
        let mut buf = [0u8; 1];
        let typ = StaticConfigType::StaticConfigIpv4;
        let n = typ.encode_parameter_data(&mut buf).unwrap();
        assert_eq!(n, 1);
        assert_eq!(buf[0], 0x01);
        let decoded = StaticConfigType::decode_parameter_data(&buf).unwrap();
        assert_eq!(decoded, typ);
    }

    #[test]
    fn should_encode_decode_broker_state() {
        let mut buf = [0u8; 1];
        let state = BrokerState::Active;
        let n = state.encode_parameter_data(&mut buf).unwrap();
        assert_eq!(n, 1);
        assert_eq!(buf[0], 0x01);
        let decoded = BrokerState::decode_parameter_data(&buf).unwrap();
        assert_eq!(decoded, state);
    }

    #[test]
    fn should_encode_decode_search_domain() {
        let sd = SearchDomain::from_str("example.com").unwrap();
        let mut buf = [0u8; SearchDomain::MAX_LENGTH];
        let n = sd.encode_parameter_data(&mut buf).unwrap();
        assert_eq!(n, sd.len());
        assert_eq!(&buf[..n], sd.as_bytes());
        let decoded = SearchDomain::decode_parameter_data(&buf).unwrap();
        assert_eq!(decoded, sd);
    }

    #[test]
    fn should_encode_decode_scope() {
        let s = Scope::from_str("my_scope").unwrap();
        let mut buf = [0u8; Scope::MAX_LENGTH];
        let n = s.encode_parameter_data(&mut buf).unwrap();
        assert_eq!(n, s.len());
        assert_eq!(&buf[..n], s.as_bytes());
        let decoded = Scope::decode_parameter_data(&buf).unwrap();
        assert_eq!(decoded, s);
    }
}
