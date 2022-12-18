use byteorder::{BigEndian, WriteBytesExt};
use std::collections::HashMap;
use thiserror::Error;
use ux::u48;

use super::{
    bsd_16_crc,
    device::{DeviceUID, DmxSlot},
    DiscoveryRequest, DisplayInvertMode, GetRequest, LampOnMode, LampState, ProductCategory,
    Protocol, SetRequest, SupportedCommandClasses,
};

#[derive(Debug, Error)]
pub enum ParameterError {
    #[error("unsupported parameter")]
    UnsupportedParameter,
    #[error("unknown parameter error")]
    Unknown,
}

// TODO add remaining parameter ids
#[derive(Copy, Clone, Debug)]
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
    DmxPersonality = 0x00e0,
    DmxPersonalityDescription = 0x00e1,
    DmxStartAddress = 0x00f0,
    SlotInfo = 0x0120,
    SlotDescription = 0x0121,
    DefaultSlotValue = 0x0122,
    SensorDefinition = 0x0200,
    SensorValue = 0x0201,
    RecordSensors = 0x0202,
    DimmerInfo = 0x0340,
    MinimumLevel = 0x0341,
    MaximumLevel = 0x0342,
    Curve = 0x0343,
    CurveDescription = 0x0344,
    OutputResponseTime = 0x0345,
    OutputResponseTimeDescription = 0x0346,
    ModulationFrequency = 0x0347,
    ModulationFrequencyDescription = 0x0348,
    OutputResponseTimeDown = 0x0371,
    OutputResponseTimeDownDescription = 0x0372,
    DeviceHours = 0x0400,
    LampHours = 0x0401,
    LampStrikes = 0x0402,
    LampState = 0x0403,
    LampOnMode = 0x0404,
    DevicePowerCycles = 0x0405,
    DisplayInvert = 0x0500,
    DisplayLevel = 0x0501,
    PanInvert = 0x0600,
    TiltInvert = 0x0601,
    PanTiltSwap = 0x0602,
    RealTimeClock = 0x0603,
    IdentifyDevice = 0x1000,
    ResetDevice = 0x1001,
    PowerState = 0x1010,
    PerformSelfTest = 0x1020,
    SelfTestDescription = 0x1021,
    CapturePreset = 0x1030,
    PresetPlayback = 0x1031,
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
            0x00e0 => ParameterId::DmxPersonality,
            0x00e1 => ParameterId::DmxPersonalityDescription,
            0x00f0 => ParameterId::DmxStartAddress,
            0x0120 => ParameterId::SlotInfo,
            0x0121 => ParameterId::SlotDescription,
            0x0122 => ParameterId::DefaultSlotValue,
            0x0200 => ParameterId::SensorDefinition,
            0x0201 => ParameterId::SensorValue,
            0x0202 => ParameterId::RecordSensors,
            0x0340 => ParameterId::DimmerInfo,
            0x0341 => ParameterId::MinimumLevel,
            0x0342 => ParameterId::MaximumLevel,
            0x0343 => ParameterId::Curve,
            0x0344 => ParameterId::CurveDescription,
            0x0345 => ParameterId::OutputResponseTime,
            0x0346 => ParameterId::OutputResponseTimeDescription,
            0x0347 => ParameterId::ModulationFrequency,
            0x0348 => ParameterId::ModulationFrequencyDescription,
            0x0400 => ParameterId::DeviceHours,
            0x0401 => ParameterId::LampHours,
            0x0402 => ParameterId::LampStrikes,
            0x0403 => ParameterId::LampState,
            0x0404 => ParameterId::LampOnMode,
            0x0405 => ParameterId::DevicePowerCycles,
            0x0500 => ParameterId::DisplayInvert,
            0x0501 => ParameterId::DisplayLevel,
            0x0600 => ParameterId::PanInvert,
            0x0601 => ParameterId::TiltInvert,
            0x0602 => ParameterId::PanTiltSwap,
            0x0603 => ParameterId::RealTimeClock,
            0x1000 => ParameterId::IdentifyDevice,
            0x1001 => ParameterId::ResetDevice,
            0x1010 => ParameterId::PowerState,
            0x1020 => ParameterId::PerformSelfTest,
            0x1021 => ParameterId::SelfTestDescription,
            0x1030 => ParameterId::CapturePreset,
            0x1031 => ParameterId::PresetPlayback,
            _ => panic!("Invalid value for ParameterId: 0x{:04X?}", bytes),
        }
    }
}

