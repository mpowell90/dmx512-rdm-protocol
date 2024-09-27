//! Data types and functionality for encoding and decoding DMX512 packets
//!
//! ### DmxUniverse
//!
//! ```rust
//! use dmx512_rdm_protocol::dmx::DmxUniverse;
//! 
//! #[cfg(feature = "alloc")]
//! fn with_alloc() {
//!     // Create a 512 channel universe
//!     let dmx_universe = DmxUniverse::default();
//!     // or create a smaller universe
//!     let dmx_universe = DmxUniverse::new(4).unwrap();
//!     // or decode a dmx packet
//!     let mut dmx_universe = DmxUniverse::decode(&[0, 255, 255, 0, 0]).unwrap();
//!
//!     assert_eq!(dmx_universe.as_slice(), &[255, 255, 0, 0]);
//!
//!     dmx_universe.set_channel_value(0, 64).unwrap();
//!     dmx_universe.set_channel_values(1, &[128, 192, 255]).unwrap();
//! 
//!     assert_eq!(dmx_universe.get_channel_value(0).unwrap(), 64);
//!     assert_eq!(dmx_universe.get_channel_values(1..=2).unwrap(), &[128, 192]);
//!     assert_eq!(dmx_universe.as_slice(), &[64, 128, 192, 255]);
//!     assert_eq!(dmx_universe.encode(), &[0, 64, 128, 192, 255]);
//! }
//! 
//! #[cfg(not(feature = "alloc"))]
//! fn without_alloc() {
//!     // Create a 512 channel universe on the stack
//!     let dmx_universe = DmxUniverse::default();
//! 
//!     // or decode a dmx packet
//!     let mut dmx_universe = DmxUniverse::decode(&[0, 255, 255, 0, 0]).unwrap();
//!
//!     assert_eq!(dmx_universe.get_channel_values(0..=1).unwrap(), &[255, 255]);
//!
//!     dmx_universe.set_channel_value(0, 64).unwrap();
//!     dmx_universe.set_channel_values(1, &[128, 192, 255]).unwrap();
//! 
//!     assert_eq!(dmx_universe.get_channel_value(0).unwrap(), 64);
//!     assert_eq!(dmx_universe.get_channel_values(1..=2).unwrap(), &[128, 192]);
//!     assert_eq!(dmx_universe.as_slice(), &[64, 128, 192, 255]);
//!     assert_eq!(dmx_universe.encode(), &[0, 64, 128, 192, 255]);
//! }
//! ```
//!
pub mod error;
pub const DMX_START_CODE: u8 = 0;
#[cfg(feature = "alloc")]
pub const MAXIMUM_CHANNEL_COUNT: u16 = 512;
#[cfg(not(feature = "alloc"))]
pub const MAXIMUM_CHANNEL_COUNT: usize = 512;

use core::ops::{Index, IndexMut, RangeInclusive};
use error::DmxError;

#[cfg(not(feature = "alloc"))]
use heapless::Vec;

#[cfg(feature = "alloc")]
#[derive(Clone, Debug, PartialEq)]
pub struct DmxUniverse {
    pub channel_count: u16,
    channels: Vec<u8>,
}

#[cfg(not(feature = "alloc"))]
#[derive(Clone, Debug, PartialEq)]
pub struct DmxUniverse(Vec<u8, MAXIMUM_CHANNEL_COUNT>);

impl DmxUniverse {
    #[cfg(feature = "alloc")]
    pub fn new(channel_count: u16) -> Result<Self, DmxError> {
        if channel_count > MAXIMUM_CHANNEL_COUNT {
            return Err(DmxError::InvalidChannelCount(channel_count));
        }

        Ok(Self {
            channel_count,
            channels: vec![0; channel_count as usize],
        })
    }
    #[cfg(not(feature = "alloc"))]
    pub fn new() -> Self {
        Self(Vec::from_slice(&[0; MAXIMUM_CHANNEL_COUNT]).unwrap())
    }

    pub fn reset(&mut self) {
        #[cfg(feature = "alloc")]
        self.channels.fill(0);
        #[cfg(not(feature = "alloc"))]
        self.0.fill(0);
    }

    #[cfg(feature = "alloc")]
    pub fn get_channel_value(&self, channel: u16) -> Result<u8, DmxError> {
        if channel < self.channel_count {
            Ok(self.channels[channel as usize])
        } else {
            Err(DmxError::ChannelOutOfBounds)
        }
    }
    #[cfg(not(feature = "alloc"))]
    pub fn get_channel_value(&self, channel: u16) -> Result<u8, DmxError> {
        if channel < MAXIMUM_CHANNEL_COUNT as u16 {
            Ok(self.0[channel as usize])
        } else {
            Err(DmxError::ChannelOutOfBounds)
        }
    }

