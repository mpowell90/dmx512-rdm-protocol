use crate::parameter_traits::{ParameterDataError, RdmParameterData};
use core::str::FromStr;

impl<T, const N: usize> RdmParameterData for heapless::Vec<T, N>
where
    T: RdmParameterData + core::fmt::Debug,
{
    fn size_of(&self) -> usize {
        self.iter().map(|v| v.size_of()).sum()
    }

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterDataError> {
        let size = self.size_of();

        if buf.len() < size {
            return Err(ParameterDataError::BufferTooSmall {
                provided: buf.len(),
                required: size,
            });
        }

        let mut offset = 0;

        for v in self {
            offset += v.encode_parameter_data(&mut buf[offset..])?;
        }

        Ok(offset)
    }

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterDataError> {
        let mut out = heapless::Vec::<T, N>::new();

        let mut offset = 0;

        while offset < buf.len() {
            let t = T::decode_parameter_data(&buf[offset..])?;

            offset += t.size_of();

            out.push(t)
                .map_err(|_| ParameterDataError::MalformedData)?;
        }

        Ok(out)
    }
}

impl<const N: usize> RdmParameterData for heapless::String<N> {
    fn size_of(&self) -> usize {
        self.len()
    }

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterDataError> {
        let size = self.size_of();

        if buf.len() < size {
            return Err(ParameterDataError::BufferTooSmall {
                provided: buf.len(),
                required: size,
            });
        }

        buf[..size].copy_from_slice(self.as_bytes());

        Ok(size)
    }

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterDataError> {
        heapless::String::<N>::from_str(core::str::from_utf8(buf)?).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use crate::parameter_traits::RdmParameterData;
    use core::str::FromStr;
    use heapless::Vec;

    fn encode_decode<T>(value: T, buf: &mut [u8])
    where
        T: RdmParameterData + PartialEq + core::fmt::Debug,
    {
        let size = value.size_of();

        let written = value.encode_parameter_data(buf).expect("encode failed");
        assert_eq!(written, size);

        let decoded = T::decode_parameter_data(&buf[..written]).expect("decode failed");
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_heapless_vec_u8() {
        encode_decode(Vec::<u8, 4>::from([0x01, 0x02, 0x03, 0x04]), &mut [0u8; 4]);
    }

    #[test]
    fn test_heapless_string() {
        encode_decode(
            heapless::String::<4>::from_str("Test").unwrap(),
            &mut [0u8; 4],
        );
        encode_decode(
            heapless::String::<32>::from_str("Test\0\0\0\0").unwrap(),
            &mut [0u8; 32],
        );
    }
}