// TODO this could use try_from and return a result rather than panic
impl From<u16> for ParameterId {
    fn from(parameter_id: u16) -> Self {
        match parameter_id {
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
            0x00e0 => ParameterId::DmxPersonality,
            0x00e1 => ParameterId::DmxPersonalityDescription,
            0x00f0 => ParameterId::DmxStartAddress,
            0x0120 => ParameterId::SlotInfo,
            0x0121 => ParameterId::SlotDescription,
            0x0122 => ParameterId::DefaultSlotValue,
            0x0200 => ParameterId::SensorDefinition,
            0x0201 => ParameterId::SensorValue,
            0x0202 => ParameterId::RecordSensors,
            0x0340 => ParameterId::DimmerInfo,
            0x0341 => ParameterId::MinimumLevel,
            0x0342 => ParameterId::MaximumLevel,
            0x0343 => ParameterId::Curve,
            0x0344 => ParameterId::CurveDescription,
            0x0345 => ParameterId::OutputResponseTime,
            0x0346 => ParameterId::OutputResponseTimeDescription,
            0x0347 => ParameterId::ModulationFrequency,
            0x0348 => ParameterId::ModulationFrequencyDescription,
            0x0400 => ParameterId::DeviceHours,
            0x0401 => ParameterId::LampHours,
            0x0402 => ParameterId::LampStrikes,
            0x0403 => ParameterId::LampState,
            0x0404 => ParameterId::LampOnMode,
            0x0405 => ParameterId::DevicePowerCycles,
            0x0500 => ParameterId::DisplayInvert,
            0x0501 => ParameterId::DisplayLevel,
            0x0600 => ParameterId::PanInvert,
            0x0601 => ParameterId::TiltInvert,
            0x0602 => ParameterId::PanTiltSwap,
            0x0603 => ParameterId::RealTimeClock,
            0x1000 => ParameterId::IdentifyDevice,
            0x1001 => ParameterId::ResetDevice,
            0x1010 => ParameterId::PowerState,
            0x1020 => ParameterId::PerformSelfTest,
            0x1021 => ParameterId::SelfTestDescription,
            0x1030 => ParameterId::CapturePreset,
            0x1031 => ParameterId::PresetPlayback,
            _ => panic!("Invalid value for ParameterId: 0x{:04X?}", parameter_id),
        }
    }
}

pub const REQUIRED_PARAMETERS: [ParameterId; 4] = [
    ParameterId::DeviceInfo,
    ParameterId::SupportedParameters,
    ParameterId::SoftwareVersionLabel,
    ParameterId::IdentifyDevice,
];

#[derive(Clone, Debug, Default)]
pub struct ManufacturerSpecificParameter {
    pub parameter_id: u16,
    pub parameter_data_size: Option<u8>, // TODO use enum
    pub data_type: Option<u8>,           // TODO use enum
    pub command_class: Option<SupportedCommandClasses>,
    pub prefix: Option<u8>, // TODO use enum
    pub minimum_valid_value: Option<u32>,
    pub maximum_valid_value: Option<u32>,
    pub default_value: Option<u32>,
    pub description: Option<String>,
}

impl From<u16> for ManufacturerSpecificParameter {
    fn from(parameter_id: u16) -> Self {
        ManufacturerSpecificParameter {
            parameter_id,
            ..Default::default()
        }
    }
}

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

pub struct ProxiedDevicesGetRequest;

impl Protocol for ProxiedDevicesGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::ProxiedDevices
    }
}

impl GetRequest for ProxiedDevicesGetRequest {}

impl From<ProxiedDevicesGetRequest> for Vec<u8> {
    fn from(_: ProxiedDevicesGetRequest) -> Self {
        Vec::new()
    }
}

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

pub struct ParameterDescriptionGetRequest {
    parameter_id: u16,
}

impl ParameterDescriptionGetRequest {
    pub fn new(parameter_id: u16) -> Self {
        ParameterDescriptionGetRequest { parameter_id }
    }
}

impl Protocol for ParameterDescriptionGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::ParameterDescription
    }
}

impl GetRequest for ParameterDescriptionGetRequest {}

impl From<ParameterDescriptionGetRequest> for Vec<u8> {
    fn from(data: ParameterDescriptionGetRequest) -> Self {
        Vec::from(data.parameter_id.to_be_bytes())
    }
}

#[derive(Clone, Debug)]
pub struct ParameterDescriptionGetResponse {
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

impl Protocol for ParameterDescriptionGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::ParameterDescription
    }
}

impl GetRequest for ParameterDescriptionGetResponse {}

impl From<Vec<u8>> for ParameterDescriptionGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        ParameterDescriptionGetResponse {
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
    pub personality_count: u8,
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
            personality_count: bytes[13],
            start_address: u16::from_be_bytes(bytes[14..=15].try_into().unwrap()),
            sub_device_count: u16::from_be_bytes(bytes[16..=17].try_into().unwrap()),
            sensor_count: u8::from_be(bytes[18]),
        }
    }
}

pub struct SoftwareVersionLabelGetRequest;

impl Protocol for SoftwareVersionLabelGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::SoftwareVersionLabel
    }
}

impl GetRequest for SoftwareVersionLabelGetRequest {}

impl From<SoftwareVersionLabelGetRequest> for Vec<u8> {
    fn from(_: SoftwareVersionLabelGetRequest) -> Self {
        Vec::new()
    }
}

#[derive(Debug)]
pub struct SoftwareVersionLabelGetResponse {
    pub software_version_label: String,
}

impl Protocol for SoftwareVersionLabelGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::SoftwareVersionLabel
    }
}

impl From<Vec<u8>> for SoftwareVersionLabelGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        SoftwareVersionLabelGetResponse {
            software_version_label: String::from_utf8_lossy(&bytes)
                .trim_end_matches("\0")
                .to_string(),
        }
    }
}

pub struct SupportedParametersGetRequest;

impl Protocol for SupportedParametersGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::SupportedParameters
    }
}

impl GetRequest for SupportedParametersGetRequest {}

impl From<SupportedParametersGetRequest> for Vec<u8> {
    fn from(_: SupportedParametersGetRequest) -> Self {
        Vec::new()
    }
}

#[derive(Debug)]
pub struct SupportedParametersGetResponse {
    pub standard_parameters: Vec<ParameterId>,
    pub manufacturer_specific_parameters: HashMap<u16, ManufacturerSpecificParameter>,
}

impl Protocol for SupportedParametersGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::SupportedParameters
    }
}

