
#[cfg(not(feature = "alloc"))]
use heapless::Vec;

#[macro_export]
macro_rules! check_msg_len {
    ($msg:ident, $min_len:literal) => {
        if $msg.len() < $min_len {
            return Err(RdmError::InvalidMessageLength($msg.len() as u8));
        }
    };
}

pub struct PackedEntryIterator<'a> {
    pos: usize,
    data: &'a [u8],
}
impl<'a> PackedEntryIterator<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            pos: 0,
            data
        }
    }
}
impl<'a> Iterator for PackedEntryIterator<'a> {
    type Item = Result<(u16, &'a [u8]), ()>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.data.len() {
            return None;
        }
        let data = &self.data[self.pos..];

        if data.len() < 3 {
            self.pos = self.data.len();
            return Some(Err(()));
        }
        let ident = u16::from_be_bytes([data[0], data[1]]);
        let len = data[2] as usize;

        self.pos += 3;
        let data = &data[3..];

        if data.len() < len {
            self.pos = self.data.len();
            return Some(Err(()));
        }

        let item_data = &data[..len];
        self.pos += len;

        Some(Ok((
            ident,
            item_data
        )))
    }
}

#[cfg(feature = "alloc")]
type T = Vec<u8>;
#[cfg(not(feature = "alloc"))]
type T = Vec<u8, 231>;

pub fn write_packed_entry(buf: &mut T, ident: u16, data: &[u8]) -> bool {
    if data.len() > 255 {
        return false;
    }

    buf.extend(ident.to_be_bytes());

    #[cfg(feature = "alloc")]
    buf.push(data.len() as u8);
    #[cfg(not(feature = "alloc"))]
    buf.push(data.len() as u8).unwrap();

    #[cfg(feature = "alloc")]
    buf.extend_from_slice(data);
    #[cfg(not(feature = "alloc"))]
    buf.extend_from_slice(data).unwrap();

    true
}

#[cfg(test)]
mod tests {
    use crate::rdm::utils::{PackedEntryIterator, write_packed_entry};


    // Test 1: DMX Start address for 3 devices
    const UNPACKED_1: &[(u16, &[u8])] = &[
        (0x0000, &[0x00, 0x00]),
        (0x0001, &[0x00, 0x02]),
        (0x0002, &[0x00, 0x04]),
    ];
    const PACKED_1: &[u8] = &[
        0x00, 0x00, // Sub-Device: Root
        0x02,       // len: 2 bytes
        0x00, 0x00, // Data: 0
        0x00, 0x01, // sub-device: 1
        0x02,       // len: 2
        0x00, 0x02, // data: 2
        0x00, 0x02, // sub-device: 1
        0x02,       // len: 2
        0x00, 0x04, // data: 2
    ];

    // Test 2: GET_PERSONALITY_DESCRIPTION by index for 4 values
    const UNPACKED_2: &[(u16, &[u8])] = &[
        (0x0001, b"\x00\x10Standard"),
        (0x0002, b"\x00\x0AReduced 8-bit"),
        (0x0003, b"\x00\x18Direct Emitters"),
        (0x0004, b"\x00\x08Macros Only"),
    ];
    const PACKED_2: &[u8] = &[
        0x00, 0x01, // Personality: 1
        10,
        // b"\x00\x10Standard",
        0x00, 0x10, 0x53, 0x74, 0x61, 0x6e, 0x64, 0x61, 0x72, 0x64,
        0x00, 0x02,
        15,
        0x00, 0x0a, 0x52, 0x65, 0x64, 0x75, 0x63, 0x65, 0x64, 0x20, 0x38, 0x2d, 0x62, 0x69, 0x74,
        // b"\x00\x0AReduced 8-bit",
        0x00, 0x03,
        17,
        0x00, 0x18, 0x44, 0x69, 0x72, 0x65, 0x63, 0x74, 0x20, 0x45, 0x6d, 0x69, 0x74, 0x74, 0x65, 0x72, 0x73,
        // b"\x00\x18Direct Emitters",
        0x00, 0x04,
        13,
        0x00, 0x08, 0x4d, 0x61, 0x63, 0x72, 0x6f, 0x73, 0x20, 0x4f, 0x6e, 0x6c, 0x79,
        // b"\x00\x08Macros Only",
    ];

    #[cfg(not(feature = "alloc"))]
    type Vec<T> = heapless::Vec<T, 231>;

    fn pack_all(unpacked_data: &[(u16, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        for (ident, data) in unpacked_data {
            write_packed_entry(&mut out, *ident, data);
        }
        out
    }
    fn unpack_all<'a>(data: &'a[u8]) -> Result<Vec<(u16, &'a [u8])>, ()> {
        let out: Result<Vec<_>, _> = PackedEntryIterator::new(data).collect();
        out
    }

    #[test]
    fn pack_data_dmx_start_address() {
        let packed = pack_all(UNPACKED_1);
        assert_eq!(packed, PACKED_1)
    }
    #[test]
    fn unpack_data_dmx_start_address() {
        let out = unpack_all(PACKED_1).unwrap();
        assert_eq!(out, UNPACKED_1)
    }

    #[test]
    fn pack_data_personality_description() {
        let packed = pack_all(UNPACKED_2);
        assert_eq!(packed, PACKED_2)
    }
    #[test]
    fn unpack_data_personality_description() {
        let out = unpack_all(PACKED_2).unwrap();
        assert_eq!(out, UNPACKED_2)
    }
}
