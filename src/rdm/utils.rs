
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

pub trait VecExt {
    fn push_u8(&mut self, value: u8);
    fn push_u16_be(&mut self, value: u16);
    fn push_u32_be(&mut self, value: u32);
}

#[cfg(feature = "alloc")]
impl VecExt for Vec<u8> {
    #[inline]
    fn push_u8(&mut self, value: u8) {
        self.push(value);
    }

    #[inline]
    fn push_u16_be(&mut self, value: u16) {
        self.extend(value.to_be_bytes());
    }

    #[inline]
    fn push_u32_be(&mut self, value: u32) {
        self.extend(value.to_be_bytes());
    }
}

#[cfg(not(feature = "alloc"))]
impl<const N: usize> VecExt for Vec<u8, N> {
    #[inline]
    fn push_u8(&mut self, value: u8) {
        self.push(value).unwrap();
    }

    #[inline]
    fn push_u16_be(&mut self, value: u16) {
        self.extend(value.to_be_bytes());
    }

    #[inline]
    fn push_u32_be(&mut self, value: u32) {
        self.extend(value.to_be_bytes());
    }
}
