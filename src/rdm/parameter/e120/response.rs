use crate::rdm::{
    DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE, DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE,
    parameter::e120::{
        BootSoftwareVersionLabel, ControlField, DefaultSlotValue, DeviceLabel,
        DeviceModelDescription, DisplayInvertMode, DmxPersonalityDescription,
        ImplementedCommandClass, Iso639_1, LampOnMode, LampState, ManufacturerLabel,
        ParameterDataType, ParameterDescriptionLabel, PowerState, PresetPlaybackMode,
        ProductCategory, ProductDetail, ProtocolVersion, SelfTest, SelfTestDescription,
        SensorDefinitionDescription, SensorType, SensorUnit, SensorUnitPrefix, SlotDescription,
        SlotInfo, SoftwareVersionLabel, StatusIdDescription, StatusMessage, StatusType,
    },
    utils::bsd_16_crc,
};
use heapless::Vec;
use rdm_core::{CommandClass, DeviceUID, ParameterId, error::RdmError};
use rdm_derive::rdm_response_parameter;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DiscoveryUniqueBranchFrameResponse {
    pub device_uid: DeviceUID,
}

impl DiscoveryUniqueBranchFrameResponse {
    pub fn size(&self) -> usize {
        24
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, RdmError> {
        buf[0..7].copy_from_slice(&[
            DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
            DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
            DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
            DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
            DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
            DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
            DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
        ]);

        buf[7] = DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE;

        let [manufacturer_id1, manufacturer_id0] = self.device_uid.manufacturer_id.to_be_bytes();

        buf[8..12].copy_from_slice(&[
            manufacturer_id1 | 0xaa,
            manufacturer_id1 | 0x55,
            manufacturer_id0 | 0xaa,
            manufacturer_id0 | 0x55,
        ]);

        let [device_id3, device_id2, device_id1, device_id0] =
            self.device_uid.device_id.to_be_bytes();

        buf[12..20].copy_from_slice(&[
            device_id3 | 0xaa,
            device_id3 | 0x55,
            device_id2 | 0xaa,
            device_id2 | 0x55,
            device_id1 | 0xaa,
            device_id1 | 0x55,
            device_id0 | 0xaa,
            device_id0 | 0x55,
        ]);

        let [checksum1, checksum0] = bsd_16_crc(&buf[8..]).to_be_bytes();

        buf[20..24].copy_from_slice(&[
            checksum1 | 0xaa,
            checksum1 | 0x55,
            checksum0 | 0xaa,
            checksum0 | 0x55,
        ]);

        Ok(24)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, RdmError> {
        let Some(frame_start_index) = bytes.iter().position(|&x| x == 0xaa) else {
            return Err(RdmError::InvalidDiscoveryUniqueBranchPreamble);
        };

        let euid = &bytes[(frame_start_index + 1)..=(frame_start_index + 12)];

        let ecs = &bytes[(frame_start_index + 13)..=(frame_start_index + 16)];

        let decoded_checksum = bsd_16_crc(euid);

        let checksum = u16::from_be_bytes([ecs[0] & ecs[1], ecs[2] & ecs[3]]);

        if checksum != decoded_checksum {
            return Err(RdmError::InvalidChecksum(decoded_checksum, checksum));
        }

        let manufacturer_id = u16::from_be_bytes([euid[0] & euid[1], euid[2] & euid[3]]);

        let device_id = u32::from_be_bytes([
            euid[4] & euid[5],
            euid[6] & euid[7],
            euid[8] & euid[9],
            euid[10] & euid[11],
        ]);

        Ok(Self {
            device_uid: DeviceUID::new(manufacturer_id, device_id),
        })
    }
}

impl TryFrom<&[u8]> for DiscoveryUniqueBranchFrameResponse {
    type Error = RdmError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        DiscoveryUniqueBranchFrameResponse::decode(bytes)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::DiscMute, command_class = CommandClass::DiscoveryResponse)]
#[repr(C)]
pub struct DiscMuteResponse {
    pub control_field: ControlField,
    pub binding_uid: Option<DeviceUID>,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::DiscUnMute, command_class = CommandClass::DiscoveryResponse)]
