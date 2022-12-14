use std::collections::HashMap;

use byteorder::{BigEndian, WriteBytesExt};
use ux::u48;

use super::{
    bsd_16_crc, device::DeviceUID, DiscoveryRequest, GetRequest, ManufacturerSpecificParameter,
    ParameterId, ProductCategory, Protocol, SetRequest, SupportedCommandClasses,
};

#[derive(Copy, Clone, Debug)]
pub struct DiscUniqueBranchRequest {
    pub lower_bound_uid: u48,
    pub upper_bound_uid: u48,
}

impl DiscUniqueBranchRequest {
    pub fn new(lower_bound_uid: u48, upper_bound_uid: u48) -> Self {
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

impl Protocol for DiscUniqueBranchRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::DiscUniqueBranch
    }
}

impl DiscoveryRequest for DiscUniqueBranchRequest {}

// TODO
#[derive(Debug)]
pub struct DiscUniqueBranchResponse {
    pub device_uid: DeviceUID,
}

impl TryFrom<&[u8]> for DiscUniqueBranchResponse {
    type Error = &'static str;

    fn try_from(packet: &[u8]) -> Result<Self, Self::Error> {
        let euid_start_index = packet
            .iter()
            .position(|&x| x == 0xaa) // &x -> accessing the element value by reference
            .unwrap();

        let euid = Vec::from(&packet[(euid_start_index + 1)..=(euid_start_index + 12)]);

        let ecs = Vec::from(&packet[(euid_start_index + 13)..=(euid_start_index + 16)]);

        let decoded_checksum = bsd_16_crc(&euid);

        let checksum = u16::from_be_bytes([ecs[0] & ecs[1], ecs[2] & ecs[3]]);

        if checksum != decoded_checksum {
            return Err("Checksum does not match decoded checksum");
        }

        let manufacturer_id = u16::from_be_bytes([euid[0] & euid[1], euid[2] & euid[3]]);

        let device_id = u32::from_be_bytes([
            euid[4] & euid[5],
            euid[6] & euid[7],
            euid[8] & euid[9],
            euid[10] & euid[11],
        ]);

        Ok(DiscUniqueBranchResponse {
            device_uid: DeviceUID::new(manufacturer_id, device_id),
        })
    }
}

pub struct DiscMuteRequest;

impl Protocol for DiscMuteRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::DiscMute
    }
}

impl DiscoveryRequest for DiscMuteRequest {}

#[derive(Debug)]
pub struct DiscMuteResponse {
    pub control_field: u16,
    pub binding_uid: Option<DeviceUID>,
}

impl From<Vec<u8>> for DiscMuteResponse {
    fn from(bytes: Vec<u8>) -> Self {
        let binding_uid = if bytes.len() > 2 {
            Some(DeviceUID::from(&bytes[2..]))
        } else {
            None
        };
        DiscMuteResponse {
            control_field: u16::from_be_bytes(bytes[..=1].try_into().unwrap()),
            binding_uid,
        }
    }
}

pub struct DiscUnmuteRequest;

impl Protocol for DiscUnmuteRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::DiscUnMute
    }
}

impl DiscoveryRequest for DiscUnmuteRequest {}

impl From<DiscUnmuteRequest> for Vec<u8> {
    fn from(_: DiscUnmuteRequest) -> Self {
        Vec::new()
    }
}

#[derive(Debug)]
pub struct DiscUnmuteResponse {
    pub control_field: Option<u16>,
    pub binding_uid: Option<DeviceUID>,
}

impl Protocol for DiscUnmuteResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::DiscUnMute
    }
}

// TODO can we make all of these From<&[u8]>
impl From<Vec<u8>> for DiscUnmuteResponse {
    fn from(bytes: Vec<u8>) -> Self {
        let binding_uid = if bytes.len() > 2 {
            Some(DeviceUID::from(&bytes[2..]))
        } else {
            None
        };
        DiscUnmuteResponse {
            control_field: Some(u16::from_be_bytes(bytes[..=1].try_into().unwrap())),
            binding_uid,
        }
    }
}

