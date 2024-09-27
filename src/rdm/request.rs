//! Data types and functionality for encoding RDM requests
//!
//! # RdmRequest
//!
//! ```rust
//! use dmx512_rdm_protocol::rdm::{
//!     request::{RdmRequest, RequestParameter},
//!     DeviceUID, SubDeviceId,
//! };
//!
//! let encoded = RdmRequest::new(
//!     DeviceUID::new(0x0102, 0x03040506),
//!     DeviceUID::new(0x0605, 0x04030201),
//!     0x00,
//!     0x01,
//!     SubDeviceId::RootDevice,
//!     RequestParameter::GetIdentifyDevice,
//! )
//! .encode();
//!
//! let expected = &[
//!     0xcc, // Start Code
//!     0x01, // Sub Start Code
//!     0x18, // Message Length
//!     0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // Destination UID
//!     0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // Source UID
//!     0x00, // Transaction Number
//!     0x01, // Port ID
//!     0x00, // Message Count
//!     0x00, 0x00, // Sub-Device ID = Root Device
//!     0x20, // Command Class = GetCommand
//!     0x10, 0x00, // Parameter ID = Identify Device
//!     0x00, // PDL
//!     0x01, 0x40, // Checksum
//! ];
//!
//! assert_eq!(encoded, expected);
//! ```
//!
//! See tests for more examples.

use super::{
    bsd_16_crc,
    parameter::{
        DisplayInvertMode, FadeTimes, LampOnMode, LampState, ParameterId, PowerState,
        PresetPlaybackMode, ResetDeviceMode, SelfTest, StatusType,
    },
    CommandClass, DeviceUID, SubDeviceId, RDM_START_CODE_BYTE, RDM_SUB_START_CODE_BYTE,
};

#[cfg(not(feature = "alloc"))]
use heapless::{String, Vec};

#[derive(Clone, Debug, PartialEq)]
pub enum RequestParameter {
    DiscMute,
    DiscUnMute,
    DiscUniqueBranch {
        lower_bound_uid: DeviceUID,
        upper_bound_uid: DeviceUID,
    },
    GetCommsStatus,
    SetCommsStatus,
    GetQueuedMessage {
        status_type: StatusType,
    },
    GetStatusMessages {
        status_type: StatusType,
    },
    GetStatusIdDescription {
        status_id: u16,
    },
    SetClearStatusId,
    GetSubDeviceIdStatusReportThreshold,
    SetSubDeviceIdStatusReportThreshold {
        status_type: StatusType,
    },
    GetSupportedParameters,
    GetParameterDescription {
        parameter_id: u16,
    },
    GetDeviceInfo,
    GetProductDetailIdList,
    GetDeviceModelDescription,
    GetManufacturerLabel,
    GetDeviceLabel,
    SetDeviceLabel {
        #[cfg(feature = "alloc")]
        device_label: String,
        #[cfg(not(feature = "alloc"))]
        device_label: String<32>,
    },
    GetFactoryDefaults,
    SetFactoryDefaults,
    GetLanguageCapabilities,
    GetLanguage,
    SetLanguage {
        #[cfg(feature = "alloc")]
        language: String,
        #[cfg(not(feature = "alloc"))]
        language: String<32>,
    },
    GetSoftwareVersionLabel,
    GetBootSoftwareVersionId,
    GetBootSoftwareVersionLabel,
    GetDmxPersonality,
    SetDmxPersonality {
        personality_id: u8,
    },
    GetDmxPersonalityDescription {
        personality: u8,
    },
    GetDmxStartAddress,
    SetDmxStartAddress {
        dmx_start_address: u16,
    },
    GetSlotInfo,
    GetSlotDescription {
        slot_id: u16,
    },
    GetDefaultSlotValue,
    GetSensorDefinition {
        sensor_id: u8,
    },
    GetSensorValue {
        sensor_id: u8,
    },
    SetSensorValue {
        sensor_id: u8,
    },
    SetRecordSensors {
        sensor_id: u8,
    },
    GetDeviceHours,
    SetDeviceHours {
        device_hours: u32,
    },
    GetLampHours,
    SetLampHours {
        lamp_hours: u32,
    },
    GetLampStrikes,
    SetLampStrikes {
        lamp_strikes: u32,
    },
    GetLampState,
    SetLampState {
        lamp_state: LampState,
    },
    GetLampOnMode,
    SetLampOnMode {
        lamp_on_mode: LampOnMode,
    },
    GetDevicePowerCycles,
    SetDevicePowerCycles {
        device_power_cycles: u32,
    },
    GetDisplayInvert,
    SetDisplayInvert {
        display_invert: DisplayInvertMode,
    },
    GetDisplayLevel,
    SetDisplayLevel {
        display_level: u8,
    },
    GetPanInvert,
    SetPanInvert {
        pan_invert: bool,
    },
    GetTiltInvert,
    SetTiltInvert {
        tilt_invert: bool,
    },
    GetPanTiltSwap,
    SetPanTiltSwap {
        pan_tilt_swap: bool,
    },
    GetRealTimeClock,
    SetRealTimeClock {
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    },
    GetIdentifyDevice,
    SetIdentifyDevice {
        identify: bool,
    },
    SetResetDevice {
        reset_device: ResetDeviceMode,
    },
    GetPowerState,
    SetPowerState {
        power_state: PowerState,
    },
    GetPerformSelfTest,
    SetPerformSelfTest {
        self_test_id: SelfTest,
    },
    SetCapturePreset {
        scene_id: u16,
        fade_times: Option<FadeTimes>,
    },
    GetSelfTestDescription {
        self_test_id: SelfTest,
    },
    GetPresetPlayback,
    SetPresetPlayback {
        mode: PresetPlaybackMode,
        level: u8,
    },
    ManufacturerSpecific {
        command_class: CommandClass,
        parameter_id: u16,
        #[cfg(feature = "alloc")]
        parameter_data: Vec<u8>,
        #[cfg(not(feature = "alloc"))]
        parameter_data: Vec<u8, 231>,
    },
    Unsupported {
        command_class: CommandClass,
        parameter_id: u16,
        #[cfg(feature = "alloc")]
        parameter_data: Vec<u8>,
        #[cfg(not(feature = "alloc"))]
        parameter_data: Vec<u8, 231>,
    },
}

