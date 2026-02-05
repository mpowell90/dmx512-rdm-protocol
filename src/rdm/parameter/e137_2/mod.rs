pub mod request;
pub mod response;

use crate::impl_rdm_string;
use core::net::{Ipv4Addr, Ipv6Addr};
use heapless::String;
use macaddr::MacAddr6;
use rdm_core::{
    error::{ParameterDataError, RdmError},
    parameter_traits::RdmParameterData,
};
use rdm_derive::ParameterData;

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

impl RdmParameterData for DhcpMode {
    fn size_of(&self) -> usize {
        1
    }

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterDataError> {
        buf[0] = *self as u8;
        Ok(1)
    }

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterDataError> {
        let dhcp_mode =
            DhcpMode::try_from(buf[0]).map_err(|_| ParameterDataError::MalformedData)?;
        Ok(dhcp_mode)
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

impl RdmParameterData for Ipv4Address {
    fn size_of(&self) -> usize {
        4
    }

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterDataError> {
        let bytes: [u8; 4] = (*self).into();
        buf[0..4].copy_from_slice(&bytes);
        Ok(4)
    }

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterDataError> {
        let address = Ipv4Address::from([buf[0], buf[1], buf[2], buf[3]]);
        Ok(address)
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

impl RdmParameterData for Ipv6Address {
    fn size_of(&self) -> usize {
        16
    }

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterDataError> {
        let bytes: [u8; 16] = (*self).into();
        buf[0..16].copy_from_slice(&bytes);
        Ok(16)
    }

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterDataError> {
        let address = Ipv6Address::from([
            buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7], buf[8], buf[9],
            buf[10], buf[11], buf[12], buf[13], buf[14], buf[15],
        ]);
        Ok(address)
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

impl RdmParameterData for Ipv4Route {
    fn size_of(&self) -> usize {
        4
    }

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterDataError> {
        let bytes: [u8; 4] = (*self).into();
        buf[0..4].copy_from_slice(&bytes);
        Ok(4)
    }

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterDataError> {
        let route = Ipv4Route::from([buf[0], buf[1], buf[2], buf[3]]);
        Ok(route)
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

impl RdmParameterData for HardwareType {
    fn size_of(&self) -> usize {
        2
    }

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterDataError> {
        buf[0..2].copy_from_slice(&u16::from(*self).to_be_bytes());
        Ok(2)
    }

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterDataError> {
        Ok(u16::from_be_bytes([buf[0], buf[1]]).into())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, ParameterData)]
pub struct NetworkInterface {
    pub interface_id: u32,
    pub hardware_type: HardwareType,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct InterfaceLabel(String<{ InterfaceLabel::MAX_LENGTH }>);

impl_rdm_string!(InterfaceLabel, 32);

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DnsHostName(String<{ DnsHostName::MAX_LENGTH }>);

impl_rdm_string!(DnsHostName, 63);

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DnsDomainName(String<{ DnsDomainName::MAX_LENGTH }>);

impl_rdm_string!(DnsDomainName, 231);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct MacAddress(pub MacAddr6);

impl From<MacAddr6> for MacAddress {
    fn from(value: MacAddr6) -> Self {
        Self(value)
    }
}

impl From<[u8; 6]> for MacAddress {
    fn from(value: [u8; 6]) -> Self {
        Self(MacAddr6::from(value))
    }
}

impl RdmParameterData for MacAddress {
    fn size_of(&self) -> usize {
        6
    }

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterDataError> {
        buf[0..6].copy_from_slice(self.0.as_bytes());
        Ok(6)
    }

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterDataError> {
        let address = MacAddress::from([buf[0], buf[1], buf[2], buf[3], buf[4], buf[5]]);
        Ok(address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::net::Ipv4Addr;

    #[test]
    fn should_encode_decode_dhcp_mode() {
        let mut buf = [0u8; 1];
        let m = DhcpMode::Active;
        m.encode_parameter_data(&mut buf).unwrap();
        assert_eq!(buf[0], 0x01);
        assert_eq!(DhcpMode::decode_parameter_data(&buf).unwrap(), m);
    }

    #[test]
    fn should_encode_decode_ipv4_address() {
        let ip = Ipv4Addr::new(1, 2, 3, 4);
        let addr = Ipv4Address::from(ip);
        let mut buf = [0u8; 4];
        addr.encode_parameter_data(&mut buf).unwrap();
        assert_eq!(&buf, &[1, 2, 3, 4]);
        let decoded = Ipv4Address::decode_parameter_data(&buf).unwrap();
        assert_eq!(decoded, addr);

        // unconfigured
        let un = Ipv4Address::from([0u8; 4]);
        let mut bufu = [0u8; 4];
        un.encode_parameter_data(&mut bufu).unwrap();
        assert_eq!(&bufu, &[0, 0, 0, 0]);
        assert_eq!(Ipv4Address::decode_parameter_data(&bufu).unwrap(), un);
    }

    #[test]
    fn should_encode_decode_ipv6_address() {
        let octets = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10,
        ];
        let addr = Ipv6Address::from(octets);
        let mut buf = [0u8; 16];
        let bytes_written = addr.encode_parameter_data(&mut buf).unwrap();
        assert_eq!(&buf, &octets);
        assert_eq!(bytes_written, 16);
        let decoded = Ipv6Address::decode_parameter_data(&buf).unwrap();
        assert_eq!(decoded, addr);
    }

    #[test]
    fn should_encode_decode_ipv4_route() {
        let r = Ipv4Route::from(Ipv4Addr::new(10, 0, 0, 1));
        let mut buf = [0u8; 4];
        r.encode_parameter_data(&mut buf).unwrap();
        assert_eq!(&buf, &[10, 0, 0, 1]);
        assert_eq!(Ipv4Route::decode_parameter_data(&buf).unwrap(), r);

        let nr = Ipv4Route::from(0u32);
        let mut bufn = [0u8; 4];
        nr.encode_parameter_data(&mut bufn).unwrap();
        assert_eq!(&bufn, &[0, 0, 0, 0]);
        assert_eq!(Ipv4Route::decode_parameter_data(&bufn).unwrap(), nr);
    }

    #[test]
    fn should_encode_decode_hardware_type() {
        let mut buf = [0u8; 2];
        let hw = HardwareType::Ethernet;
        hw.encode_parameter_data(&mut buf).unwrap();
        assert_eq!(&buf, &[0x00, 0x01]);
        assert_eq!(HardwareType::decode_parameter_data(&buf).unwrap(), hw);
    }

    #[test]
    fn should_encode_decode_mac() {
        let mac = MacAddress::from([1, 2, 3, 4, 5, 6]);
        let mut buf = [0u8; 6];
        mac.encode_parameter_data(&mut buf).unwrap();
        assert_eq!(&buf, &[1, 2, 3, 4, 5, 6]);
        assert_eq!(MacAddress::decode_parameter_data(&buf).unwrap(), mac);
    }

    #[test]
    fn should_encode_decode_interface() {
        // NetworkInterface (u32 + hardware type)
        let net = NetworkInterface {
            interface_id: 0x01020304,
            hardware_type: HardwareType::Ethernet,
        };
        let mut buf = [0u8; 6];
        net.encode_parameter_data(&mut buf).unwrap();
        assert_eq!(&buf, &[0x01, 0x02, 0x03, 0x04, 0x00, 0x01]);
        assert_eq!(NetworkInterface::decode_parameter_data(&buf).unwrap(), net);
    }
}
