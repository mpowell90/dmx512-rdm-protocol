use crate::rdm::parameter::e120::{
    DeviceLabel, DisplayInvertMode, FadeTimes, Iso639_1, LampOnMode, LampState, PowerState,
    PresetPlaybackMode, ResetDeviceMode, SelfTest, StatusType,
};
use rdm_core::{CommandClass, DeviceUID, ParameterId};
use rdm_derive::rdm_parameter;

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::DiscUniqueBranch, command_class = CommandClass::Discovery)]
#[repr(C)]
pub struct DiscUniqueBranchRequest {
    pub lower_bound_uid: DeviceUID,
    pub upper_bound_uid: DeviceUID,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::QueuedMessage, command_class = CommandClass::Get)]
#[repr(C, packed)]
pub struct GetQueuedMessageRequest {
    pub status_type: StatusType,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::StatusMessages, command_class = CommandClass::Get)]
#[repr(C, packed)]
pub struct GetStatusMessagesRequest {
    pub status_type: StatusType,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::StatusIdDescription, command_class = CommandClass::Get)]
#[repr(C)]
pub struct GetStatusIdDescriptionRequest {
    pub status_id: u16,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::SubDeviceIdStatusReportThreshold, command_class = CommandClass::Set)]
#[repr(C, packed)]
pub struct SetSubDeviceIdStatusReportThresholdRequest {
    pub status_type: StatusType,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::ParameterDescription, command_class = CommandClass::Get)]
#[repr(C)]
pub struct GetParameterDescriptionRequest {
    pub parameter_id: ParameterId,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::DeviceLabel, command_class = CommandClass::Set)]
#[repr(C)]
pub struct SetDeviceLabelRequest {
    pub device_label: DeviceLabel,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::Language, command_class = CommandClass::Set)]
#[repr(C)]
pub struct SetLanguageRequest {
    pub language: Iso639_1,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::DmxPersonality, command_class = CommandClass::Set)]
#[repr(C, packed)]
pub struct SetDmxPersonalityRequest {
    pub personality_id: u8,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::DmxPersonalityDescription, command_class = CommandClass::Get)]
#[repr(C, packed)]
pub struct GetDmxPersonalityDescriptionRequest {
    pub personality_id: u8,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::DmxStartAddress, command_class = CommandClass::Set)]
#[repr(C)]
pub struct SetDmxStartAddressRequest {
    pub dmx_start_address: u16,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::SlotDescription, command_class = CommandClass::Get)]
#[repr(C)]
pub struct GetSlotDescriptionRequest {
    pub slot_id: u16,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::SensorDefinition, command_class = CommandClass::Get)]
#[repr(C, packed)]
pub struct GetSensorDefinitionRequest {
    pub sensor_id: u8,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::SensorValue, command_class = CommandClass::Get)]
#[repr(C, packed)]
pub struct GetSensorValueRequest {
    pub sensor_id: u8,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::SensorValue, command_class = CommandClass::Set)]
#[repr(C, packed)]
pub struct SetSensorValueRequest {
    pub sensor_id: u8,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::RecordSensors, command_class = CommandClass::Set)]
#[repr(C, packed)]
pub struct SetRecordSensorsRequest {
    pub sensor_id: u8,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::DeviceHours, command_class = CommandClass::Set)]
#[repr(C)]
pub struct SetDeviceHoursRequest {
    pub device_hours: u32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::LampHours, command_class = CommandClass::Set)]
#[repr(C)]
pub struct SetLampHoursRequest {
    pub lamp_hours: u32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::LampStrikes, command_class = CommandClass::Set)]
#[repr(C)]
pub struct SetLampStrikesRequest {
    pub lamp_strikes: u32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::LampState, command_class = CommandClass::Set)]
#[repr(C, packed)]
pub struct SetLampStateRequest {
    pub lamp_state: LampState,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::LampOnMode, command_class = CommandClass::Set)]
#[repr(C, packed)]
pub struct SetLampOnModeRequest {
    pub lamp_on_mode: LampOnMode,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::DevicePowerCycles, command_class = CommandClass::Set)]
#[repr(C)]
pub struct SetDevicePowerCyclesRequest {
    pub device_power_cycles: u32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::DisplayInvert, command_class = CommandClass::Set)]
#[repr(C)]
pub struct SetDisplayInvertRequest {
    pub display_invert_mode: DisplayInvertMode,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::DisplayLevel, command_class = CommandClass::Set)]
#[repr(C, packed)]
pub struct SetDisplayLevelRequest {
    pub display_level: u8,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::PanInvert, command_class = CommandClass::Set)]
#[repr(C, packed)]
pub struct SetPanInvertRequest {
    pub pan_invert: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::TiltInvert, command_class = CommandClass::Set)]
#[repr(C, packed)]
pub struct SetTiltInvertRequest {
    pub tilt_invert: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::PanTiltSwap, command_class = CommandClass::Set)]
#[repr(C, packed)]
pub struct SetPanTiltSwapRequest {
    pub pan_tilt_swap: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::RealTimeClock, command_class = CommandClass::Set)]
#[repr(C)]
pub struct SetRealTimeClockRequest {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::IdentifyDevice, command_class = CommandClass::Set)]
#[repr(C, packed)]
pub struct SetIdentifyDeviceRequest {
    pub identify: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::ResetDevice, command_class = CommandClass::Set)]
#[repr(C)]
pub struct SetResetDeviceRequest {
    pub reset_device: ResetDeviceMode,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::PowerState, command_class = CommandClass::Set)]
#[repr(C, packed)]
pub struct SetPowerStateRequest {
    pub power_state: PowerState,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::PerformSelfTest, command_class = CommandClass::Set)]
#[repr(C, packed)]
pub struct SetPerformSelfTestRequest {
    pub self_test_id: SelfTest,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::SelfTestDescription, command_class = CommandClass::Get)]
#[repr(C, packed)]
pub struct GetSelfTestDescriptionRequest {
    pub self_test_id: SelfTest,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::CapturePreset, command_class = CommandClass::Set)]
#[repr(C)]
pub struct SetCapturePresetRequest {
    pub scene_id: u16,
    pub fade_times: Option<FadeTimes>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_parameter(pid = ParameterId::PresetPlayback, command_class = CommandClass::Set)]
#[repr(C)]
pub struct SetPresetPlaybackRequest {
    pub mode: PresetPlaybackMode,
    pub level: u8,
}
