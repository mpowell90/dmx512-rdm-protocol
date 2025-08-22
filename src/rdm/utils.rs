use crate::rdm::error::RdmError;
use core::{ops::Deref, str::FromStr};

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

pub trait RdmTruncateNullStr {
    fn encode(&self, buf: &mut [u8]) -> Result<usize, RdmError>
    where
        Self: Deref<Target = str>,
    {
        let len = self.len();

        if buf.len() > len {
            return Err(RdmError::InvalidBufferLength(buf.len(), len));
        }

        buf[0..len].copy_from_slice(self.as_bytes());

        Ok(len)
    }

    fn decode(bytes: &[u8]) -> Result<Self, RdmError>
    where
        Self: Sized + FromStr<Err = RdmError>,
    {
        core::str::from_utf8(truncate_at_null(bytes))
            .map_err(RdmError::from)?
            .parse()
    }
}
