use crate::{CommandClass, ParameterId, ResponseResult};
use heapless::Vec;

#[derive(Clone, Debug, PartialEq)]
pub struct CustomResponseParameter {
    pub command_class: CommandClass,
    pub parameter_id: ParameterId,
    pub parameter_data_length: u8,
    pub response: ResponseResult<Vec<u8, 231>>,
}
