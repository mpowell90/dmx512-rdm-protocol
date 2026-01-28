#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

// Allows using std types when desired, while keeping the crate `no_std`.
#[cfg(feature = "std")]
extern crate std;

pub mod trait_impls;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ParameterCodecError {
    BufferTooSmall { provided: usize, required: usize },
    MalformedData,
    Utf8Error(core::str::Utf8Error),
    CapacityError,
}

impl core::fmt::Display for ParameterCodecError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ParameterCodecError::BufferTooSmall { provided, required } => write!(
                f,
                "Buffer too small, provided: {}, required: {}",
                provided, required
            ),
            ParameterCodecError::MalformedData => write!(f, "Malformed data"),
            ParameterCodecError::Utf8Error(e) => write!(f, "UTF-8 error: {}", e),
            ParameterCodecError::CapacityError => write!(f, "Insufficient capacity"),
        }
    }
}

impl From<core::str::Utf8Error> for ParameterCodecError {
    fn from(err: core::str::Utf8Error) -> Self {
        ParameterCodecError::Utf8Error(err)
    }
}

impl From<heapless::CapacityError> for ParameterCodecError {
    fn from(_: heapless::CapacityError) -> Self {
        ParameterCodecError::CapacityError
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

pub trait RdmParameterCodec: Sized {
    const COMMAND_CLASS: u8;

    const PID: u16;

    fn command_class(&self) -> u8 {
        Self::COMMAND_CLASS
    }

    fn parameter_id(&self) -> u16 {
        Self::PID
    }

    fn size_of_parameter_data(&self) -> usize;

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError>;

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError>;

    fn encode_parameter(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        let size_of_parameter_data = self.size_of_parameter_data();
        let required_size = 4 + size_of_parameter_data;
        if buf.len() < required_size {
            return Err(ParameterCodecError::BufferTooSmall {
                provided: buf.len(),
                required: required_size,
            });
        }

        // Encode PID
        buf[0] = Self::COMMAND_CLASS;
        buf[1..3].copy_from_slice(&Self::PID.to_be_bytes());
        buf[3] = size_of_parameter_data as u8;

        // Encode parameter data
        let data_size = self.encode_parameter_data(&mut buf[4..])?;

        Ok(4 + data_size)
    }

    fn decode_parameter(buf: &[u8]) -> Result<(u8, u16, u8, Self), ParameterCodecError> {
        if buf.len() < 4 {
            return Err(ParameterCodecError::BufferTooSmall {
                provided: buf.len(),
                required: 4,
            });
        }

        let command_class = buf[0];
        let pid = u16::from_be_bytes([buf[1], buf[2]]);
        let parameter_data_length = buf[3] as usize;

        if buf.len() < 4 + parameter_data_length {
            return Err(ParameterCodecError::BufferTooSmall {
                provided: buf.len(),
                required: 4 + parameter_data_length,
            });
        }

        let parameter_data =
            Self::decode_parameter_data(&buf[4..4 + parameter_data_length])?;

        Ok((command_class, pid, parameter_data_length as u8, parameter_data))
    }
}
