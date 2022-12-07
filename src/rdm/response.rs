use super::{bsd_16_crc, device::DeviceUID, CommandClass, ParameterId, ResponseType};

pub struct ResponseHeader {
    destination_uid: DeviceUID,
    source_uid: DeviceUID,
    transaction_number: u8,
    message_count: u8,
    response_type: ResponseType,
    sub_device: u16,
    command_class: CommandClass,
    parameter_id: ParameterId,
    parameter_data_length: u8,
}

#[derive(Debug)]
pub struct Response<T> {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub response_type: ResponseType,
    pub message_count: u8,
    pub sub_device: u16,
    pub command_class: CommandClass,
    pub parameter_id: ParameterId,
    pub parameter_data_length: u8,
    pub parameter_data: Option<T>,
}

impl<T> Response<T> {
    // Packet Format
    // [0] Start Code = 1 byte
    // [1] Sub Start Code = 1 byte
    // [2] Message Length = 1 byte
    // [3-8] Destination UID = 6 bytes (48 bit)
    // [9-14] Source UID = 6 bytes (48 bit)
    // [15] Transaction Number (TN) = 1 byte
    // [16] Port ID / Response Type = 1 byte
    // [17] Message Count = 1 byte
    // [18-19] Sub-Device = 2 bytes
    // [20] Command-Class = 1 byte
    // [21-22] Parameter ID = 2 bytes
    // [23] Parameter Data Length = 1 byte
    // [24-N] Parameter Data = Variable Length
    // [N-N+2] Checksum = 2 bytes

    fn parse_packet_header(packet: Vec<u8>) -> ResponseHeader {
        ResponseHeader {
            destination_uid: DeviceUID::from(&packet[3..=8]),
            source_uid: DeviceUID::from(&packet[9..=14]),
            transaction_number: u8::from_be(packet[15]),
            response_type: ResponseType::try_from(packet[16]).unwrap(),
            message_count: u8::from_be(packet[17]),
            sub_device: u16::from_be_bytes(packet[18..=19].try_into().unwrap()),
            command_class: CommandClass::try_from(packet[20]).unwrap(),
            parameter_id: ParameterId::from(&packet[21..=22]),
            parameter_data_length: u8::from_be(packet[23]),
        }
    }
}

#[derive(Debug)]
pub struct DiscUniqueBranchResponse {
    pub device_uid: DeviceUID,
}

impl TryFrom<Vec<u8>> for DiscUniqueBranchResponse {
    type Error = &'static str;

    fn try_from(packet: Vec<u8>) -> Result<Self, Self::Error> {
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

        let device_id = u32::from_be_bytes([euid[4] & euid[5], euid[6] & euid[7], euid[8] & euid[9], euid[10] & euid[11]]);

        Ok(DiscUniqueBranchResponse {
            device_uid: DeviceUID::new(manufacturer_id, device_id),
        })
    }
}

pub struct DiscMuteResponse {
    control_field: u16,
    binding_uid: Option<DeviceUID>,
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

impl TryFrom<Vec<u8>> for Response<DiscMuteResponse> {
    type Error = &'static str;

    fn try_from(packet: Vec<u8>) -> Result<Self, Self::Error> {
        let ResponseHeader {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
        } = Response::<()>::parse_packet_header(packet.clone());

        let parameter_data: DiscMuteResponse =
            Vec::from(&packet[24..(packet.len() + parameter_data_length as usize)])
                .try_into()
                .unwrap();

        Ok(Response {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
            parameter_data: Some(parameter_data),
        })
    }
}

pub struct DiscUnmuteResponse {
    control_field: u16,
    binding_uid: Option<DeviceUID>,
}

impl From<Vec<u8>> for DiscUnmuteResponse {
    fn from(bytes: Vec<u8>) -> Self {
        let binding_uid = if bytes.len() > 2 {
            Some(DeviceUID::from(&bytes[2..]))
        } else {
            None
        };
        DiscUnmuteResponse {
            control_field: u16::from_be_bytes(bytes[..=1].try_into().unwrap()),
            binding_uid,
        }
    }
}

impl TryFrom<Vec<u8>> for Response<DiscUnmuteResponse> {
    type Error = &'static str;

