#![allow(unused)]
use std::{borrow::BorrowMut, process::Command, fmt::format};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use ux::u48;

const MIN_PACKET_LEN: usize = 26;

const SC_RDM: u8 = 0xcc;
const SC_SUB_MESSAGE: u8 = 0x01;

const BROADCAST_ALL_DEVICES_ID: u48 = u48::new(0xffffffffffff);
const SUB_DEVICE_ALL_CALL: u16 = 0xffff;
const ROOT_DEVICE: u8 = 0x00;

#[derive(Debug)]
pub enum CommandClass {
    DiscoveryCommand = 0x10,
    DiscoveryCommandResponse = 0x11,
    GetCommand = 0x20,
    GetCommandResponse = 0x21,
    SetCommand = 0x30,
    SetCommandResponse = 0x31,
}

impl TryFrom<u8> for CommandClass {
    type Error = &'static str;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        let command_class = match byte {
            0x10 => CommandClass::DiscoveryCommand,
            0x11 => CommandClass::DiscoveryCommandResponse,
            0x20 => CommandClass::GetCommand,
            0x21 => CommandClass::GetCommandResponse,
            0x30 => CommandClass::SetCommand,
            0x31 => CommandClass::SetCommandResponse,
            _ => return Err("Invalid value for CommandClass"),
        };
        Ok(command_class)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DeviceUID {
    manufacturer_id: u16,
    device_id: u32,
}

impl DeviceUID {
    pub fn new(manufacturer_id: u16, device_id: u32) -> Self {
        DeviceUID {
            manufacturer_id,
            device_id,
        }
    }
}

impl From<u64> for DeviceUID {
    fn from(value: u64) -> Self {
        DeviceUID {
            manufacturer_id: (value & 0xff0000) as u16,
            device_id: (value & 0xffff) as u32,
        }
    }
}

impl From<DeviceUID> for u64 {
    fn from(device_uid: DeviceUID) -> u64 {
        ((device_uid.manufacturer_id as u64) << 16u64) + device_uid.device_id as u64
    }
}

impl From<&[u8]> for DeviceUID {
    fn from(buffer: &[u8]) -> Self {
        DeviceUID {
            // TODO we might be able to simplify this use .read_u16()
            manufacturer_id: u16::from_be_bytes(buffer[..2].try_into().unwrap()),
            device_id: u32::from_be_bytes(buffer[2..].try_into().unwrap()),
        }
    }
}

#[derive(Debug)]
pub enum ParameterId {
    DiscUniqueBranch = 0x0001,
    DiscMute = 0x0002,
    DiscUnMute = 0x0003,
    ProxiedDevices = 0x0010,
    ProxiedDeviceCount = 0x0011,
    CommsStatus = 0x0015,
    QueuedMessage = 0x0020,
    StatusMessages = 0x0030,
    StatusIdDescription = 0x0031,
    ClearStatusId = 0x0032,
    SubDeviceStatusReportThreshold = 0x0033,
    SupportedParameters = 0x0050,
    ParameterDescription = 0x0051,
    DeviceInfo = 0x0060,
    ProductDetailIdList = 0x0070,
    DeviceModelDescription = 0x0080,
    ManufacturerLabel = 0x0081,
    DeviceLabel = 0x0082,
    FactoryDefaults = 0x0090,
    LanguageCapabilities = 0x00a0,
    Language = 0x00b0,
    SoftwareVersionLabel = 0x00c0,
    BootSoftwareVersionId = 0x00c1,
    BootSoftwareVersionLabel = 0x00c2,
}

