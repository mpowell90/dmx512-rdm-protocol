#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

// Allows using std types when desired, while keeping the crate `no_std`.
#[cfg(feature = "std")]
extern crate std;

pub mod trait_impls;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ParameterCodecError {
    BufferTooSmall {
        provided: usize,
        required: usize,
    },
    MalformedData,
}

impl core::fmt::Display for ParameterCodecError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ParameterCodecError::BufferTooSmall {
                provided,
                required,
            } => write!(f, "Buffer too small, provided: {}, required: {}", provided, required),
            ParameterCodecError::MalformedData => write!(f, "Malformed data"),
        }
    }
}

impl core::error::Error for ParameterCodecError {}

pub trait RdmParameterData: Sized {
    fn size_of(&self) -> usize;

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError>;

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError>;
}

pub trait RdmDiscoveryRequestParameterCodec: Sized {
    fn size_of(&self) -> usize;

    fn discovery_request_encode_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError>;

    fn discovery_request_decode_data(buf: &[u8]) -> Result<Self, ParameterCodecError>;
}

pub trait RdmDiscoveryResponseParameterCodec: Sized {
    fn size_of(&self) -> usize;

    fn discovery_response_encode_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError>;

    fn discovery_response_decode_data(buf: &[u8]) -> Result<Self, ParameterCodecError>;
}

pub trait RdmGetRequestParameterCodec: Sized {
    fn size_of(&self) -> usize;

    fn get_request_encode_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError>;

    fn get_request_decode_data(buf: &[u8]) -> Result<Self, ParameterCodecError>;
}

pub trait RdmGetResponseParameterCodec: Sized {
    fn size_of(&self) -> usize;

    fn get_response_encode_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError>;

    fn get_response_decode_data(buf: &[u8]) -> Result<Self, ParameterCodecError>;
}

pub trait RdmSetRequestParameterCodec: Sized {
    fn size_of(&self) -> usize;

    fn set_request_encode_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError>;

    fn set_request_decode_data(buf: &[u8]) -> Result<Self, ParameterCodecError>;
}

pub trait RdmSetResponseParameterCodec: Sized {
    fn size_of(&self) -> usize;

    fn set_response_encode_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError>;

    fn set_response_decode_data(buf: &[u8]) -> Result<Self, ParameterCodecError>;
}
