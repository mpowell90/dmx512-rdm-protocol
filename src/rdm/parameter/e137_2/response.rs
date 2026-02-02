use crate::rdm::parameter::e137_2::{
    DhcpMode, DnsDomainName, DnsHostName, InterfaceLabel, Ipv4Address, Ipv4Route, MacAddress,
    NetworkInterface,
};
use rdm_core::{CommandClass, ParameterId, parameter_traits::RdmParameterData};
use rdm_derive::rdm_response_parameter;

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::ListInterfaces, command_class = CommandClass::GetResponse)]
pub struct GetListInterfacesResponse {
    pub interface_list: heapless::Vec<NetworkInterface, 38>,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::InterfaceLabel, command_class = CommandClass::GetResponse)]
pub struct GetInterfaceLabelResponse {
    pub interface_id: u32,
    pub interface_label: InterfaceLabel,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::InterfaceHardwareAddressType1, command_class = CommandClass::GetResponse)]
pub struct GetInterfaceHardwareAddressType1Response {
    pub interface_id: u32,
    pub hardware_address: MacAddress,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::IpV4DhcpMode, command_class = CommandClass::GetResponse)]
pub struct GetIpV4DhcpModeResponse {
    pub interface_id: u32,
    pub dhcp_mode: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::IpV4ZeroConfMode, command_class = CommandClass::GetResponse)]
pub struct GetIpV4ZeroConfModeResponse {
    pub interface_id: u32,
    pub zero_conf_mode: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::IpV4CurrentAddress, command_class = CommandClass::GetResponse)]
pub struct GetIpV4CurrentAddressResponse {
    pub interface_id: u32,
    pub address: Ipv4Address,
    pub netmask: u8,
    pub dhcp_status: DhcpMode,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::IpV4StaticAddress, command_class = CommandClass::GetResponse)]
pub struct GetIpV4StaticAddressResponse {
    pub interface_id: u32,
    pub address: Ipv4Address,
    pub netmask: u8,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::IpV4DefaultRoute, command_class = CommandClass::GetResponse)]
pub struct GetIpV4DefaultRouteResponse {
    pub interface_id: u32,
    pub address: Ipv4Route,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::DnsIpV4NameServer, command_class = CommandClass::GetResponse)]
pub struct GetDnsIpV4NameServerResponse {
    pub name_server_index: u8,
    pub address: Ipv4Address,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::DnsHostName, command_class = CommandClass::GetResponse)]
pub struct GetDnsHostNameResponse {
    pub dns_host_name: DnsHostName,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::DnsDomainName, command_class = CommandClass::GetResponse)]
pub struct GetDnsDomainNameResponse {
    pub dns_domain_name: DnsDomainName,
}