pub struct ProxiedDeviceCountRequest;

impl Protocol for ProxiedDeviceCountRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::ProxiedDeviceCount
    }
}

impl GetRequest for ProxiedDeviceCountRequest {}

pub struct ProxiedDeviceCountResponse {
    pub device_count: u16,
    pub list_change: bool,
}

impl From<Vec<u8>> for ProxiedDeviceCountResponse {
    fn from(bytes: Vec<u8>) -> Self {
        ProxiedDeviceCountResponse {
            device_count: u16::from_be_bytes(bytes[..=1].try_into().unwrap()),
            list_change: bytes[2] != 0,
        }
    }
}

impl From<ProxiedDeviceCountResponse> for Vec<u8> {
    fn from(_: ProxiedDeviceCountResponse) -> Self {
        Vec::new()
    }
}

impl Protocol for ProxiedDeviceCountResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::ProxiedDeviceCount
    }
}

impl GetRequest for ProxiedDeviceCountResponse {}

pub struct ProxiedDevicesRequest;

impl Protocol for ProxiedDevicesRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::ProxiedDevices
    }
}

impl GetRequest for ProxiedDevicesRequest {}

pub struct ProxiedDevicesResponse {
    pub device_uids: Vec<DeviceUID>,
}

impl Protocol for ProxiedDevicesResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::ProxiedDevices
    }
}

impl GetRequest for ProxiedDevicesResponse {}

impl From<Vec<u8>> for ProxiedDevicesResponse {
    fn from(bytes: Vec<u8>) -> Self {
        ProxiedDevicesResponse {
            device_uids: bytes.chunks(6).map(DeviceUID::from).collect(),
        }
    }
}

pub struct ParameterDescriptionRequest {
    parameter_id: u16,
}

impl ParameterDescriptionRequest {
    pub fn new(parameter_id: u16) -> Self {
        ParameterDescriptionRequest { parameter_id }
    }
}

impl Protocol for ParameterDescriptionRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::ParameterDescription
    }
}

impl GetRequest for ParameterDescriptionRequest {}

impl From<ParameterDescriptionRequest> for Vec<u8> {
    fn from(data: ParameterDescriptionRequest) -> Self {
        Vec::from(data.parameter_id.to_be_bytes())
    }
}

#[derive(Clone, Debug)]
pub struct ParameterDescriptionResponse {
    pub parameter_id: u16,
    pub parameter_data_size: u8,
    pub data_type: u8,
    pub command_class: SupportedCommandClasses,
    pub prefix: u8,
    pub minimum_valid_value: u32,
    pub maximum_valid_value: u32,
    pub default_value: u32,
    pub description: String,
}

impl Protocol for ParameterDescriptionResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::ParameterDescription
    }
}

impl GetRequest for ParameterDescriptionResponse {}

impl From<Vec<u8>> for ParameterDescriptionResponse {
    fn from(bytes: Vec<u8>) -> Self {
        println!("DATA IN ParameterDescriptionResponse {:02X?}", bytes);
        println!("description {:02X?}", &bytes[20..]);

        ParameterDescriptionResponse {
            parameter_id: u16::from_be_bytes(bytes[0..=1].try_into().unwrap()),
            parameter_data_size: bytes[2],
            data_type: bytes[3],
            command_class: SupportedCommandClasses::try_from(bytes[4]).unwrap(),
            prefix: bytes[5],
            minimum_valid_value: u32::from_be_bytes(bytes[8..=11].try_into().unwrap()),
            maximum_valid_value: u32::from_be_bytes(bytes[12..=15].try_into().unwrap()),
            default_value: u32::from_be_bytes(bytes[16..=19].try_into().unwrap()),
            description: String::from_utf8_lossy(&bytes[20..])
                .trim_end_matches("\0")
                .to_string(),
        }
    }
}