    fn try_from(packet: Vec<u8>) -> Result<Self, Self::Error> {
        let ResponseHeader {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
        } = Response::<()>::parse_packet_header(packet.clone());

        let parameter_data: DiscUnmuteResponse =
            Vec::from(&packet[24..(packet.len() + parameter_data_length as usize)])
                .try_into()
                .unwrap();

        Ok(Response {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
            parameter_data: Some(parameter_data),
        })
    }
}

pub struct ProxiedDeviceCountResponse {
    device_count: u16,
    list_change: bool,
}

impl From<Vec<u8>> for ProxiedDeviceCountResponse {
    fn from(bytes: Vec<u8>) -> Self {
        ProxiedDeviceCountResponse {
            device_count: u16::from_be_bytes(bytes[..=1].try_into().unwrap()),
            list_change: bytes[2] != 0,
        }
    }
}

impl TryFrom<Vec<u8>> for Response<ProxiedDeviceCountResponse> {
    type Error = &'static str;

    fn try_from(packet: Vec<u8>) -> Result<Self, Self::Error> {
        let ResponseHeader {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
        } = Response::<()>::parse_packet_header(packet.clone());

        let parameter_data: ProxiedDeviceCountResponse =
            Vec::from(&packet[24..(packet.len() + parameter_data_length as usize)])
                .try_into()
                .unwrap();

        Ok(Response {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
            parameter_data: Some(parameter_data),
        })
    }
}

pub struct ProxiedDevicesResponse {
    device_uids: Vec<DeviceUID>,
}

impl From<Vec<u8>> for ProxiedDevicesResponse {
    fn from(bytes: Vec<u8>) -> Self {
        ProxiedDevicesResponse {
            device_uids: bytes.chunks(6).map(DeviceUID::from).collect(),
        }
    }
}

impl TryFrom<Vec<u8>> for Response<ProxiedDevicesResponse> {
    type Error = &'static str;

    fn try_from(packet: Vec<u8>) -> Result<Self, Self::Error> {
        let ResponseHeader {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
        } = Response::<()>::parse_packet_header(packet.clone());

        let parameter_data: ProxiedDevicesResponse =
            Vec::from(&packet[24..(packet.len() + parameter_data_length as usize)])
                .try_into()
                .unwrap();

        Ok(Response {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
            parameter_data: Some(parameter_data),
        })
    }
}

pub struct SupportParametersResponse {
    parameter_ids: Vec<ParameterId>,
}

impl From<Vec<u8>> for SupportParametersResponse {
    fn from(bytes: Vec<u8>) -> Self {
        SupportParametersResponse {
            parameter_ids: bytes.chunks(2).map(ParameterId::from).collect(),
        }
    }
}

impl TryFrom<Vec<u8>> for Response<SupportParametersResponse> {
    type Error = &'static str;

    fn try_from(packet: Vec<u8>) -> Result<Self, Self::Error> {
        let ResponseHeader {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
        } = Response::<()>::parse_packet_header(packet.clone());

        let parameter_data: SupportParametersResponse =
            Vec::from(&packet[24..(packet.len() + parameter_data_length as usize)])
                .try_into()
                .unwrap();

        Ok(Response {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
            parameter_data: Some(parameter_data),
        })
    }
}

pub struct ParameterDescriptionResponse {
    parameter_id: ParameterId,
    parameter_data_size: u8,
    data_type: u8,
    command_class: CommandClass,
    prefix: u8,
    minimum_valid_value: u32,
    maximum_valid_value: u32,
    default_value: u32,
    description: String,
}

impl From<Vec<u8>> for ParameterDescriptionResponse {
    fn from(bytes: Vec<u8>) -> Self {
        ParameterDescriptionResponse {
            parameter_id: ParameterId::from(&bytes[0..=1]),
            parameter_data_size: bytes[2],
            data_type: bytes[3],
            command_class: CommandClass::try_from(bytes[4]).unwrap(),
            prefix: bytes[5],
            minimum_valid_value: u32::from_be_bytes(bytes[8..=9].try_into().unwrap()),
            maximum_valid_value: u32::from_be_bytes(bytes[10..=11].try_into().unwrap()),
            default_value: u32::from_be_bytes(bytes[12..=13].try_into().unwrap()),
            description: String::from_utf8(bytes[14..].to_vec()).unwrap(),
        }
    }
}

impl TryFrom<Vec<u8>> for Response<ParameterDescriptionResponse> {
    type Error = &'static str;

