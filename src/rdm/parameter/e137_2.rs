use heapless::String;

use super::{super::utils::truncate_at_null, RdmError};
use core::{
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DhcpMode {
    Inactive = 0x00,
    Active = 0x01,
    Unknown = 0x02,
}

impl TryFrom<u8> for DhcpMode {
    type Error = RdmError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Inactive),
            0x01 => Ok(Self::Active),
            0x02 => Ok(Self::Unknown),
            value => Err(RdmError::InvalidDhcpMode(value)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Ipv4Address {
    Unconfigured,
    Configured(Ipv4Addr),
}

impl From<Ipv4Addr> for Ipv4Address {
    fn from(value: Ipv4Addr) -> Self {
        Self::Configured(value)
    }
}

impl From<u32> for Ipv4Address {
    fn from(value: u32) -> Self {
        if value == 0 {
            Self::Unconfigured
        } else {
            Self::Configured(Ipv4Addr::from(value))
        }
    }
}

impl From<[u8; 4]> for Ipv4Address {
    fn from(value: [u8; 4]) -> Self {
        if value == [0, 0, 0, 0] {
            Self::Unconfigured
        } else {
            Self::Configured(Ipv4Addr::from(value))
        }
    }
}

impl From<Ipv4Address> for [u8; 4] {
    fn from(value: Ipv4Address) -> [u8; 4] {
        match value {
            Ipv4Address::Unconfigured => [0, 0, 0, 0],
            Ipv4Address::Configured(ip) => ip.octets(),
        }
    }
}

impl From<Ipv4Address> for u32 {
    fn from(value: Ipv4Address) -> u32 {
        match value {
            Ipv4Address::Unconfigured => 0,
            Ipv4Address::Configured(ip) => ip.to_bits(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Ipv6Address {
    Unconfigured,
    Configured(Ipv6Addr),
}

impl From<Ipv6Addr> for Ipv6Address {
    fn from(value: Ipv6Addr) -> Self {
        Self::Configured(value)
    }
}

impl From<u128> for Ipv6Address {
    fn from(value: u128) -> Self {
        if value == 0 {
            Self::Unconfigured
        } else {
            Self::Configured(Ipv6Addr::from(value))
        }
    }
}

impl From<[u8; 16]> for Ipv6Address {
    fn from(value: [u8; 16]) -> Self {
        if value == [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0] {
            Self::Unconfigured
        } else {
            Self::Configured(Ipv6Addr::from(value))
        }
    }
}

impl From<Ipv6Address> for [u8; 16] {
    fn from(value: Ipv6Address) -> [u8; 16] {
        match value {
            Ipv6Address::Unconfigured => [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            Ipv6Address::Configured(ip) => ip.octets(),
        }
    }
}

impl From<Ipv6Address> for u128 {
    fn from(value: Ipv6Address) -> u128 {
        match value {
            Ipv6Address::Unconfigured => 0,
            Ipv6Address::Configured(ip) => ip.to_bits(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Ipv4Route {
    NoDefault,
    Configured(Ipv4Addr),
}

impl From<Ipv4Addr> for Ipv4Route {
    fn from(value: Ipv4Addr) -> Self {
        Self::Configured(value)
    }
}

impl From<u32> for Ipv4Route {
    fn from(value: u32) -> Self {
        if value == 0 {
            Self::NoDefault
        } else {
            Self::Configured(Ipv4Addr::from(value))
        }
    }
}

impl From<[u8; 4]> for Ipv4Route {
    fn from(value: [u8; 4]) -> Self {
        if value == [0, 0, 0, 0] {
            Self::NoDefault
        } else {
            Self::Configured(Ipv4Addr::from(value))
        }
    }
}

impl From<Ipv4Route> for [u8; 4] {
    fn from(value: Ipv4Route) -> [u8; 4] {
        match value {
            Ipv4Route::NoDefault => [0, 0, 0, 0],
            Ipv4Route::Configured(ip) => ip.octets(),
        }
    }
}

impl From<Ipv4Route> for u32 {
    fn from(value: Ipv4Route) -> u32 {
        match value {
            Ipv4Route::NoDefault => 0,
            Ipv4Route::Configured(ip) => ip.to_bits(),
        }
    }
}

// Hardware types are defined by the IANA:
// https://www.iana.org/assignments/arp-parameters/arp-parameters.xhtml
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HardwareType {
    Reserved(u16),
    Ethernet,
    ExperimentEthernet,
    AmateurRadioAx25,
    ProteonPronetTokenRing,
    Chaos,
    Ieee802Networks,
    Arcnet,
    Hyperchannel,
    Lanstar,
    AutonetShortAddress,
    LocalTalk,
    LocalNet,
    UltraLink,
    SMDS,
    FrameRelay,
    ATM,
    HDLC,
    FibreChannel,
    ATMLogical,
    SerialLine,
    ATMPhysical,
    MilStd188220,
    Metricom,
    IEEE1394,
    MAPOS,
    Twinaxial,
    EUI64,
    HIPARP,
    IPAndARPOverISO,
    ARPSec,
    IPsecTunnel,
    InfiniBand,
    TIA102,
    Wiegand,
    PureIP,
    HwExp1,
    Hf1,
    UnifiedBus,
    HwExp2,
    AEthernet,
    Unknown(u16),
}

impl From<u16> for HardwareType {
    fn from(value: u16) -> Self {
        match value {
            val @ (0 | 65535) => Self::Reserved(val),
            1 => Self::Ethernet,
            2 => Self::ExperimentEthernet,
            3 => Self::AmateurRadioAx25,
            4 => Self::ProteonPronetTokenRing,
            5 => Self::Chaos,
            6 => Self::Ieee802Networks,
            7 => Self::Arcnet,
            8 => Self::Hyperchannel,
            9 => Self::Lanstar,
            10 => Self::AutonetShortAddress,
            11 => Self::LocalTalk,
            12 => Self::LocalNet,
            13 => Self::UltraLink,
            14 => Self::SMDS,
            15 => Self::FrameRelay,
            16 => Self::ATM,
            17 => Self::HDLC,
            18 => Self::FibreChannel,
            19 => Self::ATMLogical,
            20 => Self::SerialLine,
            21 => Self::ATMPhysical,
            22 => Self::MilStd188220,
            23 => Self::Metricom,
            24 => Self::IEEE1394,
            25 => Self::MAPOS,
            26 => Self::Twinaxial,
            27 => Self::EUI64,
            28 => Self::HIPARP,
            29 => Self::IPAndARPOverISO,
            30 => Self::ARPSec,
            31 => Self::IPsecTunnel,
            32 => Self::InfiniBand,
            33 => Self::TIA102,
            34 => Self::Wiegand,
            35 => Self::PureIP,
            36 => Self::HwExp1,
            37 => Self::Hf1,
            38 => Self::UnifiedBus,
            256 => Self::HwExp2,
            257 => Self::AEthernet,
            value => Self::Unknown(value),
        }
    }
}

impl From<HardwareType> for u16 {
    fn from(value: HardwareType) -> Self {
        match value {
            HardwareType::Reserved(val) => val,
            HardwareType::Ethernet => 1,
            HardwareType::ExperimentEthernet => 2,
            HardwareType::AmateurRadioAx25 => 3,
            HardwareType::ProteonPronetTokenRing => 4,
            HardwareType::Chaos => 5,
            HardwareType::Ieee802Networks => 6,
            HardwareType::Arcnet => 7,
            HardwareType::Hyperchannel => 8,
            HardwareType::Lanstar => 9,
            HardwareType::AutonetShortAddress => 10,
            HardwareType::LocalTalk => 11,
            HardwareType::LocalNet => 12,
            HardwareType::UltraLink => 13,
            HardwareType::SMDS => 14,
            HardwareType::FrameRelay => 15,
            HardwareType::ATM => 16,
            HardwareType::HDLC => 17,
            HardwareType::FibreChannel => 18,
            HardwareType::ATMLogical => 19,
            HardwareType::SerialLine => 20,
            HardwareType::ATMPhysical => 21,
            HardwareType::MilStd188220 => 22,
            HardwareType::Metricom => 23,
            HardwareType::IEEE1394 => 24,
            HardwareType::MAPOS => 25,
            HardwareType::Twinaxial => 26,
            HardwareType::EUI64 => 27,
            HardwareType::HIPARP => 28,
            HardwareType::IPAndARPOverISO => 29,
            HardwareType::ARPSec => 30,
            HardwareType::IPsecTunnel => 31,
            HardwareType::InfiniBand => 32,
            HardwareType::TIA102 => 33,
            HardwareType::Wiegand => 34,
            HardwareType::PureIP => 35,
            HardwareType::HwExp1 => 36,
            HardwareType::Hf1 => 37,
            HardwareType::UnifiedBus => 38,
            HardwareType::HwExp2 => 256,
            HardwareType::AEthernet => 257,
            HardwareType::Unknown(val) => val,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct NetworkInterface {
    pub interface_id: u32,
    pub hardware_type: HardwareType,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DnsHostName(String<63>);

impl DnsHostName {
    pub const MAX_LENGTH: usize = 63;

    pub fn new<T: AsRef<str>>(dns_hostname: T) -> Result<Self, RdmError> {
        let dns_hostname = dns_hostname.as_ref();

        if dns_hostname.len() > Self::MAX_LENGTH {
            return Err(RdmError::InvalidStringLength(
                dns_hostname.len(),
                Self::MAX_LENGTH,
            ));
        }

        Ok(Self(String::<63>::from_str(dns_hostname).unwrap()))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn is_valid(&self) -> bool {
        !self.0.is_empty() && self.0.len() <= Self::MAX_LENGTH
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, RdmError> {
        if buf.len() < Self::MAX_LENGTH {
            return Err(RdmError::InvalidBufferLength(buf.len(), Self::MAX_LENGTH));
        }
        let len = self.0.len();

        buf[0..len].copy_from_slice(self.0.as_bytes());

        Ok(len)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, RdmError> {
        if bytes.len() > Self::MAX_LENGTH {
            return Err(RdmError::InvalidStringLength(bytes.len(), Self::MAX_LENGTH));
        }

        let dns_hostname =
            core::str::from_utf8(truncate_at_null(bytes)).map_err(RdmError::from)?;

        Ok(Self(
            String::<63>::from_str(dns_hostname).unwrap(),
        ))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DnsDomainName(String<231>);

impl DnsDomainName {
    pub const MAX_LENGTH: usize = 231;

    pub fn new<T: AsRef<str>>(dns_hostname: T) -> Result<Self, RdmError> {
        let dns_hostname = dns_hostname.as_ref();

        if dns_hostname.len() > Self::MAX_LENGTH {
            return Err(RdmError::InvalidStringLength(
                dns_hostname.len(),
                Self::MAX_LENGTH,
            ));
        }

        Ok(Self(String::<231>::from_str(dns_hostname).unwrap()))
    }
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, RdmError> {
        if buf.len() < Self::MAX_LENGTH {
            return Err(RdmError::InvalidBufferLength(buf.len(), Self::MAX_LENGTH));
        }
        let len = self.0.len();

        buf[0..len].copy_from_slice(self.0.as_bytes());

        Ok(len)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, RdmError> {
        if bytes.len() > Self::MAX_LENGTH {
            return Err(RdmError::InvalidStringLength(bytes.len(), Self::MAX_LENGTH));
        }

        let dns_domain_name =
            core::str::from_utf8(truncate_at_null(bytes)).map_err(RdmError::from)?;

        Ok(Self(
            String::<231>::from_str(dns_domain_name).unwrap(),
        ))
    }
}
