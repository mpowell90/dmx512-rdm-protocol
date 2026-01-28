use super::RdmError;
use crate::{
    impl_rdm_string,
    rdm::parameter::e137_2::{Ipv4Address, Ipv6Address},
};
use core::time::Duration;
use heapless::String;
use rdm_parameter_derive::{
    RdmGetRequestParameter, RdmGetResponseParameter, RdmSetRequestParameter,
};
use rdm_parameter_traits::{ParameterCodecError, RdmParameterData};

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

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        buf[0] = *self as u8;
        Ok(1)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
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

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        buf[0] = *self as u8;
        Ok(1)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        let broker_state =
            BrokerState::try_from(buf[0]).map_err(|_| ParameterCodecError::MalformedData)?;
        Ok(broker_state)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct SearchDomain(String<{ SearchDomain::MAX_LENGTH }>);

impl_rdm_string!(SearchDomain, 231);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Scope(String<{ Scope::MAX_LENGTH }>);

impl_rdm_string!(Scope, 63);

#[derive(Clone, Debug, PartialEq, RdmGetRequestParameter)]
pub struct GetComponentScopeRequest {
    pub scope_slot: u16,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetComponentScopeResponse {
    pub scope_slot: u16,
    pub scope_string: Scope,
    pub static_config_type: StaticConfigType,
    pub static_ipv4_address: Ipv4Address,
    pub static_ipv6_address: Ipv6Address,
    pub static_port: u16,
}

#[derive(Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetComponentScopeRequest {
    pub scope_slot: u16,
    pub scope_string: Scope,
    pub static_config_type: StaticConfigType,
    pub static_broker_ipv4_address: Ipv4Address,
    pub static_broker_ipv6_address: Ipv6Address,
    pub static_broker_port: u16,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetSearchDomainResponse {
    pub search_domain: SearchDomain,
}

#[derive(Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetSearchDomainRequest {
    pub search_domain: SearchDomain,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetTcpCommsStatusResponse {
    pub scope_string: Scope,
    pub broker_ipv4_address: Ipv4Address,
    pub broker_ipv6_address: Ipv6Address,
    pub broker_port: u16,
    pub unhealthy_tcp_events: u16,
}

#[derive(Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetTcpCommsStatusRequest {
    pub scope: Scope,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetBrokerStatusResponse {
    pub is_allowing_set_commands: bool,
    pub broker_state: BrokerState,
}

#[derive(Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetBrokerStatusRequest {
    pub broker_state: BrokerState,
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_scope_string_new_valid() {
//         let scope = "test.scope";
//         let scope_string = Scope::new(scope).unwrap();
//         assert_eq!(scope_string.as_str(), scope);
//     }

//     #[test]
//     fn test_scope_string_new_too_long() {
//         let scope = "a".repeat(Scope::MAX_LENGTH + 1);
//         let result = Scope::new(&scope);
//         assert_eq!(
//             result,
//             Err(RdmError::InvalidStringLength(
//                 scope.len(),
//                 Scope::MAX_LENGTH
//             ))
//         );
//     }

//     #[test]
//     fn test_scope_string_encode_success() {
//         let scope = "test.scope";
//         let scope_string = Scope::new(scope).unwrap();
//         let mut buffer = [0u8; Scope::MAX_LENGTH];

//         let written = scope_string.encode(&mut buffer).unwrap();

//         assert_eq!(written, Scope::MAX_LENGTH);
//         assert_eq!(&buffer[0..scope.len()], scope.as_bytes());
//         assert!(buffer[scope.len()..].iter().all(|&b| b == 0));
//     }

//     #[test]
//     fn test_scope_string_encode_buffer_too_small() {
//         let scope = "test.scope";
//         let scope_string = Scope::new(scope).unwrap();
//         let mut buffer = [0u8; Scope::MAX_LENGTH - 1];

//         let result = scope_string.encode(&mut buffer);
//         assert_eq!(
//             result,
//             Err(RdmError::InvalidBufferLength(
//                 buffer.len(),
//                 Scope::MAX_LENGTH
//             ))
//         );
//     }

//     #[test]
//     fn test_scope_string_decode_success() {
//         let scope = "test.scope";
//         let mut buffer = [0u8; Scope::MAX_LENGTH];
//         buffer[0..scope.len()].copy_from_slice(scope.as_bytes());

//         let scope_string = Scope::decode(&buffer).unwrap();
//         assert_eq!(scope_string.as_str(), scope);
//     }

//     #[test]
//     fn test_scope_string_decode_with_nulls() {
//         let scope = "test.scope";
//         let mut buffer = [0u8; Scope::MAX_LENGTH];
//         buffer[0..scope.len()].copy_from_slice(scope.as_bytes());
//         buffer[scope.len()..].fill(0);

//         let scope_string = Scope::decode(&buffer).unwrap();
//         assert_eq!(scope_string.as_str(), scope);
//     }

//     #[test]
//     fn test_scope_string_decode_too_long() {
//         let buffer = [b'a'; Scope::MAX_LENGTH + 1];
//         let result = Scope::decode(&buffer);
//         assert_eq!(
//             result,
//             Err(RdmError::InvalidStringLength(
//                 buffer.len(),
//                 Scope::MAX_LENGTH
//             ))
//         );
//     }

//     #[test]
//     fn test_scope_string_decode_invalid_utf8() {
//         let mut buffer = [0u8; Scope::MAX_LENGTH];
//         buffer[0] = 0xFF; // Invalid UTF-8 byte

//         let result = Scope::decode(&buffer);
//         assert!(result.is_err());
//     }
// }
