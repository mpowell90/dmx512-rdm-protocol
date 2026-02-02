use crate::{CommandClass, ParameterId, error::ParameterCodecError};

#[cfg(feature = "alloc")]
pub mod alloc_impl;
pub mod core_impl;
pub mod heapless_impl; // We use heapless for convenient no_std collections with fixed capacity
#[cfg(feature = "std")]
pub mod std_impl;

pub trait RdmParameterData: Sized {
    fn size_of(&self) -> usize;

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError>;

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError>;
}

pub trait RdmParameter: RdmParameterData {
    const COMMAND_CLASS: CommandClass;

    const PID: ParameterId;

    fn command_class(&self) -> CommandClass {
        Self::COMMAND_CLASS
    }

    fn parameter_id(&self) -> ParameterId {
        Self::PID
    }
}
