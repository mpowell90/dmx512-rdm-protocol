use crate::{parameter::ParameterId, ProtocolError};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SetResponseParameterData {
    DeviceLabel,
    DmxPersonality,
    DmxStartAddress,
    Curve,
    ModulationFrequency,
    OutputResponseTime,
    IdentifyDevice,
}

impl SetResponseParameterData {
    pub fn parse(_parameter_id: ParameterId, _bytes: &[u8]) -> Result<Option<Self>, ProtocolError> {
        Ok(None)
    }
}
