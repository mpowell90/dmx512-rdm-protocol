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
//! let mut dmx_universe = DmxUniverse::decode(&[0, 255, 255, 0, 0]).unwrap();
//!
//! assert_eq!(&dmx_universe.as_slice()[..4], &[255, 255, 0, 0]);
//!
//! dmx_universe.set_channel_value(0, 64).unwrap();
//! dmx_universe.set_channel_values(1, &[128, 192, 255]).unwrap();
//!
//! assert_eq!(dmx_universe.get_channel_value(0).unwrap(), 64);
//! assert_eq!(dmx_universe.get_channel_values(1..=2).unwrap(), &[128, 192]);
//! assert_eq!(&dmx_universe.as_slice()[..4], &[64, 128, 192, 255]);
//! 
//! let mut encoded = [0_u8; 513];
//! let bytes_encoded = dmx_universe.encode(&mut encoded).unwrap();
//! assert_eq!(bytes_encoded, 513);
//! assert_eq!(&encoded[..5], &[0, 64, 128, 192, 255]);   
//! ```

pub mod error;
pub const DMX_START_CODE: u8 = 0;
pub const MAXIMUM_CHANNEL_COUNT: usize = 512;

use core::ops::{Index, IndexMut, RangeInclusive};
use error::DmxError;

use heapless::Vec;

#[derive(Clone, Debug, PartialEq)]
pub struct DmxUniverse(Vec<u8, MAXIMUM_CHANNEL_COUNT>);

impl DmxUniverse {
    pub fn new() -> Self {
        Self(Vec::from_slice(&[0; MAXIMUM_CHANNEL_COUNT]).unwrap())
    }

    pub fn reset(&mut self) {
        self.0.fill(0);
    }

    pub fn get_channel_value(&self, channel: u16) -> Result<u8, DmxError> {
        if channel < MAXIMUM_CHANNEL_COUNT as u16 {
            Ok(self.0[channel as usize])
        } else {
            Err(DmxError::ChannelOutOfBounds)
        }
    }

    pub fn get_channel_values(&self, range: RangeInclusive<u16>) -> Result<&[u8], DmxError> {
        let start = *range.start();
        let end = *range.end();
        if end < MAXIMUM_CHANNEL_COUNT as u16 {
            Ok(&self.0[start as usize..=end as usize])
        } else {
            Err(DmxError::ChannelOutOfBounds)
        }
    }

    pub fn set_channel_value(&mut self, channel: u16, value: u8) -> Result<(), DmxError> {
        if channel < MAXIMUM_CHANNEL_COUNT as u16 {
            self.0[channel as usize] = value;
            Ok(())
        } else {
            Err(DmxError::ChannelOutOfBounds)
        }
    }

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
        self.0.fill(value);
    }

    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self, DmxError> {
        if bytes.len() > MAXIMUM_CHANNEL_COUNT {
            return Err(DmxError::InvalidChannelCount(bytes.len() as u16));
        }

        let mut universe = Self::new();

        universe.0[0..bytes.len()].copy_from_slice(bytes);

        Ok(universe)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, DmxError> {
        if bytes.len() < 2 || bytes.len() > MAXIMUM_CHANNEL_COUNT + 1 {
            return Err(DmxError::InvalidFrameLength(bytes.len() as u16));
        }

        if bytes[0] != DMX_START_CODE {
            return Err(DmxError::InvalidStartCode(bytes[0]));
        }

        Self::from_slice(&bytes[1..])
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, DmxError> {
        let len = self.0.len();

        buf[0] = DMX_START_CODE;

        buf[1..len + 1].copy_from_slice(&self.0[..]);

        if len < MAXIMUM_CHANNEL_COUNT {
            buf[len + 1..MAXIMUM_CHANNEL_COUNT + 1].fill(0);
        }
        Ok(MAXIMUM_CHANNEL_COUNT + 1)
    }
}

impl Default for DmxUniverse {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<u16> for DmxUniverse {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl IndexMut<u16> for DmxUniverse {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl TryFrom<&[u8]> for DmxUniverse {
    type Error = DmxError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        Self::from_slice(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_new_dmx_universe() {
        let universe = DmxUniverse::new();
        assert_eq!(universe.0, Vec::<u8, 512>::from_slice(&[0; 512]).unwrap());
    }

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

    #[test]
    fn should_encode_dmx_universe() {
        let mut encoded = [0_u8; 513];

        let mut universe = DmxUniverse::new();
        universe.0[0..4].copy_from_slice(&[0x40, 0x80, 0xc0, 0xff]);

        let bytes_encoded = universe.encode(&mut encoded).unwrap();

        assert_eq!(bytes_encoded, 513);
        assert_eq!(&encoded[..5], &[0x00, 0x40, 0x80, 0xc0, 0xff]);
    }

    #[test]
    fn should_reset_dmx_universe() {
        let mut universe = DmxUniverse(Vec::<u8, 512>::from_slice(&[255; 512]).unwrap());

        universe.reset();

        assert_eq!(universe.0, Vec::<u8, 512>::from_slice(&[0; 512]).unwrap());
    }

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

    #[test]
    fn should_get_channel_values() {
        let universe = DmxUniverse::from_slice(&[0x40, 0x80, 0xc0, 0xff]).unwrap();

        assert_eq!(universe.get_channel_values(2..=3).unwrap(), &[192, 255]);
        assert_eq!(
            universe.get_channel_values(510..=513),
            Err(DmxError::ChannelOutOfBounds)
        );
    }

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
}
