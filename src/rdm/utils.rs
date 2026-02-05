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

#[macro_export]
macro_rules! impl_rdm_string {
    ($t:ty, $e:expr) => {
        impl $t {
            pub const MAX_LENGTH: usize = {$e};

            // #[allow(clippy::new_without_default)]
            // pub const fn new() -> Self {
            //     Self(String::new())
            // }
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
            type Err = rdm_core::error::ParameterDataError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(String::<{ Self::MAX_LENGTH }>::from_str(s.trim_end_matches('\0'))?))
            }
        }

        impl RdmParameterData for $t {
            fn size_of(&self) -> usize {
                $crate::rdm::utils::truncate_at_null(
                    self.0.as_bytes()
                ).len()
            }

            fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, rdm_core::error::ParameterDataError> {
                let size = self.size_of();

                if buf.len() < size {
                    return Err(rdm_core::error::ParameterDataError::BufferTooSmall {
                        provided: buf.len(),
                        required: size,
                    });
                }

                buf[..size].copy_from_slice($crate::rdm::utils::truncate_at_null(self.0.as_bytes()));

                Ok(size)
            }

            fn decode_parameter_data(buf: &[u8]) -> Result<Self, rdm_core::error::ParameterDataError> {
                Ok(Self(String::decode_parameter_data($crate::rdm::utils::truncate_at_null(
                    buf,
                ))?))
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use heapless::String;
    use rdm_core::parameter_traits::RdmParameterData;
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
