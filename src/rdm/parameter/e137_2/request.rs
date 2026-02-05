use crate::rdm::parameter::e137_2::{DnsDomainName, DnsHostName, Ipv4Address, Ipv4Route};
use rdm_core::{CommandClass, ParameterId};
use rdm_derive::rdm_parameter;

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::InterfaceLabel, command_class = CommandClass::Get)]
pub struct GetInterfaceLabelRequest {
    pub interface_id: u32,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::InterfaceHardwareAddressType1, command_class = CommandClass::Get)]
pub struct GetInterfaceHardwareAddressType1Request {
    pub interface_id: u32,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::IpV4DhcpMode, command_class = CommandClass::Get)]
pub struct GetIpV4DhcpModeRequest {
    pub interface_id: u32,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::IpV4DhcpMode, command_class = CommandClass::Set)]
pub struct SetIpV4DhcpModeRequest {
    pub interface_id: u32,
    pub dhcp_mode: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::IpV4ZeroConfMode, command_class = CommandClass::Get)]
pub struct GetIpV4ZeroConfModeRequest {
    pub interface_id: u32,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::IpV4ZeroConfMode, command_class = CommandClass::Set)]
pub struct SetIpV4ZeroConfModeRequest {
    pub interface_id: u32,
    pub zero_conf_mode: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::IpV4CurrentAddress, command_class = CommandClass::Get)]
pub struct GetIpV4CurrentAddressRequest {
    pub interface_id: u32,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::IpV4StaticAddress, command_class = CommandClass::Get)]
pub struct GetIpV4StaticAddressRequest {
    pub interface_id: u32,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::IpV4StaticAddress, command_class = CommandClass::Set)]
pub struct SetIpV4StaticAddressRequest {
    pub interface_id: u32,
    pub address: Ipv4Address,
    pub netmask: u8,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::InterfaceRenewDhcp, command_class = CommandClass::Set)]
pub struct SetInterfaceRenewDhcpRequest {
    pub interface_id: u32,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::InterfaceReleaseDhcp, command_class = CommandClass::Set)]
pub struct SetInterfaceReleaseDhcpRequest {
    pub interface_id: u32,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::InterfaceApplyConfiguration, command_class = CommandClass::Set)]
pub struct SetInterfaceApplyConfigurationRequest {
    pub interface_id: u32,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::IpV4DefaultRoute, command_class = CommandClass::Set)]
pub struct SetIpV4DefaultRouteRequest {
    pub interface_id: u32,
    pub address: Ipv4Route,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::DnsIpV4NameServer, command_class = CommandClass::Get)]
pub struct GetDnsIpV4NameServerRequest {
    pub name_server_index: u8,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::DnsIpV4NameServer, command_class = CommandClass::Set)]
pub struct SetDnsIpv4NameServerRequest {
    pub name_server_index: u8,
    pub name_server_address: Ipv4Address,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::DnsHostName, command_class = CommandClass::Set)]
pub struct SetDnsHostNameRequest {
    pub dns_host_name: DnsHostName,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::DnsDomainName, command_class = CommandClass::Set)]
pub struct SetDnsDomainNameRequest {
    pub dns_domain_name: DnsDomainName,
}