impl RequestParameter {
    pub fn command_class(&self) -> CommandClass {
        match self {
            Self::DiscMute | Self::DiscUnMute | Self::DiscUniqueBranch { .. } => {
                CommandClass::DiscoveryCommand
            }
            Self::GetCommsStatus
            | Self::GetQueuedMessage { .. }
            | Self::GetStatusMessages { .. }
            | Self::GetStatusIdDescription { .. }
            | Self::GetSubDeviceIdStatusReportThreshold
            | Self::GetSupportedParameters
            | Self::GetParameterDescription { .. }
            | Self::GetDeviceInfo
            | Self::GetProductDetailIdList
            | Self::GetDeviceModelDescription
            | Self::GetManufacturerLabel
            | Self::GetDeviceLabel
            | Self::GetFactoryDefaults
            | Self::GetLanguageCapabilities
            | Self::GetLanguage
            | Self::GetSoftwareVersionLabel
            | Self::GetBootSoftwareVersionId
            | Self::GetBootSoftwareVersionLabel
            | Self::GetDmxPersonality
            | Self::GetDmxPersonalityDescription { .. }
            | Self::GetDmxStartAddress
            | Self::GetSlotInfo
            | Self::GetSlotDescription { .. }
            | Self::GetDefaultSlotValue
            | Self::GetSensorDefinition { .. }
            | Self::GetSensorValue { .. }
            | Self::GetDeviceHours
            | Self::GetLampHours
            | Self::GetLampStrikes
            | Self::GetLampState
            | Self::GetLampOnMode
            | Self::GetDevicePowerCycles
            | Self::GetDisplayInvert
            | Self::GetDisplayLevel
            | Self::GetPanInvert
            | Self::GetTiltInvert
            | Self::GetPanTiltSwap
            | Self::GetRealTimeClock
            | Self::GetIdentifyDevice
            | Self::GetPowerState
            | Self::GetPerformSelfTest
            | Self::GetSelfTestDescription { .. }
            | Self::GetPresetPlayback => CommandClass::GetCommand,
            Self::SetCommsStatus
            | Self::SetClearStatusId
            | Self::SetSubDeviceIdStatusReportThreshold { .. }
            | Self::SetDeviceLabel { .. }
            | Self::SetFactoryDefaults
            | Self::SetLanguage { .. }
            | Self::SetDmxPersonality { .. }
            | Self::SetDmxStartAddress { .. }
            | Self::SetSensorValue { .. }
            | Self::SetRecordSensors { .. }
            | Self::SetDeviceHours { .. }
            | Self::SetLampHours { .. }
            | Self::SetLampStrikes { .. }
            | Self::SetLampState { .. }
            | Self::SetLampOnMode { .. }
            | Self::SetDevicePowerCycles { .. }
            | Self::SetDisplayInvert { .. }
            | Self::SetDisplayLevel { .. }
            | Self::SetPanInvert { .. }
            | Self::SetTiltInvert { .. }
            | Self::SetPanTiltSwap { .. }
            | Self::SetRealTimeClock { .. }
            | Self::SetIdentifyDevice { .. }
            | Self::SetResetDevice { .. }
            | Self::SetPowerState { .. }
            | Self::SetPerformSelfTest { .. }
            | Self::SetCapturePreset { .. }
            | Self::SetPresetPlayback { .. } => CommandClass::SetCommand,
            Self::ManufacturerSpecific { command_class, .. } => *command_class,
            Self::Unsupported { command_class, .. } => *command_class,
        }
    }

