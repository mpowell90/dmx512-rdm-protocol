use crate::rdm::parameter::e137_7::{
    BackgroundQueuedStatusPolicyDescription, DiscoveryCountStatus, DiscoveryState, EndpointEntry,
    EndpointId, EndpointLabel, EndpointMode, EndpointTimingDescription,
};
use heapless::Vec;
use rdm_core::{CommandClass, DeviceUID, ParameterId};
use rdm_derive::rdm_parameter;

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::EndpointList, command_class = CommandClass::Get)]
pub struct GetEndpointListResponse {
    pub list_change_number: u32,
    pub endpoint_list: Vec<EndpointEntry, 75>,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::EndpointListChange, command_class = CommandClass::Get)]
pub struct GetEndpointListChangeResponse {
    pub list_change_number: u32,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::IdentifyEndpoint, command_class = CommandClass::Get)]
pub struct GetIdentifyEndpointResponse {
    pub endpoint_id: EndpointId,
    pub identify: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::IdentifyEndpoint, command_class = CommandClass::Set)]
pub struct SetIdentifyEndpointResponse {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::EndpointToUniverse, command_class = CommandClass::Get)]
pub struct GetEndpointToUniverseResponse {
    pub endpoint_id: EndpointId,
    pub universe: u16,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::EndpointToUniverse, command_class = CommandClass::Set)]
pub struct SetEndpointToUniverseResponse {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::EndpointMode, command_class = CommandClass::Get)]
pub struct GetEndpointModeResponse {
    pub endpoint_id: EndpointId,
    pub mode: EndpointMode,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::EndpointMode, command_class = CommandClass::Set)]
pub struct SetEndpointModeResponse {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::EndpointLabel, command_class = CommandClass::Get)]
pub struct GetEndpointLabelResponse {
    pub endpoint_id: EndpointId,
    pub label: EndpointLabel,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::EndpointLabel, command_class = CommandClass::Set)]
pub struct SetEndpointLabelResponse {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::RdmTrafficEnable, command_class = CommandClass::Get)]
pub struct GetRdmTrafficEnableResponse {
    pub endpoint_id: EndpointId,
    pub enable: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::RdmTrafficEnable, command_class = CommandClass::Set)]
pub struct SetRdmTrafficEnableResponse {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::DiscoveryState, command_class = CommandClass::Get)]
pub struct GetDiscoveryStateResponse {
    pub endpoint_id: EndpointId,
    pub device_count: DiscoveryCountStatus,
    pub discovery_state: DiscoveryState,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::DiscoveryState, command_class = CommandClass::Set)]
pub struct SetDiscoveryStateResponse {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::BackgroundDiscovery, command_class = CommandClass::Get)]
pub struct GetBackgroundDiscoveryResponse {
    pub endpoint_id: EndpointId,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::BackgroundDiscovery, command_class = CommandClass::Set)]
pub struct SetBackgroundDiscoveryResponse {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::EndpointTiming, command_class = CommandClass::Get)]
pub struct GetEndpointTimingResponse {
    pub endpoint_id: EndpointId,
    pub current_setting_id: u8,
    pub setting_count: u8,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::EndpointTiming, command_class = CommandClass::Set)]
pub struct SetEndpointTimingResponse {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::EndpointTimingDescription, command_class = CommandClass::Get)]
pub struct GetEndpointTimingDescriptionResponse {
    pub setting_id: u8,
    pub description: EndpointTimingDescription,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::EndpointResponders, command_class = CommandClass::Get)]
pub struct GetEndpointRespondersResponse {
    pub endpoint_id: EndpointId,
    pub list_change_number: u32,
    pub responders: Vec<DeviceUID, 37>,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::EndpointResponderListChange, command_class = CommandClass::Get)]
pub struct GetEndpointResponderListChangeResponse {
    pub endpoint_id: EndpointId,
    pub list_change_number: u32,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::BindingControlFields, command_class = CommandClass::Get)]
pub struct GetBindingControlFieldsResponse {
    pub endpoint_id: EndpointId,
    pub uid: DeviceUID,
    pub control_field: u16,
    pub binding_uid: DeviceUID,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::BackgroundQueuedStatusPolicy, command_class = CommandClass::Get)]
pub struct GetBackgroundQueuedStatusPolicyResponse {
    pub current_policy_id: u8,
    pub policy_count: u8,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::BackgroundQueuedStatusPolicyDescription, command_class = CommandClass::Get)]
pub struct GetBackgroundQueuedStatusPolicyDescriptionResponse {
    pub policy_id: u8,
    pub description: BackgroundQueuedStatusPolicyDescription,
}
