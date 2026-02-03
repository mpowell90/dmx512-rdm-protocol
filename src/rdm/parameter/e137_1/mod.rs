pub mod request;
pub mod response;

use rdm_core::{
    error::{ParameterCodecError, RdmError},
    parameter_traits::RdmParameterData,
};
use rdm_derive::RdmParameterData;

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

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        buf[0] = *self as u8;
        Ok(1)
    }

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
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

impl RdmParameterData for MergeMode {
    fn size_of(&self) -> usize {
        1
    }

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        buf[0] = *self as u8;
        Ok(1)
    }

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        let merge_mode =
            MergeMode::try_from(buf[0]).map_err(|_| ParameterCodecError::MalformedData)?;
        Ok(merge_mode)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, RdmParameterData)]
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

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        let value: u16 = (*self).into();
        buf[0..2].copy_from_slice(&value.to_be_bytes());
        Ok(2)
    }

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
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

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        let value: u16 = (*self).into();
        buf[0..2].copy_from_slice(&value.to_be_bytes());
        Ok(2)
    }

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        let value = u16::from_be_bytes([buf[0], buf[1]]);
        Ok(TimeMode::from(value))
    }
}
