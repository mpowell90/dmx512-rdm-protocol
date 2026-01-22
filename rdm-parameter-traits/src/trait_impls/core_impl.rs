use crate::{ParameterCodecError, RdmParameterData};

macro_rules! impl_rdm_data_primitive {
    ($($t:ty),*) => {
        $(
            impl RdmParameterData for $t {
                #[inline]
                fn size_of(&self) -> usize {
                    core::mem::size_of::<$t>()
                }

                fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
                    let size = self.size_of();

                    if buf.len() < size {
                        return Err(ParameterCodecError::BufferTooSmall {
                            provided: buf.len(),
                            required: size,
                        });
                    }

                    buf[..size].copy_from_slice(&self.to_be_bytes());
                    Ok(size)
                }

                fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
                    let size = core::mem::size_of::<$t>();

                    if buf.len() < size {
                        return Err(ParameterCodecError::BufferTooSmall {
                            provided: buf.len(),
                            required: size,
                        });
                    }

                    let val = <$t>::from_be_bytes(
                        buf[..size]
                            .try_into()
                            .map_err(|_| ParameterCodecError::MalformedData)?
                    );
                    Ok(val)
                }
            }

            impl<const N: usize> RdmParameterData for [$t; N] {
                #[inline]
                fn size_of(&self) -> usize {
                    N * core::mem::size_of::<$t>()
                }

                fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
                    let size = self.size_of();

                    if buf.len() < size {
                        return Err(ParameterCodecError::BufferTooSmall {
                            provided: buf.len(),
                            required: size,
                        });
                    }

                    let item_size = core::mem::size_of::<$t>();

                    for (i, item) in self.iter().enumerate() {
                        let start = i * item_size;
                        let end = start + item_size;
                        buf[start..end].copy_from_slice(&item.to_be_bytes());
                    }

                    Ok(size)
                }

                fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
                    let size = core::mem::size_of::<$t>() * N;

                    if buf.len() < size {
                        return Err(ParameterCodecError::BufferTooSmall {
                            provided: buf.len(),
                            required: size,
                        });
                    }

                    let mut arr: [$t; N] = [0 as $t; N];

                    let item_size = core::mem::size_of::<$t>();

                    for (i, item) in arr.iter_mut().enumerate() {
                        let start = i * item_size;
                        let end = start + item_size;
                        *item = <$t>::from_be_bytes(
                            buf[start..end]
                                .try_into()
                                .map_err(|_| ParameterCodecError::MalformedData)?
                        );
                    }

                    Ok(arr)
                }
            }
        )*
    };
}

impl_rdm_data_primitive!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);

impl RdmParameterData for bool {
    #[inline]
    fn size_of(&self) -> usize {
        1
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        if buf.is_empty() {
            return Err(ParameterCodecError::BufferTooSmall {
                provided: buf.len(),
                required: 1,
            });
        }

        buf[0] = *self as u8;

        Ok(1)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        if buf.is_empty() {
            return Err(ParameterCodecError::BufferTooSmall {
                provided: buf.len(),
                required: 1,
            });
        }

        let val = buf[0] != 0;

        Ok(val)
    }
}

impl<T> RdmParameterData for Option<T>
where
    T: RdmParameterData,
{
    fn size_of(&self) -> usize {
        match self {
            Some(v) => v.size_of(),
            None => 0,
        }
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        match self {
            Some(v) => v.encode_rdm_parameter_data(buf),
            None => Ok(0),
        }
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        if buf.is_empty() {
            Ok(None)
        } else {
            let val = T::decode_rdm_parameter_data(buf)?;
            Ok(Some(val))
        }
    }
}

impl<const N: usize> RdmParameterData for [bool; N] {
    #[inline]
    fn size_of(&self) -> usize {
        N * core::mem::size_of::<bool>()
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        let size = self.size_of();

        if buf.len() < size {
            return Err(ParameterCodecError::BufferTooSmall {
                provided: buf.len(),
                required: size,
            });
        }

        for (i, item) in self.iter().enumerate() {
            let start = i * core::mem::size_of::<bool>();
            buf[start] = if *item { 1 } else { 0 };
        }

        Ok(size)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        let size = core::mem::size_of::<bool>() * N;

        if buf.len() < size {
            return Err(ParameterCodecError::BufferTooSmall {
                provided: buf.len(),
                required: size,
            });
        }

        let mut arr: [bool; N] = [false; N];

        for (i, slot) in arr.iter_mut().enumerate() {
            let start = i * core::mem::size_of::<bool>();
            let item = buf[start] != 0;
            *slot = item;
        }

        Ok(arr)
    }
}

#[cfg(test)]
mod tests {
    use crate::RdmParameterData;

    fn encode_decode<T>(value: T, buf: &mut [u8])
    where
        T: RdmParameterData + PartialEq + core::fmt::Debug,
    {
        let size = value.size_of();

        let written = value.encode_rdm_parameter_data(buf).expect("encode failed");
        assert_eq!(written, size);

        let decoded = T::decode_rdm_parameter_data(&buf[..written]).expect("decode failed");
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_primitives_encode_decode() {
        encode_decode(0u8, &mut [0u8; 1]);
        encode_decode(0x1234u16, &mut [0u8; 2]);
        encode_decode(0x1234_5678u32, &mut [0u8; 4]);
        encode_decode(0x1234_5678_9ABCu64, &mut [0u8; 8]);
        encode_decode(0x1234_5678_9ABC_DEF0_1234_5678u128, &mut [0u8; 16]);
        encode_decode(0i8, &mut [0u8; 1]);
        encode_decode(-0x1234i16, &mut [0u8; 2]);
        encode_decode(-0x1234_5678i32, &mut [0u8; 4]);
        encode_decode(-0x1234_5678_9ABCi64, &mut [0u8; 8]);
        encode_decode(-0x1234_5678_9ABC_DEF0_1234_5678i128, &mut [0u8; 16]);
        encode_decode(1.5f32, &mut [0u8; 4]);
        encode_decode(-2.25f64, &mut [0u8; 8]);
    }

    #[test]
    fn test_bool_and_bool_array() {
        encode_decode(true, &mut [0u8; 1]);
        encode_decode([true, false, true, false, false], &mut [0u8; 5]);
    }

    #[test]
    fn test_primitive_arrays() {
        encode_decode([1u8, 2u8, 3u8, 4u8], &mut [0u8; 4]);
        encode_decode([0x1234i16, -0x1234i16, 0x7FFFi16], &mut [0u8; 6]);
        encode_decode([1.5f32, -2.5f32], &mut [0u8; 8]);
    }

    #[test]
    fn test_option_some_none() {
        encode_decode(Option::<u8>::Some(0x42), &mut [0u8; 1]);
        encode_decode(Option::<u8>::None, &mut [0u8; 0]);
    }
}
