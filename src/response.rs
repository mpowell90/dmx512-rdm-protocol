use thiserror::Error;

use crate::{
    device::{DeviceUID, DmxSlot},
    parameter::{
        DisplayInvertMode, LampOnMode, LampState, ManufacturerSpecificParameter, ParameterId,
        PowerState, ProductCategory,
    },
    sensor::Sensor,
    CommandClass,
};
use std::collections::HashMap;

// TODO the following is a quick and dirty test
#[derive(Debug, PartialEq)]
pub enum PacketResponseType {
    SuccessResponse = 0x05,
    NullResponse = 0x0c,
}

pub enum ResponseNackReasonCode {
    UnknownPid = 0x0000,
    FormatError = 0x0001,
    HardwareFault = 0x0002,
    ProxyReject = 0x0003,
    WriteProtect = 0x0004,
    UnsupportedCommandClass = 0x0005,
    DataOutOfRange = 0x0006,
    BufferFull = 0x0007,
    PacketSizeUnsupported = 0x0008,
    SubDeviceOutOfRange = 0x0009,
    ProxyBufferFull = 0x000a,
}

#[derive(Clone, Debug)]
pub enum ResponseType {
    Ack = 0x00,
    AckTimer = 0x01,
    NackReason = 0x02,
    AckOverflow = 0x03,
}

impl TryFrom<u8> for ResponseType {
    type Error = &'static str;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        let response_type = match byte {
            0x00 => ResponseType::Ack,
            0x01 => ResponseType::AckTimer,
            0x02 => ResponseType::NackReason,
            0x03 => ResponseType::AckOverflow,
            _ => return Err("Invalid value for ResponseType"),
        };
        Ok(response_type)
    }
}

#[derive(Debug, Error)]
pub enum DriverError {
    #[error("invalid data length")]
    InvalidDataLength,
    #[error("invalid start byte")]
    InvalidStartByte,
    #[error("invalid stop byte")]
    InvalidStopByte,
    #[error("invalid packet type")]
    UnsupportedPacketType,
    #[error("malformed packet")]
    MalformedPacket,
    #[error("unknown driver error")]
    Unknown,
}

impl TryFrom<u8> for PacketResponseType {
    type Error = DriverError;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        let packet_type = match byte {
            0x05 => PacketResponseType::SuccessResponse,
            0x0c => PacketResponseType::NullResponse,
            _ => return Err(DriverError::UnsupportedPacketType),
        };
        Ok(packet_type)
    }
}

#[derive(Clone, Debug)]
pub enum DiscoveryResponseParameterData {
    DiscMute {
        control_field: u16,
        binding_uid: Option<DeviceUID>,
    },
    DiscUnmute {
        control_field: u16,
        binding_uid: Option<DeviceUID>,
    },
}

