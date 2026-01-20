#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ParameterCodecError {
    BufferTooSmall,
    MalformedData,
}

impl core::fmt::Display for ParameterCodecError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ParameterCodecError::BufferTooSmall => write!(f, "Buffer too small"),
            ParameterCodecError::MalformedData => write!(f, "Malformed data"),
        }
    }
}

impl core::error::Error for ParameterCodecError {}

pub trait RdmGetRequestParameterCodec: Sized {
    // const BODY_SIZE: usize;

    // fn get_request_data_length(&self, _buf: &mut [u8]) -> usize;

    fn get_request_encode_data(&self, _buf: &mut [u8]) -> Result<usize, ParameterCodecError>;

    fn get_request_decode_data(_bytes: &[u8]) -> Result<Self, ParameterCodecError>;
}

pub trait RdmSetRequestParameterCodec: Sized {
    // const BODY_SIZE: usize;

    // fn get_request_data_length(&self, _buf: &mut [u8]) -> usize;

    fn set_request_encode_data(&self, _buf: &mut [u8]) -> Result<usize, ParameterCodecError>;

    fn set_request_decode_data(_bytes: &[u8]) -> Result<Self, ParameterCodecError>;
}

pub trait RdmGetResponseParameterCodec: Sized {
    // const BODY_SIZE: usize;

    // fn get_request_data_length(&self, _buf: &mut [u8]) -> usize;

    fn get_response_encode_data(&self, _buf: &mut [u8]) -> Result<usize, ParameterCodecError>;

    fn get_response_decode_data(_bytes: &[u8]) -> Result<Self, ParameterCodecError>;
}

pub trait RdmSetResponseParameterCodec: Sized {
    // const BODY_SIZE: usize;

    // fn get_response_data_length(&self, _buf: &mut [u8]) -> usize;

    fn set_response_encode_data(&self, _buf: &mut [u8]) -> Result<usize, ParameterCodecError>;

    fn set_response_decode_data(_bytes: &[u8]) -> Result<Self, ParameterCodecError>;
}