    pub fn parameter_id(&self) -> ParameterId {
        match self {
            Self::DiscMute => ParameterId::DiscMute,
            Self::DiscUnMute => ParameterId::DiscUnMute,
            Self::DiscUniqueBranch { .. } => ParameterId::DiscUniqueBranch,
            Self::GetCommsStatus | Self::SetCommsStatus => ParameterId::CommsStatus,
            Self::GetQueuedMessage { .. } => ParameterId::QueuedMessage,
            Self::GetStatusMessages { .. } => ParameterId::StatusMessages,
            Self::GetStatusIdDescription { .. } => ParameterId::StatusIdDescription,
            Self::SetClearStatusId => ParameterId::ClearStatusId,
            Self::GetSubDeviceIdStatusReportThreshold
            | Self::SetSubDeviceIdStatusReportThreshold { .. } => {
                ParameterId::SubDeviceIdStatusReportThreshold
            }
            Self::GetSupportedParameters => ParameterId::SupportedParameters,
            Self::GetParameterDescription { .. } => ParameterId::ParameterDescription,
            Self::GetDeviceInfo => ParameterId::DeviceInfo,
            Self::GetProductDetailIdList => ParameterId::ProductDetailIdList,
            Self::GetDeviceModelDescription => ParameterId::DeviceModelDescription,
            Self::GetManufacturerLabel => ParameterId::ManufacturerLabel,
            Self::GetDeviceLabel | Self::SetDeviceLabel { .. } => ParameterId::DeviceLabel,
            Self::GetFactoryDefaults | Self::SetFactoryDefaults => ParameterId::FactoryDefaults,
            Self::GetLanguageCapabilities => ParameterId::LanguageCapabilities,
            Self::GetLanguage | Self::SetLanguage { .. } => ParameterId::Language,
            Self::GetSoftwareVersionLabel => ParameterId::SoftwareVersionLabel,
            Self::GetBootSoftwareVersionId => ParameterId::BootSoftwareVersionId,
            Self::GetBootSoftwareVersionLabel => ParameterId::BootSoftwareVersionLabel,
            Self::GetDmxPersonality | Self::SetDmxPersonality { .. } => ParameterId::DmxPersonality,
            Self::GetDmxPersonalityDescription { .. } => ParameterId::DmxPersonalityDescription,
            Self::GetDmxStartAddress | Self::SetDmxStartAddress { .. } => {
                ParameterId::DmxStartAddress
            }
            Self::GetSlotInfo => ParameterId::SlotInfo,
            Self::GetSlotDescription { .. } => ParameterId::SlotDescription,
            Self::GetDefaultSlotValue => ParameterId::DefaultSlotValue,
            Self::GetSensorDefinition { .. } => ParameterId::SensorDefinition,
            Self::GetSensorValue { .. } | Self::SetSensorValue { .. } => ParameterId::SensorValue,
            Self::SetRecordSensors { .. } => ParameterId::RecordSensors,
            Self::GetDeviceHours | Self::SetDeviceHours { .. } => ParameterId::DeviceHours,
            Self::GetLampHours | Self::SetLampHours { .. } => ParameterId::LampHours,
            Self::GetLampStrikes | Self::SetLampStrikes { .. } => ParameterId::LampStrikes,
            Self::GetLampState | Self::SetLampState { .. } => ParameterId::LampState,
            Self::GetLampOnMode | Self::SetLampOnMode { .. } => ParameterId::LampOnMode,
            Self::GetDevicePowerCycles | Self::SetDevicePowerCycles { .. } => {
                ParameterId::DevicePowerCycles
            }
            Self::GetDisplayInvert | Self::SetDisplayInvert { .. } => ParameterId::DisplayInvert,
            Self::GetDisplayLevel | Self::SetDisplayLevel { .. } => ParameterId::DisplayLevel,
            Self::GetPanInvert | Self::SetPanInvert { .. } => ParameterId::PanInvert,
            Self::GetTiltInvert | Self::SetTiltInvert { .. } => ParameterId::TiltInvert,
            Self::GetPanTiltSwap | Self::SetPanTiltSwap { .. } => ParameterId::PanTiltSwap,
            Self::GetRealTimeClock | Self::SetRealTimeClock { .. } => ParameterId::RealTimeClock,
            Self::GetIdentifyDevice | Self::SetIdentifyDevice { .. } => ParameterId::IdentifyDevice,
            Self::SetResetDevice { .. } => ParameterId::ResetDevice,
            Self::GetPowerState | Self::SetPowerState { .. } => ParameterId::PowerState,
            Self::GetPerformSelfTest | Self::SetPerformSelfTest { .. } => {
                ParameterId::PerformSelfTest
            }
            Self::SetCapturePreset { .. } => ParameterId::CapturePreset,
            Self::GetSelfTestDescription { .. } => ParameterId::SelfTestDescription,
            Self::GetPresetPlayback | Self::SetPresetPlayback { .. } => ParameterId::PresetPlayback,
            Self::ManufacturerSpecific { parameter_id, .. } => {
                ParameterId::ManufacturerSpecific(*parameter_id)
            }
            Self::Unsupported { parameter_id, .. } => ParameterId::Unsupported(*parameter_id),
        }
    }