#[derive(Clone, Debug)]
pub enum GetResponseParameterData {
    ProxiedDeviceCount {
        device_count: u16,
        list_change: bool,
    },
    ProxiedDevices {
        device_uids: Vec<DeviceUID>,
    },
    ParameterDescription {
        parameter_id: u16,
        parameter_data_size: u8,
        data_type: u8,
        command_class: CommandClass,
        prefix: u8,
        minimum_valid_value: u32,
        maximum_valid_value: u32,
        default_value: u32,
        description: String,
    },
    DeviceLabel {
        device_label: String,
    },
    DeviceInfo {
        protocol_version: String,
        model_id: u16,
        product_category: ProductCategory,
        software_version_id: u32,
        footprint: u16,
        current_personality: u8,
        personality_count: u8,
        start_address: u16,
        sub_device_count: u16,
        sensor_count: u8,
    },
    SoftwareVersionLabel {
        software_version_label: String,
    },
    SupportedParameters {
        standard_parameters: Vec<ParameterId>,
        manufacturer_specific_parameters: HashMap<u16, ManufacturerSpecificParameter>,
    },
    SensorDefinition {
        sensor: Sensor,
    },
    IdentifyDevice {
        is_identifying: bool,
    },
    ManufacturerLabel {
        manufacturer_label: String,
    },
    FactoryDefaults {
        factory_default: bool,
    },
    DeviceModelDescription {
        device_model_description: String,
    },
    ProductDetailIdList {
        product_detail_id_list: Vec<u16>,
    },
    DmxPersonality {
        current_personality: u8,
        personality_count: u8,
    },
    DmxPersonalityDescription {
        id: u8,
        dmx_slots_required: u16,
        description: String,
    },
    DmxStartAddress {
        dmx_start_address: u16,
    },
    SlotInfo {
        dmx_slots: Option<Vec<DmxSlot>>,
    },
    SlotDescription {
        slot_id: u16,
        description: String,
    },
    DeviceHours {
        device_hours: u32,
    },
    LampHours {
        lamp_hours: u32,
    },
    LampStrikes {
        lamp_strikes: u32,
    },
    LampState {
        lamp_state: LampState,
    },
    LampOnMode {
        lamp_on_mode: LampOnMode,
    },
    DevicePowerCycles {
        power_cycle_count: u32,
    },
    DisplayInvert {
        display_invert_mode: DisplayInvertMode,
    },
    Curve {
        current_curve: u8,
        curve_count: u8,
    },
    CurveDescription {
        id: u8,
        description: String,
    },
    ModulationFrequency {
        current_modulation_frequency: u8,
        modulation_frequency_count: u8,
    },
    ModulationFrequencyDescription {
        id: u8,
        frequency: u32,
        description: String,
    },
    DimmerInfo {
        minimum_level_lower_limit: u16,
        minimum_level_upper_limit: u16,
        maximum_level_lower_limit: u16,
        maximum_level_upper_limit: u16,
        num_of_supported_curves: u8,
        levels_resolution: u8,
        minimum_levels_split_levels_supports: u8,
    },
    MinimumLevel {
        minimum_level_increasing: u16,
        minimum_level_decreasing: u16,
        on_below_minimum: u8, // TODO could be bool
    },
    MaximumLevel {
        maximum_level: u16,
    },
    OutputResponseTime {
        current_output_response_time: u8,
        output_response_time_count: u8,
    },
    OutputResponseTimeDescription {
        id: u8,
        description: String,
    },
    PowerState {
        power_state: PowerState,
    },
    PerformSelfTest {
        is_active: bool,
    },
    SelfTestDescription {
        self_test_id: u8,
        description: String,
    },
    PresetPlayback {
        mode: u16,
        level: u8,
    },
}

#[derive(Clone, Debug)]
pub enum SetResponseParameterData {
    DeviceLabel,
    DmxPersonality,
    DmxStartAddress,
    Curve,
    ModulationFrequency,
    OutputResponseTime,
    IdentifyDevice,
}

#[derive(Clone, Debug)]
pub struct GetResponse {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub response_type: ResponseType,
    pub message_count: u8,
    pub sub_device_id: u16,
    pub command_class: CommandClass,
    pub parameter_id: ParameterId,
    pub parameter_data: Option<GetResponseParameterData>,
}

#[derive(Clone, Debug)]
pub struct SetResponse {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub response_type: ResponseType,
    pub message_count: u8,
    pub sub_device_id: u16,
    pub command_class: CommandClass,
    pub parameter_id: ParameterId,
    pub parameter_data: Option<SetResponseParameterData>,
}

#[derive(Clone, Debug)]
pub struct DiscoveryResponse {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub response_type: ResponseType,
    pub message_count: u8,
    pub sub_device_id: u16,
    pub command_class: CommandClass,
    pub parameter_id: ParameterId,
    pub parameter_data: Option<DiscoveryResponseParameterData>,
}

#[derive(Clone, Debug)]
pub enum RdmResponseMessage {
    DiscoveryUniqueBranchResponse(DeviceUID),
    DiscoveryResponse(DiscoveryResponse),
    GetResponse(GetResponse),
    SetResponse(SetResponse),
}
