use crate::{parameter::ParameterId, ProtocolError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SetResponseParameterData {
    SensorValue {
        sensor_id: u8,
        current_value: u16,
        lowest_detected_value: u16,
        highest_detected_value: u16,
        recorded_value: u16,
    },
}

impl SetResponseParameterData {
    pub fn parse(parameter_id: ParameterId, bytes: &[u8]) -> Result<Self, ProtocolError> {
        match parameter_id {
            ParameterId::SensorValue => {
                Ok(SetResponseParameterData::SensorValue {
                    sensor_id: bytes[0],
                    current_value: u16::from_be_bytes(bytes[1..=2].try_into().unwrap()),
                    lowest_detected_value: u16::from_be_bytes(bytes[3..=4].try_into().unwrap()),
                    highest_detected_value: u16::from_be_bytes(bytes[5..=6].try_into().unwrap()),
                    recorded_value: u16::from_be_bytes(bytes[7..=8].try_into().unwrap()),
                })
            },
            _ => Err(ProtocolError::UnsupportedParameterId(parameter_id as u16)),
        }
    }
}