    #[cfg(feature = "alloc")]
    pub fn get_channel_values(&self, range: RangeInclusive<u16>) -> Result<&[u8], DmxError> {
        let start = *range.start();
        let end = *range.end();
        if end < self.channel_count {
            Ok(&self.channels[start as usize..=end as usize])
        } else {
            Err(DmxError::ChannelOutOfBounds)
        }
    }

    #[cfg(not(feature = "alloc"))]
    pub fn get_channel_values(&self, range: RangeInclusive<u16>) -> Result<&[u8], DmxError> {
        let start = *range.start();
        let end = *range.end();
        if end < MAXIMUM_CHANNEL_COUNT as u16 {
            Ok(&self.0[start as usize..=end as usize])
        } else {
            Err(DmxError::ChannelOutOfBounds)
        }
    }

    #[cfg(feature = "alloc")]
    pub fn set_channel_value(&mut self, channel: u16, value: u8) -> Result<(), DmxError> {
        if channel < self.channel_count {
            self.channels[channel as usize] = value;

            Ok(())
        } else {
            Err(DmxError::ChannelOutOfBounds)
        }
    }
    #[cfg(not(feature = "alloc"))]
    pub fn set_channel_value(&mut self, channel: u16, value: u8) -> Result<(), DmxError> {
        if channel < MAXIMUM_CHANNEL_COUNT as u16 {
            self.0[channel as usize] = value;
            Ok(())
        } else {
            Err(DmxError::ChannelOutOfBounds)
        }
    }

    #[cfg(feature = "alloc")]
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
    #[cfg(not(feature = "alloc"))]
    pub fn set_channel_values(&mut self, channel: u16, values: &[u8]) -> Result<(), DmxError> {
        if channel + (values.len() as u16) <= MAXIMUM_CHANNEL_COUNT as u16 {
            for (i, &value) in values.iter().enumerate() {
                self.0[channel as usize + i] = value;
            }
            Ok(())
        } else {
            Err(DmxError::ChannelOutOfBounds)
        }
    }

    pub fn set_all_channel_values(&mut self, value: u8) {
        #[cfg(feature = "alloc")]
        self.channels.fill(value);
        #[cfg(not(feature = "alloc"))]
        self.0.fill(value);
    }

    pub fn as_slice(&self) -> &[u8] {
        #[cfg(feature = "alloc")]
        return self.channels.as_slice();
        #[cfg(not(feature = "alloc"))]
        self.0.as_slice()
    }

    #[cfg(not(feature = "alloc"))]
    pub fn from_slice(bytes: &[u8]) -> Result<Self, DmxError> {
        if bytes.len() > MAXIMUM_CHANNEL_COUNT {
            return Err(DmxError::InvalidChannelCount(bytes.len() as u16));
        }

        let mut universe = Self::new();

        universe.0[0..bytes.len()].copy_from_slice(bytes);

        Ok(universe)
    }

    #[cfg(feature = "alloc")]
    pub fn extend(&mut self, values: &[u8]) -> Result<(), DmxError> {
        if self.channel_count as usize + values.len() > MAXIMUM_CHANNEL_COUNT as usize {
            return Err(DmxError::InvalidChannelCount(
                self.channels.len() as u16 + values.len() as u16,
            ));
        }

        self.channels.extend(values);
        self.channel_count += values.len() as u16;

        Ok(())
    }

    #[cfg(feature = "alloc")]
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
    #[cfg(not(feature = "alloc"))]
    pub fn decode(bytes: &[u8]) -> Result<Self, DmxError> {
        if bytes.len() < 2 || bytes.len() > MAXIMUM_CHANNEL_COUNT + 1 {
            return Err(DmxError::InvalidFrameLength(bytes.len() as u16));
        }

        if bytes[0] != DMX_START_CODE {
            return Err(DmxError::InvalidStartCode(bytes[0]));
        }

        Self::from_slice(&bytes[1..])
    }

    #[cfg(feature = "alloc")]
    pub fn encode(&self) -> Vec<u8> {
        let mut frame: Vec<u8> = Vec::with_capacity(self.channel_count as usize + 1);

        frame.push(DMX_START_CODE);
        frame.extend(self.channels.iter());

        frame
    }
    #[cfg(not(feature = "alloc"))]
    pub fn encode(&self) -> Vec<u8, 513> {
        let mut frame = Vec::<u8, 513>::new();

        frame.push(DMX_START_CODE).unwrap();
        frame.extend_from_slice(&self.0[..]).unwrap();

        frame
    }
}

impl Default for DmxUniverse {
    #[cfg(feature = "alloc")]
    fn default() -> Self {
        Self {
            channel_count: MAXIMUM_CHANNEL_COUNT,
            channels: vec![0; MAXIMUM_CHANNEL_COUNT as usize],
        }
    }
    #[cfg(not(feature = "alloc"))]
    fn default() -> Self {
        Self::new()
    }
}

