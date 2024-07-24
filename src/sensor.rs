use crate::ProtocolError;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SensorType {
    Temperature = 0x00,
    Voltage = 0x01,
    Current = 0x02,
    Frequency = 0x03,
    Resistance = 0x04,
    Power = 0x05,
    Mass = 0x06,
    Length = 0x07,
    Area = 0x08,
    Volume = 0x09,
    Density = 0x0a,
    Velocity = 0x0b,
    Acceleration = 0x0c,
    Force = 0x0d,
    Energy = 0x0e,
    Pressure = 0x0f,
    Time = 0x10,
    Angle = 0x11,
    PositionX = 0x12,
    PositionY = 0x13,
    PositionZ = 0x14,
    AngularVelocity = 0x15,
    LuminousIntensity = 0x16,
    LuminousFlux = 0x17,
    Illuminance = 0x18,
    ChrominanceRed = 0x19,
    ChrominanceGreen = 0x1a,
    ChrominanceBlue = 0x1b,
    Contacts = 0x1c,
    Memory = 0x1d,
    Items = 0x1e,
    Humidity = 0x1f,
    Counter16Bit = 0x20,
    Other = 0x7f,
}

impl TryFrom<u8> for SensorType {
    type Error = ProtocolError;
    fn try_from(value: u8) -> Result<Self, ProtocolError> {
        match value {
            0x00 => Ok(Self::Temperature),
            0x01 => Ok(Self::Voltage),
            0x02 => Ok(Self::Current),
            0x03 => Ok(Self::Frequency),
            0x04 => Ok(Self::Resistance),
            0x05 => Ok(Self::Power),
            0x06 => Ok(Self::Mass),
            0x07 => Ok(Self::Length),
            0x08 => Ok(Self::Area),
            0x09 => Ok(Self::Volume),
            0x0a => Ok(Self::Density),
            0x0b => Ok(Self::Velocity),
            0x0c => Ok(Self::Acceleration),
            0x0d => Ok(Self::Force),
            0x0e => Ok(Self::Energy),
            0x0f => Ok(Self::Pressure),
            0x10 => Ok(Self::Time),
            0x11 => Ok(Self::Angle),
            0x12 => Ok(Self::PositionX),
            0x13 => Ok(Self::PositionY),
            0x14 => Ok(Self::PositionZ),
            0x15 => Ok(Self::AngularVelocity),
            0x16 => Ok(Self::LuminousIntensity),
            0x17 => Ok(Self::LuminousFlux),
            0x18 => Ok(Self::Illuminance),
            0x19 => Ok(Self::ChrominanceRed),
            0x1a => Ok(Self::ChrominanceGreen),
            0x1b => Ok(Self::ChrominanceBlue),
            0x1c => Ok(Self::Contacts),
            0x1d => Ok(Self::Memory),
            0x1e => Ok(Self::Items),
            0x1f => Ok(Self::Humidity),
            0x20 => Ok(Self::Counter16Bit),
            0x7f => Ok(Self::Other),
            _ => Err(ProtocolError::InvalidSensorType(value)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SensorValue {
    pub sensor_id: u8,
    pub current_value: i16,
    pub lowest_detected_value: i16,
    pub highest_detected_value: i16,
    pub recorded_value: i16,
}

impl SensorValue {
    pub fn new(
        sensor_id: u8,
        current_value: i16,
        lowest_detected_value: i16,
        highest_detected_value: i16,
        recorded_value: i16
    ) -> Self {
        Self {
            sensor_id,
            current_value,
            lowest_detected_value,
            highest_detected_value,
            recorded_value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sensor {
    pub id: u8,
    pub kind: SensorType,
    pub unit: u8,
    pub prefix: u8,
    pub range_minimum_value: i16,
    pub range_maximum_value: i16,
    pub normal_minimum_value: i16,
    pub normal_maximum_value: i16,
    pub recorded_value_support: u8,
    pub description: String,
}
