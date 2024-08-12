pub mod error;

use core::ops::{Index, IndexMut, RangeInclusive};
use error::DmxError;

const DMX_START_CODE: u8 = 0;
const MAXIMUM_CHANNEL_COUNT: u16 = 512;

/// ## Usage
/// 
///  ```rust
/// use dmx512_rdm_protocol::dmx::DmxUniverse;
/// 
/// // Create a 512 channel universe
/// let dmx_universe = DmxUniverse::default();
/// // or create a smaller universe
/// let mut dmx_universe = DmxUniverse::new(4).unwrap();
/// 
/// dmx_universe.set_channel_value(0, 64).unwrap();
/// dmx_universe.set_channel_values(1, &[128, 192, 255]).unwrap();
/// 
/// assert_eq!(dmx_universe.get_channel_value(0).unwrap(), 64);
/// assert_eq!(dmx_universe.get_channel_values(1..=2).unwrap(), &[128, 192]);
/// assert_eq!(dmx_universe.as_bytes(), &[64, 128, 192, 255]);
/// assert_eq!(dmx_universe.encode(), &[0, 64, 128, 192, 255]);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct DmxUniverse {
    pub channel_count: u16,
    channels: Vec<u8>,
}

impl DmxUniverse {
    pub fn new(channel_count: u16) -> Result<Self, DmxError> {
        if channel_count > MAXIMUM_CHANNEL_COUNT {
            return Err(DmxError::InvalidChannelCount(channel_count));
        }

        Ok(Self {
            channel_count,
            channels: vec![0; channel_count as usize],
        })
    }

    pub fn reset(&mut self) {
        self.channels.fill(0);
    }

    pub fn get_channel_value(&self, channel: u16) -> Result<u8, DmxError> {
        if channel < self.channel_count {
            Ok(self.channels[channel as usize])
        } else {
            Err(DmxError::ChannelOutOfBounds)
        }
    }

    pub fn get_channel_values(&self, range: RangeInclusive<u16>) -> Result<&[u8], DmxError> {
        let start = *range.start();
        let end = *range.end();
        if end < self.channel_count {
            Ok(&self.channels[start as usize..=end as usize])
        } else {
            Err(DmxError::ChannelOutOfBounds)
        }
    }

    pub fn set_channel_value(&mut self, channel: u16, value: u8) -> Result<(), DmxError> {
        if channel < self.channel_count {
            self.channels[channel as usize] = value;
            Ok(())
        } else {
            Err(DmxError::ChannelOutOfBounds)
        }
    }

    pub fn set_channel_values(&mut self, channel: u16, values: &[u8]) -> Result<(), DmxError> {
        if channel + (values.len() as u16) <= self.channel_count {
            for (i, &value) in values.iter().enumerate() {
                self.channels[channel as usize + i] = value;
            }
            Ok(())
        } else {
            Err(DmxError::ChannelOutOfBounds)
        }
    }

    pub fn set_all_channel_values(&mut self, value: u8) {
        self.channels.fill(value)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.channels
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, DmxError> {
        if bytes.len() < 2 || bytes.len() > MAXIMUM_CHANNEL_COUNT as usize + 1 {
            return Err(DmxError::InvalidFrameLength(bytes.len() as u16));
        }

        if bytes[0] != DMX_START_CODE {
            return Err(DmxError::InvalidStartCode(bytes[0]));
        }

        Ok(Self {
            channel_count: (bytes.len() - 1) as u16,
            channels: bytes[1..].to_vec(),
        })
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut frame: Vec<u8> = Vec::with_capacity(self.channel_count as usize + 1);
        frame.push(DMX_START_CODE);
        frame.extend(self.channels.iter());
        frame
    }
}

impl Default for DmxUniverse {
    fn default() -> Self {
        Self {
            channel_count: MAXIMUM_CHANNEL_COUNT,
            channels: vec![0; MAXIMUM_CHANNEL_COUNT as usize],
        }
    }
}

impl Index<u16> for DmxUniverse {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        &self.channels[index as usize]
    }
}

impl IndexMut<u16> for DmxUniverse {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.channels[index as usize]
    }
}

impl TryFrom<&[u8]> for DmxUniverse {
    type Error = DmxError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        DmxUniverse::decode(bytes)
    }
}

impl From<DmxUniverse> for Vec<u8> {
    fn from(universe: DmxUniverse) -> Self {
        universe.encode()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_new_dmx_universe() {
        let universe = DmxUniverse::new(4).unwrap();
        assert_eq!(universe.channel_count, 4);
        assert_eq!(universe.channels, vec![0; 4]);
    }

    #[test]
    fn should_decode_dmx_frame() {
        let decoded = DmxUniverse::decode(&[0x00, 0x40, 0x80, 0xc0, 0xff]);

        let expected: Result<DmxUniverse, DmxError> = Ok(DmxUniverse {
            channel_count: 4,
            channels: vec![0x40, 0x80, 0xc0, 0xff],
        });

        assert_eq!(decoded, expected);
    }

    #[test]
    fn should_encode_dmx_universe() {
        let encoded = DmxUniverse {
            channel_count: 4,
            channels: vec![0x40, 0x80, 0xc0, 0xff],
        }.encode();

        let expected = vec![0x00, 0x40, 0x80, 0xc0, 0xff];

        assert_eq!(encoded, expected);
    }

    #[test]
    fn should_reset_dmx_universe() {
        let mut universe = DmxUniverse {
            channel_count: 4,
            channels: vec![255; 4],
        };

        universe.reset();

        assert_eq!(universe.channel_count, 4);
        assert_eq!(universe.channels, vec![0; 4]);
    }

    #[test]
    fn should_get_channel_value() {
        let universe = DmxUniverse {
            channel_count: 4,
            channels: vec![64, 128, 192, 255],
        };

        assert_eq!(universe.get_channel_value(2).unwrap(), 192);
    }

    #[test]
    fn should_get_channel_values() {
        let universe = DmxUniverse {
            channel_count: 4,
            channels: vec![64, 128, 192, 255],
        };

        assert_eq!(universe.get_channel_values(2..=3).unwrap(), &[192, 255]);
    }

    #[test]
    fn should_set_channel_value() {
        let mut universe = DmxUniverse {
            channel_count: 4,
            channels: vec![0; 4],
        };

        universe.set_channel_value(2, 255).unwrap();

        assert_eq!(universe.channels, vec![0, 0, 255, 0]);
    }

    #[test]
    fn should_set_channel_values() {
        let mut universe = DmxUniverse {
            channel_count: 4,
            channels: vec![0; 4],
        };

        universe.set_channel_values(0, &[64, 128, 192]).unwrap();

        assert_eq!(universe.channels, vec![64, 128, 192, 0]);
    }

    #[test]
    fn should_set_all_channel_values() {
        let mut universe = DmxUniverse {
            channel_count: 4,
            channels: vec![0; 4],
        };

        universe.set_all_channel_values(255);

        assert_eq!(universe.channels, vec![255, 255, 255, 255]);
    }

    #[test]
    fn should_return_all_channels_as_bytes() {
        let universe = DmxUniverse {
            channel_count: 4,
            channels: vec![255; 4],
        };

        assert_eq!(universe.as_bytes(), &[255, 255, 255, 255]);
    }
}
