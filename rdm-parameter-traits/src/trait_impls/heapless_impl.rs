use crate::{ParameterCodecError, RdmParameterData};
use core::str::FromStr;

impl<T, const N: usize> RdmParameterData for heapless::Vec<T, N>
where
    T: RdmParameterData,
{
    fn size_of(&self) -> usize {
        self.iter().map(|v| v.size_of()).sum::<usize>()
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        let size = self.size_of();

        if buf.len() < size {
            return Err(ParameterCodecError::BufferTooSmall {
                provided: buf.len(),
                required: size,
            });
        }

        let mut offset = 0;

        for v in self {
            offset += v.encode_rdm_parameter_data(&mut buf[offset..])?;
        }

        Ok(offset)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        let size = core::mem::size_of::<T>();

        let count = buf.len() / size;

        let mut out = heapless::Vec::<T, N>::new();
        let mut offset = 0;

        for _ in 0..count {
            let val = T::decode_rdm_parameter_data(&buf[offset..])?;
            offset += size;
            out.push(val)
                .map_err(|_| ParameterCodecError::MalformedData)?;
        }

        Ok(out)
    }
}

impl<const N: usize> RdmParameterData for heapless::String<N> {
    fn size_of(&self) -> usize {
        self.len()
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        let size = self.size_of();

        if buf.len() < size {
            return Err(ParameterCodecError::BufferTooSmall {
                provided: buf.len(),
                required: size,
            });
        }

        buf[..size].copy_from_slice(self.as_bytes());

        Ok(size)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        heapless::String::<N>::from_str(core::str::from_utf8(buf)?).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use crate::RdmParameterData;
    use heapless::Vec;

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
    fn test_heapless_vec_u8() {
        encode_decode(Vec::<u8, 4>::from([0x01, 0x02, 0x03, 0x04]), &mut [0u8; 4]);
    }

    #[test]
    fn test_heapless_string() {
        encode_decode(heapless::String::<4>::from_str("Test").unwrap(), &mut [0u8; 4]);
        encode_decode(heapless::String::<32>::from_str("Test\0\0\0\0").unwrap(), &mut [0u8; 32]);
    }
}
