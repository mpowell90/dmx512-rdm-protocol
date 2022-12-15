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

pub struct FactoryDefaultsRequest;

impl Protocol for FactoryDefaultsRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::FactoryDefaults
    }
}

impl GetRequest for FactoryDefaultsRequest {}

impl From<FactoryDefaultsRequest> for Vec<u8> {
    fn from(_: FactoryDefaultsRequest) -> Self {
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
    pub manufacturer_label: String,
}

impl Protocol for DeviceModelDescriptionGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::ManufacturerLabel
    }
}

impl From<Vec<u8>> for DeviceModelDescriptionGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        DeviceModelDescriptionGetResponse {
            manufacturer_label: String::from_utf8_lossy(&bytes)
                .trim_end_matches("\0")
                .to_string(),
        }
    }
}

pub struct ProductDetailIdListRequest;

impl Protocol for ProductDetailIdListRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::ProductDetailIdList
    }
}

impl GetRequest for ProductDetailIdListRequest {}

impl From<ProductDetailIdListRequest> for Vec<u8> {
    fn from(_: ProductDetailIdListRequest) -> Self {
        Vec::new()
    }
}

#[derive(Clone, Debug)]
pub struct ProductDetailIdListResponse {
    pub product_detail_id_list: Vec<u16>,
}

impl Protocol for ProductDetailIdListResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::ProductDetailIdList
    }
}

impl From<Vec<u8>> for ProductDetailIdListResponse {
    fn from(bytes: Vec<u8>) -> Self {
        ProductDetailIdListResponse {
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

struct DmxPersonalityGetResponse {
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

struct DmxPersonalityDescriptionGetRequest {
    personality: u8,
}

impl Protocol for DmxPersonalityDescriptionGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::DmxPersonality
    }
}

impl GetRequest for DmxPersonalityDescriptionGetRequest {}

impl From<DmxPersonalityDescriptionGetRequest> for Vec<u8> {
    fn from(dmx_personality_description: DmxPersonalityDescriptionGetRequest) -> Self {
        Vec::from(dmx_personality_description.personality.to_be_bytes())
    }
}

struct DmxPersonalityDescriptionGetResponse {
    pub personality: u8,
    pub dmx_slots_required: u16,
    pub description: String,
}

impl Protocol for DmxPersonalityDescriptionGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::DmxPersonality
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

struct DeviceHoursGetResponse {
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
    personality: u8,
}

impl Protocol for CurveSetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::Curve
    }
}

impl SetRequest for CurveSetRequest {}

impl From<CurveSetRequest> for Vec<u8> {
    fn from(dmx_personality: CurveSetRequest) -> Self {
        Vec::from(dmx_personality.personality.to_be_bytes())
    }
}

struct CurveGetResponse {
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

struct CurveDescriptionGetRequest {
    curve: u8,
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

struct CurveDescriptionGetResponse {
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

struct ModulationFrequencyGetResponse {
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

struct ModulationFrequencyDescriptionGetRequest {
    modulation_frequency: u8,
}

impl Protocol for ModulationFrequencyDescriptionGetRequest {
    fn parameter_id() -> ParameterId {
        ParameterId::ModulationFrequency
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

struct ModulationFrequencyDescriptionGetResponse {
    pub modulation_frequency: u8,
    pub description: String,
}

impl Protocol for ModulationFrequencyDescriptionGetResponse {
    fn parameter_id() -> ParameterId {
        ParameterId::ModulationFrequency
    }
}

impl From<Vec<u8>> for ModulationFrequencyDescriptionGetResponse {
    fn from(bytes: Vec<u8>) -> Self {
        ModulationFrequencyDescriptionGetResponse {
            modulation_frequency: bytes[0],
            description: String::from_utf8_lossy(&bytes[1..])
                .trim_end_matches("\0")
                .to_string(),
        }
    }
}