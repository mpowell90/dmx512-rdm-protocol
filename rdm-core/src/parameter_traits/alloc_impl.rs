use crate::parameter_traits::{ParameterCodecError, RdmParameterData};
use alloc::{
    collections::{BTreeMap, BTreeSet, LinkedList, VecDeque},
    vec::Vec,
};

impl<T> RdmParameterData for Vec<T>
where
    T: RdmParameterData,
{
    fn size_of(&self) -> usize {
        self.iter().map(|v| v.size_of()).sum::<usize>()
    }

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        let size = self.size_of();

        if buf.len() < size {
            return Err(ParameterCodecError::BufferTooSmall {
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

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        let size = core::mem::size_of::<T>();

        let count = buf.len() / size;

        let mut out = Vec::with_capacity(count);
        let mut offset = 0;

        for _ in 0..count {
            let val = T::decode_parameter_data(&buf[offset..])?;
            offset += size;
            out.push(val);
        }

        Ok(out)
    }
}

impl<T> RdmParameterData for VecDeque<T>
where
    T: RdmParameterData,
{
    fn size_of(&self) -> usize {
        self.iter().map(|v| v.size_of()).sum::<usize>()
    }

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        let size = self.size_of();

        if buf.len() < size {
            return Err(ParameterCodecError::BufferTooSmall {
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

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        let size = core::mem::size_of::<T>();

        let count = buf.len() / size;

        let mut out = VecDeque::with_capacity(count);
        let mut offset = 0;

        for _ in 0..count {
            let val = T::decode_parameter_data(&buf[offset..])?;
            offset += size;
            out.push_back(val);
        }

        Ok(out)
    }
}

impl<T> RdmParameterData for LinkedList<T>
where
    T: RdmParameterData,
{
    fn size_of(&self) -> usize {
        self.iter().map(|v| v.size_of()).sum::<usize>()
    }

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        let size = self.size_of();

        if buf.len() < size {
            return Err(ParameterCodecError::BufferTooSmall {
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

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        let size = core::mem::size_of::<T>();

        let count = buf.len() / size;

        let mut offset = 0;

        let mut out = LinkedList::new();

        for _ in 0..count {
            let val = T::decode_parameter_data(&buf[offset..])?;
            offset += size;
            out.push_back(val);
        }

        Ok(out)
    }
}

impl<T> RdmParameterData for BTreeSet<T>
where
    T: RdmParameterData + Ord,
{
    fn size_of(&self) -> usize {
        self.iter().map(|v| v.size_of()).sum::<usize>()
    }

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        let size = self.size_of();

        if buf.len() < size {
            return Err(ParameterCodecError::BufferTooSmall {
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

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        let size = core::mem::size_of::<T>();

        let count = buf.len() / size;

        let mut offset = 0;

        let mut out = BTreeSet::new();

        for _ in 0..count {
            let val = T::decode_parameter_data(&buf[offset..])?;
            offset += size;
            out.insert(val);
        }

        Ok(out)
    }
}

impl<K, V> RdmParameterData for BTreeMap<K, V>
where
    K: RdmParameterData + Ord,
    V: RdmParameterData,
{
    fn size_of(&self) -> usize {
        self.iter()
            .map(|(k, v)| k.size_of() + v.size_of())
            .sum::<usize>()
    }

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        let size = self.size_of();

        if buf.len() < size {
            return Err(ParameterCodecError::BufferTooSmall {
                provided: buf.len(),
                required: size,
            });
        }

        let mut offset = 0;

        for (k, v) in self.iter() {
            offset += k.encode_parameter_data(&mut buf[offset..])?;
            offset += v.encode_parameter_data(&mut buf[offset..])?;
        }

        Ok(offset)
    }

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        let size = core::mem::size_of::<K>() + core::mem::size_of::<V>();

        let count = buf.len() / size;

        let mut offset = 0;

        let mut out = BTreeMap::new();

        for _ in 0..count {
            let k = K::decode_parameter_data(&buf[offset..])?;
            offset += core::mem::size_of::<K>();

            let v = V::decode_parameter_data(&buf[offset..])?;
            offset += core::mem::size_of::<V>();

            out.insert(k, v);
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use crate::parameter_traits::RdmParameterData;
    use alloc::collections::{BTreeMap, BTreeSet, LinkedList, VecDeque};
    use alloc::vec::Vec;

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
    fn test_vec() {
        encode_decode(Vec::from([0x01u8, 0x02u8, 0x03u8]), &mut [0u8; 3]);
    }

    #[test]
    fn test_vecdeque() {
        encode_decode(VecDeque::from([0x01u8, 0x02u8, 0x03u8]), &mut [0u8; 6]);
    }

    #[test]
    fn test_linkedlist() {
        encode_decode(LinkedList::from([0x01u8, 0x02u8, 0x03u8]), &mut [0u8; 6]);
    }

    #[test]
    fn test_btreeset() {
        encode_decode(BTreeSet::from([0x01u8, 0x02u8, 0x03u8]), &mut [0u8; 6]);
    }

    #[test]
    fn test_btreemap() {
        encode_decode(
            BTreeMap::from([(0x01u8, 0x01u8), (0x02u8, 0x02u8), (0x03u8, 0x03u8)]),
            &mut [0u8; 6],
        );
    }
}