// TODO this could use try_from and return a result rather than panic
impl From<&[u8]> for ParameterId {
    fn from(bytes: &[u8]) -> Self {
        match u16::from_be_bytes(bytes.try_into().unwrap()) {
            0x0001 => ParameterId::DiscUniqueBranch,
            0x0002 => ParameterId::DiscMute,
            0x0003 => ParameterId::DiscUnMute,
            0x0010 => ParameterId::ProxiedDevices,
            0x0011 => ParameterId::ProxiedDeviceCount,
            0x0015 => ParameterId::CommsStatus,
            0x0020 => ParameterId::QueuedMessage,
            0x0030 => ParameterId::StatusMessages,
            0x0031 => ParameterId::StatusIdDescription,
            0x0032 => ParameterId::ClearStatusId,
            0x0033 => ParameterId::SubDeviceStatusReportThreshold,
            0x0050 => ParameterId::SupportedParameters,
            0x0051 => ParameterId::ParameterDescription,
            0x0060 => ParameterId::DeviceInfo,
            0x0070 => ParameterId::ProductDetailIdList,
            0x0080 => ParameterId::DeviceModelDescription,
            0x0081 => ParameterId::ManufacturerLabel,
            0x0082 => ParameterId::DeviceLabel,
            0x0090 => ParameterId::FactoryDefaults,
            0x00a0 => ParameterId::LanguageCapabilities,
            0x00b0 => ParameterId::Language,
            0x00c0 => ParameterId::SoftwareVersionLabel,
            0x00c1 => ParameterId::BootSoftwareVersionId,
            0x00c2 => ParameterId::BootSoftwareVersionLabel,
            _ => panic!("Invalid value for ParameterId: {:?}", bytes),
        }
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
        vec.write_u48::<BigEndian>(disc_unique_branch_data.lower_bound_uid.into());
        vec.write_u48::<BigEndian>(disc_unique_branch_data.upper_bound_uid.into());
        vec
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

pub struct ProxiedDeviceCountResponse {
    device_count: u16,
    list_change: bool,
}

pub struct ProxiedDevicesResponse {
    device_uids: Vec<DeviceUID>,
}

pub struct SupportParametersResponse {
    parameter_ids: Vec<ParameterId>,
}

pub struct ParameterDescriptionRequest {
    parameter_id: ParameterId,
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

pub struct DeviceInfoResponse {
    rdm_protocol_version: u16,
    device_model_id: u16,
    product_category: u16,
    software_version_id: u32,
    dmx_footprint: u16,
    dmx_personality: u16,
    dmx_start_address: u16,
    sub_device_count: u16,
    sensor_count: u8,
}

pub struct ProductDetailIdListResponse {
    product_detail_ids: Vec<u16>,
}

pub struct DeviceModelDescriptionResponse {
    description: String,
}

pub struct DeviceModelManufacturerLabelResponse {
    manufacturer_label: String,
}

pub struct DeviceLabelRequest {
    label: Option<String>,
}

pub struct DeviceLabelResponse {
    label: Option<String>,
}

pub struct FactoryDefaultsResponse {
    factory_default: Option<bool>,
}

pub struct SoftwareVersionLabelResponse {
    software_label_version: Option<String>,
}

fn bsd_16_crc(packet: &Vec<u8>) -> u16 {
    packet
        .iter()
        .fold(0_u16, |sum, byte| (sum.overflowing_add(*byte as u16).0))
}

enum ResponseType {
    Ack = 0x00,
    AckTimer = 0x01,
    NackReason = 0x02,
    AckOverflow = 0x03,
}

enum ResponseNackReasonCode {
    UnknownPid = 0x0000,
    FormatError = 0x0001,
    HardwareFault = 0x0002,
    ProxyReject = 0x0003,
    WriteProtect = 0x0004,
    UnsupportedCommandClass = 0x0005,
    DataOutOfRange = 0x0006,
    BufferFull = 0x0007,
    PacketSizeUnsupported = 0x0008,
    SubDeviceOutOfRange = 0x0009,
    ProxyBufferFull = 0x000a,
}

#[derive(Debug)]
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

    fn create_packet(self, parameter_data: Vec<u8>) -> Vec<u8> {
        let mut packet = Vec::new();
        packet.write_u8(SC_RDM).unwrap(); // Start Code
        packet.write_u8(SC_SUB_MESSAGE).unwrap(); // Sub Start Code

        packet.write_u8(24_u8 + parameter_data.len() as u8); // Message Length: Range 24 to 255 excluding the checksum
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
        packet.extend(parameter_data);

        packet.write_u16::<BigEndian>(bsd_16_crc(&packet)).unwrap();
        packet
    }
}

impl From<Request<String>> for Vec<u8> {
    fn from(request: Request<String>) -> Self {
        let parameter_data = if let Some(data) = request.parameter_data.clone() {
            data.into_bytes()
        } else {
            Vec::<u8>::new()
        };
        request.create_packet(parameter_data)
    }
}

impl From<Request<DiscUniqueBranchRequest>> for Vec<u8> {
    fn from(request: Request<DiscUniqueBranchRequest>) -> Vec<u8> {
        let parameter_data: Vec<u8> = request.parameter_data.unwrap().into();
        request.create_packet(parameter_data)
    }
}

pub struct ResponseHeader {
    destination_uid: DeviceUID,
    source_uid: DeviceUID,
    transaction_number: u8,
    message_count: u8,
    response_type: u8,
    sub_device: u16,
    command_class: CommandClass,
    parameter_id: ParameterId,
    parameter_data_length: u8,
}

pub struct Response<T> {
    destination_uid: DeviceUID,
    source_uid: DeviceUID,
    transaction_number: u8,
    response_type: u8,
    message_count: u8,
    sub_device: u16,
    command_class: CommandClass,
    parameter_id: ParameterId,
    parameter_data_length: u8,
    parameter_data: Option<T>,
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
    // fn parse_packet(packet: Vec<u8>) -> Response<T> {
    //     // the first 2 bytes can ignored
    //     let destination_uid = DeviceUID::from(&packet[3..=8]);
    //     let source_uid = DeviceUID::from(&packet[9..=14]);
    //     let transaction_number = u8::from_be(packet[15]);
    //     let response_type = u8::from_be(packet[16]); // TODO This can be an enum
    //     let message_count = u8::from_be(packet[17]);
    //     let sub_device = u16::from_be_bytes(packet[18..=19].try_into().unwrap());
    //     let command_class = CommandClass::from(packet[20]);
    //     let parameter_id = ParameterId::from(&packet[21..=22]);
    //     let parameter_data_length = u8::from_be(packet[23]);
    //     let parameter_data = Vec::from(&packet[24..(packet.len() + parameter_data_length as usize)]).try_into().unwrap();

    //     Response {
    //         destination_uid,
    //         source_uid,
    //         transaction_number,
    //         response_type,
    //         sub_device,
    //         command_class,
    //         parameter_id,
    //         parameter_data: Some(parameter_data.into())
    //     }
    // }

    fn parse_packer_header(packet: Vec<u8>) -> ResponseHeader {
        ResponseHeader {
            destination_uid: DeviceUID::from(&packet[3..=8]),
            source_uid: DeviceUID::from(&packet[9..=14]),
            transaction_number: u8::from_be(packet[15]),
            response_type: u8::from_be(packet[16]),
            message_count: u8::from_be(packet[17]),
            sub_device: u16::from_be_bytes(packet[18..=19].try_into().unwrap()),
            command_class: CommandClass::try_from(packet[20]).unwrap(),
            parameter_id: ParameterId::from(&packet[21..=22]),
            parameter_data_length: u8::from_be(packet[23]),
        }
    }
}

struct DiscUniqueBranchResponse {
    device_uid: DeviceUID,
    euid: Vec<u8>,
    ecs: Vec<u8>,
    checksum: u16,
}

// TODO add error handling for TryFrom Result
impl TryFrom<Vec<u8>> for DiscUniqueBranchResponse {
    type Error = &'static str;

    fn try_from(packet: Vec<u8>) -> Result<Self, Self::Error> {
        let euid_start_index = packet
            .iter()
            .position(|&x| x == 0xaa) // &x -> accessing the element value by reference
            .unwrap();

        let euid = Vec::from(&packet[euid_start_index..packet.len() - 4]);

        let ecs = Vec::from(&packet[(packet.len() - 4)..packet.len()]);

        let decoded_checksum = bsd_16_crc(&euid);

        // let checksum = Buffer.from([ecs.0 & ecs[1], ecs[2] & ecs[3]]).readUInt16BE(0);
        let checksum = (((ecs[0] & ecs[1]) << 8_u16) + ecs[2] & ecs[3]) as u16;

        if checksum != decoded_checksum {
            return Err("Checksum does not match decoded checksum");
        }

        // TODO if checksum is incorrect then multiple devices have responded to the current branch
        let manufacturer_id = (((euid[0] & euid[1]) << 8_u16) + euid[2] & euid[3]) as u16;

        let device_id = (((euid[4] & euid[5]) << 24_u32)
            + ((euid[6] & euid[7]) << 16_u32)
            + ((euid[8] & euid[9]) << 8_u32)
            + (euid[10] & euid[11])) as u32;

        Ok(DiscUniqueBranchResponse {
            device_uid: DeviceUID::new(manufacturer_id, device_id),
            euid,
            ecs,
            checksum,
        })
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
        } = Response::<()>::parse_packer_header(packet.clone());

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
        } = Response::<()>::parse_packer_header(packet.clone());

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

pub struct Device {
    uid: DeviceUID,
    protocol_version: Option<u16>,
    protocol_version_string: Option<String>,
    model_id: Option<u16>,
    model_description: Option<String>,
    product_category: Option<u16>,
    software_version_id: Option<u32>,
    footprint: Option<u16>,
    personality: Option<u16>,
    start_address: Option<u16>,
    sub_device_count: Option<u16>,
    sensor_count: Option<u8>,
}
