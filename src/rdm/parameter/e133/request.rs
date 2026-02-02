use crate::rdm::parameter::{
    e133::{BrokerState, Scope, SearchDomain, StaticConfigType},
    e137_2::{Ipv4Address, Ipv6Address},
};
use rdm_core::{CommandClass, ParameterId, parameter_traits::RdmParameterData};
use rdm_derive::rdm_request_parameter;

#[derive(Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::ComponentScope, command_class = CommandClass::Get)]
pub struct GetComponentScopeRequest {
    pub scope_slot: u16,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::ComponentScope, command_class = CommandClass::Set)]
pub struct SetComponentScopeRequest {
    pub scope_slot: u16,
    pub scope_string: Scope,
    pub static_config_type: StaticConfigType,
    pub static_broker_ipv4_address: Ipv4Address,
    pub static_broker_ipv6_address: Ipv6Address,
    pub static_broker_port: u16,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::SearchDomain, command_class = CommandClass::Set)]
pub struct SetSearchDomainRequest {
    pub search_domain: SearchDomain,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::TcpCommsStatus, command_class = CommandClass::Set)]
pub struct SetTcpCommsStatusRequest {
    pub scope: Scope,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::BrokerStatus, command_class = CommandClass::Set)]
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