    fn try_from(packet: Vec<u8>) -> Result<Self, Self::Error> {
        let ResponseHeader {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
        } = Response::<()>::parse_packet_header(packet.clone());

        let parameter_data: ParameterDescriptionResponse =
            Vec::from(&packet[24..(packet.len() + parameter_data_length as usize)])
                .try_into()
                .unwrap();

        Ok(Response {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
            parameter_data: Some(parameter_data),
        })
    }
}

#[derive(Debug)]
pub struct DeviceInfoResponse {
    pub protocol_version: String,
    pub model_id: u16,
    pub product_category_coarse: u8,
    pub product_category_fine: u8,
    pub software_version_id: u32,
    pub footprint: u16,
    pub current_personality: u8,
    pub total_personalities: u8,
    pub start_address: u16,
    pub sub_device_count: u16,
    pub sensor_count: u8,
}

impl From<Vec<u8>> for DeviceInfoResponse {
    fn from(bytes: Vec<u8>) -> Self {
        let bytes = bytes[..=18].to_vec();

        DeviceInfoResponse {
            protocol_version: format!("{}.{}", bytes[0], bytes[1]),
            model_id: u16::from_be_bytes(bytes[2..=3].try_into().unwrap()),
            product_category_coarse: bytes[4], // TODO use enum type
            product_category_fine: bytes[5], // TODO use enum type
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

impl TryFrom<Vec<u8>> for Response<DeviceInfoResponse> {
    type Error = &'static str;

    fn try_from(packet: Vec<u8>) -> Result<Self, Self::Error> {
        let ResponseHeader {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
        } = Response::<()>::parse_packet_header(packet.clone());

        let parameter_data: DeviceInfoResponse = DeviceInfoResponse::from(packet[24..=(24 + parameter_data_length as usize)].to_vec());

        Ok(Response {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
            parameter_data: Some(parameter_data),
        })
    }
}

pub struct ProductDetailIdListResponse {
    product_detail_ids: Vec<u16>,
}

impl From<Vec<u8>> for ProductDetailIdListResponse {
    fn from(bytes: Vec<u8>) -> Self {
        ProductDetailIdListResponse {
            product_detail_ids: bytes
                .chunks(2)
                .map(|id| u16::from_be_bytes(id.try_into().unwrap()))
                .collect(),
        }
    }
}

impl TryFrom<Vec<u8>> for Response<ProductDetailIdListResponse> {
    type Error = &'static str;

    fn try_from(packet: Vec<u8>) -> Result<Self, Self::Error> {
        let ResponseHeader {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
        } = Response::<()>::parse_packet_header(packet.clone());

        let parameter_data: ProductDetailIdListResponse =
            Vec::from(&packet[24..(packet.len() + parameter_data_length as usize)])
                .try_into()
                .unwrap();

        Ok(Response {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
            parameter_data: Some(parameter_data),
        })
    }
}

pub struct DeviceModelDescriptionResponse {
    description: String,
}

impl From<Vec<u8>> for DeviceModelDescriptionResponse {
    fn from(bytes: Vec<u8>) -> Self {
        DeviceModelDescriptionResponse {
            description: String::from_utf8(bytes).unwrap(),
        }
    }
}

impl TryFrom<Vec<u8>> for Response<DeviceModelDescriptionResponse> {
    type Error = &'static str;

    fn try_from(packet: Vec<u8>) -> Result<Self, Self::Error> {
        let ResponseHeader {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
        } = Response::<()>::parse_packet_header(packet.clone());

        let parameter_data: DeviceModelDescriptionResponse =
            Vec::from(&packet[24..(packet.len() + parameter_data_length as usize)])
                .try_into()
                .unwrap();

        Ok(Response {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
            parameter_data: Some(parameter_data),
        })
    }
}

pub struct DeviceModelManufacturerLabelResponse {
    manufacturer_label: String,
}

impl From<Vec<u8>> for DeviceModelManufacturerLabelResponse {
    fn from(bytes: Vec<u8>) -> Self {
        DeviceModelManufacturerLabelResponse {
            manufacturer_label: String::from_utf8(bytes).unwrap(),
        }
    }
}

impl TryFrom<Vec<u8>> for Response<DeviceModelManufacturerLabelResponse> {
    type Error = &'static str;

    fn try_from(packet: Vec<u8>) -> Result<Self, Self::Error> {
        let ResponseHeader {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
        } = Response::<()>::parse_packet_header(packet.clone());

        let parameter_data: DeviceModelManufacturerLabelResponse =
            Vec::from(&packet[24..(packet.len() + parameter_data_length as usize)])
                .try_into()
                .unwrap();

        Ok(Response {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
            parameter_data: Some(parameter_data),
        })
    }
}

#[derive(Debug)]
pub struct DeviceLabelResponse {
    label: String,
}

impl From<Vec<u8>> for DeviceLabelResponse {
    fn from(bytes: Vec<u8>) -> Self {
        let bytes = bytes[..=32].to_vec();
        DeviceLabelResponse {
            label: String::from_utf8(bytes).unwrap(),
        }
    }
}

impl TryFrom<Vec<u8>> for Response<DeviceLabelResponse> {
    type Error = &'static str;

    fn try_from(packet: Vec<u8>) -> Result<Self, Self::Error> {
        let ResponseHeader {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
        } = Response::<()>::parse_packet_header(packet.clone());

        let parameter_data: DeviceLabelResponse =
            Vec::from(&packet[24..(packet.len() + parameter_data_length as usize)])
                .try_into()
                .unwrap();

        Ok(Response {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
            parameter_data: Some(parameter_data),
        })
    }
}

pub struct FactoryDefaultsResponse {
    factory_default: bool,
}

impl From<Vec<u8>> for FactoryDefaultsResponse {
    fn from(bytes: Vec<u8>) -> Self {
        FactoryDefaultsResponse {
            factory_default: bytes[0] != 0,
        }
    }
}

impl TryFrom<Vec<u8>> for Response<FactoryDefaultsResponse> {
    type Error = &'static str;

    fn try_from(packet: Vec<u8>) -> Result<Self, Self::Error> {
        let ResponseHeader {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
        } = Response::<()>::parse_packet_header(packet.clone());

        let parameter_data: FactoryDefaultsResponse =
            Vec::from(&packet[24..(packet.len() + parameter_data_length as usize)])
                .try_into()
                .unwrap();

        Ok(Response {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
            parameter_data: Some(parameter_data),
        })
    }
}

pub struct SoftwareVersionLabelResponse {
    software_label_version: String,
}

impl From<Vec<u8>> for SoftwareVersionLabelResponse {
    fn from(bytes: Vec<u8>) -> Self {
        SoftwareVersionLabelResponse {
            software_label_version: String::from_utf8(bytes).unwrap(),
        }
    }
}

impl TryFrom<Vec<u8>> for Response<SoftwareVersionLabelResponse> {
    type Error = &'static str;

    fn try_from(packet: Vec<u8>) -> Result<Self, Self::Error> {
        let ResponseHeader {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
        } = Response::<()>::parse_packet_header(packet.clone());

        let parameter_data: SoftwareVersionLabelResponse =
            Vec::from(&packet[24..(packet.len() + parameter_data_length as usize)])
                .try_into()
                .unwrap();

        Ok(Response {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device,
            command_class,
            parameter_id,
            parameter_data_length,
            parameter_data: Some(parameter_data),
        })
    }
}
