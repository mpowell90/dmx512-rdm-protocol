use crate::device::DeviceUID;

pub enum DiscoveryRequestParameterData {
    DiscUniqueBranch {
        lower_bound_uid: DeviceUID,
        upper_bound_uid: DeviceUID,
    },
}

pub struct DiscoveryRequest {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub port_id: u8,
    pub sub_device_id: u16,
    pub parameter_id: u16,
    pub parameter_data: Option<DiscoveryRequestParameterData>,
}

pub enum GetRequestParameterData {
    ParameterDescription { parameter_id: u16 },
    SensorDefinition { sensor_id: u8 },
    DmxPersonalityDescription { personality: u8 },
    CurveDescription { curve: u8 },
    ModulationFrequencyDescription { modulation_frequency: u8 },
    OutputResponseTimeDescription { output_response_time: u8 },
    SelfTestDescription { self_test_id: u8 },
    SlotDescription { slot_id: u16 },
}

pub struct GetRequest {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub port_id: u8,
    pub sub_device_id: u16,
    pub parameter_id: u16,
    pub parameter_data: Option<GetRequestParameterData>,
}

pub enum SetRequestParameterData {
    DeviceLabel { device_label: String },
    DmxPersonality { personality_id: u8 },
    DmxStartAddress { dmx_start_address: u16 },
    Curve { curve_id: u8 },
    ModulationFrequency { modulation_frequency_id: u8 },
    OutputResponseTime { output_response_time_id: u8 },
    IdentifyDevice { identify: bool },
}

pub struct SetRequest {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub port_id: u8,
    pub sub_device_id: u16,
    pub parameter_id: u16,
    pub parameter_data: Option<SetRequestParameterData>,
}

pub enum RdmRequestMessage {
    DiscoveryRequest(DiscoveryRequest),
    GetRequest(GetRequest),
    SetRequest(SetRequest),
}