impl From<Vec<u8>> for SupportedParametersGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        let parameters = bytes
            .chunks(2)
            .map(|chunk| u16::from_be_bytes(chunk.try_into().unwrap()));
        SupportedParametersGetResponse {
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

pub struct FactoryDefaultsGetRequest;

impl Protocol for FactoryDefaultsGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::FactoryDefaults
    }
}

impl GetRequest for FactoryDefaultsGetRequest {}

impl From<FactoryDefaultsGetRequest> for Vec<u8> {
    fn from(_: FactoryDefaultsGetRequest) -> Self {
        Vec::new()
    }
}

pub struct FactoryDefaultsGetResponse {
    pub factory_default: bool,
}

impl Protocol for FactoryDefaultsGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::FactoryDefaults
    }
}

impl From<Vec<u8>> for FactoryDefaultsGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        FactoryDefaultsGetResponse {
            factory_default: bytes[0] != 0,
        }
    }
}

pub struct DeviceModelDescriptionGetRequest;

impl Protocol for DeviceModelDescriptionGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::DeviceModelDescription
    }
}

impl GetRequest for DeviceModelDescriptionGetRequest {}

impl From<DeviceModelDescriptionGetRequest> for Vec<u8> {
    fn from(_: DeviceModelDescriptionGetRequest) -> Self {
        Vec::new()
    }
}

#[derive(Clone, Debug)]
pub struct DeviceModelDescriptionGetResponse {
    pub device_model_description: String,
}

impl Protocol for DeviceModelDescriptionGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::ManufacturerLabel
    }
}

impl From<Vec<u8>> for DeviceModelDescriptionGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        DeviceModelDescriptionGetResponse {
            device_model_description: String::from_utf8_lossy(&bytes)
                .trim_end_matches("\0")
                .to_string(),
        }
    }
}

pub struct ProductDetailIdListGetRequest;

impl Protocol for ProductDetailIdListGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::ProductDetailIdList
    }
}

impl GetRequest for ProductDetailIdListGetRequest {}

impl From<ProductDetailIdListGetRequest> for Vec<u8> {
    fn from(_: ProductDetailIdListGetRequest) -> Self {
        Vec::new()
    }
}

#[derive(Clone, Debug)]
pub struct ProductDetailIdListGetResponse {
    pub product_detail_id_list: Vec<u16>,
}

impl Protocol for ProductDetailIdListGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::ProductDetailIdList
    }
}

impl From<Vec<u8>> for ProductDetailIdListGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        ProductDetailIdListGetResponse {
            product_detail_id_list: bytes
                .chunks(2)
                .map(|id| u16::from_be_bytes(id.try_into().unwrap()))
                .collect(),
        }
    }
}

struct DmxPersonalityGetRequest;

impl Protocol for DmxPersonalityGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::DmxPersonality
    }
}

impl GetRequest for DmxPersonalityGetRequest {}

impl From<DmxPersonalityGetRequest> for Vec<u8> {
    fn from(_: DmxPersonalityGetRequest) -> Self {
        Vec::new()
    }
}

struct DmxPersonalitySetRequest {
    personality: u8,
}

impl Protocol for DmxPersonalitySetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::DmxPersonality
    }
}

impl SetRequest for DmxPersonalitySetRequest {}

impl From<DmxPersonalitySetRequest> for Vec<u8> {
    fn from(dmx_personality: DmxPersonalitySetRequest) -> Self {
        Vec::from(dmx_personality.personality.to_be_bytes())
    }
}

pub struct DmxPersonalityGetResponse {
    pub current_personality: u8,
    pub personality_count: u8,
}

impl Protocol for DmxPersonalityGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::DmxPersonality
    }
}

impl From<Vec<u8>> for DmxPersonalityGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        DmxPersonalityGetResponse {
            current_personality: bytes[0],
            personality_count: bytes[1],
        }
    }
}

struct DmxPersonalitySetResponse;

impl Protocol for DmxPersonalitySetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::DmxPersonality
    }
}

impl From<Vec<u8>> for DmxPersonalitySetResponse {
    fn from(_: Vec<u8>) -> Self {
        DmxPersonalitySetResponse
    }
}

pub struct DmxPersonalityDescriptionGetRequest {
    personality: u8,
}

impl DmxPersonalityDescriptionGetRequest {
    pub fn new(personality: u8) -> Self {
        DmxPersonalityDescriptionGetRequest { personality }
    }
}

impl Protocol for DmxPersonalityDescriptionGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::DmxPersonalityDescription
    }
}

impl GetRequest for DmxPersonalityDescriptionGetRequest {}

impl From<DmxPersonalityDescriptionGetRequest> for Vec<u8> {
    fn from(dmx_personality_description: DmxPersonalityDescriptionGetRequest) -> Self {
        Vec::from(dmx_personality_description.personality.to_be_bytes())
    }
}

pub struct DmxPersonalityDescriptionGetResponse {
    pub personality: u8,
    pub dmx_slots_required: u16,
    pub description: String,
}

impl Protocol for DmxPersonalityDescriptionGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::DmxPersonalityDescription
    }
}

impl From<Vec<u8>> for DmxPersonalityDescriptionGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        DmxPersonalityDescriptionGetResponse {
            personality: bytes[0],
            dmx_slots_required: u16::from_be_bytes(bytes[1..=2].try_into().unwrap()),
            description: String::from_utf8_lossy(&bytes[3..])
                .trim_end_matches("\0")
                .to_string(),
        }
    }
}

pub struct SlotInfoRequest;

impl Protocol for SlotInfoRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::SlotInfo
    }
}

