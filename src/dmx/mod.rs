//! Data types and functionality for encoding and decoding DMX512 packets
//!
//! ### DmxUniverse
//!
//! ```rust
//! use dmx512_rdm_protocol::dmx::DmxUniverse;
//!
//! // Create a 512 channel universe
//! let dmx_universe = DmxUniverse::default();
//! // or create a smaller universe
//! let dmx_universe = DmxUniverse::new(4).unwrap();
//! // or decode a dmx packet
//! let mut dmx_universe = DmxUniverse::decode(&[0, 0, 0, 0, 0]).unwrap();
//! 
//! assert_eq!(dmx_universe.as_slice(), &[0, 0, 0, 0]);
//! 
//! dmx_universe.set_channel_value(0, 64).unwrap();
//! dmx_universe.set_channel_values(1, &[128, 192, 255]).unwrap();
//!
//! assert_eq!(dmx_universe.get_channel_value(0).unwrap(), 64);
//! assert_eq!(dmx_universe.get_channel_values(1..=2).unwrap(), &[128, 192]);
//! assert_eq!(dmx_universe.as_slice(), &[64, 128, 192, 255]);
//! assert_eq!(dmx_universe.encode(), &[0, 64, 128, 192, 255]);
//! ```

pub mod error;

use core::ops::{Index, IndexMut, RangeInclusive};
use error::DmxError;

const DMX_START_CODE: u8 = 0;
const MAXIMUM_CHANNEL_COUNT: u16 = 512;

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

    pub fn as_slice(&self) -> &[u8] {
        self.channels.as_slice()
    }

    pub fn extend(&mut self, values: &[u8]) -> Result<(), DmxError> {
        if self.channel_count as usize + values.len() > MAXIMUM_CHANNEL_COUNT as usize {
            return Err(DmxError::InvalidChannelCount(self.channels.len() as u16 + values.len() as u16));
        }

        self.channels.extend(values);
        self.channel_count += values.len() as u16;

        Ok(())
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
        if bytes.len() as u16 > MAXIMUM_CHANNEL_COUNT {
            return Err(DmxError::InvalidChannelCount(bytes.len() as u16));
        }

        Ok(DmxUniverse {
            channel_count: bytes.len() as u16,
            channels: bytes.to_vec(),
        })
    }
}

impl TryFrom<Vec<u8>> for DmxUniverse {
    type Error = DmxError;

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        if bytes.len() as u16 > MAXIMUM_CHANNEL_COUNT {
            return Err(DmxError::InvalidChannelCount(bytes.len() as u16));
        }

        Ok(DmxUniverse {
            channel_count: bytes.len() as u16,
            channels: bytes,
        })
    }
}

