use crate::{device::DeviceUID, parameter::ParameterId, ProtocolError};

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiscoveryResponseParameterData {
    DiscMute {
        control_field: u16,
        binding_uid: Option<DeviceUID>,
    },
    DiscUnmute {
        control_field: u16,
        binding_uid: Option<DeviceUID>,
    },
}

impl DiscoveryResponseParameterData {
    pub fn parse(parameter_id: ParameterId, bytes: &[u8]) -> Result<Self, ProtocolError> {
        // TODO dedupe code
        match parameter_id {
            ParameterId::DiscMute => {
                let binding_uid = if bytes.len() > 2 {
                    let manufacturer_id = u16::from_be_bytes(bytes[2..=3].try_into().unwrap());
                    let device_id = u32::from_be_bytes(bytes[4..=7].try_into().unwrap());
                    Some(DeviceUID::new(manufacturer_id, device_id))
                } else {
                    None
                };

                Ok(DiscoveryResponseParameterData::DiscMute {
                    control_field: u16::from_be_bytes(bytes[..=1].try_into().unwrap()),
                    binding_uid,
                })
            }
            ParameterId::DiscUnMute => {
                let binding_uid = if bytes.len() > 2 {
                    let manufacturer_id = u16::from_be_bytes(bytes[2..=3].try_into().unwrap());
                    let device_id = u32::from_be_bytes(bytes[4..=7].try_into().unwrap());
                    Some(DeviceUID::new(manufacturer_id, device_id))
                } else {
                    None
                };

                Ok(DiscoveryResponseParameterData::DiscMute {
                    control_field: u16::from_be_bytes(bytes[..=1].try_into().unwrap()),
                    binding_uid,
                })
            }
            _ => Err(ProtocolError::UnsupportedParameterId(parameter_id as u16)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_parse_discovery_mute() {
        assert_eq!(
            DiscoveryResponseParameterData::parse(ParameterId::DiscMute, &[0x00, 0x01]),
            Ok(DiscoveryResponseParameterData::DiscMute {
                control_field: 0x0001,
                binding_uid: None
            })
        );
        assert_eq!(
            DiscoveryResponseParameterData::parse(
                ParameterId::DiscMute,
                &[0x00, 0x01, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06]
            ),
            Ok(DiscoveryResponseParameterData::DiscMute {
                control_field: 0x0001,
                binding_uid: Some(DeviceUID::new(0x0102, 0x03040506))
            })
        );
    }

    #[test]
    fn should_parse_discovery_unmute() {
        assert_eq!(
            DiscoveryResponseParameterData::parse(ParameterId::DiscUnMute, &[0x00, 0x01]),
            Ok(DiscoveryResponseParameterData::DiscMute {
                control_field: 0x0001,
                binding_uid: None
            })
        );
        assert_eq!(
            DiscoveryResponseParameterData::parse(
                ParameterId::DiscUnMute,
                &[0x00, 0x01, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06]
            ),
            Ok(DiscoveryResponseParameterData::DiscMute {
                control_field: 0x0001,
                binding_uid: Some(DeviceUID::new(0x0102, 0x03040506))
            })
        );
    }
}
