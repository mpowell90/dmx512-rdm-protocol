use super::RdmError;
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
    pub fn new(search_domain: &'a str) -> Result<Self, RdmError> {
        if search_domain.len() > 231 {
            return Err(RdmError::InvalidStringLength(
                search_domain.len() as u8,
                231,
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
        if search_domain.len() > 231 {
            return Err(RdmError::InvalidStringLength(
                search_domain.len() as u8,
                231,
            ));
        }
        Ok(Self(search_domain))
    }
}