impl Index<u16> for DmxUniverse {
    type Output = u8;

    #[cfg(feature = "alloc")]
    fn index(&self, index: u16) -> &Self::Output {
        &self.channels[index as usize]
    }
    #[cfg(not(feature = "alloc"))]
    fn index(&self, index: u16) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<u16> for DmxUniverse {
    #[cfg(feature = "alloc")]
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.channels[index as usize]
    }
    #[cfg(not(feature = "alloc"))]
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl TryFrom<&[u8]> for DmxUniverse {
    type Error = DmxError;

    #[cfg(feature = "alloc")]
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() as u16 > MAXIMUM_CHANNEL_COUNT {
            return Err(DmxError::InvalidChannelCount(bytes.len() as u16));
        }

        Ok(DmxUniverse {
            channel_count: bytes.len() as u16,
            channels: bytes.to_vec(),
        })
    }
    #[cfg(not(feature = "alloc"))]
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::from_slice(bytes)
    }
}

#[cfg(feature = "alloc")]
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

#[cfg(feature = "alloc")]
impl From<DmxUniverse> for Vec<u8> {
    fn from(universe: DmxUniverse) -> Self {
        universe.channels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "alloc")]
    #[test]
    fn should_create_new_dmx_universe() {
        let universe = DmxUniverse::new(4).unwrap();
        assert_eq!(universe.channel_count, 4);
        assert_eq!(universe.channels, vec![0; 4]);
    }

    #[cfg(not(feature = "alloc"))]
    #[test]
    fn should_create_new_dmx_universe() {
        let universe = DmxUniverse::new();
        assert_eq!(universe.0, Vec::<u8, 512>::from_slice(&[0; 512]).unwrap());
    }

    #[cfg(feature = "alloc")]
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

    #[cfg(not(feature = "alloc"))]
    #[test]
    fn should_create_new_dmx_universe_from_byte_slice() {
        let bytes = [0_u8; 513];

        let universe = DmxUniverse::try_from(&bytes[..]);
        assert_eq!(universe, Err(DmxError::InvalidChannelCount(513)));

        let bytes = [0x40, 0x80, 0xc0, 0xff];

        let mut expected = Vec::<u8, 512>::from_slice(&[0; 512]).unwrap();
        expected[0..4].copy_from_slice(&bytes);

        let universe = DmxUniverse::try_from(&bytes[..]).unwrap();
        assert_eq!(universe.0, expected);

        let universe: DmxUniverse = (&bytes[..]).try_into().unwrap();
        assert_eq!(universe.0, expected);
    }

    #[cfg(feature = "alloc")]
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

    #[cfg(feature = "alloc")]
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

    #[cfg(feature = "alloc")]
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

    #[cfg(not(feature = "alloc"))]
    #[test]
    fn should_decode_dmx_frame() {
        let bytes = [0_u8; 514];

        let universe = DmxUniverse::decode(&bytes[..]);
        assert_eq!(universe, Err(DmxError::InvalidFrameLength(514)));

        let decoded = DmxUniverse::decode(&[0x00, 0x40, 0x80, 0xc0, 0xff]).unwrap();

        let mut expected = DmxUniverse(Vec::<u8, 512>::from_slice(&[0; 512]).unwrap());
        expected.0[0..4].copy_from_slice(&[0x40, 0x80, 0xc0, 0xff]);

        assert_eq!(decoded, expected);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn should_encode_dmx_universe() {
        let encoded = DmxUniverse {
            channel_count: 4,
            channels: vec![0x40, 0x80, 0xc0, 0xff],
        }
        .encode();

        let expected = vec![0x00, 0x40, 0x80, 0xc0, 0xff];

        assert_eq!(encoded, expected);
    }

    #[cfg(not(feature = "alloc"))]
    #[test]
    fn should_encode_dmx_universe() {
        let mut universe = DmxUniverse::new();
        universe.0[0..4].copy_from_slice(&[0x40, 0x80, 0xc0, 0xff]);

        let encoded = universe.encode();

        let mut expected = Vec::<u8, 513>::from_slice(&[0; 513]).unwrap();
        expected[0..5].copy_from_slice(&[0x00, 0x40, 0x80, 0xc0, 0xff]);

        assert_eq!(encoded, expected);
    }

    #[cfg(feature = "alloc")]
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

    #[cfg(not(feature = "alloc"))]
    #[test]
    fn should_reset_dmx_universe() {
        let mut universe = DmxUniverse(Vec::<u8, 512>::from_slice(&[255; 512]).unwrap());

        universe.reset();

        assert_eq!(universe.0, Vec::<u8, 512>::from_slice(&[0; 512]).unwrap());
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn should_get_channel_value() {
        let universe = DmxUniverse {
            channel_count: 4,
            channels: vec![0x40, 0x80, 0xc0, 0xff],
        };

        assert_eq!(universe.get_channel_value(2).unwrap(), 192);

        assert_eq!(
            universe.get_channel_value(4),
            Err(DmxError::ChannelOutOfBounds)
        );
    }

    #[cfg(not(feature = "alloc"))]
    #[test]
    fn should_get_channel_value() {
        let mut universe = DmxUniverse::new();
        universe.0[0..4].copy_from_slice(&[0x40, 0x80, 0xc0, 0xff]);

        assert_eq!(universe.get_channel_value(2).unwrap(), 192);

        assert_eq!(
            universe.get_channel_value(513),
            Err(DmxError::ChannelOutOfBounds)
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn should_get_channel_values() {
        let universe = DmxUniverse {
            channel_count: 4,
            channels: vec![0x40, 0x80, 0xc0, 0xff],
        };

        assert_eq!(universe.get_channel_values(2..=3).unwrap(), &[192, 255]);

        assert_eq!(
            universe.get_channel_values(2..=5),
            Err(DmxError::ChannelOutOfBounds)
        );
        assert_eq!(
            universe.get_channel_values(4..=5),
            Err(DmxError::ChannelOutOfBounds)
        );
    }

    #[cfg(not(feature = "alloc"))]
    #[test]
    fn should_get_channel_values() {
        let universe = DmxUniverse::from_slice(&[0x40, 0x80, 0xc0, 0xff]).unwrap();

        assert_eq!(universe.get_channel_values(2..=3).unwrap(), &[192, 255]);
        assert_eq!(
            universe.get_channel_values(510..=513),
            Err(DmxError::ChannelOutOfBounds)
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn should_set_channel_value() {
        let mut universe = DmxUniverse {
            channel_count: 4,
            channels: vec![0; 4],
        };

        universe.set_channel_value(2, 0xff).unwrap();

        assert_eq!(universe.channels, vec![0x00, 0x00, 0xff, 0x00]);
        assert_eq!(
            universe.set_channel_value(4, 0xff),
            Err(DmxError::ChannelOutOfBounds)
        );
    }

    #[cfg(not(feature = "alloc"))]
    #[test]
    fn should_set_channel_value() {
        let mut universe = DmxUniverse::new();

        universe.set_channel_value(2, 0xff).unwrap();

        assert_eq!(universe.0[2], 0xff);
        assert_eq!(
            universe.set_channel_value(512, 0xff),
            Err(DmxError::ChannelOutOfBounds)
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn should_set_channel_values() {
        let mut universe = DmxUniverse {
            channel_count: 4,
            channels: vec![0; 4],
        };

        universe.set_channel_values(0, &[0x40, 0x80, 0xc0]).unwrap();

        assert_eq!(universe.channels, vec![0x40, 0x80, 0xc0, 0]);

        assert_eq!(
            universe.set_channel_values(2, &[0xff, 0xff, 0xff]),
            Err(DmxError::ChannelOutOfBounds)
        );
        assert_eq!(
            universe.set_channel_values(4, &[0xff]),
            Err(DmxError::ChannelOutOfBounds)
        );
    }

    #[cfg(not(feature = "alloc"))]
    #[test]
    fn should_set_channel_values() {
        let mut universe = DmxUniverse::new();

        universe.set_channel_values(0, &[0x40, 0x80, 0xc0]).unwrap();

        let mut expected = DmxUniverse::new();
        expected.0[0..3].copy_from_slice(&[0x40, 0x80, 0xc0]);

        assert_eq!(universe.0, expected.0);
        assert_eq!(
            universe.set_channel_values(510, &[0xff, 0xff, 0xff]),
            Err(DmxError::ChannelOutOfBounds)
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn should_set_all_channel_values() {
        let mut universe = DmxUniverse {
            channel_count: 4,
            channels: vec![0; 4],
        };

        universe.set_all_channel_values(0xff);

        assert_eq!(universe.channels, vec![0xff, 0xff, 0xff, 0xff]);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn should_return_all_channels_as_slice() {
        let universe = DmxUniverse {
            channel_count: 4,
            channels: vec![255; 4],
        };

        assert_eq!(universe.as_slice(), &[0xff, 0xff, 0xff, 0xff]);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn should_extend_channels_with_byte_slice() {
        let mut universe = DmxUniverse {
            channel_count: 4,
            channels: vec![255; 4],
        };

        universe.extend(&[0, 0, 0, 0]).unwrap();

        assert_eq!(
            universe.channels,
            vec![0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00]
        );
        assert_eq!(universe.channel_count, 8);

        assert_eq!(
            universe.extend(&[0xff; 512][..]),
            Err(DmxError::InvalidChannelCount(520))
        );
    }
}
