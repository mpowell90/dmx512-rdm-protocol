pub fn bsd_16_crc(packet: &[u8]) -> u16 {
    packet
        .iter()
        .fold(0_u16, |sum, byte| sum.wrapping_add(*byte as u16))
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
