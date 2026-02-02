use crate::{CommandClass, ParameterId};
use heapless::Vec;

#[derive(Clone, Debug, PartialEq)]
pub struct CustomRequestParameter {
    pub command_class: CommandClass,
    pub parameter_id: ParameterId,
    pub parameter_data_length: u8,
    pub request: Vec<u8, 231>,
}
