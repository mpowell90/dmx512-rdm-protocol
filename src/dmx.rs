use core::ops::{Index, IndexMut, RangeInclusive};
use thiserror::Error;

const MAXIMUM_CHANNEL_COUNT: u16 = 512;

#[derive(Clone, Debug, Error)]
pub enum DmxError {
    #[error("Channel out of bounds")]
    ChannelOutOfBounds,
}

#[derive(Clone, Debug)]
pub struct DmxUniverse {
    pub channel_count: u16,
    channels: Vec<u8>,
}

impl DmxUniverse {
    pub fn new(channel_count: u16) -> Result<Self, &'static str> {
        if channel_count > MAXIMUM_CHANNEL_COUNT {
            return Err("Channel count exceeds maximum of 512");
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

    pub fn as_bytes(&self) -> &[u8] {
        &self.channels
    }

    pub fn set_channel_value(&mut self, channel: u16, value: u8) -> Result<(), DmxError>{
        if channel < self.channel_count {
            self.channels[channel as usize] = value;
            Ok(())
        } else {
            Err(DmxError::ChannelOutOfBounds)
        }
    }

    pub fn set_channel_values(&mut self, channel: u16, values: &[u8]) -> Result<(), DmxError>{
        if channel + (values.len() as u16) < self.channel_count {
            for (i, &value) in values.iter().enumerate() {
                self.channels[channel as usize + i] = value;
            }
            Ok(())
        } else {
            Err(DmxError::ChannelOutOfBounds)
        }
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
    fn should_return_all_channels_as_bytes() {
        let universe = DmxUniverse {
            channel_count: 4,
            channels: vec![255; 4],
        };

        assert_eq!(universe.as_bytes(), &[255, 255, 255, 255]);
    }
}
