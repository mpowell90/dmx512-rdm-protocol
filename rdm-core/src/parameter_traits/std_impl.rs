use crate::parameter_traits::{ParameterCodecError, RdmParameterData};
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, Hash};

impl<K, V, S> RdmParameterData for HashMap<K, V, S>
where
    K: RdmParameterData + Ord + Clone + Eq + Hash,
    V: RdmParameterData,
    S: BuildHasher + Default,
{
    fn size_of(&self) -> usize {
        self.iter()
            .map(|(k, v)| k.size_of() + v.size_of())
            .sum::<usize>()
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

        for (k, v) in self.iter() {
            offset += k.encode_rdm_parameter_data(&mut buf[offset..])?;
            offset += v.encode_rdm_parameter_data(&mut buf[offset..])?;
        }

        Ok(offset)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        let size = core::mem::size_of::<K>() + core::mem::size_of::<V>();

        let count = buf.len() / size;

        let mut offset = 0;

        let mut out = HashMap::with_capacity_and_hasher(count, S::default());

        for _ in 0..count {
            let k = K::decode_rdm_parameter_data(&buf[offset..])?;
            offset += core::mem::size_of::<K>();

            let v = V::decode_rdm_parameter_data(&buf[offset..])?;
            offset += core::mem::size_of::<V>();
            out.insert(k, v);
        }

        Ok(out)
    }
}

impl<T, S> RdmParameterData for HashSet<T, S>
where
    T: RdmParameterData + Ord + Clone + Eq + Hash,
    S: BuildHasher + Default,
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

        let mut out = HashSet::with_capacity_and_hasher(count, S::default());
        let mut offset = 0;

        for _ in 0..count {
            let val = T::decode_rdm_parameter_data(&buf[offset..])?;
            offset += core::mem::size_of::<T>();
            out.insert(val);
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use crate::parameter_traits::RdmParameterData;
    use std::collections::{HashMap, HashSet};

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
    fn test_hashmap_roundtrip() {
        encode_decode(
            HashMap::from([(0x01u8, 0x01u8), (0x02u8, 0x02u8), (0x03u8, 0x03u8)]),
            &mut [0u8; 6],
        );
    }

    #[test]
    fn test_hashset_roundtrip() {
        encode_decode(HashSet::from([0x01u8, 0x02u8, 0x03u8]), &mut [0u8; 6]);
    }
}
