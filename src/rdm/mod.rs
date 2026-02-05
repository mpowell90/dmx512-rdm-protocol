//! Data types and functionality for encoding and decoding RDM packets

pub mod core {
    pub use rdm_core::*;
}
pub mod derive {
    pub use rdm_derive::*;
}
pub mod parameter;
pub mod request;
pub mod response;

use crate::rdm::{request::RdmRequest, response::RdmResponse};
use core::{CommandClass, RdmFrameKind, error::RdmError};
pub use macaddr;

pub const RDM_START_CODE_BYTE: u8 = 0xcc;
pub const RDM_SUB_START_CODE_BYTE: u8 = 0x01;

pub const DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE: u8 = 0xfe;
pub const DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE: u8 = 0xaa;

pub const MIN_DISC_FRAME_LENGTH: usize = 17;
pub const MIN_RDM_FRAME_LENGTH: usize = 25;
pub const MAX_RDM_FRAME_LENGTH: usize = 257;
pub const MAX_RDM_PARAMETER_DATA_LENGTH: usize = 231;

#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum RdmFrame {
    Request(RdmRequest),
    Response(RdmResponse),
}

impl RdmFrame {
    pub fn frame_kind(&self) -> RdmFrameKind {
        match self {
            Self::Request(_) => RdmFrameKind::Request,
            Self::Response(_) => RdmFrameKind::Response,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Self::Request(request) => request.size(),
            Self::Response(response) => response.size(),
        }
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, RdmError> {
        match self {
            Self::Request(request) => request.encode(buf),
            Self::Response(response) => response.encode(buf),
        }
    }

    pub fn decode(buf: &[u8]) -> Result<Self, RdmError> {
        RdmFrameView::from_bytes(buf)?.decode()
    }
}

pub struct RdmFrameView<'a>(&'a [u8]);

impl<'a> RdmFrameView<'a> {
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, RdmError> {
        // if data.len() > MAX_RDM_FRAME_LENGTH {
        //     return Err(RdmError::MalformedPacket);
        // }

        if data[0] != RDM_START_CODE_BYTE {
            return Err(RdmError::InvalidStartCode);
        }

        // TODO need to sort out RdmError's
        // if data[1] != RDM_SUB_START_CODE_BYTE {
        //     return Err(RdmError::InvalidSubStartCode);
        // }

        let message_length = data[2] as usize;

        if data.len() < message_length + 2 {
            return Err(RdmError::MalformedPacket);
        }

        Ok(Self(&data[0..message_length + 2]))
    }

    pub fn message_length(&self) -> usize {
        self.0[2] as usize
    }

    pub fn command_class(&self) -> Result<CommandClass, RdmError> {
        self.0[20].try_into().map_err(Into::into)
    }

    pub fn frame_kind(&self) -> Result<RdmFrameKind, RdmError> {
        self.command_class().map(|cc| cc.rdm_frame_kind())
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0
    }

    pub fn decode(&self) -> Result<RdmFrame, RdmError> {
        match self.command_class()?.rdm_frame_kind() {
            RdmFrameKind::Request => Ok(RdmFrame::Request(RdmRequest::decode(self.0)?)),
            RdmFrameKind::Response => Ok(RdmFrame::Response(RdmResponse::decode(self.0)?)),
        }
    }
}

#[macro_export]
macro_rules! impl_rdm_string {
    ($t:ty, $e:expr) => {
        impl $t {
            pub const MAX_LENGTH: usize = $e;
        }

        impl core::ops::Deref for $t {
            type Target = str;

            fn deref(&self) -> &Self::Target {
                self.0.as_str()
            }
        }

        impl core::ops::DerefMut for $t {
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.0.as_mut_str()
            }
        }

        impl core::str::FromStr for $t {
            type Err = $crate::rdm::core::error::ParameterDataError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(String::<{ Self::MAX_LENGTH }>::from_str(s.trim_end_matches('\0'))?))
            }
        }

        impl $crate::rdm::core::parameter_traits::RdmParameterData for $t {
            fn size_of(&self) -> usize {
                $crate::rdm::core::utils::truncate_at_null(
                    self.0.as_bytes()
                ).len()
            }

            fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, $crate::rdm::core::error::ParameterDataError> {
                let size = self.size_of();

                if buf.len() < size {
                    return Err($crate::rdm::core::error::ParameterDataError::BufferTooSmall {
                        provided: buf.len(),
                        required: size,
                    });
                }

                buf[..size].copy_from_slice($crate::rdm::core::utils::truncate_at_null(self.0.as_bytes()));

                Ok(size)
            }

            fn decode_parameter_data(buf: &[u8]) -> Result<Self, $crate::rdm::core::error::ParameterDataError> {
                Ok(Self(String::decode_parameter_data($crate::rdm::core::utils::truncate_at_null(
                    buf,
                ))?))
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::rdm::core::parameter_traits::RdmParameterData;
    use heapless::String;
    use std::str::FromStr;

    const MAX_LEN: usize = 10;
    #[derive(Clone, Debug, PartialEq)]
    struct TestString(String<MAX_LEN>);
    impl_rdm_string!(TestString, MAX_LEN);

    #[test]
    fn should_encode_decode_rdm_string() {
        let original = TestString::from_str("Hello").unwrap();
        let mut buffer = [0u8; 10];
        let bytes_written = original.encode_parameter_data(&mut buffer).unwrap();
        assert_eq!(bytes_written, 5);
        let decoded = TestString::decode_parameter_data(&buffer[..bytes_written]).unwrap();
        assert_eq!(original, decoded);

        let original = TestString::from_str("Hello\0\0\0\0").unwrap();
        let mut buffer = [0u8; 10];
        let bytes_written = original.encode_parameter_data(&mut buffer).unwrap();
        assert_eq!(bytes_written, 5);
        let decoded = TestString::decode_parameter_data(&buffer[..bytes_written]).unwrap();
        assert_eq!(original, decoded);
    }
}
