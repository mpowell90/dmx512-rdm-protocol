use crate::rdm::{
    core::{CommandClass, DeviceUID, ParameterId},
    derive::rdm_parameter,
    parameter::e137_7::{DiscoveryState, EndpointId, EndpointLabel, EndpointMode},
};

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::IdentifyEndpoint, command_class = CommandClass::Get)]
pub struct GetIdentifyEndpointRequest {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::IdentifyEndpoint, command_class = CommandClass::Set)]
pub struct SetIdentifyEndpointRequest {
    pub endpoint_id: EndpointId,
    pub identify: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::EndpointToUniverse, command_class = CommandClass::Get)]
pub struct GetEndpointToUniverseRequest {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::EndpointToUniverse, command_class = CommandClass::Set)]
pub struct SetEndpointToUniverseRequest {
    pub endpoint_id: EndpointId,
    pub universe: u16,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::EndpointMode, command_class = CommandClass::Get)]
pub struct GetEndpointModeRequest {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::EndpointMode, command_class = CommandClass::Set)]
pub struct SetEndpointModeRequest {
    pub endpoint_id: EndpointId,
    pub mode: EndpointMode,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::EndpointLabel, command_class = CommandClass::Get)]
pub struct GetEndpointLabelRequest {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::EndpointLabel, command_class = CommandClass::Set)]
pub struct SetEndpointLabelRequest {
    pub endpoint_id: EndpointId,
    pub label: EndpointLabel,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::RdmTrafficEnable, command_class = CommandClass::Get)]
pub struct GetRdmTrafficEnableRequest {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::RdmTrafficEnable, command_class = CommandClass::Set)]
pub struct SetRdmTrafficEnableRequest {
    pub endpoint_id: EndpointId,
    pub enable: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::DiscoveryState, command_class = CommandClass::Get)]
pub struct GetDiscoveryStateRequest {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::DiscoveryState, command_class = CommandClass::Set)]
pub struct SetDiscoveryStateRequest {
    pub endpoint_id: EndpointId,
    pub discovery_state: DiscoveryState,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::BackgroundDiscovery, command_class = CommandClass::Get)]
pub struct GetBackgroundDiscoveryRequest {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::BackgroundDiscovery, command_class = CommandClass::Set)]
pub struct SetBackgroundDiscoveryRequest {
    pub endpoint_id: EndpointId,
    pub enable: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::EndpointTiming, command_class = CommandClass::Get)]
pub struct GetEndpointTimingRequest {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::EndpointTiming, command_class = CommandClass::Set)]
pub struct SetEndpointTimingRequest {
    pub endpoint_id: EndpointId,
    pub setting_id: u8,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::EndpointTimingDescription, command_class = CommandClass::Get)]
pub struct GetEndpointTimingDescriptionRequest {
    pub setting_id: u8,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::EndpointResponders, command_class = CommandClass::Get)]
pub struct GetEndpointRespondersRequest {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::EndpointResponderListChange, command_class = CommandClass::Get)]
pub struct GetEndpointResponderListChangeRequest {
    pub endpoint_id: EndpointId,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::BindingControlFields, command_class = CommandClass::Get)]
pub struct GetBindingControlFieldsRequest {
    pub endpoint_id: EndpointId,
    pub uid: DeviceUID,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::BackgroundQueuedStatusPolicy, command_class = CommandClass::Get)]
pub struct GetBackgroundQueuedStatusPolicyRequest {
    pub policy_id: u8,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::BackgroundQueuedStatusPolicy, command_class = CommandClass::Set)]
pub struct SetBackgroundQueuedStatusPolicyRequest {
    pub policy_id: u8,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::BackgroundQueuedStatusPolicyDescription, command_class = CommandClass::Get)]
pub struct GetBackgroundQueuedStatusPolicyDescriptionRequest {
    pub policy_id: u8,
}
