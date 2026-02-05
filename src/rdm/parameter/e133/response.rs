use crate::rdm::parameter::{
    e133::{BrokerState, Scope, SearchDomain, StaticConfigType},
    e137_2::{Ipv4Address, Ipv6Address},
};
use rdm_core::{CommandClass, ParameterId};
use rdm_derive::rdm_parameter;

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::ComponentScope, command_class = CommandClass::GetResponse)]
pub struct GetComponentScopeResponse {
    pub scope_slot: u16,
    pub scope_string: Scope,
    pub static_config_type: StaticConfigType,
    pub static_ipv4_address: Ipv4Address,
    pub static_ipv6_address: Ipv6Address,
    pub static_port: u16,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::SearchDomain, command_class = CommandClass::GetResponse)]
pub struct GetSearchDomainResponse {
    pub search_domain: SearchDomain,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::TcpCommsStatus, command_class = CommandClass::GetResponse)]
pub struct GetTcpCommsStatusResponse {
    pub scope_string: Scope,
    pub broker_ipv4_address: Ipv4Address,
    pub broker_ipv6_address: Ipv6Address,
    pub broker_port: u16,
    pub unhealthy_tcp_events: u16,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::BrokerStatus, command_class = CommandClass::GetResponse)]
pub struct GetBrokerStatusResponse {
    pub is_allowing_set_commands: bool,
    pub broker_state: BrokerState,
}