    #[cfg(feature = "alloc")]
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        match self {
            Self::DiscMute => {}
            Self::DiscUnMute => {}
            Self::DiscUniqueBranch {
                lower_bound_uid,
                upper_bound_uid,
            } => {
                buf.reserve(0x0c);
                buf.extend(lower_bound_uid.manufacturer_id.to_be_bytes().iter());
                buf.extend(lower_bound_uid.device_id.to_be_bytes().iter());
                buf.extend(upper_bound_uid.manufacturer_id.to_be_bytes().iter());
                buf.extend(upper_bound_uid.device_id.to_be_bytes().iter());
            }
            Self::GetCommsStatus => {}
            Self::SetCommsStatus => {}
            Self::GetQueuedMessage { status_type } => {
                buf.reserve(0x01);
                buf.push(*status_type as u8)
            }
            Self::GetStatusMessages { status_type } => {
                buf.reserve(0x01);
                buf.push(*status_type as u8)
            }
            Self::GetStatusIdDescription { status_id } => {
                buf.reserve(0x02);
                buf.extend((*status_id).to_be_bytes().iter());
            }
            Self::SetClearStatusId => {}
            Self::GetSubDeviceIdStatusReportThreshold => {}
            Self::SetSubDeviceIdStatusReportThreshold { status_type } => {
                buf.reserve(0x01);
                buf.push(*status_type as u8)
            }
            Self::GetSupportedParameters => {}
            Self::GetParameterDescription { parameter_id } => {
                buf.reserve(0x02);
                buf.extend((*parameter_id).to_be_bytes().iter());
            }
            Self::GetDeviceInfo => {}
            Self::GetProductDetailIdList => {}
            Self::GetDeviceModelDescription => {}
            Self::GetManufacturerLabel => {}
            Self::GetDeviceLabel => {}
            Self::SetDeviceLabel { device_label } => {
                buf.reserve(device_label.len());
                buf.extend(device_label.as_bytes())
            }
            Self::GetFactoryDefaults => {}
            Self::SetFactoryDefaults => {}
            Self::GetLanguageCapabilities => {}
            Self::GetLanguage => {}
            Self::SetLanguage { language } => {
                buf.reserve(language.len());
                buf.extend(language.as_bytes())
            }
            Self::GetSoftwareVersionLabel => {}
            Self::GetBootSoftwareVersionId => {}
            Self::GetBootSoftwareVersionLabel => {}
            Self::GetDmxPersonality => {}
            Self::SetDmxPersonality { personality_id } => {
                buf.reserve(0x01);
                buf.push(*personality_id)
            }
            Self::GetDmxPersonalityDescription { personality } => {
                buf.reserve(0x01);
                buf.push(*personality)
            }
            Self::GetDmxStartAddress => {}
            Self::SetDmxStartAddress { dmx_start_address } => {
                buf.reserve(0x02);
                buf.extend((*dmx_start_address).to_be_bytes().iter());
            }
            Self::GetSlotInfo => {}
            Self::GetSlotDescription { slot_id } => {
                buf.reserve(0x02);
                buf.extend((*slot_id).to_be_bytes().iter());
            }
            Self::GetDefaultSlotValue => {}
            Self::GetSensorDefinition { sensor_id } => {
                buf.reserve(0x01);
                buf.push(*sensor_id)
            }
            Self::GetSensorValue { sensor_id } => {
                buf.reserve(0x01);
                buf.push(*sensor_id)
            }
            Self::SetSensorValue { sensor_id } => {
                buf.reserve(0x01);
                buf.push(*sensor_id)
            }
            Self::SetRecordSensors { sensor_id } => {
                buf.reserve(0x01);
                buf.push(*sensor_id)
            }
            Self::GetDeviceHours => {}
            Self::SetDeviceHours { device_hours } => {
                buf.reserve(0x04);
                buf.extend((*device_hours).to_be_bytes().iter());
            }
            Self::GetLampHours => {}
            Self::SetLampHours { lamp_hours } => {
                buf.reserve(0x04);
                buf.extend((*lamp_hours).to_be_bytes().iter());
            }
            Self::GetLampStrikes => {}
            Self::SetLampStrikes { lamp_strikes } => {
                buf.reserve(0x04);
                buf.extend((*lamp_strikes).to_be_bytes().iter());
            }
            Self::GetLampState => {}
            Self::SetLampState { lamp_state } => {
                buf.reserve(0x01);
                buf.push(u8::from(*lamp_state))
            }
            Self::GetLampOnMode => {}
            Self::SetLampOnMode { lamp_on_mode } => {
                buf.reserve(0x01);
                buf.push(u8::from(*lamp_on_mode))
            }
            Self::GetDevicePowerCycles => {}
            Self::SetDevicePowerCycles {
                device_power_cycles,
            } => {
                buf.reserve(0x04);
                buf.extend((*device_power_cycles).to_be_bytes().iter());
            }
            Self::GetDisplayInvert => {}
            Self::SetDisplayInvert { display_invert } => {
                buf.reserve(0x01);
                buf.push(*display_invert as u8)
            }
            Self::GetDisplayLevel => {}
            Self::SetDisplayLevel { display_level } => {
                buf.reserve(0x01);
                buf.push(*display_level)
            }
            Self::GetPanInvert => {}
            Self::SetPanInvert { pan_invert } => {
                buf.reserve(0x01);
                buf.push(*pan_invert as u8)
            }
            Self::GetTiltInvert => {}
            Self::SetTiltInvert { tilt_invert } => {
                buf.reserve(0x01);
                buf.push(*tilt_invert as u8)
            }
            Self::GetPanTiltSwap => {}
            Self::SetPanTiltSwap { pan_tilt_swap } => {
                buf.reserve(0x01);
                buf.push(*pan_tilt_swap as u8)
            }
            Self::GetRealTimeClock => {}
            Self::SetRealTimeClock {
                year,
                month,
                day,
                hour,
                minute,
                second,
            } => {
                buf.reserve(0x07);
                buf.extend((*year).to_be_bytes().iter());
                buf.push(*month);
                buf.push(*day);
                buf.push(*hour);
                buf.push(*minute);
                buf.push(*second);
            }
            Self::GetIdentifyDevice => {}
            Self::SetIdentifyDevice { identify } => {
                buf.reserve(0x01);
                buf.push(*identify as u8)
            }
            Self::SetResetDevice { reset_device } => {
                buf.reserve(0x01);
                buf.push(*reset_device as u8)
            }
            Self::GetPowerState => {}
            Self::SetPowerState { power_state } => {
                buf.reserve(0x01);
                buf.push(*power_state as u8)
            }
            Self::GetPerformSelfTest => {}
            Self::SetPerformSelfTest { self_test_id } => {
                buf.reserve(0x01);
                buf.push((*self_test_id).into())
            }
            Self::SetCapturePreset {
                scene_id,
                fade_times,
            } => {
                buf.reserve(if fade_times.is_some() { 0x08 } else { 0x02 });
                buf.extend((*scene_id).to_be_bytes().iter());

                if let Some(fade_times) = fade_times {
                    buf.extend((fade_times.up_fade_time).to_be_bytes().iter());
                    buf.extend((fade_times.down_fade_time).to_be_bytes().iter());
                    buf.extend((fade_times.wait_time).to_be_bytes().iter());
                }
            }
            Self::GetSelfTestDescription { self_test_id } => {
                buf.reserve(0x01);
                buf.push((*self_test_id).into())
            }
            Self::GetPresetPlayback => {}
            Self::SetPresetPlayback { mode, level } => {
                buf.reserve(0x03);
                buf.extend(u16::from(*mode).to_be_bytes().iter());
                buf.push(*level);
            }
            Self::ManufacturerSpecific { parameter_data, .. } => {
                buf.reserve(parameter_data.len());
                buf.extend(parameter_data);
            }
            Self::Unsupported { parameter_data, .. } => {
                buf.reserve(parameter_data.len());
                buf.extend(parameter_data);
            }
        };

