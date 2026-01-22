use super::RdmError;
use crate::rdm::parameter::e120::{
    CurveDescription, LockStateDescription, ModulationFrequencyDescription,
    OutputResponseTimeDescription, PresetPlaybackMode,
};
use rdm_parameter_derive::{
    RdmGetRequestParameter, RdmGetResponseParameter, RdmSetRequestParameter,
};
use rdm_parameter_traits::{ParameterCodecError, RdmParameterData};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum IdentifyMode {
    Quiet = 0x00,
    Loud = 0xff,
}

impl TryFrom<u8> for IdentifyMode {
    type Error = RdmError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Quiet),
            0xff => Ok(Self::Loud),
            value => Err(RdmError::InvalidIdentifyMode(value)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PresetProgrammed {
    NotProgrammed = 0x00,
    Programmed = 0x01,
    ReadOnly = 0x02,
}

impl TryFrom<u8> for PresetProgrammed {
    type Error = RdmError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::NotProgrammed),
            0x01 => Ok(Self::Programmed),
            0x02 => Ok(Self::ReadOnly),
            value => Err(RdmError::InvalidPresetProgrammed(value)),
        }
    }
}

impl RdmParameterData for PresetProgrammed {
    fn size_of(&self) -> usize {
        1
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        buf[0] = *self as u8;
        Ok(1)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        let programmed =
            PresetProgrammed::try_from(buf[0]).map_err(|_| ParameterCodecError::MalformedData)?;
        Ok(programmed)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MergeMode {
    Default = 0x00,
    Htp = 0x01,
    Ltp = 0x02,
    DmxOnly = 0x03,
    Other = 0xff,
}

impl TryFrom<u8> for MergeMode {
    type Error = RdmError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Default),
            0x01 => Ok(Self::Htp),
            0x02 => Ok(Self::Ltp),
            0x03 => Ok(Self::DmxOnly),
            0xff => Ok(Self::Other),
            value => Err(RdmError::InvalidMergeMode(value)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PinCode(pub u16);

impl TryFrom<u16> for PinCode {
    type Error = RdmError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value > 9999 {
            Err(RdmError::InvalidPinCode(value))
        } else {
            Ok(Self(value))
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SupportedTimes {
    NotSupported,
    Time(u16),
}

impl From<u16> for SupportedTimes {
    fn from(value: u16) -> Self {
        match value {
            0xffff => Self::NotSupported,
            value => Self::Time(value),
        }
    }
}

impl From<SupportedTimes> for u16 {
    fn from(value: SupportedTimes) -> u16 {
        match value {
            SupportedTimes::NotSupported => 0xffff,
            SupportedTimes::Time(value) => value,
        }
    }
}

impl RdmParameterData for SupportedTimes {
    fn size_of(&self) -> usize {
        2
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        let value: u16 = (*self).into();
        buf[0..2].copy_from_slice(&value.to_be_bytes());
        Ok(2)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        let value = u16::from_be_bytes([buf[0], buf[1]]);
        Ok(SupportedTimes::from(value))
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TimeMode {
    Infinite,
    TenthOfSeconds(u16),
}

impl From<u16> for TimeMode {
    fn from(value: u16) -> Self {
        match value {
            0xffff => Self::Infinite,
            value => Self::TenthOfSeconds(value),
        }
    }
}

impl From<TimeMode> for u16 {
    fn from(value: TimeMode) -> u16 {
        match value {
            TimeMode::Infinite => 0xffff,
            TimeMode::TenthOfSeconds(value) => value,
        }
    }
}

impl RdmParameterData for TimeMode {
    fn size_of(&self) -> usize {
        2
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        let value: u16 = (*self).into();
        buf[0..2].copy_from_slice(&value.to_be_bytes());
        Ok(2)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        let value = u16::from_be_bytes([buf[0], buf[1]]);
        Ok(TimeMode::from(value))
    }
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetDmxBlockAddress {
    pub total_sub_device_footprint: u16,
    pub base_dmx_address: u16,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetDmxBlockAddressRequest {
    pub dmx_block_address: u16,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetDmxFailMode {
    pub scene_id: PresetPlaybackMode,
    pub loss_of_signal_delay: TimeMode,
    pub hold_time: TimeMode,
    pub level: u8,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetDmxStartupMode {
    pub scene_id: PresetPlaybackMode,
    pub startup_delay: TimeMode,
    pub hold_time: TimeMode,
    pub level: u8,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetLockState {
    pub lock_state_id: u8,
    pub lock_state_count: u8,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetLockStateDescription {
    pub lock_state_id: u8,
    pub description: LockStateDescription,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetDimmerInfo {
    pub minimum_level_lower_limit: u16,
    pub minimum_level_upper_limit: u16,
    pub maximum_level_lower_limit: u16,
    pub maximum_level_upper_limit: u16,
    pub number_of_supported_curves: u8,
    pub levels_resolution: u8,
    pub minimum_level_split_levels_supported: bool,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetMinimumLevel {
    pub minimum_level_increasing: u16,
    pub minimum_level_decreasing: u16,
    pub on_below_minimum: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetMinimumLevelRequest {
    pub minimum_level_increasing: u16,
    pub minimum_level_decreasing: u16,
    pub on_below_minimum: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetMaximumLevelRequest {
    pub maximum_level: u16,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetCurve {
    pub curve_id: u8,
    pub curve_count: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetCurveRequest {
    pub curve_id: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmGetRequestParameter)]
pub struct GetCurveDescriptionRequest {
    pub curve_id: u8,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetCurveDescription {
    pub curve_id: u8,
    pub description: CurveDescription,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetOutputResponseTime {
    pub response_time_id: u8,
    pub response_time_count: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetOutputResponseTimeRequest {
    pub output_response_time_id: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmGetRequestParameter)]
pub struct GetOutputResponseTimeDescriptionRequest {
    pub output_response_time_id: u8,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetOutputResponseTimeDescription {
    pub response_time_id: u8,
    pub description: OutputResponseTimeDescription,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetModulationFrequency {
    pub modulation_frequency_id: u8,
    pub modulation_frequency_count: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetModulationFrequencyRequest {
    pub modulation_frequency_id: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmGetRequestParameter)]
pub struct GetModulationFrequencyDescriptionRequest {
    pub modulation_frequency_id: u8,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetModulationFrequencyDescription {
    pub modulation_frequency_id: u8,
    pub frequency: u32,
    pub description: ModulationFrequencyDescription,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetBurnInRequest {
    pub hours: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetIdentifyModeRequest {
    pub identify_mode: u8,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetPresetInfo {
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

#[derive(Copy, Clone, Debug, PartialEq, RdmGetRequestParameter)]
pub struct GetPresetStatusRequest {
    pub scene_id: u16,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
pub struct GetPresetStatus {
    pub scene_id: u16,
    pub up_fade_time: u16,
    pub down_fade_time: u16,
    pub wait_time: u16,
    pub programmed: PresetProgrammed,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetPresetStatusRequest {
    pub scene_id: u16,
    pub up_fade_time: u16,
    pub down_fade_time: u16,
    pub wait_time: u16,
    pub clear_preset: bool,
}
