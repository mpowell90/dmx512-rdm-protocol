use crate::{parameter::ParameterId, sensor::SensorValue, ProtocolError};

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SetResponseParameterData {
    SensorValue(SensorValue),
}

impl SetResponseParameterData {
    pub fn parse(parameter_id: ParameterId, bytes: &[u8]) -> Result<Self, ProtocolError> {
        match parameter_id {
            ParameterId::SensorValue => {
                Ok(SetResponseParameterData::SensorValue(SensorValue::new(
                    bytes[0],
                    i16::from_be_bytes(
                        bytes[1..=2]
                            .try_into()
                            .map_err(|_| ProtocolError::TryFromSliceError)?,
                    ),
                    i16::from_be_bytes(
                        bytes[3..=4]
                            .try_into()
                            .map_err(|_| ProtocolError::TryFromSliceError)?,
                    ),
                    i16::from_be_bytes(
                        bytes[5..=6]
                            .try_into()
                            .map_err(|_| ProtocolError::TryFromSliceError)?,
                    ),
                    i16::from_be_bytes(
                        bytes[7..=8]
                            .try_into()
                            .map_err(|_| ProtocolError::TryFromSliceError)?,
                    ),
                )))
            }
            _ => Err(ProtocolError::UnsupportedParameterId(parameter_id as u16)),
        }
    }
}
