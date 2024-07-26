use crate::{
    bsd_16_crc,
    device::DeviceUID,
    parameter::{LampOnMode, LampState, ParameterId},
    CommandClass, StatusType, SC_RDM, SC_SUB_MESSAGE,
};
use bytes::{BufMut, BytesMut};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FadeTimes {
    pub up_fade_time: u16,
    pub down_fade_time: u16,
    pub wait_time: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
    GetSubDeviceStatusReportThreshold,
    SetSubDeviceStatusReportThreshold {
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
        device_label: String,
    },
    GetFactoryDefaults,
    SetFactoryDefaults,
    GetLanguageCapabilities,
    GetLanguage,
    SetLanguage {
        language: String,
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
        display_invert: u8,
    }, // TODO could be an enum instead of u8 On/Off/Auto
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
        reset_device: u8,
    }, // TODO could be an enum instead of u8, 0x01 = Warn Reset, 0xff = Cold Reset
    GetPowerState,
    SetPowerState {
        power_state: u8,
    }, // TODO could be an enum instead of u8
    GetPerformSelfTest,
    SetPerformSelfTest {
        self_test_id: u8,
    },
    SetCapturePreset {
        scene_id: u16,
        fade_times: Option<FadeTimes>,
    },
    GetSelfTestDescription {
        self_test_id: u8,
    },
    GetPresetPlayback,
    SetPresetPlayback {
        mode: u16,
        level: u8,
    }, // TODO could be an enum instead of u16
    SetCurve {
        curve_id: u8,
    },
    GetCurveDescription {
        curve: u8,
    },
    GetModulationFrequencyDescription {
        modulation_frequency: u8,
    },
    SetModulationFrequency {
        modulation_frequency_id: u8,
    },
    GetOutputResponseTimeDescription {
        output_response_time: u8,
    },
    SetOutputResponseTime {
        output_response_time_id: u8,
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
            | Self::GetSubDeviceStatusReportThreshold
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
            | Self::GetPresetPlayback
            | Self::GetCurveDescription { .. }
            | Self::GetModulationFrequencyDescription { .. }
            | Self::GetOutputResponseTimeDescription { .. } => CommandClass::GetCommand,
            Self::SetCommsStatus
            | Self::SetClearStatusId
            | Self::SetSubDeviceStatusReportThreshold { .. }
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
            | Self::SetPresetPlayback { .. }
            | Self::SetCurve { .. }
            | Self::SetModulationFrequency { .. }
            | Self::SetOutputResponseTime { .. } => CommandClass::SetCommand,
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
            Self::GetSubDeviceStatusReportThreshold
            | Self::SetSubDeviceStatusReportThreshold { .. } => {
                ParameterId::SubDeviceStatusReportThreshold
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
            Self::SetCurve { .. } => ParameterId::Curve,
            Self::GetCurveDescription { .. } => ParameterId::CurveDescription,
            Self::SetModulationFrequency { .. } => ParameterId::ModulationFrequency,
            Self::GetModulationFrequencyDescription { .. } => {
                ParameterId::ModulationFrequencyDescription
            }
            Self::SetOutputResponseTime { .. } => ParameterId::OutputResponseTime,
            Self::GetOutputResponseTimeDescription { .. } => {
                ParameterId::OutputResponseTimeDescription
            }
        }
    }

    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::new();

        match self {
            Self::DiscMute => {}
            Self::DiscUnMute => {}
            Self::DiscUniqueBranch {
                lower_bound_uid,
                upper_bound_uid,
            } => {
                buf.reserve(0x0c);
                buf.put_u16(lower_bound_uid.manufacturer_id);
                buf.put_u32(lower_bound_uid.device_id);
                buf.put_u16(upper_bound_uid.manufacturer_id);
                buf.put_u32(upper_bound_uid.device_id);
            }
            Self::GetCommsStatus => {}
            Self::SetCommsStatus => {}
            Self::GetQueuedMessage { status_type } => {
                buf.reserve(0x01);
                buf.put_u8(*status_type as u8)
            }
            Self::GetStatusMessages { status_type } => {
                buf.reserve(0x01);
                buf.put_u8(*status_type as u8)
            }
            Self::GetStatusIdDescription { status_id } => {
                buf.reserve(0x02);
                buf.put_u16(*status_id)
            }
            Self::SetClearStatusId => {}
            Self::GetSubDeviceStatusReportThreshold => {}
            Self::SetSubDeviceStatusReportThreshold { status_type } => {
                buf.reserve(0x01);
                buf.put_u8(*status_type as u8)
            }
            Self::GetSupportedParameters => {}
            Self::GetParameterDescription { parameter_id } => {
                buf.reserve(0x02);
                buf.put_u16(*parameter_id)
            }
            Self::GetDeviceInfo => {}
            Self::GetProductDetailIdList => {}
            Self::GetDeviceModelDescription => {}
            Self::GetManufacturerLabel => {}
            Self::GetDeviceLabel => {}
            Self::SetDeviceLabel { device_label } => {
                buf.reserve(device_label.len());
                buf.put(device_label.as_ref())
            }
            Self::GetFactoryDefaults => {}
            Self::SetFactoryDefaults => {}
            Self::GetLanguageCapabilities => {}
            Self::GetLanguage => {}
            Self::SetLanguage { language } => {
                buf.reserve(language.len());
                buf.put(language.as_ref())
            }
            Self::GetSoftwareVersionLabel => {}
            Self::GetBootSoftwareVersionId => {}
            Self::GetBootSoftwareVersionLabel => {}
            Self::GetDmxPersonality => {}
            Self::SetDmxPersonality { personality_id } => {
                buf.reserve(0x01);
                buf.put_u8(*personality_id)
            }
            Self::GetDmxPersonalityDescription { personality } => {
                buf.reserve(0x01);
                buf.put_u8(*personality)
            }
            Self::GetDmxStartAddress => {}
            Self::SetDmxStartAddress { dmx_start_address } => {
                buf.reserve(0x02);
                buf.put_u16(*dmx_start_address)
            }
            Self::GetSlotInfo => {}
            Self::GetSlotDescription { slot_id } => {
                buf.reserve(0x02);
                buf.put_u16(*slot_id)
            }
            Self::GetDefaultSlotValue => {}
            Self::GetSensorDefinition { sensor_id } => {
                buf.reserve(0x01);
                buf.put_u8(*sensor_id)
            }
            Self::GetSensorValue { sensor_id } => {
                buf.reserve(0x01);
                buf.put_u8(*sensor_id)
            }
            Self::SetSensorValue { sensor_id } => {
                buf.reserve(0x01);
                buf.put_u8(*sensor_id)
            }
            Self::SetRecordSensors { sensor_id } => {
                buf.reserve(0x01);
                buf.put_u8(*sensor_id)
            }
            Self::GetDeviceHours => {}
            Self::SetDeviceHours { device_hours } => {
                buf.reserve(0x04);
                buf.put_u32(*device_hours)
            }
            Self::GetLampHours => {}
            Self::SetLampHours { lamp_hours } => {
                buf.reserve(0x04);
                buf.put_u32(*lamp_hours)
            }
            Self::GetLampStrikes => {}
            Self::SetLampStrikes { lamp_strikes } => {
                buf.reserve(0x04);
                buf.put_u32(*lamp_strikes)
            }
            Self::GetLampState => {}
            Self::SetLampState { lamp_state } => {
                buf.reserve(0x01);
                buf.put_u8(*lamp_state as u8)
            }
            Self::GetLampOnMode => {}
            Self::SetLampOnMode { lamp_on_mode } => {
                buf.reserve(0x01);
                buf.put_u8(*lamp_on_mode as u8)
            }
            Self::GetDevicePowerCycles => {}
            Self::SetDevicePowerCycles {
                device_power_cycles,
            } => {
                buf.reserve(0x04);
                buf.put_u32(*device_power_cycles)
            }
            Self::GetDisplayInvert => {}
            Self::SetDisplayInvert { display_invert } => {
                buf.reserve(0x01);
                buf.put_u8(*display_invert)
            }
            Self::GetDisplayLevel => {}
            Self::SetDisplayLevel { display_level } => {
                buf.reserve(0x01);
                buf.put_u8(*display_level)
            }
            Self::GetPanInvert => {}
            Self::SetPanInvert { pan_invert } => {
                buf.reserve(0x01);
                buf.put_u8(*pan_invert as u8)
            }
            Self::GetTiltInvert => {}
            Self::SetTiltInvert { tilt_invert } => {
                buf.reserve(0x01);
                buf.put_u8(*tilt_invert as u8)
            }
            Self::GetPanTiltSwap => {}
            Self::SetPanTiltSwap { pan_tilt_swap } => {
                buf.reserve(0x01);
                buf.put_u8(*pan_tilt_swap as u8)
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
                buf.put_u16(*year);
                buf.put_u8(*month);
                buf.put_u8(*day);
                buf.put_u8(*hour);
                buf.put_u8(*minute);
                buf.put_u8(*second);
            }
            Self::GetIdentifyDevice => {}
            Self::SetIdentifyDevice { identify } => {
                buf.reserve(0x01);
                buf.put_u8(*identify as u8)
            }
            Self::SetResetDevice { reset_device } => {
                buf.reserve(0x01);
                buf.put_u8(*reset_device)
            }
            Self::GetPowerState => {}
            Self::SetPowerState { power_state } => {
                buf.reserve(0x01);
                buf.put_u8(*power_state)
            }
            Self::GetPerformSelfTest => {}
            Self::SetPerformSelfTest { self_test_id } => {
                buf.reserve(0x01);
                buf.put_u8(*self_test_id)
            }
            Self::SetCapturePreset {
                scene_id,
                fade_times,
            } => {
                buf.reserve(if fade_times.is_some() { 0x08 } else { 0x02 });
                buf.put_u16(*scene_id);

                if let Some(fade_times) = fade_times {
                    buf.put_u16(fade_times.up_fade_time);
                    buf.put_u16(fade_times.down_fade_time);
                    buf.put_u16(fade_times.wait_time);
                }
            }
            Self::GetSelfTestDescription { self_test_id } => {
                buf.reserve(0x01);
                buf.put_u8(*self_test_id)
            }
            Self::GetPresetPlayback => {}
            Self::SetPresetPlayback { mode, level } => {
                buf.reserve(0x03);
                buf.put_u16(*mode);
                buf.put_u8(*level);
            }
            Self::SetCurve { curve_id } => {
                buf.reserve(0x01);
                buf.put_u8(*curve_id);
            }
            Self::GetCurveDescription { curve } => {
                buf.reserve(0x01);
                buf.put_u8(*curve)
            }
            Self::GetModulationFrequencyDescription {
                modulation_frequency,
            } => {
                buf.reserve(0x01);
                buf.put_u8(*modulation_frequency)
            }
            Self::SetModulationFrequency {
                modulation_frequency_id,
            } => {
                buf.reserve(0x01);
                buf.put_u8(*modulation_frequency_id);
            }
            Self::GetOutputResponseTimeDescription {
                output_response_time,
            } => {
                buf.reserve(0x01);
                buf.put_u8(*output_response_time)
            }
            Self::SetOutputResponseTime {
                output_response_time_id,
            } => {
                buf.reserve(0x01);
                buf.put_u8(*output_response_time_id);
            }
        };

        buf
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RdmRequest {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub port_id: u8,
    pub sub_device_id: u16,
    pub parameter: RequestParameter,
}