impl GetRequest for SlotInfoRequest {}

impl From<SlotInfoRequest> for Vec<u8> {
    fn from(_: SlotInfoRequest) -> Self {
        Vec::new()
    }
}

#[derive(Debug)]
pub struct SlotInfoResponse {
    pub dmx_slots: Vec<DmxSlot>,
}

impl Protocol for SlotInfoResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::SlotInfo
    }
}

impl From<Vec<u8>> for SlotInfoResponse {
    fn from(bytes: Vec<u8>) -> Self {
        println!("{:02X?}", bytes);
        SlotInfoResponse {
            dmx_slots: bytes.chunks(5).map(DmxSlot::from).collect(),
        }
    }
}

struct DeviceHoursGetRequest;

impl Protocol for DeviceHoursGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::DeviceHours
    }
}

impl GetRequest for DeviceHoursGetRequest {}

impl From<DeviceHoursGetRequest> for Vec<u8> {
    fn from(_: DeviceHoursGetRequest) -> Self {
        Vec::new()
    }
}

struct DeviceHoursSetRequest {
    device_hours: u32,
}

impl Protocol for DeviceHoursSetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::DeviceHours
    }
}

impl SetRequest for DeviceHoursSetRequest {}

impl From<DeviceHoursSetRequest> for Vec<u8> {
    fn from(dmx_personality: DeviceHoursSetRequest) -> Self {
        Vec::from(dmx_personality.device_hours.to_be_bytes())
    }
}

pub struct DeviceHoursGetResponse {
    pub device_hours: u32,
}

impl Protocol for DeviceHoursGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::DeviceHours
    }
}

impl From<Vec<u8>> for DeviceHoursGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        DeviceHoursGetResponse {
            device_hours: u32::from_be_bytes(bytes[0..=3].try_into().unwrap()),
        }
    }
}

struct DeviceHoursSetResponse;

impl Protocol for DeviceHoursSetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::DeviceHours
    }
}

impl From<Vec<u8>> for DeviceHoursSetResponse {
    fn from(_: Vec<u8>) -> Self {
        DeviceHoursSetResponse
    }
}

struct LampHoursGetRequest;

impl Protocol for LampHoursGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::LampHours
    }
}

impl GetRequest for LampHoursGetRequest {}

impl From<LampHoursGetRequest> for Vec<u8> {
    fn from(_: LampHoursGetRequest) -> Self {
        Vec::new()
    }
}

struct LampHoursSetRequest {
    pub lamp_hours: u32,
}

impl Protocol for LampHoursSetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::LampHours
    }
}

impl SetRequest for LampHoursSetRequest {}

impl From<LampHoursSetRequest> for Vec<u8> {
    fn from(lamp_hours: LampHoursSetRequest) -> Self {
        Vec::from(lamp_hours.lamp_hours.to_be_bytes())
    }
}

pub struct LampHoursGetResponse {
    pub lamp_hours: u32,
}

impl Protocol for LampHoursGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::LampHours
    }
}

impl From<Vec<u8>> for LampHoursGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        LampHoursGetResponse {
            lamp_hours: u32::from_be_bytes(bytes[0..=3].try_into().unwrap()),
        }
    }
}

struct LampHoursSetResponse;

impl Protocol for LampHoursSetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::LampHours
    }
}

impl From<Vec<u8>> for LampHoursSetResponse {
    fn from(_: Vec<u8>) -> Self {
        LampHoursSetResponse
    }
}

struct LampStrikesGetRequest;

impl Protocol for LampStrikesGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::LampStrikes
    }
}

impl GetRequest for LampStrikesGetRequest {}

impl From<LampStrikesGetRequest> for Vec<u8> {
    fn from(_: LampStrikesGetRequest) -> Self {
        Vec::new()
    }
}

struct LampStrikesSetRequest {
    pub lamp_strikes: u32,
}

impl Protocol for LampStrikesSetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::LampStrikes
    }
}

impl SetRequest for LampStrikesSetRequest {}

impl From<LampStrikesSetRequest> for Vec<u8> {
    fn from(lamp_strikes: LampStrikesSetRequest) -> Self {
        Vec::from(lamp_strikes.lamp_strikes.to_be_bytes())
    }
}

pub struct LampStrikesGetResponse {
    pub lamp_strikes: u32,
}

impl Protocol for LampStrikesGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::LampStrikes
    }
}

impl From<Vec<u8>> for LampStrikesGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        LampStrikesGetResponse {
            lamp_strikes: u32::from_be_bytes(bytes[0..=3].try_into().unwrap()),
        }
    }
}

struct LampStrikesSetResponse;

impl Protocol for LampStrikesSetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::LampStrikes
    }
}

impl From<Vec<u8>> for LampStrikesSetResponse {
    fn from(_: Vec<u8>) -> Self {
        LampStrikesSetResponse
    }
}

struct LampStateGetRequest;

impl Protocol for LampStateGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::LampState
    }
}

impl GetRequest for LampStateGetRequest {}

impl From<LampStateGetRequest> for Vec<u8> {
    fn from(_: LampStateGetRequest) -> Self {
        Vec::new()
    }
}

struct LampStateSetRequest {
    pub lamp_state: LampState,
}

impl Protocol for LampStateSetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::LampStrikes
    }
}

impl SetRequest for LampStateSetRequest {}

impl From<LampStateSetRequest> for Vec<u8> {
    fn from(lamp_state: LampStateSetRequest) -> Self {
        Vec::from([lamp_state.lamp_state as u8])
    }
}