        buf
    }

    #[cfg(not(feature = "alloc"))]
    pub fn encode(&self) -> Vec<u8, 231> {
        let mut buf: Vec<u8, 231> = Vec::new();

        match self {
            Self::DiscMute => {}
            Self::DiscUnMute => {}
            Self::DiscUniqueBranch {
                lower_bound_uid,
                upper_bound_uid,
            } => {
                buf.extend(lower_bound_uid.manufacturer_id.to_be_bytes());
                buf.extend(lower_bound_uid.device_id.to_be_bytes());
                buf.extend(upper_bound_uid.manufacturer_id.to_be_bytes());
                buf.extend(upper_bound_uid.device_id.to_be_bytes());
            }
            Self::GetCommsStatus => {}
            Self::SetCommsStatus => {}
            Self::GetQueuedMessage { status_type } => buf.push(*status_type as u8).unwrap(),
            Self::GetStatusMessages { status_type } => buf.push(*status_type as u8).unwrap(),
            Self::GetStatusIdDescription { status_id } => {
                buf.extend((*status_id).to_be_bytes());
            }
            Self::SetClearStatusId => {}
            Self::GetSubDeviceIdStatusReportThreshold => {}
            Self::SetSubDeviceIdStatusReportThreshold { status_type } => {
                buf.push(*status_type as u8).unwrap()
            }
            Self::GetSupportedParameters => {}
            Self::GetParameterDescription { parameter_id } => {
                buf.extend((*parameter_id).to_be_bytes());
            }
            Self::GetDeviceInfo => {}
            Self::GetProductDetailIdList => {}
            Self::GetDeviceModelDescription => {}
            Self::GetManufacturerLabel => {}
            Self::GetDeviceLabel => {}
            Self::SetDeviceLabel { device_label } => buf.extend(device_label.bytes()),
            Self::GetFactoryDefaults => {}
            Self::SetFactoryDefaults => {}
            Self::GetLanguageCapabilities => {}
            Self::GetLanguage => {}
            Self::SetLanguage { language } => buf.extend(language.bytes()),
            Self::GetSoftwareVersionLabel => {}
            Self::GetBootSoftwareVersionId => {}
            Self::GetBootSoftwareVersionLabel => {}
            Self::GetDmxPersonality => {}
            Self::SetDmxPersonality { personality_id } => buf.push(*personality_id).unwrap(),
            Self::GetDmxPersonalityDescription { personality } => buf.push(*personality).unwrap(),
            Self::GetDmxStartAddress => {}
            Self::SetDmxStartAddress { dmx_start_address } => {
                buf.extend((*dmx_start_address).to_be_bytes());
            }
            Self::GetSlotInfo => {}
            Self::GetSlotDescription { slot_id } => {
                buf.extend((*slot_id).to_be_bytes());
            }
            Self::GetDefaultSlotValue => {}
            Self::GetSensorDefinition { sensor_id } => buf.push(*sensor_id).unwrap(),
            Self::GetSensorValue { sensor_id } => buf.push(*sensor_id).unwrap(),
            Self::SetSensorValue { sensor_id } => buf.push(*sensor_id).unwrap(),
            Self::SetRecordSensors { sensor_id } => buf.push(*sensor_id).unwrap(),
            Self::GetDeviceHours => {}
            Self::SetDeviceHours { device_hours } => {
                buf.extend((*device_hours).to_be_bytes());
            }
            Self::GetLampHours => {}
            Self::SetLampHours { lamp_hours } => {
                buf.extend((*lamp_hours).to_be_bytes());
            }
            Self::GetLampStrikes => {}
            Self::SetLampStrikes { lamp_strikes } => {
                buf.extend((*lamp_strikes).to_be_bytes());
            }
            Self::GetLampState => {}
            Self::SetLampState { lamp_state } => buf.push(u8::from(*lamp_state)).unwrap(),
            Self::GetLampOnMode => {}
            Self::SetLampOnMode { lamp_on_mode } => buf.push(u8::from(*lamp_on_mode)).unwrap(),
            Self::GetDevicePowerCycles => {}
            Self::SetDevicePowerCycles {
                device_power_cycles,
            } => {
                buf.extend((*device_power_cycles).to_be_bytes());
            }
            Self::GetDisplayInvert => {}
            Self::SetDisplayInvert { display_invert } => buf.push(*display_invert as u8).unwrap(),
            Self::GetDisplayLevel => {}
            Self::SetDisplayLevel { display_level } => buf.push(*display_level).unwrap(),
            Self::GetPanInvert => {}
            Self::SetPanInvert { pan_invert } => buf.push(*pan_invert as u8).unwrap(),
            Self::GetTiltInvert => {}
            Self::SetTiltInvert { tilt_invert } => buf.push(*tilt_invert as u8).unwrap(),
            Self::GetPanTiltSwap => {}
            Self::SetPanTiltSwap { pan_tilt_swap } => buf.push(*pan_tilt_swap as u8).unwrap(),
            Self::GetRealTimeClock => {}
            Self::SetRealTimeClock {
                year,
                month,
                day,
                hour,
                minute,
                second,
            } => {
                buf.extend((*year).to_be_bytes());
                buf.push(*month).unwrap();
                buf.push(*day).unwrap();
                buf.push(*hour).unwrap();
                buf.push(*minute).unwrap();
                buf.push(*second).unwrap();
            }
            Self::GetIdentifyDevice => {}
            Self::SetIdentifyDevice { identify } => buf.push(*identify as u8).unwrap(),
            Self::SetResetDevice { reset_device } => buf.push(*reset_device as u8).unwrap(),
            Self::GetPowerState => {}
            Self::SetPowerState { power_state } => buf.push(*power_state as u8).unwrap(),
            Self::GetPerformSelfTest => {}
            Self::SetPerformSelfTest { self_test_id } => buf.push((*self_test_id).into()).unwrap(),
            Self::SetCapturePreset {
                scene_id,
                fade_times,
            } => {
                buf.extend((*scene_id).to_be_bytes());

                if let Some(fade_times) = fade_times {
                    buf.extend((fade_times.up_fade_time).to_be_bytes());
                    buf.extend((fade_times.down_fade_time).to_be_bytes());
                    buf.extend((fade_times.wait_time).to_be_bytes());
                }
            }
            Self::GetSelfTestDescription { self_test_id } => {
                buf.push((*self_test_id).into()).unwrap()
            }
            Self::GetPresetPlayback => {}
            Self::SetPresetPlayback { mode, level } => {
                buf.extend(u16::from(*mode).to_be_bytes());
                buf.push(*level).unwrap();
            }
            Self::ManufacturerSpecific { parameter_data, .. } => {
                buf.extend_from_slice(parameter_data).unwrap();
            }
            Self::Unsupported { parameter_data, .. } => {
                buf.extend_from_slice(parameter_data).unwrap();
            }
        };

        buf
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RdmRequest {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub port_id: u8,
    pub sub_device_id: SubDeviceId,
    pub parameter: RequestParameter,
}

impl RdmRequest {
    pub fn new(
        destination_uid: DeviceUID,
        source_uid: DeviceUID,
        transaction_number: u8,
        port_id: u8,
        sub_device_id: SubDeviceId,
        parameter: RequestParameter,
    ) -> Self {
        RdmRequest {
            destination_uid,
            source_uid,
            transaction_number,
            port_id,
            sub_device_id,
            parameter,
        }
    }

    pub fn command_class(&self) -> CommandClass {
        self.parameter.command_class()
    }

    pub fn parameter_id(&self) -> ParameterId {
        self.parameter.parameter_id()
    }

    #[cfg(feature = "alloc")]
    pub fn encode(&self) -> Vec<u8> {
        let parameter_data = self.parameter.encode();

        let message_length = 24 + parameter_data.len();

        let mut buf = Vec::with_capacity(message_length + 2);

        buf.push(RDM_START_CODE_BYTE);
        buf.push(RDM_SUB_START_CODE_BYTE);
        buf.push(message_length as u8);
        buf.extend(self.destination_uid.manufacturer_id.to_be_bytes().iter());
        buf.extend(self.destination_uid.device_id.to_be_bytes().iter());
        buf.extend(self.source_uid.manufacturer_id.to_be_bytes().iter());
        buf.extend(self.source_uid.device_id.to_be_bytes().iter());
        buf.push(self.transaction_number);
        buf.push(self.port_id);
        buf.push(0x00); // Message Count shall be set to 0x00 in all controller generated requests
        buf.extend(u16::from(self.sub_device_id).to_be_bytes().iter());
        buf.push(self.parameter.command_class() as u8);
        buf.extend(
            u16::from(self.parameter.parameter_id())
                .to_be_bytes()
                .iter(),
        );
        buf.push(parameter_data.len() as u8);
        buf.extend(parameter_data);
        buf.extend(bsd_16_crc(&buf[..]).to_be_bytes().iter());

        buf
    }

    #[cfg(not(feature = "alloc"))]
    pub fn encode(&self) -> Vec<u8, 257> {
        let parameter_data = self.parameter.encode();

        let message_length = 24 + parameter_data.len();

        let mut buf = Vec::new();

        buf.push(RDM_START_CODE_BYTE).unwrap();
        buf.push(RDM_SUB_START_CODE_BYTE).unwrap();
        buf.push(message_length as u8).unwrap();
        buf.extend(self.destination_uid.manufacturer_id.to_be_bytes());
        buf.extend(self.destination_uid.device_id.to_be_bytes());
        buf.extend(self.source_uid.manufacturer_id.to_be_bytes());
        buf.extend(self.source_uid.device_id.to_be_bytes());
        buf.push(self.transaction_number).unwrap();
        buf.push(self.port_id).unwrap();
        buf.push(0x00).unwrap(); // Message Count shall be set to 0x00 in all controller generated requests
        buf.extend(u16::from(self.sub_device_id).to_be_bytes());
        buf.push(self.parameter.command_class() as u8).unwrap();
        buf.extend(u16::from(self.parameter.parameter_id()).to_be_bytes());
        buf.push(parameter_data.len() as u8).unwrap();
        buf.extend(parameter_data);
        buf.extend(bsd_16_crc(&buf[..]).to_be_bytes());

        buf
    }
}

#[cfg(feature = "alloc")]
impl From<RdmRequest> for Vec<u8> {
    fn from(request: RdmRequest) -> Self {
        request.encode()
    }
}

#[cfg(not(feature = "alloc"))]
impl From<RdmRequest> for Vec<u8, 257> {
    fn from(request: RdmRequest) -> Self {
        request.encode()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_encode_discovery_unique_branch_request() {
        let encoded = RdmRequest::new(
            DeviceUID::new(0x0102, 0x03040506),
            DeviceUID::new(0x0605, 0x04030201),
            0x00,
            0x01,
            SubDeviceId::Id(0x01),
            RequestParameter::DiscUniqueBranch {
                lower_bound_uid: DeviceUID::new(0x0000, 0x00000000),
                upper_bound_uid: DeviceUID::new(0xffff, 0xffffffff),
            },
        )
        .encode();

        let expected = &[
            0xcc, // Start Code
            0x01, // Sub Start Code
            0x24, // Message Length
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // Destination UID
            0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // Source UID
            0x00, // Transaction Number
            0x01, // Port ID
            0x00, // Message Count
            0x00, 0x01, // Sub-Device ID
            0x10, // Command Class
            0x00, 0x01, // Parameter ID
            0x0c, // PDL
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Parameter Data - Lower Bound UID
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // Parameter Data - Upper Bound UID
            0x07, 0x34, // Checksum
        ];

        assert_eq!(encoded, expected);
    }

    #[test]
    fn should_encode_valid_rdm_request() {
        let encoded = RdmRequest::new(
            DeviceUID::new(0x0102, 0x03040506),
            DeviceUID::new(0x0605, 0x04030201),
            0x00,
            0x01,
            SubDeviceId::RootDevice,
            RequestParameter::GetIdentifyDevice,
        )
        .encode();

        let expected = &[
            0xcc, // Start Code
            0x01, // Sub Start Code
            0x18, // Message Length
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // Destination UID
            0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // Source UID
            0x00, // Transaction Number
            0x01, // Port ID
            0x00, // Message Count
            0x00, 0x00, // Sub-Device ID = Root Device
            0x20, // Command Class = GetCommand
            0x10, 0x00, // Parameter ID = Identify Device
            0x00, // PDL
            0x01, 0x40, // Checksum
        ];

        assert_eq!(encoded, expected);
    }

    #[test]
    fn should_encode_manufacturer_specific_rdm_request() {
        let encoded = RdmRequest::new(
            DeviceUID::new(0x0102, 0x03040506),
            DeviceUID::new(0x0605, 0x04030201),
            0x00,
            0x01,
            SubDeviceId::RootDevice,
            RequestParameter::ManufacturerSpecific {
                command_class: CommandClass::SetCommand,
                parameter_id: 0x8080,
                #[cfg(feature = "alloc")]
                parameter_data: vec![0x01, 0x02, 0x03, 0x04],
                #[cfg(not(feature = "alloc"))]
                parameter_data: Vec::<u8, 231>::from_slice(&[0x01, 0x02, 0x03, 0x04]).unwrap(),
            },
        )
        .encode();

        let expected = &[
            0xcc, // Start Code
            0x01, // Sub Start Code
            0x1c, // Message Length
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // Destination UID
            0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // Source UID
            0x00, // Transaction Number
            0x01, // Port ID
            0x00, // Message Count
            0x00, 0x00, // Sub-Device ID = Root Device
            0x30, // Command Class = SetCommand
            0x80, 0x80, // Parameter ID = Identify Device
            0x04, // PDL
            0x01, 0x02, 0x03, 0x04, // Parameter Data
            0x02, 0x52, // Checksum
        ];

        assert_eq!(encoded, expected);
    }
}
