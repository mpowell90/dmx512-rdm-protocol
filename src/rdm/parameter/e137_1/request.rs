use crate::rdm::parameter::{
    e120::{PresetPlaybackMode, SelfTest},
    e137_1::{MergeMode, PinCode, TimeMode},
};
use rdm_core::{CommandClass, ParameterId, parameter_traits::RdmParameterData};
use rdm_derive::rdm_request_parameter;

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::DmxBlockAddress, command_class = CommandClass::Set)]
pub struct SetDmxBlockAddressRequest {
    pub dmx_block_address: u16,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::DmxFailMode, command_class = CommandClass::Set)]
pub struct SetDmxFailModeRequest {
    pub scene_id: PresetPlaybackMode,
    pub loss_of_signal_delay: TimeMode,
    pub hold_time: TimeMode,
    pub level: u8,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::DmxStartupMode, command_class = CommandClass::Set)]
pub struct SetDmxStartupModeRequest {
    pub scene_id: PresetPlaybackMode,
    pub startup_delay: TimeMode,
    pub hold_time: TimeMode,
    pub level: u8,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::MinimumLevel, command_class = CommandClass::Set)]
pub struct SetMinimumLevelRequest {
    pub minimum_level_increasing: u16,
    pub minimum_level_decreasing: u16,
    pub on_below_minimum: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::MaximumLevel, command_class = CommandClass::Set)]
pub struct SetMaximumLevelRequest {
    pub maximum_level: u16,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::Curve, command_class = CommandClass::Set)]
pub struct SetCurveRequest {
    pub curve_id: u8,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::CurveDescription, command_class = CommandClass::Get)]
pub struct GetCurveDescriptionRequest {
    pub curve_id: u8,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::OutputResponseTime, command_class = CommandClass::Set)]
pub struct SetOutputResponseTimeRequest {
    pub output_response_time_id: u8,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::OutputResponseTimeDescription, command_class = CommandClass::Get)]
pub struct GetOutputResponseTimeDescriptionRequest {
    pub output_response_time_id: u8,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::ModulationFrequency, command_class = CommandClass::Set)]
pub struct SetModulationFrequencyRequest {
    pub modulation_frequency_id: u8,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::ModulationFrequencyDescription, command_class = CommandClass::Get)]
pub struct GetModulationFrequencyDescriptionRequest {
    pub modulation_frequency_id: u8,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::BurnIn, command_class = CommandClass::Set)]
pub struct SetBurnInRequest {
    pub hours: u8,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::LockPin, command_class = CommandClass::Set)]
pub struct SetLockPinRequest {
    pub new_pin_code: PinCode,
    pub current_pin_code: PinCode,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::LockState, command_class = CommandClass::Set)]
pub struct SetLockStateRequest {
    pub pin_code: PinCode,
    pub lock_state: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::LockStateDescription, command_class = CommandClass::Get)]
pub struct GetLockStateDescriptionRequest {
    pub lock_state_id: u8,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::IdentifyMode, command_class = CommandClass::Set)]
pub struct SetIdentifyModeRequest {
    pub identify_mode: u8, // TODO use enum
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::PresetStatus, command_class = CommandClass::Get)]
pub struct GetPresetStatusRequest {
    pub scene_id: u16,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::PresetStatus, command_class = CommandClass::Set)]
pub struct SetPresetStatusRequest {
    pub scene_id: u16,
    pub up_fade_time: u16,
    pub down_fade_time: u16,
    pub wait_time: u16,
    pub clear_preset: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::PresetMergeMode, command_class = CommandClass::Set)]
pub struct SetPresetMergeModeRequest {
    pub merge_mode: MergeMode,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_request_parameter(pid = ParameterId::PowerOnSelfTest, command_class = CommandClass::Set)]
pub struct SetPowerOnSelfTestRequest {
    pub self_test_id: SelfTest,
}
