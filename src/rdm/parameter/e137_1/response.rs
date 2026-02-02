use crate::rdm::parameter::{
    e120::{
        CurveDescription, LockStateDescription, ModulationFrequencyDescription,
        OutputResponseTimeDescription, PresetPlaybackMode,
    },
    e137_1::{MergeMode, PinCode, PresetProgrammed, SupportedTimes, TimeMode},
};
use rdm_core::{CommandClass, ParameterId, parameter_traits::RdmParameterData};
use rdm_derive::rdm_response_parameter;

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::DmxBlockAddress, command_class = CommandClass::GetResponse)]
pub struct GetDmxBlockAddressResponse {
    pub total_sub_device_footprint: u16,
    pub base_dmx_address: u16,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::DmxFailMode, command_class = CommandClass::GetResponse)]
pub struct GetDmxFailModeResponse {
    pub scene_id: PresetPlaybackMode,
    pub loss_of_signal_delay: TimeMode,
    pub hold_time: TimeMode,
    pub level: u8,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::DmxStartupMode, command_class = CommandClass::GetResponse)]
pub struct GetDmxStartupModeResponse {
    pub scene_id: PresetPlaybackMode,
    pub startup_delay: TimeMode,
    pub hold_time: TimeMode,
    pub level: u8,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::DimmerInfo, command_class = CommandClass::GetResponse)]
pub struct GetDimmerInfoResponse {
    pub minimum_level_lower_limit: u16,
    pub minimum_level_upper_limit: u16,
    pub maximum_level_lower_limit: u16,
    pub maximum_level_upper_limit: u16,
    pub number_of_supported_curves: u8,
    pub levels_resolution: u8,
    pub minimum_level_split_levels_supported: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::MinimumLevel, command_class = CommandClass::GetResponse)]
pub struct GetMinimumLevelResponse {
    pub minimum_level_increasing: u16,
    pub minimum_level_decreasing: u16,
    pub on_below_minimum: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::MaximumLevel, command_class = CommandClass::GetResponse)]
pub struct GetMaximumLevelResponse {
    pub maximum_level: u16,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::Curve, command_class = CommandClass::GetResponse)]
pub struct GetCurveResponse {
    pub curve_id: u8,
    pub curve_count: u8,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::CurveDescription, command_class = CommandClass::GetResponse)]
pub struct GetCurveDescriptionResponse {
    pub curve_id: u8,
    pub description: CurveDescription,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::OutputResponseTime, command_class = CommandClass::GetResponse)]
pub struct GetOutputResponseTimeResponse {
    pub response_time_id: u8,
    pub response_time_count: u8,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::OutputResponseTimeDescription, command_class = CommandClass::GetResponse)]
pub struct GetOutputResponseTimeDescriptionResponse {
    pub response_time_id: u8,
    pub description: OutputResponseTimeDescription,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::ModulationFrequency, command_class = CommandClass::GetResponse)]
pub struct GetModulationFrequencyResponse {
    pub modulation_frequency_id: u8,
    pub modulation_frequency_count: u8,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::ModulationFrequencyDescription, command_class = CommandClass::GetResponse)]
pub struct GetModulationFrequencyDescriptionResponse {
    pub modulation_frequency_id: u8,
    pub frequency: u32,
    pub description: ModulationFrequencyDescription,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::BurnIn, command_class = CommandClass::GetResponse)]
pub struct GetBurnInResponse {
    pub hours_remaining: u8,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::LockPin, command_class = CommandClass::GetResponse)]
pub struct GetLockPinResponse {
    pub current_pin_code: PinCode,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::LockState, command_class = CommandClass::GetResponse)]
pub struct GetLockStateResponse {
    pub lock_state_id: u8,
    pub lock_state_count: u8,
}
#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::LockStateDescription, command_class = CommandClass::GetResponse)]
pub struct GetLockStateDescriptionResponse {
    pub lock_state_id: u8,
    pub description: LockStateDescription,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::IdentifyMode, command_class = CommandClass::GetResponse)]
pub struct GetIdentifyModeResponse {
    pub identify_mode: u8, // TODO use enum
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::PresetInfo, command_class = CommandClass::GetResponse)]
pub struct GetPresetInfoResponse {
    pub level_field_supported: bool,
    pub preset_sequence_supported: bool,
    pub split_times_supported: bool,
    pub dmx_fail_infinite_delay_time_supported: bool,
    pub dmx_fail_infinite_hold_time_supported: bool,
    pub startup_infinite_hold_time_supported: bool,
    pub maximum_scene_number: u16,
    pub minimum_preset_fade_time_supported: u16,
    pub maximum_preset_fade_time_supported: u16,
    pub minimum_preset_wait_time_supported: u16,
    pub maximum_preset_wait_time_supported: u16,
    pub minimum_dmx_fail_delay_time_supported: SupportedTimes,
    pub maximum_dmx_fail_delay_time_supported: SupportedTimes,
    pub minimum_dmx_fail_hold_time_supported: SupportedTimes,
    pub maximum_dmx_fail_hold_time_supported: SupportedTimes,
    pub minimum_startup_delay_time_supported: SupportedTimes,
    pub maximum_startup_delay_time_supported: SupportedTimes,
    pub minimum_startup_hold_time_supported: SupportedTimes,
    pub maximum_startup_hold_time_supported: SupportedTimes,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::PresetMergeMode, command_class = CommandClass::GetResponse)]
pub struct GetPresetMergeModeResponse {
    pub merge_mode: MergeMode,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::PresetStatus, command_class = CommandClass::GetResponse)]
pub struct GetPresetStatusResponse {
    pub scene_id: u16,
    pub up_fade_time: u16,
    pub down_fade_time: u16,
    pub wait_time: u16,
    pub programmed: PresetProgrammed,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::PowerOnSelfTest, command_class = CommandClass::GetResponse)]
pub struct GetPowerOnSelfTestResponse {
    pub power_on_self_test: bool,
}
