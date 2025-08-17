use super::RdmError;

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
