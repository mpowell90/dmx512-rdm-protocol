use crate::rdm::error::RdmError;
#[cfg(not(feature = "alloc"))]
use core::str::FromStr;
#[cfg(not(feature = "alloc"))]
use heapless::{String, Vec};

#[cfg(feature = "alloc")]
pub fn decode_string_bytes(bytes: &[u8]) -> Result<String, RdmError> {
    let utf8 = String::from_utf8_lossy(bytes);

    if utf8.contains(char::from(0)) {
        Ok(utf8.split_once(char::from(0)).unwrap().0.to_string())
    } else {
        Ok(utf8.to_string())
    }
}

#[cfg(not(feature = "alloc"))]
pub fn decode_string_bytes<const N: usize>(bytes: &[u8]) -> Result<String<N>, RdmError> {
    let utf8 = String::<N>::from_utf8(Vec::<u8, N>::from_slice(bytes).unwrap())?;

    if utf8.contains(char::from(0)) {
        Ok(String::<N>::from_str(utf8.split_once(char::from(0)).unwrap().0).unwrap())
    } else {
        Ok(utf8)
    }
}

pub fn bsd_16_crc(packet: &[u8]) -> u16 {
    packet
        .iter()
        .fold(0_u16, |sum, byte| (sum.wrapping_add(*byte as u16)))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "alloc")]
    fn should_decode_string_bytes() {
        assert_eq!(
            decode_string_bytes(&b"null terminated string\0"[..]).unwrap(),
            "null terminated string".to_string()
        );
        assert_eq!(
            decode_string_bytes(&b"not null terminated string"[..]).unwrap(),
            "not null terminated string".to_string()
        );
        assert_eq!(
            decode_string_bytes(&b"early terminated\0string"[..]).unwrap(),
            "early terminated".to_string()
        );
    }

    #[test]
    #[cfg(not(feature = "alloc"))]
    fn should_decode_string_bytes() {
        assert_eq!(
            decode_string_bytes::<32>(&b"null terminated string\0"[..]).unwrap(),
            String::from_utf8(Vec::<u8, 32>::from_slice(b"null terminated string").unwrap())
                .unwrap()
        );
        assert_eq!(
            decode_string_bytes::<32>(&b"not null terminated string"[..]).unwrap(),
            String::from_utf8(Vec::<u8, 32>::from_slice(b"not null terminated string").unwrap())
                .unwrap()
        );
        assert_eq!(
            decode_string_bytes::<32>(&b"early terminated\0string"[..]).unwrap(),
            String::from_utf8(Vec::<u8, 32>::from_slice(b"early terminated").unwrap()).unwrap()
        );
    }
}