pub struct DeviceLabelRequest {
    device_label: String,
}

impl Protocol for DeviceLabelRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::DeviceLabel
    }
}

impl GetRequest for DeviceLabelRequest {}

impl SetRequest for DeviceLabelRequest {}

impl From<DeviceLabelRequest> for Vec<u8> {
    fn from(data: DeviceLabelRequest) -> Self {
        data.device_label.as_bytes().to_vec()
    }
}

#[derive(Clone, Debug)]
pub struct DeviceLabelResponse {
    pub device_label: String,
}

impl Protocol for DeviceLabelResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::DeviceLabel
    }
}

impl From<Vec<u8>> for DeviceLabelResponse {
    fn from(bytes: Vec<u8>) -> Self {
        DeviceLabelResponse {
            device_label: String::from_utf8_lossy(&bytes)
                .trim_end_matches("\0")
                .to_string(),
        }
    }
}

pub struct DeviceInfoRequest;

impl Protocol for DeviceInfoRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::DeviceInfo
    }
}

impl GetRequest for DeviceInfoRequest {}

impl From<DeviceInfoRequest> for Vec<u8> {
    fn from(_: DeviceInfoRequest) -> Self {
        Vec::new()
    }
}

#[derive(Debug)]
pub struct DeviceInfoResponse {
    pub protocol_version: String,
    pub model_id: u16,
    pub product_category: ProductCategory,
    pub software_version_id: u32,
    pub footprint: u16,
    pub current_personality: u8,
    pub total_personalities: u8,
    pub start_address: u16,
    pub sub_device_count: u16,
    pub sensor_count: u8,
}

impl Protocol for DeviceInfoResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::DeviceInfo
    }
}

impl From<Vec<u8>> for DeviceInfoResponse {
    fn from(bytes: Vec<u8>) -> Self {
        DeviceInfoResponse {
            protocol_version: format!("{}.{}", bytes[0], bytes[1]),
            model_id: u16::from_be_bytes(bytes[2..=3].try_into().unwrap()),
            product_category: ProductCategory::from(&bytes[4..=5]),
            software_version_id: u32::from_be_bytes(bytes[6..=9].try_into().unwrap()),
            footprint: u16::from_be_bytes(bytes[10..=11].try_into().unwrap()),
            current_personality: bytes[12],
            total_personalities: bytes[13],
            start_address: u16::from_be_bytes(bytes[14..=15].try_into().unwrap()),
            sub_device_count: u16::from_be_bytes(bytes[16..=17].try_into().unwrap()),
            sensor_count: u8::from_be(bytes[18]),
        }
    }
}

pub struct SoftwareVersionLabelRequest;

impl Protocol for SoftwareVersionLabelRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::SoftwareVersionLabel
    }
}

impl GetRequest for SoftwareVersionLabelRequest {}

impl From<SoftwareVersionLabelRequest> for Vec<u8> {
    fn from(_: SoftwareVersionLabelRequest) -> Self {
        Vec::new()
    }
}

#[derive(Debug)]
pub struct SoftwareVersionLabelResponse {
    pub software_version_label: String,
}

impl Protocol for SoftwareVersionLabelResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::SoftwareVersionLabel
    }
}

impl From<Vec<u8>> for SoftwareVersionLabelResponse {
    fn from(bytes: Vec<u8>) -> Self {
        SoftwareVersionLabelResponse {
            software_version_label: String::from_utf8_lossy(&bytes)
                .trim_end_matches("\0")
                .to_string(),
        }
    }
}

pub struct SupportedParametersRequest;

impl Protocol for SupportedParametersRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::SupportedParameters
    }
}

impl GetRequest for SupportedParametersRequest {}

impl From<SupportedParametersRequest> for Vec<u8> {
    fn from(_: SupportedParametersRequest) -> Self {
        Vec::new()
    }
}

