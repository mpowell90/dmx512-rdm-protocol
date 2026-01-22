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

                    for (i, item) in self.iter().enumerate() {
                        let start = i * core::mem::size_of::<$t>();
                        let end = start + core::mem::size_of::<$t>();
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

                    for (i, slot) in arr.iter_mut().enumerate() {
                        let start = i * core::mem::size_of::<$t>();
                        let end = start + core::mem::size_of::<$t>();
                        let item = <$t>::from_be_bytes(
                            buf[start..end]
                                .try_into()
                                .map_err(|_| ParameterCodecError::MalformedData)?
                        );
                        *slot = item;
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
        core::mem::size_of::<bool>()
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        let size = self.size_of();

        if buf.len() < size {
            return Err(ParameterCodecError::BufferTooSmall {
                provided: buf.len(),
                required: size,
            });
        }

        buf[0] = if *self { 1 } else { 0 };

        Ok(size)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        let size = core::mem::size_of::<bool>();

        if buf.len() < size {
            return Err(ParameterCodecError::BufferTooSmall {
                provided: buf.len(),
                required: size,
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

// impl<T> RdmParameterData for heapless::VecView<T>
// where
//     T: RdmParameterData
// {
//     fn size_of(&self) -> usize {
//         self.iter().map(|v| v.size_of()).sum::<usize>()
//     }

//     fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
//         let size = self.size_of();

//         if buf.len() < size {
//             return Err(ParameterCodecError::BufferTooSmall);
//         }

//         let mut offset = 0;

//         for v in self {
//             offset += v.encode_rdm_parameter_data(&mut buf[offset..])?;
//         }

//         Ok(offset)
//     }

//     fn decode_rdm_parameter_data(buf: &[u8]) -> Result<(Self, usize), ParameterCodecError> {
//         let size = core::mem::size_of::<T>();

//         let count = buf.len() / size;

//         let mut out = heapless::Vec::<T, N>::new();
//         let mut offset = 0;

//         for _ in 0..count {
//             let (val, used) = T::decode_rdm_parameter_data(&buf[offset..])?;
//             offset += used;
//             out.push(val).map_err(|_| ParameterCodecError::MalformedData)?;
//         }

//         Ok((out, offset))
//     }
// }

// impl<T, const N: usize> RdmParameterData for heapless::String<N>
// where
//     T: RdmParameterData + From<u8> + Into<u8> + Copy,
// {
//     fn size_of(&self) -> usize {
//         self.len()
//     }

//     fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
//         let size = self.size_of();

//         if buf.len() < size {
//             return Err(ParameterCodecError::BufferTooSmall);
//         }

//         for (i, c) in self.as_bytes().iter().enumerate() {
//             buf[i] = (*c).into();
//         }

//         Ok(size)
//     }

//     fn decode_rdm_parameter_data(buf: &[u8]) -> Result<(Self, usize), ParameterCodecError> {
//         let mut out = heapless::String::<N>::new();

//         for &b in buf {
//             let c: T = b.try_into().map_err(|_| ParameterCodecError::MalformedData)?;
//             out.push(c.into()).map_err(|_| ParameterCodecError::MalformedData)?;
//         }

//         Ok((out, out.len()))
//     }
// }

#[cfg(feature = "alloc")]
mod alloc_impls {
    use crate::{ParameterCodecError, RdmParameterData};
    use alloc::{
        collections::{BTreeMap, BTreeSet, BinaryHeap, LinkedList, VecDeque},
        vec::Vec,
    };

    impl<T> RdmParameterData for Vec<T>
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

            let mut out = Vec::with_capacity(count);
            let mut offset = 0;

            for _ in 0..count {
                let val = T::decode_rdm_parameter_data(&buf[offset..])?;
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

            let mut out = VecDeque::with_capacity(count);
            let mut offset = 0;

            for _ in 0..count {
                let val = T::decode_rdm_parameter_data(&buf[offset..])?;
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

            let mut offset = 0;

            let mut out = LinkedList::new();

            for _ in 0..count {
                let val = T::decode_rdm_parameter_data(&buf[offset..])?;
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

            let mut offset = 0;

            let mut out = BTreeSet::new();

            for _ in 0..count {
                let val = T::decode_rdm_parameter_data(&buf[offset..])?;
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

            let mut out = BTreeMap::new();

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

    impl<T> RdmParameterData for BinaryHeap<T>
    where
        T: RdmParameterData + Ord,
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
            let v = Vec::<T>::decode_rdm_parameter_data(buf)?;
            Ok(BinaryHeap::from(v))
        }
    }
}

#[cfg(feature = "std")]
mod std_impls {
    use crate::{ParameterCodecError, RdmParameterData};
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
}