pub struct LampStateGetResponse {
    pub lamp_state: LampState,
}

impl Protocol for LampStateGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::LampStrikes
    }
}

impl From<Vec<u8>> for LampStateGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        LampStateGetResponse {
            lamp_state: LampState::from(bytes[0]),
        }
    }
}

struct LampStateSetResponse;

impl Protocol for LampStateSetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::LampStrikes
    }
}

impl From<Vec<u8>> for LampStateSetResponse {
    fn from(_: Vec<u8>) -> Self {
        LampStateSetResponse
    }
}

struct LampOnModeGetRequest;

impl Protocol for LampOnModeGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::LampOnMode
    }
}

impl GetRequest for LampOnModeGetRequest {}

impl From<LampOnModeGetRequest> for Vec<u8> {
    fn from(_: LampOnModeGetRequest) -> Self {
        Vec::new()
    }
}

struct LampOnModeSetRequest {
    lamp_on_mode: LampOnMode,
}

impl Protocol for LampOnModeSetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::LampOnMode
    }
}

impl SetRequest for LampOnModeSetRequest {}

impl From<LampOnModeSetRequest> for Vec<u8> {
    fn from(lamp_on_mode: LampOnModeSetRequest) -> Self {
        Vec::from([lamp_on_mode.lamp_on_mode as u8])
    }
}

pub struct LampOnModeGetResponse {
    pub lamp_on_mode: LampOnMode,
}

impl Protocol for LampOnModeGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::LampOnMode
    }
}

impl From<Vec<u8>> for LampOnModeGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        LampOnModeGetResponse {
            lamp_on_mode: LampOnMode::from(bytes[0]),
        }
    }
}

struct LampOnModeSetResponse;

impl Protocol for LampOnModeSetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::LampOnMode
    }
}

impl From<Vec<u8>> for LampOnModeSetResponse {
    fn from(_: Vec<u8>) -> Self {
        LampOnModeSetResponse
    }
}

struct DevicePowerCyclesGetRequest;

impl Protocol for DevicePowerCyclesGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::DevicePowerCycles
    }
}

impl GetRequest for DevicePowerCyclesGetRequest {}

impl From<DevicePowerCyclesGetRequest> for Vec<u8> {
    fn from(_: DevicePowerCyclesGetRequest) -> Self {
        Vec::new()
    }
}

struct DevicePowerCyclesSetRequest {
    power_cycle_count: u32,
}

impl Protocol for DevicePowerCyclesSetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::DevicePowerCycles
    }
}

impl SetRequest for DevicePowerCyclesSetRequest {}

impl From<DevicePowerCyclesSetRequest> for Vec<u8> {
    fn from(device_power_cycles: DevicePowerCyclesSetRequest) -> Self {
        Vec::from(device_power_cycles.power_cycle_count.to_be_bytes())
    }
}

pub struct DevicePowerCyclesGetResponse {
    pub power_cycle_count: u32,
}

impl Protocol for DevicePowerCyclesGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::DevicePowerCycles
    }
}

impl From<Vec<u8>> for DevicePowerCyclesGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        DevicePowerCyclesGetResponse {
            power_cycle_count: u32::from_be_bytes(bytes[0..=3].try_into().unwrap()),
        }
    }
}

struct DevicePowerCyclesSetResponse;

impl Protocol for DevicePowerCyclesSetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::DevicePowerCycles
    }
}

impl From<Vec<u8>> for DevicePowerCyclesSetResponse {
    fn from(_: Vec<u8>) -> Self {
        DevicePowerCyclesSetResponse
    }
}

struct DisplayInvertGetRequest;

impl Protocol for DisplayInvertGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::DisplayInvert
    }
}

impl GetRequest for DisplayInvertGetRequest {}

impl From<DisplayInvertGetRequest> for Vec<u8> {
    fn from(_: DisplayInvertGetRequest) -> Self {
        Vec::new()
    }
}

struct DisplayInvertSetRequest {
    display_invert_mode: DisplayInvertMode,
}

impl Protocol for DisplayInvertSetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::DisplayInvert
    }
}

impl SetRequest for DisplayInvertSetRequest {}

impl From<DisplayInvertSetRequest> for Vec<u8> {
    fn from(display_invert: DisplayInvertSetRequest) -> Self {
        Vec::from([display_invert.display_invert_mode as u8])
    }
}

pub struct DisplayInvertGetResponse {
    pub display_invert_mode: DisplayInvertMode,
}

impl Protocol for DisplayInvertGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::DisplayInvert
    }
}

impl From<Vec<u8>> for DisplayInvertGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        DisplayInvertGetResponse {
            display_invert_mode: DisplayInvertMode::from(bytes[0]),
        }
    }
}

struct DisplayInvertSetResponse;

impl Protocol for DisplayInvertSetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::DisplayInvert
    }
}

impl From<Vec<u8>> for DisplayInvertSetResponse {
    fn from(_: Vec<u8>) -> Self {
        DisplayInvertSetResponse
    }
}

struct CurveGetRequest;

impl Protocol for CurveGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::Curve
    }
}

impl GetRequest for CurveGetRequest {}

impl From<CurveGetRequest> for Vec<u8> {
    fn from(_: CurveGetRequest) -> Self {
        Vec::new()
    }
}

struct CurveSetRequest {
    curve: u8,
}

impl Protocol for CurveSetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::Curve
    }
}

impl SetRequest for CurveSetRequest {}