#[derive(Debug)]
pub struct SupportedParametersResponse {
    pub standard_parameters: Vec<ParameterId>,
    pub manufacturer_specific_parameters: HashMap<u16, ManufacturerSpecificParameter>,
}

impl Protocol for SupportedParametersResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::SupportedParameters
    }
}

impl From<Vec<u8>> for SupportedParametersResponse {
    fn from(bytes: Vec<u8>) -> Self {
        let parameters = bytes
            .chunks(2)
            .map(|chunk| u16::from_be_bytes(chunk.try_into().unwrap()));
        SupportedParametersResponse {
            standard_parameters: parameters
                .clone()
                .filter(|parameter_id| {
                    // TODO consider if we should filter parameters here or before we add to the queue
                    let parameter_id = *parameter_id;
                    parameter_id >= 0x0060_u16 && parameter_id < 0x8000_u16
                })
                .map(ParameterId::from)
                .collect(),
            manufacturer_specific_parameters: parameters
                .filter(|parameter_id| *parameter_id >= 0x8000_u16)
                .map(|parameter_id| {
                    (
                        parameter_id,
                        ManufacturerSpecificParameter {
                            parameter_id,
                            ..Default::default()
                        },
                    )
                })
                .collect(),
        }
    }
}

pub struct IdentifyDeviceGetRequest;

impl Protocol for IdentifyDeviceGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::IdentifyDevice
    }
}

impl GetRequest for IdentifyDeviceGetRequest {}

impl From<IdentifyDeviceGetRequest> for Vec<u8> {
    fn from(_: IdentifyDeviceGetRequest) -> Self {
        Vec::new()
    }
}

pub struct IdentifyDeviceSetRequest {
    pub is_identifying: bool,
}

impl Protocol for IdentifyDeviceSetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::IdentifyDevice
    }
}

impl SetRequest for IdentifyDeviceSetRequest {}

impl From<IdentifyDeviceSetRequest> for Vec<u8> {
    fn from(data: IdentifyDeviceSetRequest) -> Self {
        Vec::from([data.is_identifying as u8])
    }
}

#[derive(Debug)]
pub struct IdentifyDeviceResponse {
    pub is_identifying: bool,
}

impl Protocol for IdentifyDeviceResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::IdentifyDevice
    }
}

impl From<Vec<u8>> for IdentifyDeviceResponse {
    fn from(bytes: Vec<u8>) -> Self {
        IdentifyDeviceResponse {
            is_identifying: bytes[0] != 0,
        }
    }
}

pub struct ManufacturerLabelGetRequest;

impl Protocol for ManufacturerLabelGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::ManufacturerLabel
    }
}

impl GetRequest for ManufacturerLabelGetRequest {}

impl From<ManufacturerLabelGetRequest> for Vec<u8> {
    fn from(_: ManufacturerLabelGetRequest) -> Self {
        Vec::new()
    }
}

pub struct ManufacturerLabelSetRequest {
    pub manufacturer_label: String,
}

impl Protocol for ManufacturerLabelSetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::ManufacturerLabel
    }
}

impl SetRequest for ManufacturerLabelSetRequest {}

impl From<ManufacturerLabelSetRequest> for Vec<u8> {
    fn from(data: ManufacturerLabelSetRequest) -> Vec<u8> {
        data.manufacturer_label.as_bytes().to_vec()
    }
}

#[derive(Clone, Debug)]
pub struct ManufacturerLabelResponse {
    pub manufacturer_label: String,
}

impl Protocol for ManufacturerLabelResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::ManufacturerLabel
    }
}

impl From<Vec<u8>> for ManufacturerLabelResponse {
    fn from(bytes: Vec<u8>) -> Self {
        ManufacturerLabelResponse {
            manufacturer_label: String::from_utf8_lossy(&bytes)
                .trim_end_matches("\0")
                .to_string(),
        }
    }
}

// pub struct FactoryDefaultsResponse {
//     factory_default: bool,
// }

