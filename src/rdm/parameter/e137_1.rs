use rdm_parameter_derive::{RdmGetRequestParameter, RdmSetRequestParameter};

use super::RdmError;

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

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetDmxBlockAddressRequest {
    pub dmx_block_address: u16,
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

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetCurveRequest {
    pub curve_id: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmGetRequestParameter)]
pub struct GetCurveDescriptionRequest {
    pub curve_id: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetOutputResponseTimeRequest {
    pub output_response_time_id: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmGetRequestParameter)]
pub struct GetOutputResponseTimeDescriptionRequest {
    pub output_response_time_id: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetModulationFrequencyRequest {
    pub modulation_frequency_id: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmGetRequestParameter)]
pub struct GetModulationFrequencyDescriptionRequest {
    pub modulation_frequency_id: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetBurnInRequest {
    pub hours: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetIdentifyModeRequest {
    pub identify_mode: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmGetRequestParameter)]
pub struct GetPresetStatusRequest {
    pub scene_id: u16,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
pub struct SetPresetStatusRequest {
    pub scene_id: u16,
    pub up_fade_time: u16,
    pub down_fade_time: u16,
    pub wait_time: u16,
    pub clear_preset: bool,
}
