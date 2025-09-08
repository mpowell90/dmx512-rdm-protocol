use super::RdmError;
use crate::rdm::utils::{RdmPadNullStr};
use core::{ops::Deref, str::FromStr, time::Duration};
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

pub const SEARCH_DOMAIN_MAX_LENGTH: usize = 231;

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct SearchDomain(String<SEARCH_DOMAIN_MAX_LENGTH>);

impl SearchDomain {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self(String::<SEARCH_DOMAIN_MAX_LENGTH>::new())
    }
}

impl RdmPadNullStr for SearchDomain {
    const MAX_LENGTH: usize = SEARCH_DOMAIN_MAX_LENGTH;
    type Error = RdmError;
}

impl Deref for SearchDomain {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl FromStr for SearchDomain {
    type Err = RdmError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > Self::MAX_LENGTH {
            return Err(RdmError::InvalidStringLength(s.len(), Self::MAX_LENGTH));
        }
        Ok(Self(
            String::<SEARCH_DOMAIN_MAX_LENGTH>::from_str(s).unwrap(),
        ))
    }
}

pub const SCOPE_MAX_LENGTH: usize = 63;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Scope(String<SCOPE_MAX_LENGTH>);

impl Scope {
    #[allow(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self(String::<SCOPE_MAX_LENGTH>::new())
    }
}

impl RdmPadNullStr for Scope {
    const MAX_LENGTH: usize = SCOPE_MAX_LENGTH;
    type Error = RdmError;
}

impl Deref for Scope {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl FromStr for Scope {
    type Err = RdmError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() > SCOPE_MAX_LENGTH {
            return Err(RdmError::InvalidStringLength(s.len(), SCOPE_MAX_LENGTH));
        }
        Ok(Self(String::<{ SCOPE_MAX_LENGTH }>::from_str(s).unwrap()))
    }
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
