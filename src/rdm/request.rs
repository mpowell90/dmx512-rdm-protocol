use byteorder::{BigEndian, WriteBytesExt};
use ux::u48;

use super::{bsd_16_crc, device::DeviceUID, CommandClass, ParameterId, SC_RDM, SC_SUB_MESSAGE};

#[derive(Clone, Debug)]
pub struct Request<T> {
    destination_uid: DeviceUID,
    source_uid: DeviceUID,
    transaction_number: u8,
    port_id: u8,
    sub_device: u16,
    command_class: CommandClass,
    parameter_id: ParameterId,
    parameter_data: Option<T>,
}

impl<T> Request<T> {
    pub fn new(
        destination_uid: DeviceUID,
        source_uid: DeviceUID,
        transaction_number: u8,
        port_id: u8,
        sub_device: u16,
        command_class: CommandClass,
        parameter_id: ParameterId,
        parameter_data: Option<T>,
    ) -> Request<T> {
        Request {
            destination_uid,
            source_uid,
            transaction_number,
            port_id,
            sub_device,
            command_class,
            parameter_id,
            parameter_data,
        }
    }

    fn create_packet(self, parameter_data: Option<Vec<u8>>) -> Vec<u8> {
        // TODO it might be possible to remove this clone()
        let parameter_data_len = if let Some(data) = parameter_data.clone() {
            data.len() as u8
        } else {
            0
        };

        let mut packet = Vec::new();
        packet.write_u8(SC_RDM).unwrap(); // Start Code
        packet.write_u8(SC_SUB_MESSAGE).unwrap(); // Sub Start Code
        packet.write_u8(24_u8 + parameter_data_len).unwrap(); // Message Length: Range 24 to 255 excluding the checksum
        packet
            .write_u48::<BigEndian>(self.destination_uid.into())
            .unwrap();
        packet
            .write_u48::<BigEndian>(self.source_uid.into())
            .unwrap();
        packet.write_u8(self.transaction_number).unwrap(); // Transaction Number
        packet.write_u8(self.port_id).unwrap(); // Port Id / Response Type
        packet.write_u8(0x00).unwrap(); // Message Count
        packet.write_u16::<BigEndian>(self.sub_device).unwrap(); // Sub Device
        packet.write_u8(self.command_class as u8).unwrap();
        packet
            .write_u16::<BigEndian>(self.parameter_id as u16)
            .unwrap();

        packet.write_u8(parameter_data_len as u8).unwrap();

        if let Some(data) = parameter_data {
            packet.extend(data);
        }

        packet.write_u16::<BigEndian>(bsd_16_crc(&packet)).unwrap();
        packet
    }
}

impl From<Request<String>> for Vec<u8> {
    fn from(request: Request<String>) -> Self {
        let parameter_data: Option<Vec<u8>> = request
            .parameter_data
            .clone()
            .and_then(|data| Some(data.into_bytes()));
        request.create_packet(parameter_data)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct DiscUniqueBranchRequest {
    lower_bound_uid: u48,
    upper_bound_uid: u48,
}

impl DiscUniqueBranchRequest {
    pub fn new(lower_bound_uid: u48, upper_bound_uid: u48) -> DiscUniqueBranchRequest {
        DiscUniqueBranchRequest {
            lower_bound_uid,
            upper_bound_uid,
        }
    }
}

impl From<DiscUniqueBranchRequest> for Vec<u8> {
    fn from(disc_unique_branch_data: DiscUniqueBranchRequest) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::new();
        vec.write_u48::<BigEndian>(disc_unique_branch_data.lower_bound_uid.into())
            .unwrap();
        vec.write_u48::<BigEndian>(disc_unique_branch_data.upper_bound_uid.into())
            .unwrap();
        vec
    }
}

impl From<Request<DiscUniqueBranchRequest>> for Vec<u8> {
    fn from(request: Request<DiscUniqueBranchRequest>) -> Vec<u8> {
        let parameter_data: Option<Vec<u8>> =
            request.parameter_data.and_then(|data| Some(data.into()));
        request.create_packet(parameter_data)
    }
}

#[derive(Debug)]
pub struct DiscUnmuteRequest {}

impl From<Request<DiscUnmuteRequest>> for Vec<u8> {
    fn from(request: Request<DiscUnmuteRequest>) -> Vec<u8> {
        request.create_packet(None)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ParameterDescriptionRequest {
    parameter_id: ParameterId,
}

impl From<ParameterDescriptionRequest> for Vec<u8> {
    fn from(parameter_description_request: ParameterDescriptionRequest) -> Vec<u8> {
        Vec::from([parameter_description_request.parameter_id as u8])
    }
}

impl From<Request<ParameterDescriptionRequest>> for Vec<u8> {
    fn from(request: Request<ParameterDescriptionRequest>) -> Vec<u8> {
        let parameter_data = request.parameter_data.and_then(|data| Some(data.into()));
        request.create_packet(parameter_data)
    }
}

#[derive(Clone, Debug)]
pub struct DeviceLabelRequest {
    label: Option<String>,
}

impl From<Request<DeviceLabelRequest>> for Vec<u8> {
    fn from(request: Request<DeviceLabelRequest>) -> Vec<u8> {
        let parameter_data = request
            .parameter_data
            .clone()
            .and_then(|device_label| device_label.label)
            .and_then(|label| Some(label.into_bytes()));

        request.create_packet(parameter_data)
    }
}

pub struct DeviceInfoRequest;

impl From<Request<DeviceInfoRequest>> for Vec<u8> {
    fn from(request: Request<DeviceInfoRequest>) -> Vec<u8> {
        request.create_packet(None)
    }
}

pub struct SoftwareVersionLabelRequest;

impl From<Request<SoftwareVersionLabelRequest>> for Vec<u8> {
    fn from(request: Request<SoftwareVersionLabelRequest>) -> Vec<u8> {
        request.create_packet(None)
    }
}

pub struct SupportedParametersRequest;

impl From<Request<SupportedParametersRequest>> for Vec<u8> {
    fn from(request: Request<SupportedParametersRequest>) -> Vec<u8> {
        request.create_packet(None)
    }
}

pub struct IdentifyDeviceRequest;

impl From<Request<IdentifyDeviceRequest>> for Vec<u8> {
    fn from(request: Request<IdentifyDeviceRequest>) -> Vec<u8> {
        request.create_packet(None)
    }
}