impl From<DmxUniverse> for Vec<u8> {
    fn from(universe: DmxUniverse) -> Self {
        universe.channels
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
    fn should_create_new_dmx_universe_from_byte_slice() {
        let bytes = [0_u8; 513];

        let universe = DmxUniverse::try_from(&bytes[..]);
        assert_eq!(universe, Err(DmxError::InvalidChannelCount(513)));
        
        let bytes = [0x40, 0x80, 0xc0, 0xff];

        let universe = DmxUniverse::try_from(&bytes[..]).unwrap();
        assert_eq!(universe.channel_count, 4);
        assert_eq!(universe.channels, vec![0x40, 0x80, 0xc0, 0xff]);

        let universe: DmxUniverse = (&bytes[..]).try_into().unwrap();
        assert_eq!(universe.channel_count, 4);
        assert_eq!(universe.channels, vec![0x40, 0x80, 0xc0, 0xff]);
    }

    #[test]
    fn should_create_new_dmx_universe_from_byte_vec() {
        let bytes = vec![0_u8; 513];

        let universe = DmxUniverse::try_from(bytes);
        assert_eq!(universe, Err(DmxError::InvalidChannelCount(513)));

        let bytes = vec![0x40, 0x80, 0xc0, 0xff];
        
        let universe = DmxUniverse::try_from(bytes.clone()).unwrap();
        assert_eq!(universe.channel_count, 4);
        assert_eq!(universe.channels, vec![0x40, 0x80, 0xc0, 0xff]);

        let universe: DmxUniverse = bytes.try_into().unwrap();
        assert_eq!(universe.channel_count, 4);
        assert_eq!(universe.channels, vec![0x40, 0x80, 0xc0, 0xff]);
    }

    #[test]
    fn should_create_byte_vec_from_new_dmx_universe() {
        let universe = DmxUniverse {
            channel_count: 4,
            channels: vec![0x40, 0x80, 0xc0, 0xff],
        };
        
        assert_eq!(Vec::from(universe.clone()), vec![0x40, 0x80, 0xc0, 0xff]);

        let bytes: Vec<u8> = universe.into();
        assert_eq!(bytes, vec![0x40, 0x80, 0xc0, 0xff]);
    }

    #[test]
    fn should_decode_dmx_frame() {
        let bytes = [0_u8; 514];

        let universe = DmxUniverse::decode(&bytes[..]);
        assert_eq!(universe, Err(DmxError::InvalidFrameLength(514)));

        let decoded = DmxUniverse::decode(&[0x00, 0x40, 0x80, 0xc0, 0xff]).unwrap();

        let expected = DmxUniverse {
            channel_count: 4,
            channels: vec![0x40, 0x80, 0xc0, 0xff],
        };

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
            channels: vec![0x40, 0x80, 0xc0, 0xff],
        };

        assert_eq!(universe.get_channel_value(2).unwrap(), 192);

        assert_eq!(universe.get_channel_value(4), Err(DmxError::ChannelOutOfBounds));
    }

    #[test]
    fn should_get_channel_values() {
        let universe = DmxUniverse {
            channel_count: 4,
            channels: vec![0x40, 0x80, 0xc0, 0xff],
        };

        assert_eq!(universe.get_channel_values(2..=3).unwrap(), &[192, 255]);

        assert_eq!(universe.get_channel_values(2..=5), Err(DmxError::ChannelOutOfBounds));
        assert_eq!(universe.get_channel_values(4..=5), Err(DmxError::ChannelOutOfBounds));
    }

    #[test]
    fn should_set_channel_value() {
        let mut universe = DmxUniverse {
            channel_count: 4,
            channels: vec![0; 4],
        };

        universe.set_channel_value(2, 0xff).unwrap();

        assert_eq!(universe.channels, vec![0x00, 0x00, 0xff, 0x00]);
        assert_eq!(universe.set_channel_value(4, 0xff), Err(DmxError::ChannelOutOfBounds));
    }

    #[test]
    fn should_set_channel_values() {
        let mut universe = DmxUniverse {
            channel_count: 4,
            channels: vec![0; 4],
        };

        universe.set_channel_values(0, &[0x40, 0x80, 0xc0]).unwrap();

        assert_eq!(universe.channels, vec![0x40, 0x80, 0xc0, 0]);

        assert_eq!(universe.set_channel_values(2, &[0xff, 0xff, 0xff]), Err(DmxError::ChannelOutOfBounds));
        assert_eq!(universe.set_channel_values(4, &[0xff]), Err(DmxError::ChannelOutOfBounds));
    }

    #[test]
    fn should_set_all_channel_values() {
        let mut universe = DmxUniverse {
            channel_count: 4,
            channels: vec![0; 4],
        };

        universe.set_all_channel_values(0xff);

        assert_eq!(universe.channels, vec![0xff, 0xff, 0xff, 0xff]);
    }

    #[test]
    fn should_return_all_channels_as_slice() {
        let universe = DmxUniverse {
            channel_count: 4,
            channels: vec![255; 4],
        };

        assert_eq!(universe.as_slice(), &[0xff, 0xff, 0xff, 0xff]);
    }

    #[test]
    fn should_extend_channels_with_byte_slice() {
        let mut universe = DmxUniverse {
            channel_count: 4,
            channels: vec![255; 4],
        };

        universe.extend(&[0, 0, 0, 0]).unwrap();

        assert_eq!(universe.channels, vec![0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(universe.channel_count, 8);

        assert_eq!(universe.extend(&[0xff; 512][..]), Err(DmxError::InvalidChannelCount(520)));
    }
}