#[repr(C)]
pub struct DiscUnMuteResponse {
    pub control_field: ControlField,
    pub binding_uid: Option<DeviceUID>,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::ProxiedDevices, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetProxiedDevicesResponse {
    pub device_uids: Vec<DeviceUID, 38>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::ProxiedDeviceCount, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetProxiedDeviceCountResponse {
    pub device_count: u16,
    pub list_change: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::CommsStatus, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetCommsStatusResponse {
    pub short_message: u16,
    pub length_mismatch: u16,
    pub checksum_fail: u16,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::StatusMessages, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetStatusMessagesResponse {
    pub messages: Vec<StatusMessage, 25>,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::StatusIdDescription, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetStatusIdDescriptionResponse {
    pub status_id_description: StatusIdDescription,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::SubDeviceIdStatusReportThreshold, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetSubDeviceIdStatusReportThresholdResponse {
    pub status_type: StatusType,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::SupportedParameters, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetSupportedParametersResponse {
    pub supported_parameters: Vec<ParameterId, 115>,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::ParameterDescription, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetParameterDescriptionResponse {
    pub parameter_id: ParameterId,
    pub parameter_data_length: u8,
    pub data_type: ParameterDataType,
    pub command_class: ImplementedCommandClass,
    pub unit_type: SensorUnit,
    pub prefix: SensorUnitPrefix,
    pub raw_minimum_valid_value: [u8; 4],
    pub raw_maximum_valid_value: [u8; 4],
    pub raw_default_value: [u8; 4],
    pub description: ParameterDescriptionLabel,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::DeviceInfo, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetDeviceInfoResponse {
    pub protocol_version: ProtocolVersion,
    pub device_model_id: u16,
    pub product_category: ProductCategory,
    pub software_version_id: u32,
    pub dmx512_footprint: u16,
    pub current_personality: u8,
    pub personality_count: u8,
    pub dmx512_start_address: u16,
    pub sub_device_count: u16,
    pub sensor_count: u8,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::ProductDetailIdList, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetProductDetailIdListResponse {
    pub product_detail_ids: Vec<ProductDetail, 6>,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::DeviceModelDescription, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetDeviceModelDescriptionResponse {
    pub device_model_description: DeviceModelDescription,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::ManufacturerLabel, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetManufacturerLabelResponse {
    pub manufacturer_label: ManufacturerLabel,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::DeviceLabel, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetDeviceLabelResponse {
    pub device_label: DeviceLabel,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::FactoryDefaults, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetFactoryDefaultsResponse {
    pub factory_reset: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::Language, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetLanguageResponse {
    pub language: Iso639_1,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::LanguageCapabilities, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetLanguageCapabilitiesResponse {
    pub language_capabilities: Vec<Iso639_1, 115>,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::SoftwareVersionLabel, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetSoftwareVersionLabelResponse {
    pub software_version_label: SoftwareVersionLabel,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::BootSoftwareVersionId, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetBootSoftwareVersionIdResponse {
    pub boot_software_version_id: u32,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::BootSoftwareVersionLabel, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetBootSoftwareVersionLabelResponse {
    pub boot_software_version_label: BootSoftwareVersionLabel,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::DmxPersonality, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetDmxPersonalityResponse {
    pub current_personality: u8,
    pub personality_count: u8,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::DmxPersonalityDescription, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetDmxPersonalityDescriptionResponse {
    pub id: u8,
    pub dmx_slots_required: u16,
    pub description: DmxPersonalityDescription,
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::DmxStartAddress, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetDmxStartAddressResponse {
    pub dmx_start_address: u16,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::SlotInfo, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetSlotInfoResponse {
    pub slot_info: Vec<SlotInfo, 46>,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::SlotDescription, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetSlotDescriptionResponse {
    pub slot_id: u16,
    pub description: SlotDescription,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::DefaultSlotValue, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetDefaultSlotValueResponse {
    pub default_slot_values: Vec<DefaultSlotValue, 77>,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::SensorDefinition, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetSensorDefinitionResponse {
    pub id: u8,
    pub kind: SensorType,
    pub unit: SensorUnit,
    pub prefix: SensorUnitPrefix,
    pub range_minimum_value: i16,
    pub range_maximum_value: i16,
    pub normal_minimum_value: i16,
    pub normal_maximum_value: i16,
    pub is_lowest_highest_detected_value_supported: bool,
    pub is_recorded_value_supported: bool,
    pub description: SensorDefinitionDescription,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::SensorValue, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetSensorValueResponse {
    pub sensor_id: u8,
    pub current_value: i16,
    pub lowest_detected_value: i16,
    pub highest_detected_value: i16,
    pub recorded_value: i16,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::SensorValue, command_class = CommandClass::SetResponse)]
#[repr(C)]
pub struct SetSensorValueResponse {
    pub sensor_id: u8,
    pub current_value: i16,
    pub lowest_detected_value: i16,
    pub highest_detected_value: i16,
    pub recorded_value: i16,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::DeviceHours, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetDeviceHoursResponse {
    pub device_hours: u32,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::LampHours, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetLampHoursResponse {
    pub lamp_hours: u32,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::LampStrikes, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetLampStrikesResponse {
    pub lamp_strikes: u32,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::LampState, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetLampStateResponse {
    pub lamp_state: LampState,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::LampOnMode, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetLampOnModeResponse {
    pub lamp_on_mode: LampOnMode,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::DevicePowerCycles, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetDevicePowerCyclesResponse {
    pub device_power_cycles: u32,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::DisplayInvert, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetDisplayInvertResponse {
    pub display_invert_mode: DisplayInvertMode,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::DisplayLevel, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetDisplayLevelResponse {
    pub display_level: u8,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::PanInvert, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetPanInvertResponse {
    pub pan_invert: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::TiltInvert, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetTiltInvertResponse {
    pub tilt_invert: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::PanTiltSwap, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetPanTiltSwapResponse {
    pub pan_tilt_swap: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::RealTimeClock, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetRealTimeClockResponse {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::IdentifyDevice, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetIdentifyDeviceResponse {
    pub identify: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::PowerState, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetPowerStateResponse {
    pub power_state: PowerState,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::PerformSelfTest, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetPerformSelfTestResponse {
    pub perform_self_test: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::SelfTestDescription, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetSelfTestDescriptionResponse {
    pub self_test_id: SelfTest,
    pub description: SelfTestDescription,
}

#[derive(Clone, Debug, PartialEq)]
#[rdm_response_parameter(pid = ParameterId::PresetPlayback, command_class = CommandClass::GetResponse)]
#[repr(C)]
pub struct GetPresetPlaybackResponse {
    pub mode: PresetPlaybackMode,
    pub level: u8,
}
