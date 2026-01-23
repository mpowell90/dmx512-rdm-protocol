use crate::rdm::error::RdmError;
use core::{
    error::Error,
    ops::Deref,
    str::{FromStr, Utf8Error},
};

pub fn bsd_16_crc(packet: &[u8]) -> u16 {
    packet
        .iter()
        .fold(0_u16, |sum, byte| sum.wrapping_add(*byte as u16))
}

pub fn trim_trailing_nulls(slice: &[u8]) -> &[u8] {
    match slice.iter().rposition(|&byte| byte != 0) {
        Some(index) => &slice[..=index],
        None => &[],
    }
}

pub fn truncate_at_null(slice: &[u8]) -> &[u8] {
    match slice.iter().position(|&byte| byte == 0) {
        Some(index) => &slice[..index],
        None => slice,
    }
}

pub trait RdmTruncateNullStr {
    type Error: Error + From<RdmError> + From<Utf8Error>;

    fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error>
    where
        Self: Deref<Target = str>,
    {
        let buffer_len = buf.len();
        let str_len = self.len();

        if str_len > buffer_len {
            return Err(RdmError::InvalidBufferLength(buffer_len, str_len).into());
        }

        buf[0..str_len].copy_from_slice(self.as_bytes());

        Ok(str_len)
    }

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error>
    where
        Self: Sized + FromStr<Err = Self::Error>,
    {
        core::str::from_utf8(truncate_at_null(bytes))
            .map_err(Self::Error::from)?
            .parse()
    }
}

pub trait RdmPadNullStr {
    const MAX_LENGTH: usize;

    type Error: Error + From<RdmError> + From<Utf8Error>;

    fn encode(&self, buf: &mut [u8]) -> Result<usize, Self::Error>
    where
        Self: Deref<Target = str>,
    {
        let buffer_len = buf.len();
        let str_len = self.len();

        if Self::MAX_LENGTH > buffer_len {
            return Err(RdmError::InvalidBufferLength(buffer_len, Self::MAX_LENGTH).into());
        }

        buf[0..str_len].copy_from_slice(self.as_bytes());

        let remaining_len = Self::MAX_LENGTH - str_len;

        if remaining_len > 0 {
            buf[str_len..Self::MAX_LENGTH].fill(0);
        }

        Ok(str_len)
    }

    fn decode(bytes: &[u8]) -> Result<Self, Self::Error>
    where
        Self: Sized + FromStr<Err = Self::Error>,
    {
        core::str::from_utf8(trim_trailing_nulls(bytes))
            .map_err(Self::Error::from)?
            .parse()
    }
}

#[macro_export]
macro_rules! impl_rdm_string {
    ($t:ty, $e:expr) => {
        impl $t {
            pub const MAX_LENGTH: usize = {$e};

            #[allow(clippy::new_without_default)]
            pub const fn new() -> Self {
                Self(String::new())
            }
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
            type Err = rdm_parameter_traits::ParameterCodecError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(String::<{ Self::MAX_LENGTH }>::from_str(s)?))
            }
        }

        impl RdmParameterData for $t {
            fn size_of(&self) -> usize {
                self.0.len()
            }

            fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, rdm_parameter_traits::ParameterCodecError> {
                let size = self.size_of();

                if buf.len() < size {
                    return Err(rdm_parameter_traits::ParameterCodecError::BufferTooSmall {
                        provided: buf.len(),
                        required: size,
                    });
                }

                buf[..size].copy_from_slice(self.0.as_bytes());

                Ok(size)
            }

            fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
                Ok(Self(String::decode_rdm_parameter_data($crate::rdm::utils::truncate_at_null(
                    buf,
                ))?))
            }
        }
    };
}
