//! Data types and functionality for encoding and decoding RDM packets

pub mod error;
pub mod header;
pub mod parameter;
pub mod request;
pub mod response;
pub mod utils;

use crate::rdm::{header::RdmFrameKind, request::RdmRequest, response::RdmResponse};
pub use error::RdmError;
pub use header::{CommandClass, DeviceUID, SubDeviceId};
pub use macaddr;

pub const RDM_START_CODE_BYTE: u8 = 0xcc;
pub const RDM_SUB_START_CODE_BYTE: u8 = 0x01;

pub const DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE: u8 = 0xfe;
pub const DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE: u8 = 0xaa;

pub const MAX_RDM_FRAME_LENGTH: usize = 257;
pub const MAX_RDM_PARAMETER_DATA_LENGTH: usize = 231;

#[derive(Clone, Debug)]
pub enum RdmFrame {
    Request(RdmRequest),
    Response(RdmResponse),
}

impl RdmFrame {
    pub fn frame_kind(&self) -> RdmFrameKind {
        match self {
            Self::Request(_) => RdmFrameKind::Request,
            Self::Response(_) => RdmFrameKind::Response,
        }
    }

    pub fn size(&self) -> usize {
        match self {
            Self::Request(request) => request.size(),
            Self::Response(response) => response.size(),
        }
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, RdmError> {
        match self {
            Self::Request(request) => request.encode(buf),
            Self::Response(response) => response.encode(buf),
        }
    }

    pub fn decode(buf: &[u8]) -> Result<Self, RdmError> {
        RdmFrameView::from_bytes(buf)?.decode()
    }
}

pub struct RdmFrameView<'a>(&'a [u8]);

impl<'a> RdmFrameView<'a> {
    pub fn from_bytes(data: &'a [u8]) -> Result<Self, RdmError> {
        // if data.len() > MAX_RDM_FRAME_LENGTH {
        //     return Err(RdmError::MalformedPacket);
        // }

        if data[0] != RDM_START_CODE_BYTE {
            return Err(RdmError::InvalidStartCode);
        }

        // TODO need to sort out RdmError's
        // if data[1] != RDM_SUB_START_CODE_BYTE {
        //     return Err(RdmError::InvalidSubStartCode);
        // }

        let message_length = data[2] as usize;

        if data.len() < message_length + 2 {
            return Err(RdmError::MalformedPacket);
        }

        Ok(Self(&data[0..message_length + 2]))
    }

    pub fn message_length(&self) -> usize {
        self.0[2] as usize
    }

    pub fn command_class(&self) -> Result<CommandClass, RdmError> {
        self.0[20].try_into()
    }

    pub fn frame_kind(&self) -> Result<RdmFrameKind, RdmError> {
        self.command_class().map(|cc| cc.rdm_frame_kind())
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0
    }

    pub fn decode(&self) -> Result<RdmFrame, RdmError> {
        match self.command_class()?.rdm_frame_kind() {
            RdmFrameKind::Request => Ok(RdmFrame::Request(RdmRequest::decode(self.0)?)),
            RdmFrameKind::Response => Ok(RdmFrame::Response(RdmResponse::decode(self.0)?)),
        }
    }
}