impl From<CurveSetRequest> for Vec<u8> {
    fn from(curve: CurveSetRequest) -> Self {
        Vec::from(curve.curve.to_be_bytes())
    }
}

pub struct CurveGetResponse {
    pub current_curve: u8,
    pub curve_count: u8,
}

impl Protocol for CurveGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::Curve
    }
}

impl From<Vec<u8>> for CurveGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        CurveGetResponse {
            current_curve: bytes[0],
            curve_count: bytes[1],
        }
    }
}

struct CurveSetResponse;

impl Protocol for CurveSetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::Curve
    }
}

impl From<Vec<u8>> for CurveSetResponse {
    fn from(_: Vec<u8>) -> Self {
        CurveSetResponse
    }
}

pub struct CurveDescriptionGetRequest {
    curve: u8,
}

impl CurveDescriptionGetRequest {
    pub fn new(curve: u8) -> Self {
        CurveDescriptionGetRequest { curve }
    }
}

impl Protocol for CurveDescriptionGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::CurveDescription
    }
}

impl GetRequest for CurveDescriptionGetRequest {}

impl From<CurveDescriptionGetRequest> for Vec<u8> {
    fn from(curve_description: CurveDescriptionGetRequest) -> Self {
        Vec::from(curve_description.curve.to_be_bytes())
    }
}

pub struct CurveDescriptionGetResponse {
    pub curve: u8,
    pub description: String,
}

impl Protocol for CurveDescriptionGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::CurveDescription
    }
}

impl From<Vec<u8>> for CurveDescriptionGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        CurveDescriptionGetResponse {
            curve: bytes[0],
            description: String::from_utf8_lossy(&bytes[1..])
                .trim_end_matches("\0")
                .to_string(),
        }
    }
}

struct ModulationFrequencyGetRequest;

impl Protocol for ModulationFrequencyGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::ModulationFrequency
    }
}

impl GetRequest for ModulationFrequencyGetRequest {}

impl From<ModulationFrequencyGetRequest> for Vec<u8> {
    fn from(_: ModulationFrequencyGetRequest) -> Self {
        Vec::new()
    }
}

struct ModulationFrequencySetRequest {
    modulation_frequency: u8,
}

impl Protocol for ModulationFrequencySetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::ModulationFrequency
    }
}

impl SetRequest for ModulationFrequencySetRequest {}

impl From<ModulationFrequencySetRequest> for Vec<u8> {
    fn from(modulation_frequency: ModulationFrequencySetRequest) -> Self {
        Vec::from(modulation_frequency.modulation_frequency.to_be_bytes())
    }
}

pub struct ModulationFrequencyGetResponse {
    pub current_modulation_frequency: u8,
    pub modulation_frequency_count: u8,
}

impl Protocol for ModulationFrequencyGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::ModulationFrequency
    }
}

impl From<Vec<u8>> for ModulationFrequencyGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        ModulationFrequencyGetResponse {
            current_modulation_frequency: bytes[0],
            modulation_frequency_count: bytes[1],
        }
    }
}

struct ModulationFrequencySetResponse;

impl Protocol for ModulationFrequencySetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::ModulationFrequency
    }
}

impl From<Vec<u8>> for ModulationFrequencySetResponse {
    fn from(_: Vec<u8>) -> Self {
        ModulationFrequencySetResponse
    }
}

pub struct ModulationFrequencyDescriptionGetRequest {
    modulation_frequency: u8,
}

impl ModulationFrequencyDescriptionGetRequest {
    pub fn new(modulation_frequency: u8) -> Self {
        ModulationFrequencyDescriptionGetRequest {
            modulation_frequency,
        }
    }
}

impl Protocol for ModulationFrequencyDescriptionGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::ModulationFrequencyDescription
    }
}

impl GetRequest for ModulationFrequencyDescriptionGetRequest {}

impl From<ModulationFrequencyDescriptionGetRequest> for Vec<u8> {
    fn from(modulation_frequency_description: ModulationFrequencyDescriptionGetRequest) -> Self {
        Vec::from(
            modulation_frequency_description
                .modulation_frequency
                .to_be_bytes(),
        )
    }
}

pub struct ModulationFrequencyDescriptionGetResponse {
    pub modulation_frequency: u8,
    pub frequency: u32,
    pub description: String,
}

impl Protocol for ModulationFrequencyDescriptionGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::ModulationFrequencyDescription
    }
}

impl From<Vec<u8>> for ModulationFrequencyDescriptionGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        ModulationFrequencyDescriptionGetResponse {
            modulation_frequency: bytes[0],
            frequency: u32::from_be_bytes(bytes[1..=4].try_into().unwrap()),
            description: String::from_utf8_lossy(&bytes[5..])
                .trim_end_matches("\0")
                .to_string(),
        }
    }
}

pub struct DimmerInfoRequest;

impl Protocol for DimmerInfoRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::DimmerInfo
    }
}

impl GetRequest for DimmerInfoRequest {}

impl From<DimmerInfoRequest> for Vec<u8> {
    fn from(_: DimmerInfoRequest) -> Self {
        Vec::new()
    }
}

#[derive(Debug)]
pub struct DimmerInfoResponse {
    pub minimum_level_lower_limit: u16,
    pub minimum_level_upper_limit: u16,
    pub maximum_level_lower_limit: u16,
    pub maximum_level_upper_limit: u16,
    pub num_of_supported_curves: u8,
    pub levels_resolution: u8,
    pub minimum_levels_split_levels_supports: u8,
}

impl Protocol for DimmerInfoResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::DimmerInfo
    }
}

