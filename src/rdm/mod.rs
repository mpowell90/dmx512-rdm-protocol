//! Data types and functionality for encoding and decoding RDM packets

pub mod error;
pub mod header;
pub mod parameter;
pub mod request;
pub mod response;
pub mod utils;

pub use error::RdmError;
pub use header::{CommandClass, DeviceUID, SubDeviceId};
pub use macaddr;

pub const RDM_START_CODE_BYTE: u8 = 0xcc;
pub const RDM_SUB_START_CODE_BYTE: u8 = 0x01;

pub const DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE: u8 = 0xfe;
pub const DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE: u8 = 0xaa;

pub const MAX_RDM_FRAME_LENGTH: usize = 257;
pub const MAX_RDM_PARAMETER_DATA_LENGTH: usize = 231;
