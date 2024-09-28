use core::{error::Error, fmt};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DmxError {
    InvalidFrameLength(u16),
    InvalidStartCode(u8),
    InvalidChannelCount(u16),
    ChannelOutOfBounds,
    FailedToAllocate,
}

impl fmt::Display for DmxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFrameLength(length) => write!(f, "Invalid frame length: {}", length),
            Self::InvalidStartCode(start_code) => write!(f, "Invalid start code: {}", start_code),
            Self::InvalidChannelCount(channel_count) => {
                write!(f, "Invalid channel count: {}", channel_count)
            }
            Self::ChannelOutOfBounds => write!(f, "Channel out of bounds"),
            Self::FailedToAllocate => write!(f, "Failed to allocate memory"),
        }
    }
}

impl Error for DmxError {}
