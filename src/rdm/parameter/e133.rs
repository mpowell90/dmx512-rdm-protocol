use super::{super::utils::trim_trailing_nulls, RdmError};
use core::ops::Deref;

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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SearchDomain<'a>(&'a str);

impl<'a> SearchDomain<'a> {
    pub const MAX_LENGTH: usize = 231;

    pub fn new(search_domain: &'a str) -> Result<Self, RdmError> {
        if search_domain.len() > Self::MAX_LENGTH {
            return Err(RdmError::InvalidStringLength(
                search_domain.len(),
                Self::MAX_LENGTH,
            ));
        }
        Ok(Self(search_domain))
    }

    pub fn as_str(&self) -> &str {
        self.0
    }
}

impl Deref for SearchDomain<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> From<SearchDomain<'a>> for &'a [u8] {
    fn from(value: SearchDomain<'a>) -> Self {
        value.0.as_bytes()
    }
}

impl<'a> TryFrom<&'a [u8]> for SearchDomain<'a> {
    type Error = RdmError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let search_domain = core::str::from_utf8(value).map_err(RdmError::from)?;
        if search_domain.len() > Self::MAX_LENGTH {
            return Err(RdmError::InvalidStringLength(
                search_domain.len(),
                Self::MAX_LENGTH,
            ));
        }
        Ok(Self(search_domain))
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ScopeString<'a>(&'a str);

impl<'a> ScopeString<'a> {
    pub const MAX_LENGTH: usize = 63;

    pub fn new(scope_string: &'a str) -> Result<Self, RdmError> {
        if scope_string.len() > Self::MAX_LENGTH {
            return Err(RdmError::InvalidStringLength(
                scope_string.len(),
                Self::MAX_LENGTH,
            ));
        }
        Ok(Self(scope_string))
    }

    pub fn as_str(&self) -> &str {
        self.0
    }

    pub fn is_valid(&self) -> bool {
        !self.0.is_empty()
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, RdmError> {
        if buf.len() < Self::MAX_LENGTH {
            return Err(RdmError::InvalidBufferLength(buf.len(), Self::MAX_LENGTH));
        }
        let len = self.0.len();

        buf[0..len].copy_from_slice(self.0.as_bytes());

        if len < Self::MAX_LENGTH {
            buf[len..Self::MAX_LENGTH].fill(0);
        }

        Ok(Self::MAX_LENGTH)
    }

    pub fn decode(bytes: &'a [u8]) -> Result<Self, RdmError> {
        if bytes.len() > Self::MAX_LENGTH {
            return Err(RdmError::InvalidStringLength(bytes.len(), Self::MAX_LENGTH));
        }

        let scope_string =
            core::str::from_utf8(trim_trailing_nulls(bytes)).map_err(RdmError::from)?;

        Ok(Self(scope_string))
    }
}

impl Deref for ScopeString<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> TryFrom<&'a str> for ScopeString<'a> {
    type Error = RdmError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl<'a> From<ScopeString<'a>> for &'a str {
    fn from(value: ScopeString<'a>) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_string_new_valid() {
        let scope = "test.scope";
        let scope_string = ScopeString::new(scope).unwrap();
        assert_eq!(scope_string.as_str(), scope);
    }

    #[test]
    fn test_scope_string_new_too_long() {
        let scope = "a".repeat(ScopeString::MAX_LENGTH + 1);
        let result = ScopeString::new(&scope);
        assert_eq!(
            result,
            Err(RdmError::InvalidStringLength(
                scope.len(),
                ScopeString::MAX_LENGTH
            ))
        );
    }

    #[test]
    fn test_scope_string_encode_success() {
        let scope = "test.scope";
        let scope_string = ScopeString::new(scope).unwrap();
        let mut buffer = [0u8; ScopeString::MAX_LENGTH];

        let written = scope_string.encode(&mut buffer).unwrap();

        assert_eq!(written, ScopeString::MAX_LENGTH);
        assert_eq!(&buffer[0..scope.len()], scope.as_bytes());
        assert!(buffer[scope.len()..].iter().all(|&b| b == 0));
    }

    #[test]
    fn test_scope_string_encode_buffer_too_small() {
        let scope = "test.scope";
        let scope_string = ScopeString::new(scope).unwrap();
        let mut buffer = [0u8; ScopeString::MAX_LENGTH - 1];

        let result = scope_string.encode(&mut buffer);
        assert_eq!(
            result,
            Err(RdmError::InvalidBufferLength(
                buffer.len(),
                ScopeString::MAX_LENGTH
            ))
        );
    }

    #[test]
    fn test_scope_string_decode_success() {
        let scope = "test.scope";
        let mut buffer = [0u8; ScopeString::MAX_LENGTH];
        buffer[0..scope.len()].copy_from_slice(scope.as_bytes());

        let scope_string = ScopeString::decode(&buffer).unwrap();
        assert_eq!(scope_string.as_str(), scope);
    }

    #[test]
    fn test_scope_string_decode_with_nulls() {
        let scope = "test.scope";
        let mut buffer = [0u8; ScopeString::MAX_LENGTH];
        buffer[0..scope.len()].copy_from_slice(scope.as_bytes());
        buffer[scope.len()..].fill(0);

        let scope_string = ScopeString::decode(&buffer).unwrap();
        assert_eq!(scope_string.as_str(), scope);
    }

    #[test]
    fn test_scope_string_decode_too_long() {
        let buffer = [b'a'; ScopeString::MAX_LENGTH + 1];
        let result = ScopeString::decode(&buffer);
        assert_eq!(
            result,
            Err(RdmError::InvalidStringLength(
                buffer.len(),
                ScopeString::MAX_LENGTH
            ))
        );
    }

    #[test]
    fn test_scope_string_decode_invalid_utf8() {
        let mut buffer = [0u8; ScopeString::MAX_LENGTH];
        buffer[0] = 0xFF; // Invalid UTF-8 byte

        let result = ScopeString::decode(&buffer);
        assert!(result.is_err());
    }
}