impl From<Vec<u8>> for DimmerInfoResponse {
    fn from(bytes: Vec<u8>) -> Self {
        DimmerInfoResponse {
            minimum_level_lower_limit: u16::from_be_bytes(bytes[0..=1].try_into().unwrap()),
            minimum_level_upper_limit: u16::from_be_bytes(bytes[2..=3].try_into().unwrap()),
            maximum_level_lower_limit: u16::from_be_bytes(bytes[4..=5].try_into().unwrap()),
            maximum_level_upper_limit: u16::from_be_bytes(bytes[6..=7].try_into().unwrap()),
            num_of_supported_curves: bytes[8],
            levels_resolution: bytes[9],
            minimum_levels_split_levels_supports: bytes[10], // TODO could be bool
        }
    }
}

pub struct MinimumLevelGetRequest;

impl Protocol for MinimumLevelGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::MinimumLevel
    }
}

impl GetRequest for MinimumLevelGetRequest {}

impl From<MinimumLevelGetRequest> for Vec<u8> {
    fn from(_: MinimumLevelGetRequest) -> Self {
        Vec::new()
    }
}

#[derive(Debug)]
pub struct MinimumLevelGetResponse {
    pub minimum_level_increasing: u16,
    pub minimum_level_decreasing: u16,
    pub on_below_minimum: u8, // TODO could be bool
}

impl Protocol for MinimumLevelGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::MinimumLevel
    }
}

impl From<Vec<u8>> for MinimumLevelGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        MinimumLevelGetResponse {
            minimum_level_increasing: u16::from_be_bytes(bytes[0..=1].try_into().unwrap()),
            minimum_level_decreasing: u16::from_be_bytes(bytes[2..=3].try_into().unwrap()),
            on_below_minimum: bytes[4],
        }
    }
}

pub struct MinimumLevelSetRequest {
    pub minimum_level_increasing: u16,
    pub minimum_level_decreasing: u16,
    pub on_below_minimum: u8, // TODO could be bool
}

impl Protocol for MinimumLevelSetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::MinimumLevel
    }
}

impl GetRequest for MinimumLevelSetRequest {}

impl From<MinimumLevelSetRequest> for Vec<u8> {
    fn from(minimum_level: MinimumLevelSetRequest) -> Self {
        let mut packet = Vec::new();
        packet
            .write_u16::<BigEndian>(minimum_level.minimum_level_increasing)
            .unwrap();
        packet
            .write_u16::<BigEndian>(minimum_level.minimum_level_decreasing)
            .unwrap();
        packet.write_u8(minimum_level.on_below_minimum).unwrap();
        packet
    }
}

#[derive(Debug)]
pub struct MinimumLevelSetResponse;

impl Protocol for MinimumLevelSetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::MinimumLevel
    }
}

impl From<Vec<u8>> for MinimumLevelSetResponse {
    fn from(_: Vec<u8>) -> Self {
        MinimumLevelSetResponse
    }
}

pub struct MaximumLevelGetRequest;

impl Protocol for MaximumLevelGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::MaximumLevel
    }
}

impl GetRequest for MaximumLevelGetRequest {}

impl From<MaximumLevelGetRequest> for Vec<u8> {
    fn from(_: MaximumLevelGetRequest) -> Self {
        Vec::new()
    }
}

#[derive(Debug)]
pub struct MaximumLevelGetResponse {
    pub maximum_level: u16,
}

impl Protocol for MaximumLevelGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::MaximumLevel
    }
}

impl From<Vec<u8>> for MaximumLevelGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        MaximumLevelGetResponse {
            maximum_level: u16::from_be_bytes(bytes[0..=1].try_into().unwrap()),
        }
    }
}

pub struct MaximumLevelSetRequest {
    pub maximum_level: u16,
}

impl Protocol for MaximumLevelSetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::MaximumLevel
    }
}

impl GetRequest for MaximumLevelSetRequest {}

impl From<MaximumLevelSetRequest> for Vec<u8> {
    fn from(maximum_level: MaximumLevelSetRequest) -> Self {
        Vec::from(maximum_level.maximum_level.to_be_bytes())
    }
}

#[derive(Debug)]
pub struct MaximumLevelSetResponse;

impl Protocol for MaximumLevelSetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::MaximumLevel
    }
}

impl From<Vec<u8>> for MaximumLevelSetResponse {
    fn from(_: Vec<u8>) -> Self {
        MaximumLevelSetResponse
    }
}

pub struct OutputResponseTimeGetRequest;

impl Protocol for OutputResponseTimeGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::OutputResponseTime
    }
}

impl GetRequest for OutputResponseTimeGetRequest {}

impl From<OutputResponseTimeGetRequest> for Vec<u8> {
    fn from(_: OutputResponseTimeGetRequest) -> Self {
        Vec::new()
    }
}

pub struct OutputResponseTimeGetResponse {
    pub current_output_response_time: u8,
    pub output_response_time_count: u8,
}

impl Protocol for OutputResponseTimeGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::OutputResponseTime
    }
}

impl From<Vec<u8>> for OutputResponseTimeGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        OutputResponseTimeGetResponse {
            current_output_response_time: bytes[0],
            output_response_time_count: bytes[1],
        }
    }
}

pub struct OutputResponseTimeSetRequest {
    output_response_time: u8,
}

impl Protocol for OutputResponseTimeSetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::OutputResponseTime
    }
}

impl SetRequest for OutputResponseTimeSetRequest {}