// impl From<Vec<u8>> for FactoryDefaultsResponse {
//     fn from(bytes: Vec<u8>) -> Self {
//         FactoryDefaultsResponse {
//             factory_default: bytes[0] != 0,
//         }
//     }
// }

// impl TryFrom<Vec<u8>> for Response<FactoryDefaultsResponse> {
//     type Error = &'static str;

//     fn try_from(packet: Vec<u8>) -> Result<Self, Self::Error> {
//         let ResponseHeader {
//             destination_uid,
//             source_uid,
//             transaction_number,
//             response_type,
//             message_count,
//             sub_device,
//             command_class,
//             parameter_id,
//             parameter_data_length,
//         } = Response::<()>::parse_packet_header(packet.clone());

//         let parameter_data: FactoryDefaultsResponse =
//             Vec::from(&packet[24..(packet.len() + parameter_data_length as usize)])
//                 .try_into()
//                 .unwrap();

//         Ok(Response {
//             destination_uid,
//             source_uid,
//             transaction_number,
//             response_type,
//             message_count,
//             sub_device,
//             command_class,
//             parameter_id,
//             parameter_data_length,
//             parameter_data: Some(parameter_data),
//         })
//     }
// }

// pub struct DeviceModelDescriptionResponse {
//     description: String,
// }

// impl From<Vec<u8>> for DeviceModelDescriptionResponse {
//     fn from(bytes: Vec<u8>) -> Self {
//         DeviceModelDescriptionResponse {
//             description: String::from_utf8(bytes).unwrap(),
//         }
//     }
// }

// impl TryFrom<Vec<u8>> for Response<DeviceModelDescriptionResponse> {
//     type Error = &'static str;

//     fn try_from(packet: Vec<u8>) -> Result<Self, Self::Error> {
//         let ResponseHeader {
//             destination_uid,
//             source_uid,
//             transaction_number,
//             response_type,
//             message_count,
//             sub_device,
//             command_class,
//             parameter_id,
//             parameter_data_length,
//         } = Response::<()>::parse_packet_header(packet.clone());

//         let parameter_data: DeviceModelDescriptionResponse =
//             Vec::from(&packet[24..(packet.len() + parameter_data_length as usize)])
//                 .try_into()
//                 .unwrap();

//         Ok(Response {
//             destination_uid,
//             source_uid,
//             transaction_number,
//             response_type,
//             message_count,
//             sub_device,
//             command_class,
//             parameter_id,
//             parameter_data_length,
//             parameter_data: Some(parameter_data),
//         })
//     }
// }

// pub struct ProductDetailIdListResponse {
//     product_detail_ids: Vec<u16>,
// }

// impl From<Vec<u8>> for ProductDetailIdListResponse {
//     fn from(bytes: Vec<u8>) -> Self {
//         ProductDetailIdListResponse {
//             product_detail_ids: bytes
//                 .chunks(2)
//                 .map(|id| u16::from_be_bytes(id.try_into().unwrap()))
//                 .collect(),
//         }
//     }
// }

// impl TryFrom<Vec<u8>> for Response<ProductDetailIdListResponse> {
//     type Error = &'static str;

//     fn try_from(packet: Vec<u8>) -> Result<Self, Self::Error> {
//         let ResponseHeader {
//             destination_uid,
//             source_uid,
//             transaction_number,
//             response_type,
//             message_count,
//             sub_device,
//             command_class,
//             parameter_id,
//             parameter_data_length,
//         } = Response::<()>::parse_packet_header(packet.clone());

//         let parameter_data: ProductDetailIdListResponse =
//             Vec::from(&packet[24..(packet.len() + parameter_data_length as usize)])
//                 .try_into()
//                 .unwrap();

//         Ok(Response {
//             destination_uid,
//             source_uid,
//             transaction_number,
//             response_type,
//             message_count,
//             sub_device,
//             command_class,
//             parameter_id,
//             parameter_data_length,
//             parameter_data: Some(parameter_data),
//         })
//     }
// }