impl RdmRequest {
    pub fn new(
        destination_uid: DeviceUID,
        source_uid: DeviceUID,
        transaction_number: u8,
        port_id: u8,
        sub_device_id: u16,
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

    pub fn encode(self) -> BytesMut {
        let mut buf = BytesMut::new();

        let parameter_data = self.parameter.encode();
        dbg!(&parameter_data);

        buf.put_u8(SC_RDM);
        buf.put_u8(SC_SUB_MESSAGE);
        buf.put_u8(24 + parameter_data.len() as u8);
        buf.put_u16(self.destination_uid.manufacturer_id);
        buf.put_u32(self.destination_uid.device_id);
        buf.put_u16(self.source_uid.manufacturer_id);
        buf.put_u32(self.source_uid.device_id);
        buf.put_u8(self.transaction_number);
        buf.put_u8(self.port_id);
        buf.put_u8(0x00); // Message Count shall be set to 0x00 in all controller generated requests
        buf.put_u16(self.sub_device_id);
        buf.put_u8(self.parameter.command_class() as u8);
        buf.put_u16(self.parameter.parameter_id() as u16);
        buf.put_u8(parameter_data.len() as u8);
        buf.extend(parameter_data);
        buf.put_u16(bsd_16_crc(&buf[..]));

        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn should_encode_discovery_unique_branch_request() {
        let encoded = RdmRequest::new(
            DeviceUID::new(0x0102, 0x03040506),
            DeviceUID::new(0x0605, 0x04030201),
            0x00,
            0x01,
            0x0001,
            RequestParameter::DiscUniqueBranch {
                lower_bound_uid: DeviceUID::new(0x0000, 0x00000000),
                upper_bound_uid: DeviceUID::new(0xffff, 0xffffffff),
            },
        )
        .encode();

        let expected = BytesMut::from_iter(&[
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
        ]);

        assert_eq!(encoded, expected);
    }
}
