
#[macro_export]
macro_rules! check_msg_len {
    ($msg:ident, $min_len:literal) => {
        if $msg.len() < $min_len {
            return Err(RdmError::InvalidMessageLength($msg.len() as u8));
        }
    };
}