impl From<OutputResponseTimeSetRequest> for Vec<u8> {
    fn from(output_response_time: OutputResponseTimeSetRequest) -> Self {
        Vec::from(output_response_time.output_response_time.to_be_bytes())
    }
}

pub struct OutputResponseTimeSetResponse;

impl Protocol for OutputResponseTimeSetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::OutputResponseTime
    }
}

impl From<Vec<u8>> for OutputResponseTimeSetResponse {
    fn from(_: Vec<u8>) -> Self {
        OutputResponseTimeSetResponse
    }
}

pub struct OutputResponseTimeDescriptionGetRequest {
    output_response_time: u8,
}

impl OutputResponseTimeDescriptionGetRequest {
    pub fn new(output_response_time: u8) -> Self {
        OutputResponseTimeDescriptionGetRequest {
            output_response_time,
        }
    }
}

impl Protocol for OutputResponseTimeDescriptionGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::OutputResponseTimeDescription
    }
}

impl GetRequest for OutputResponseTimeDescriptionGetRequest {}

impl From<OutputResponseTimeDescriptionGetRequest> for Vec<u8> {
    fn from(output_response_time_description: OutputResponseTimeDescriptionGetRequest) -> Self {
        Vec::from(
            output_response_time_description
                .output_response_time
                .to_be_bytes(),
        )
    }
}

pub struct OutputResponseTimeDescriptionGetResponse {
    pub output_response_time: u8,
    pub description: String,
}

impl Protocol for OutputResponseTimeDescriptionGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::OutputResponseTimeDescription
    }
}

impl From<Vec<u8>> for OutputResponseTimeDescriptionGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        OutputResponseTimeDescriptionGetResponse {
            output_response_time: bytes[0],
            description: String::from_utf8_lossy(&bytes[1..])
                .trim_end_matches("\0")
                .to_string(),
        }
    }
}

pub fn create_standard_parameter_get_request_packet(
    parameter_id: ParameterId,
    destination_uid: DeviceUID,
    source_uid: DeviceUID,
    transaction_number: u8,
    port_id: u8,
    sub_device: u16,
) -> Result<Vec<u8>, ParameterError> {
    match parameter_id {
        // DiscUniqueBranch => ,
        // DiscMute => ,
        // DiscUnMute => ,
        ParameterId::ProxiedDevices => Ok(ProxiedDevicesGetRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        ParameterId::ProxiedDeviceCount => Ok(ProxiedDevicesGetRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        // CommsStatus => ,
        // QueuedMessage => ,
        // StatusMessages => ,
        // StatusIdDescription => ,
        // ClearStatusId => ,
        // SubDeviceStatusReportThreshold => ,
        ParameterId::SupportedParameters => Ok(SupportedParametersGetRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        // ParameterDescription => ,
        ParameterId::DeviceInfo => Ok(DeviceInfoRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        ParameterId::ProductDetailIdList => Ok(ProductDetailIdListGetRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        ParameterId::DeviceModelDescription => Ok(DeviceModelDescriptionGetRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        ParameterId::ManufacturerLabel => Ok(ManufacturerLabelGetRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        ParameterId::DeviceLabel => Ok(ManufacturerLabelGetRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        ParameterId::FactoryDefaults => Ok(FactoryDefaultsGetRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        // LanguageCapabilities => ,
        // Language => ,
        ParameterId::SoftwareVersionLabel => Ok(SoftwareVersionLabelGetRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        // BootSoftwareVersionId => ,
        // BootSoftwareVersionLabel => ,
        ParameterId::DmxPersonality => Ok(DmxPersonalityGetRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        // DmxPersonalityDescription => ,
        // DmxStartAddress => ,
        ParameterId::SlotInfo => Ok(SlotInfoRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        // SlotDescription => ,
        // DefaultSlotValue => ,
        // SensorDefinition => ,
        // SensorValue => ,
        // RecordSensors => ,
        ParameterId::DimmerInfo => Ok(DimmerInfoRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        ParameterId::MinimumLevel => Ok(MinimumLevelGetRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        ParameterId::MaximumLevel => Ok(MaximumLevelGetRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        ParameterId::Curve => Ok(CurveGetRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        // CurveDescription => ,
        ParameterId::ModulationFrequency => Ok(ModulationFrequencyGetRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        // ModulationFrequencyDescription => ,
        ParameterId::OutputResponseTime => Ok(OutputResponseTimeGetRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        // OutputResponseTimeDescription => ,
        ParameterId::DeviceHours => Ok(DeviceHoursGetRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        ParameterId::LampHours => Ok(LampHoursGetRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        ParameterId::LampStrikes => Ok(LampStrikesGetRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        ParameterId::LampState => Ok(LampStateGetRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        ParameterId::LampOnMode => Ok(LampOnModeGetRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        ParameterId::DevicePowerCycles => Ok(DevicePowerCyclesGetRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        ParameterId::DisplayInvert => Ok(DisplayInvertGetRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        // DisplayLevel => ,
        // PanInvert => ,
        // TiltInvert => ,
        // PanTiltSwap => ,
        // RealTimeClock => ,
        ParameterId::IdentifyDevice => Ok(IdentifyDeviceGetRequest
            .get_request(
                destination_uid,
                source_uid,
                transaction_number,
                port_id,
                sub_device,
            )
            .into()),
        // ResetDevice => ,
        // PowerState => ,
        // PerformSelfTest => ,
        // SelfTestDescription => ,
        // CapturePreset => ,
        // PresetPlayback => ,
        _ => Err(ParameterError::UnsupportedParameter),
    }
}
