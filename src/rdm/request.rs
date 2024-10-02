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
    error::RdmError,
    parameter::{
        decode_string_bytes, DisplayInvertMode, FadeTimes, Ipv4Address, Ipv4Route, LampOnMode,
        LampState, MergeMode, ParameterId, PinCode, PowerState, PresetPlaybackMode,
        ResetDeviceMode, SelfTest, StatusType, TimeMode,
    },
    CommandClass, DeviceUID, EncodedFrame, EncodedParameterData, SubDeviceId, RDM_START_CODE_BYTE,
    RDM_SUB_START_CODE_BYTE,
};

#[cfg(not(feature = "alloc"))]
use heapless::{String, Vec};

#[derive(Clone, Debug, PartialEq)]
pub enum RequestParameter {
    // E1.20
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
    // E1.37-1
    GetDmxBlockAddress,
    SetDmxBlockAddress {
        dmx_block_address: u16,
    },
    GetDmxFailMode,
    SetDmxFailMode {
        scene_id: PresetPlaybackMode,
        loss_of_signal_delay_time: TimeMode,
        hold_time: TimeMode,
        level: u8,
    },
    GetDmxStartupMode,
    SetDmxStartupMode {
        scene_id: PresetPlaybackMode,
        startup_delay: TimeMode,
        hold_time: TimeMode,
        level: u8,
    },
    GetDimmerInfo,
    GetMinimumLevel,
    SetMinimumLevel {
        minimum_level_increasing: u16,
        minimum_level_decreasing: u16,
        on_below_minimum: bool,
    },
    GetMaximumLevel,
    SetMaximumLevel {
        maximum_level: u16,
    },
    GetCurve,
    SetCurve {
        curve_id: u8,
    },
    GetCurveDescription {
        curve_id: u8,
    },
    GetOutputResponseTime,
    SetOutputResponseTime {
        output_response_time_id: u8,
    },
    GetOutputResponseTimeDescription {
        output_response_time_id: u8,
    },
    GetModulationFrequency,
    SetModulationFrequency {
        modulation_frequency_id: u8,
    },
    GetModulationFrequencyDescription {
        modulation_frequency_id: u8,
    },
    GetPowerOnSelfTest,
    SetPowerOnSelfTest {
        self_test_id: SelfTest,
    },
    GetLockState,
    SetLockState {
        pin_code: PinCode,
        lock_state: bool,
    },
    GetLockStateDescription,
    GetLockPin,
    SetLockPin {
        new_pin_code: PinCode,
        current_pin_code: PinCode,
    },
    GetBurnIn,
    SetBurnIn {
        hours: u8,
    },
    GetIdentifyMode,
    SetIdentifyMode {
        identify_mode: u8,
    },
    GetPresetInfo,
    GetPresetStatus {
        scene_id: u16,
    },
    GetPresetMergeMode,
    SetPresetMergeMode {
        merge_mode: MergeMode,
    },
    SetPresetStatus {
        scene_id: u16,
        up_fade_time: u16,
        down_fade_time: u16,
        wait_time: u16,
        clear_preset: bool,
    },
    // E1.37-2
    GetListInterfaces,
    GetInterfaceLabel {
        interface_id: u32,
    },
    GetInterfaceHardwareAddressType1 {
        interface_id: u32,
    },
    GetIpV4DhcpMode {
        interface_id: u32,
    },
    SetIpV4DhcpMode {
        interface_id: u32,
        dhcp_mode: bool,
    },
    GetIpV4ZeroConfMode {
        interface_id: u32,
    },
    SetIpV4ZeroConfMode {
        interface_id: u32,
        zero_conf_mode: bool,
    },
    GetIpV4CurrentAddress {
        interface_id: u32,
    },
    GetIpV4StaticAddress {
        interface_id: u32,
    },
    SetIpV4StaticAddress {
        interface_id: u32,
        address: Ipv4Address,
        netmask: u8,
    },
    SetInterfaceApplyConfiguration {
        interface_id: u32,
    },
    SetInterfaceRenewDhcp {
        interface_id: u32,
    },
    SetInterfaceReleaseDhcp {
        interface_id: u32,
    },
    GetIpV4DefaultRoute,
    SetIpV4DefaultRoute {
        interface_id: u32,
        ipv4_default_route: Ipv4Route,
    },
    GetDnsIpV4NameServer {
        name_server_index: u8,
    },
    SetDnsIpV4NameServer {
        name_server_index: u8,
        name_server_address: Ipv4Address,
    },
    GetDnsHostName,
    SetDnsHostName {
        #[cfg(feature = "alloc")]
        host_name: String,
        #[cfg(not(feature = "alloc"))]
        host_name: String<63>,
    },
    GetDnsDomainName,
    SetDnsDomainName {
        #[cfg(feature = "alloc")]
        domain_name: String,
        #[cfg(not(feature = "alloc"))]
        domain_name: String<231>,
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
            // E1.20
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
            | Self::GetPresetPlayback
            // E1.37-1
            | Self::GetDmxBlockAddress
            | Self::GetDmxFailMode
            | Self::GetDmxStartupMode
            | Self::GetDimmerInfo
            | Self::GetMinimumLevel
            | Self::GetMaximumLevel
            | Self::GetCurve
            | Self::GetCurveDescription { .. }
            | Self::GetOutputResponseTime
            | Self::GetOutputResponseTimeDescription { .. }
            | Self::GetModulationFrequency
            | Self::GetModulationFrequencyDescription { .. }
            | Self::GetPowerOnSelfTest
            | Self::GetLockState
            | Self::GetLockStateDescription
            | Self::GetLockPin
            | Self::GetBurnIn
            | Self::GetIdentifyMode
            | Self::GetPresetInfo
            | Self::GetPresetStatus { .. }
            | Self::GetPresetMergeMode
            // E1.37-2
            | Self::GetListInterfaces
            | Self::GetInterfaceLabel { .. }
            | Self::GetInterfaceHardwareAddressType1 { .. }
            | Self::GetIpV4DhcpMode { .. }
            | Self::GetIpV4ZeroConfMode { .. }
            | Self::GetIpV4CurrentAddress { .. }
            | Self::GetIpV4StaticAddress { .. }
            | Self::GetIpV4DefaultRoute
            | Self::GetDnsIpV4NameServer { .. }
            | Self::GetDnsHostName
            | Self::GetDnsDomainName
            => CommandClass::GetCommand,
            // E1.20
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
            | Self::SetPresetPlayback { .. }
            // E1.37-1
            | Self::SetDmxBlockAddress { .. }
            | Self::SetDmxFailMode { .. }
            | Self::SetDmxStartupMode { .. }
            | Self::SetMinimumLevel { .. }
            | Self::SetMaximumLevel { .. }
            | Self::SetCurve { .. }
            | Self::SetOutputResponseTime { .. }
            | Self::SetModulationFrequency { .. }
            | Self::SetPowerOnSelfTest { .. }
            | Self::SetLockState { .. }
            | Self::SetLockPin { .. }
            | Self::SetBurnIn { .. }
            | Self::SetIdentifyMode { .. }
            | Self::SetPresetMergeMode { .. }
            | Self::SetPresetStatus { .. }
            // E1.37-2
            | Self::SetIpV4DhcpMode { .. }
            | Self::SetIpV4ZeroConfMode { .. }
            | Self::SetIpV4StaticAddress { .. }
            | Self::SetInterfaceApplyConfiguration { .. }
            | Self::SetInterfaceRenewDhcp { .. }
            | Self::SetInterfaceReleaseDhcp { .. }
            | Self::SetIpV4DefaultRoute { .. }
            | Self::SetDnsIpV4NameServer { .. }
            | Self::SetDnsHostName { .. }
            | Self::SetDnsDomainName { .. }
            => CommandClass::SetCommand,
            Self::ManufacturerSpecific { command_class, .. } => *command_class,
            Self::Unsupported { command_class, .. } => *command_class,
        }
    }

    pub fn parameter_id(&self) -> ParameterId {
        match self {
            // E1.20
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
            // E1.37-1
            Self::GetDmxBlockAddress | Self::SetDmxBlockAddress { .. } => {
                ParameterId::DmxBlockAddress
            }
            Self::GetDmxFailMode | Self::SetDmxFailMode { .. } => ParameterId::DmxFailMode,
            Self::GetDmxStartupMode | Self::SetDmxStartupMode { .. } => ParameterId::DmxStartupMode,
            Self::GetDimmerInfo => ParameterId::DimmerInfo,
            Self::GetMinimumLevel | Self::SetMinimumLevel { .. } => ParameterId::MinimumLevel,
            Self::GetMaximumLevel | Self::SetMaximumLevel { .. } => ParameterId::MaximumLevel,
            Self::GetCurve | Self::SetCurve { .. } => ParameterId::Curve,
            Self::GetCurveDescription { .. } => ParameterId::CurveDescription,
            Self::GetOutputResponseTime | Self::SetOutputResponseTime { .. } => {
                ParameterId::OutputResponseTime
            }
            Self::GetOutputResponseTimeDescription { .. } => {
                ParameterId::OutputResponseTimeDescription
            }
            Self::GetModulationFrequency | Self::SetModulationFrequency { .. } => {
                ParameterId::ModulationFrequency
            }
            Self::GetModulationFrequencyDescription { .. } => {
                ParameterId::ModulationFrequencyDescription
            }
            Self::GetPowerOnSelfTest | Self::SetPowerOnSelfTest { .. } => {
                ParameterId::PowerOnSelfTest
            }
            Self::GetLockState | Self::SetLockState { .. } => ParameterId::LockState,
            Self::GetLockStateDescription => ParameterId::LockStateDescription,
            Self::GetLockPin | Self::SetLockPin { .. } => ParameterId::LockPin,
            Self::GetBurnIn | Self::SetBurnIn { .. } => ParameterId::BurnIn,
            Self::GetIdentifyMode | Self::SetIdentifyMode { .. } => ParameterId::IdentifyMode,
            Self::GetPresetInfo => ParameterId::PresetInfo,
            Self::GetPresetStatus { .. } | Self::SetPresetStatus { .. } => {
                ParameterId::PresetStatus
            }
            Self::GetPresetMergeMode | Self::SetPresetMergeMode { .. } => {
                ParameterId::PresetMergeMode
            }
            // E1.37-2
            Self::GetListInterfaces => ParameterId::ListInterfaces,
            Self::GetInterfaceLabel { .. } => ParameterId::InterfaceLabel,
            Self::GetInterfaceHardwareAddressType1 { .. } => {
                ParameterId::InterfaceHardwareAddressType1
            }
            Self::GetIpV4DhcpMode { .. } | Self::SetIpV4DhcpMode { .. } => {
                ParameterId::IpV4DhcpMode
            }
            Self::GetIpV4ZeroConfMode { .. } | Self::SetIpV4ZeroConfMode { .. } => {
                ParameterId::IpV4ZeroConfMode
            }
            Self::GetIpV4CurrentAddress { .. } => ParameterId::IpV4CurrentAddress,
            Self::GetIpV4StaticAddress { .. } | Self::SetIpV4StaticAddress { .. } => {
                ParameterId::IpV4StaticAddress
            }
            Self::SetInterfaceApplyConfiguration { .. } => ParameterId::InterfaceApplyConfiguration,
            Self::SetInterfaceRenewDhcp { .. } => ParameterId::InterfaceRenewDhcp,
            Self::SetInterfaceReleaseDhcp { .. } => ParameterId::InterfaceReleaseDhcp,
            Self::GetIpV4DefaultRoute | Self::SetIpV4DefaultRoute { .. } => {
                ParameterId::IpV4DefaultRoute
            }
            Self::GetDnsIpV4NameServer { .. } | Self::SetDnsIpV4NameServer { .. } => {
                ParameterId::DnsIpV4NameServer
            }
            Self::GetDnsHostName | Self::SetDnsHostName { .. } => ParameterId::DnsHostName,
            Self::GetDnsDomainName | Self::SetDnsDomainName { .. } => ParameterId::DnsDomainName,
            Self::ManufacturerSpecific { parameter_id, .. } => {
                ParameterId::ManufacturerSpecific(*parameter_id)
            }
            Self::Unsupported { parameter_id, .. } => ParameterId::Unsupported(*parameter_id),
        }
    }

    pub fn encode(&self) -> EncodedParameterData {
        #[cfg(feature = "alloc")]
        let mut buf = Vec::new();

        #[cfg(not(feature = "alloc"))]
        let mut buf: Vec<u8, 231> = Vec::new();

        match self {
            // E1.20
            Self::DiscMute => {}
            Self::DiscUnMute => {}
            Self::DiscUniqueBranch {
                lower_bound_uid,
                upper_bound_uid,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x0c);

                buf.extend(lower_bound_uid.manufacturer_id.to_be_bytes());
                buf.extend(lower_bound_uid.device_id.to_be_bytes());
                buf.extend(upper_bound_uid.manufacturer_id.to_be_bytes());
                buf.extend(upper_bound_uid.device_id.to_be_bytes());
            }
            Self::GetCommsStatus => {}
            Self::SetCommsStatus => {}
            Self::GetQueuedMessage { status_type } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*status_type as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*status_type as u8).unwrap();
            }
            Self::GetStatusMessages { status_type } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*status_type as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*status_type as u8).unwrap();
            }
            Self::GetStatusIdDescription { status_id } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x02);

                buf.extend((*status_id).to_be_bytes());
            }
            Self::SetClearStatusId => {}
            Self::GetSubDeviceIdStatusReportThreshold => {}
            Self::SetSubDeviceIdStatusReportThreshold { status_type } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*status_type as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*status_type as u8).unwrap();
            }
            Self::GetSupportedParameters => {}
            Self::GetParameterDescription { parameter_id } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x02);

                buf.extend((*parameter_id).to_be_bytes());
            }
            Self::GetDeviceInfo => {}
            Self::GetProductDetailIdList => {}
            Self::GetDeviceModelDescription => {}
            Self::GetManufacturerLabel => {}
            Self::GetDeviceLabel => {}
            Self::SetDeviceLabel { device_label } => {
                #[cfg(feature = "alloc")]
                buf.reserve(device_label.len());

                buf.extend(device_label.bytes());
            }
            Self::GetFactoryDefaults => {}
            Self::SetFactoryDefaults => {}
            Self::GetLanguageCapabilities => {}
            Self::GetLanguage => {}
            Self::SetLanguage { language } => {
                #[cfg(feature = "alloc")]
                buf.reserve(language.len());

                buf.extend(language.bytes());
            }
            Self::GetSoftwareVersionLabel => {}
            Self::GetBootSoftwareVersionId => {}
            Self::GetBootSoftwareVersionLabel => {}
            Self::GetDmxPersonality => {}
            Self::SetDmxPersonality { personality_id } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*personality_id);
                #[cfg(not(feature = "alloc"))]
                buf.push(*personality_id).unwrap();
            }
            Self::GetDmxPersonalityDescription { personality } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*personality);
                #[cfg(not(feature = "alloc"))]
                buf.push(*personality).unwrap();
            }
            Self::GetDmxStartAddress => {}
            Self::SetDmxStartAddress { dmx_start_address } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x02);

                buf.extend((*dmx_start_address).to_be_bytes());
            }
            Self::GetSlotInfo => {}
            Self::GetSlotDescription { slot_id } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x02);

                buf.extend((*slot_id).to_be_bytes());
            }
            Self::GetDefaultSlotValue => {}
            Self::GetSensorDefinition { sensor_id } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*sensor_id);
                #[cfg(not(feature = "alloc"))]
                buf.push(*sensor_id).unwrap();
            }
            Self::GetSensorValue { sensor_id } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*sensor_id);
                #[cfg(not(feature = "alloc"))]
                buf.push(*sensor_id).unwrap();
            }
            Self::SetSensorValue { sensor_id } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*sensor_id);
                #[cfg(not(feature = "alloc"))]
                buf.push(*sensor_id).unwrap();
            }
            Self::SetRecordSensors { sensor_id } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*sensor_id);
                #[cfg(not(feature = "alloc"))]
                buf.push(*sensor_id).unwrap();
            }
            Self::GetDeviceHours => {}
            Self::SetDeviceHours { device_hours } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x04);

                buf.extend((*device_hours).to_be_bytes());
            }
            Self::GetLampHours => {}
            Self::SetLampHours { lamp_hours } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x04);

                buf.extend((*lamp_hours).to_be_bytes());
            }
            Self::GetLampStrikes => {}
            Self::SetLampStrikes { lamp_strikes } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x04);

                buf.extend((*lamp_strikes).to_be_bytes());
            }
            Self::GetLampState => {}
            Self::SetLampState { lamp_state } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(u8::from(*lamp_state));
                #[cfg(not(feature = "alloc"))]
                buf.push(u8::from(*lamp_state)).unwrap();
            }
            Self::GetLampOnMode => {}
            Self::SetLampOnMode { lamp_on_mode } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(u8::from(*lamp_on_mode));
                #[cfg(not(feature = "alloc"))]
                buf.push(u8::from(*lamp_on_mode)).unwrap();
            }
            Self::GetDevicePowerCycles => {}
            Self::SetDevicePowerCycles {
                device_power_cycles,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x04);

                buf.extend((*device_power_cycles).to_be_bytes());
            }
            Self::GetDisplayInvert => {}
            Self::SetDisplayInvert { display_invert } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*display_invert as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*display_invert as u8).unwrap();
            }
            Self::GetDisplayLevel => {}
            Self::SetDisplayLevel { display_level } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*display_level);
                #[cfg(not(feature = "alloc"))]
                buf.push(*display_level).unwrap();
            }
            Self::GetPanInvert => {}
            Self::SetPanInvert { pan_invert } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*pan_invert as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*pan_invert as u8).unwrap();
            }
            Self::GetTiltInvert => {}
            Self::SetTiltInvert { tilt_invert } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*tilt_invert as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*tilt_invert as u8).unwrap();
            }
            Self::GetPanTiltSwap => {}
            Self::SetPanTiltSwap { pan_tilt_swap } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*pan_tilt_swap as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*pan_tilt_swap as u8).unwrap();
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
                #[cfg(feature = "alloc")]
                buf.reserve(0x07);

                buf.extend((*year).to_be_bytes());

                #[cfg(feature = "alloc")]
                buf.push(*month);
                #[cfg(not(feature = "alloc"))]
                buf.push(*month).unwrap();
                #[cfg(feature = "alloc")]
                buf.push(*day);
                #[cfg(not(feature = "alloc"))]
                buf.push(*day).unwrap();
                #[cfg(feature = "alloc")]
                buf.push(*hour);
                #[cfg(not(feature = "alloc"))]
                buf.push(*hour).unwrap();
                #[cfg(feature = "alloc")]
                buf.push(*minute);
                #[cfg(not(feature = "alloc"))]
                buf.push(*minute).unwrap();
                #[cfg(feature = "alloc")]
                buf.push(*second);
                #[cfg(not(feature = "alloc"))]
                buf.push(*second).unwrap();
            }
            Self::GetIdentifyDevice => {}
            Self::SetIdentifyDevice { identify } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*identify as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*identify as u8).unwrap();
            }
            Self::SetResetDevice { reset_device } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*reset_device as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*reset_device as u8).unwrap();
            }
            Self::GetPowerState => {}
            Self::SetPowerState { power_state } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*power_state as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*power_state as u8).unwrap();
            }
            Self::GetPerformSelfTest => {}
            Self::SetPerformSelfTest { self_test_id } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push((*self_test_id).into());
                #[cfg(not(feature = "alloc"))]
                buf.push((*self_test_id).into()).unwrap();
            }
            Self::SetCapturePreset {
                scene_id,
                fade_times,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(if fade_times.is_some() { 0x08 } else { 0x02 });

                buf.extend((*scene_id).to_be_bytes());

                if let Some(fade_times) = fade_times {
                    buf.extend((fade_times.up_fade_time).to_be_bytes());
                    buf.extend((fade_times.down_fade_time).to_be_bytes());
                    buf.extend((fade_times.wait_time).to_be_bytes());
                }
            }
            Self::GetSelfTestDescription { self_test_id } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push((*self_test_id).into());
                #[cfg(not(feature = "alloc"))]
                buf.push((*self_test_id).into()).unwrap();
            }
            Self::GetPresetPlayback => {}
            Self::SetPresetPlayback { mode, level } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x03);

                buf.extend(u16::from(*mode).to_be_bytes());

                #[cfg(feature = "alloc")]
                buf.push(*level);
                #[cfg(not(feature = "alloc"))]
                buf.push(*level).unwrap();
            }
            // E1.37-1
            Self::GetDmxBlockAddress => {}
            Self::SetDmxBlockAddress { dmx_block_address } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x02);

                buf.extend((*dmx_block_address).to_be_bytes());
            }
            Self::GetDmxFailMode => {}
            Self::SetDmxFailMode {
                scene_id,
                loss_of_signal_delay_time,
                hold_time,
                level,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x07);

                buf.extend(u16::from(*scene_id).to_be_bytes());
                buf.extend(u16::from(*loss_of_signal_delay_time).to_be_bytes());
                buf.extend(u16::from(*hold_time).to_be_bytes());

                #[cfg(feature = "alloc")]
                buf.push(*level);
                #[cfg(not(feature = "alloc"))]
                buf.push(*level).unwrap();
            }
            Self::GetDmxStartupMode => {}
            Self::SetDmxStartupMode {
                scene_id,
                startup_delay,
                hold_time,
                level,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x07);

                buf.extend(u16::from(*scene_id).to_be_bytes());
                buf.extend(u16::from(*startup_delay).to_be_bytes());
                buf.extend(u16::from(*hold_time).to_be_bytes());

                #[cfg(feature = "alloc")]
                buf.push(*level);
                #[cfg(not(feature = "alloc"))]
                buf.push(*level).unwrap();
            }
            Self::GetDimmerInfo => {}
            Self::GetMinimumLevel => {}
            Self::SetMinimumLevel {
                minimum_level_increasing,
                minimum_level_decreasing,
                on_below_minimum,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x05);

                buf.extend((*minimum_level_increasing).to_be_bytes());
                buf.extend((*minimum_level_decreasing).to_be_bytes());

                #[cfg(feature = "alloc")]
                buf.push(*on_below_minimum as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*on_below_minimum as u8).unwrap();
            }
            Self::GetMaximumLevel => {}
            Self::SetMaximumLevel { maximum_level } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x02);

                buf.extend((*maximum_level).to_be_bytes());
            }
            Self::GetCurve => {}
            Self::SetCurve { curve_id } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*curve_id);
                #[cfg(not(feature = "alloc"))]
                buf.push(*curve_id).unwrap();
            }
            Self::GetCurveDescription { curve_id } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*curve_id);
                #[cfg(not(feature = "alloc"))]
                buf.push(*curve_id).unwrap();
            }
            Self::GetOutputResponseTime => {}
            Self::SetOutputResponseTime {
                output_response_time_id,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*output_response_time_id);
                #[cfg(not(feature = "alloc"))]
                buf.push(*output_response_time_id).unwrap();
            }
            Self::GetOutputResponseTimeDescription {
                output_response_time_id,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*output_response_time_id);
                #[cfg(not(feature = "alloc"))]
                buf.push(*output_response_time_id).unwrap();
            }
            Self::GetModulationFrequency => {}
            Self::SetModulationFrequency {
                modulation_frequency_id,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*modulation_frequency_id);
                #[cfg(not(feature = "alloc"))]
                buf.push(*modulation_frequency_id).unwrap();
            }
            Self::GetModulationFrequencyDescription {
                modulation_frequency_id,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*modulation_frequency_id);
                #[cfg(not(feature = "alloc"))]
                buf.push(*modulation_frequency_id).unwrap();
            }
            Self::GetPowerOnSelfTest => {}
            Self::SetPowerOnSelfTest { self_test_id } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push((*self_test_id).into());
                #[cfg(not(feature = "alloc"))]
                buf.push((*self_test_id).into()).unwrap();
            }
            Self::GetLockState => {}
            Self::SetLockState {
                pin_code,
                lock_state,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x03);

                buf.extend((pin_code.0).to_be_bytes());

                #[cfg(feature = "alloc")]
                buf.push(*lock_state as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*lock_state as u8).unwrap();
            }
            Self::GetLockStateDescription => {}
            Self::GetLockPin => {}
            Self::SetLockPin {
                new_pin_code,
                current_pin_code,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x04);

                buf.extend((new_pin_code.0).to_be_bytes());
                buf.extend((current_pin_code.0).to_be_bytes());
            }
            Self::GetBurnIn => {}
            Self::SetBurnIn { hours } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*hours);
                #[cfg(not(feature = "alloc"))]
                buf.push(*hours).unwrap();
            }
            Self::GetIdentifyMode => {}
            Self::SetIdentifyMode { identify_mode } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*identify_mode);
                #[cfg(not(feature = "alloc"))]
                buf.push(*identify_mode).unwrap();
            }
            Self::GetPresetInfo => {}
            Self::GetPresetStatus { scene_id } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x02);

                buf.extend((*scene_id).to_be_bytes());
            }
            Self::SetPresetStatus {
                scene_id,
                up_fade_time,
                down_fade_time,
                wait_time,
                clear_preset,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x0a);

                buf.extend((*scene_id).to_be_bytes());
                buf.extend((*up_fade_time).to_be_bytes());
                buf.extend((*down_fade_time).to_be_bytes());
                buf.extend((*wait_time).to_be_bytes());

                #[cfg(feature = "alloc")]
                buf.push(*clear_preset as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*clear_preset as u8).unwrap();
            }
            Self::GetPresetMergeMode => {}
            Self::SetPresetMergeMode { merge_mode } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x01);

                #[cfg(feature = "alloc")]
                buf.push(*merge_mode as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*merge_mode as u8).unwrap();
            }
            // E1.37-2
            Self::GetListInterfaces => {}
            Self::GetInterfaceLabel { interface_id } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x04);

                buf.extend((*interface_id).to_be_bytes());
            }
            Self::GetInterfaceHardwareAddressType1 { interface_id } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x04);

                buf.extend((*interface_id).to_be_bytes());
            }
            Self::GetIpV4DhcpMode { interface_id } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x04);

                buf.extend((*interface_id).to_be_bytes());
            }
            Self::SetIpV4DhcpMode {
                interface_id,
                dhcp_mode,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x05);

                buf.extend((*interface_id).to_be_bytes());

                #[cfg(feature = "alloc")]
                buf.push(*dhcp_mode as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*dhcp_mode as u8).unwrap();
            }
            Self::GetIpV4ZeroConfMode { interface_id } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x04);

                buf.extend((*interface_id).to_be_bytes());
            }
            Self::SetIpV4ZeroConfMode {
                interface_id,
                zero_conf_mode,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x05);

                buf.extend((*interface_id).to_be_bytes());

                #[cfg(feature = "alloc")]
                buf.push(*zero_conf_mode as u8);
                #[cfg(not(feature = "alloc"))]
                buf.push(*zero_conf_mode as u8).unwrap();
            }
            Self::GetIpV4CurrentAddress { interface_id } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x04);

                buf.extend((*interface_id).to_be_bytes());
            }
            Self::GetIpV4StaticAddress { interface_id } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x04);

                buf.extend((*interface_id).to_be_bytes());
            }
            Self::SetIpV4StaticAddress {
                interface_id,
                address,
                netmask,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x09);

                buf.extend((*interface_id).to_be_bytes());
                buf.extend::<[u8; 4]>((*address).into());

                #[cfg(feature = "alloc")]
                buf.push(*netmask);
                #[cfg(not(feature = "alloc"))]
                buf.push(*netmask).unwrap();
            }
            Self::SetInterfaceApplyConfiguration { interface_id } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x04);

                buf.extend((*interface_id).to_be_bytes());
            }
            Self::SetInterfaceRenewDhcp { interface_id } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x04);

                buf.extend((*interface_id).to_be_bytes());
            }
            Self::SetInterfaceReleaseDhcp { interface_id } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x04);

                buf.extend((*interface_id).to_be_bytes());
            }
            Self::GetIpV4DefaultRoute => {}
            Self::SetIpV4DefaultRoute {
                interface_id,
                ipv4_default_route,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x08);

                buf.extend((*interface_id).to_be_bytes());
                buf.extend::<[u8; 4]>((*ipv4_default_route).into());
            }
            Self::GetDnsIpV4NameServer { name_server_index } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x04);

                #[cfg(feature = "alloc")]
                buf.push(*name_server_index);
                #[cfg(not(feature = "alloc"))]
                buf.push(*name_server_index).unwrap();
            }
            Self::SetDnsIpV4NameServer {
                name_server_index,
                name_server_address,
            } => {
                #[cfg(feature = "alloc")]
                buf.reserve(0x08);

                #[cfg(feature = "alloc")]
                buf.push(*name_server_index);
                #[cfg(not(feature = "alloc"))]
                buf.push(*name_server_index).unwrap();

                buf.extend::<[u8; 4]>((*name_server_address).into());
            }
            Self::GetDnsHostName => {}
            Self::SetDnsHostName { host_name } => {
                #[cfg(feature = "alloc")]
                buf.reserve(host_name.len());

                buf.extend(host_name.bytes())
            }
            Self::GetDnsDomainName => {}
            Self::SetDnsDomainName { domain_name } => {
                #[cfg(feature = "alloc")]
                buf.reserve(domain_name.len());

                buf.extend(domain_name.bytes())
            }
            Self::ManufacturerSpecific { parameter_data, .. } => {
                #[cfg(feature = "alloc")]
                buf.reserve(parameter_data.len());

                #[cfg(feature = "alloc")]
                buf.extend(parameter_data);
                #[cfg(not(feature = "alloc"))]
                buf.extend_from_slice(parameter_data).unwrap();
            }
            Self::Unsupported { parameter_data, .. } => {
                #[cfg(feature = "alloc")]
                buf.reserve(parameter_data.len());

                #[cfg(feature = "alloc")]
                buf.extend(parameter_data);
                #[cfg(not(feature = "alloc"))]
                buf.extend_from_slice(parameter_data).unwrap();
            }
        };

        buf
    }

    pub fn decode(
        command_class: CommandClass,
        parameter_id: ParameterId,
        bytes: &[u8],
    ) -> Result<Self, RdmError> {
        match (command_class, parameter_id) {
            // E1.20
            (CommandClass::DiscoveryCommand, ParameterId::DiscMute) => Ok(Self::DiscMute),
            (CommandClass::DiscoveryCommand, ParameterId::DiscUnMute) => Ok(Self::DiscUnMute),
            (CommandClass::DiscoveryCommand, ParameterId::DiscUniqueBranch) => {
                if bytes.len() < 12 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                let lower_bound_uid = DeviceUID::new(
                    u16::from_be_bytes([bytes[0], bytes[1]]),
                    u32::from_be_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]),
                );
                let upper_bound_uid = DeviceUID::new(
                    u16::from_be_bytes([bytes[6], bytes[7]]),
                    u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
                );
                Ok(Self::DiscUniqueBranch {
                    lower_bound_uid,
                    upper_bound_uid,
                })
            }
            (CommandClass::GetCommand, ParameterId::CommsStatus) => Ok(Self::GetCommsStatus),
            (CommandClass::SetCommand, ParameterId::CommsStatus) => Ok(Self::SetCommsStatus),
            (CommandClass::GetCommand, ParameterId::QueuedMessage) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetQueuedMessage {
                    status_type: bytes[0].try_into()?,
                })
            }
            (CommandClass::GetCommand, ParameterId::StatusMessages) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetStatusMessages {
                    status_type: bytes[0].try_into()?,
                })
            }
            (CommandClass::GetCommand, ParameterId::StatusIdDescription) => {
                if bytes.len() < 2 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetStatusIdDescription {
                    status_id: u16::from_be_bytes([bytes[0], bytes[1]]),
                })
            }
            (CommandClass::SetCommand, ParameterId::ClearStatusId) => Ok(Self::SetClearStatusId),
            (CommandClass::GetCommand, ParameterId::SubDeviceIdStatusReportThreshold) => {
                Ok(Self::GetSubDeviceIdStatusReportThreshold)
            }
            (CommandClass::SetCommand, ParameterId::SubDeviceIdStatusReportThreshold) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetSubDeviceIdStatusReportThreshold {
                    status_type: bytes[0].try_into()?,
                })
            }
            (CommandClass::GetCommand, ParameterId::SupportedParameters) => {
                Ok(Self::GetSupportedParameters)
            }
            (CommandClass::GetCommand, ParameterId::ParameterDescription) => {
                if bytes.len() < 2 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetParameterDescription {
                    parameter_id: u16::from_be_bytes([bytes[0], bytes[1]]),
                })
            }
            (CommandClass::GetCommand, ParameterId::DeviceInfo) => Ok(Self::GetDeviceInfo),
            (CommandClass::GetCommand, ParameterId::ProductDetailIdList) => {
                Ok(Self::GetProductDetailIdList)
            }
            (CommandClass::GetCommand, ParameterId::DeviceModelDescription) => {
                Ok(Self::GetDeviceModelDescription)
            }
            (CommandClass::GetCommand, ParameterId::ManufacturerLabel) => {
                Ok(Self::GetManufacturerLabel)
            }
            (CommandClass::GetCommand, ParameterId::DeviceLabel) => Ok(Self::GetDeviceLabel),
            (CommandClass::SetCommand, ParameterId::DeviceLabel) => Ok(Self::SetDeviceLabel {
                #[cfg(feature = "alloc")]
                device_label: decode_string_bytes(bytes)?,
                #[cfg(not(feature = "alloc"))]
                device_label: decode_string_bytes(bytes)?,
            }),
            (CommandClass::GetCommand, ParameterId::FactoryDefaults) => {
                Ok(Self::GetFactoryDefaults)
            }
            (CommandClass::SetCommand, ParameterId::FactoryDefaults) => {
                Ok(Self::SetFactoryDefaults)
            }
            (CommandClass::GetCommand, ParameterId::LanguageCapabilities) => {
                Ok(Self::GetLanguageCapabilities)
            }
            (CommandClass::GetCommand, ParameterId::Language) => Ok(Self::GetLanguage),
            (CommandClass::SetCommand, ParameterId::Language) => Ok(Self::SetLanguage {
                #[cfg(feature = "alloc")]
                language: decode_string_bytes(bytes)?,
                #[cfg(not(feature = "alloc"))]
                language: decode_string_bytes(bytes)?,
            }),
            (CommandClass::GetCommand, ParameterId::SoftwareVersionLabel) => {
                Ok(Self::GetSoftwareVersionLabel)
            }
            (CommandClass::GetCommand, ParameterId::BootSoftwareVersionId) => {
                Ok(Self::GetBootSoftwareVersionId)
            }
            (CommandClass::GetCommand, ParameterId::BootSoftwareVersionLabel) => {
                Ok(Self::GetBootSoftwareVersionLabel)
            }
            (CommandClass::GetCommand, ParameterId::DmxPersonality) => Ok(Self::GetDmxPersonality),
            (CommandClass::SetCommand, ParameterId::DmxPersonality) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetDmxPersonality {
                    personality_id: bytes[0],
                })
            }
            (CommandClass::GetCommand, ParameterId::DmxPersonalityDescription) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetDmxPersonalityDescription {
                    personality: bytes[0],
                })
            }
            (CommandClass::GetCommand, ParameterId::DmxStartAddress) => {
                Ok(Self::GetDmxStartAddress)
            }
            (CommandClass::SetCommand, ParameterId::DmxStartAddress) => {
                if bytes.len() < 2 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetDmxStartAddress {
                    dmx_start_address: u16::from_be_bytes([bytes[0], bytes[1]]),
                })
            }
            (CommandClass::GetCommand, ParameterId::SlotInfo) => Ok(Self::GetSlotInfo),
            (CommandClass::GetCommand, ParameterId::SlotDescription) => {
                if bytes.len() < 2 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetSlotDescription {
                    slot_id: u16::from_be_bytes([bytes[0], bytes[1]]),
                })
            }
            (CommandClass::GetCommand, ParameterId::DefaultSlotValue) => {
                Ok(Self::GetDefaultSlotValue)
            }
            (CommandClass::GetCommand, ParameterId::SensorDefinition) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetSensorDefinition {
                    sensor_id: bytes[0],
                })
            }
            (CommandClass::GetCommand, ParameterId::SensorValue) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetSensorValue {
                    sensor_id: bytes[0],
                })
            }
            (CommandClass::SetCommand, ParameterId::SensorValue) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetSensorValue {
                    sensor_id: bytes[0],
                })
            }
            (CommandClass::SetCommand, ParameterId::RecordSensors) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetRecordSensors {
                    sensor_id: bytes[0],
                })
            }
            (CommandClass::GetCommand, ParameterId::DeviceHours) => Ok(Self::GetDeviceHours),
            (CommandClass::SetCommand, ParameterId::DeviceHours) => {
                if bytes.len() < 4 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetDeviceHours {
                    device_hours: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                })
            }
            (CommandClass::GetCommand, ParameterId::LampHours) => Ok(Self::GetLampHours),
            (CommandClass::SetCommand, ParameterId::LampHours) => {
                if bytes.len() < 4 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetLampHours {
                    lamp_hours: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                })
            }
            (CommandClass::GetCommand, ParameterId::LampStrikes) => Ok(Self::GetLampStrikes),
            (CommandClass::SetCommand, ParameterId::LampStrikes) => {
                if bytes.len() < 4 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetLampStrikes {
                    lamp_strikes: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                })
            }
            (CommandClass::GetCommand, ParameterId::LampState) => Ok(Self::GetLampState),
            (CommandClass::SetCommand, ParameterId::LampState) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetLampState {
                    lamp_state: bytes[0].try_into()?,
                })
            }
            (CommandClass::GetCommand, ParameterId::LampOnMode) => Ok(Self::GetLampOnMode),
            (CommandClass::SetCommand, ParameterId::LampOnMode) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetLampOnMode {
                    lamp_on_mode: bytes[0].try_into()?,
                })
            }
            (CommandClass::GetCommand, ParameterId::DevicePowerCycles) => {
                Ok(Self::GetDevicePowerCycles)
            }
            (CommandClass::SetCommand, ParameterId::DevicePowerCycles) => {
                if bytes.len() < 4 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetDevicePowerCycles {
                    device_power_cycles: u32::from_be_bytes([
                        bytes[0], bytes[1], bytes[2], bytes[3],
                    ]),
                })
            }
            (CommandClass::GetCommand, ParameterId::DisplayInvert) => Ok(Self::GetDisplayInvert),
            (CommandClass::SetCommand, ParameterId::DisplayInvert) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetDisplayInvert {
                    display_invert: bytes[0].try_into()?,
                })
            }
            (CommandClass::GetCommand, ParameterId::DisplayLevel) => Ok(Self::GetDisplayLevel),
            (CommandClass::SetCommand, ParameterId::DisplayLevel) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetDisplayLevel {
                    display_level: bytes[0],
                })
            }
            (CommandClass::GetCommand, ParameterId::PanInvert) => Ok(Self::GetPanInvert),
            (CommandClass::SetCommand, ParameterId::PanInvert) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetPanInvert {
                    pan_invert: bytes[0] != 0,
                })
            }
            (CommandClass::GetCommand, ParameterId::TiltInvert) => Ok(Self::GetTiltInvert),
            (CommandClass::SetCommand, ParameterId::TiltInvert) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetTiltInvert {
                    tilt_invert: bytes[0] != 0,
                })
            }
            (CommandClass::GetCommand, ParameterId::PanTiltSwap) => Ok(Self::GetPanTiltSwap),
            (CommandClass::SetCommand, ParameterId::PanTiltSwap) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetPanTiltSwap {
                    pan_tilt_swap: bytes[0] != 0,
                })
            }
            (CommandClass::GetCommand, ParameterId::RealTimeClock) => Ok(Self::GetRealTimeClock),
            (CommandClass::SetCommand, ParameterId::RealTimeClock) => {
                if bytes.len() < 7 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetRealTimeClock {
                    year: u16::from_be_bytes([bytes[0], bytes[1]]),
                    month: bytes[2],
                    day: bytes[3],
                    hour: bytes[4],
                    minute: bytes[5],
                    second: bytes[6],
                })
            }
            (CommandClass::GetCommand, ParameterId::IdentifyDevice) => Ok(Self::GetIdentifyDevice),
            (CommandClass::SetCommand, ParameterId::IdentifyDevice) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetIdentifyDevice {
                    identify: bytes[0] != 0,
                })
            }
            (CommandClass::SetCommand, ParameterId::ResetDevice) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetResetDevice {
                    reset_device: bytes[0].try_into()?,
                })
            }
            (CommandClass::GetCommand, ParameterId::PowerState) => Ok(Self::GetPowerState),
            (CommandClass::SetCommand, ParameterId::PowerState) => Ok(Self::SetPowerState {
                power_state: bytes[0].try_into()?,
            }),
            (CommandClass::GetCommand, ParameterId::PerformSelfTest) => {
                Ok(Self::GetPerformSelfTest)
            }
            (CommandClass::SetCommand, ParameterId::PerformSelfTest) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetPerformSelfTest {
                    self_test_id: bytes[0].into(),
                })
            }
            (CommandClass::SetCommand, ParameterId::CapturePreset) => {
                if bytes.len() < 2 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }

                let scene_id = u16::from_be_bytes([bytes[0], bytes[1]]);
                let fade_times = if bytes.len() > 2 {
                    Some(FadeTimes {
                        up_fade_time: u16::from_be_bytes([bytes[2], bytes[3]]),
                        down_fade_time: u16::from_be_bytes([bytes[4], bytes[5]]),
                        wait_time: u16::from_be_bytes([bytes[6], bytes[7]]),
                    })
                } else {
                    None
                };

                Ok(Self::SetCapturePreset {
                    scene_id,
                    fade_times,
                })
            }
            (CommandClass::GetCommand, ParameterId::SelfTestDescription) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetSelfTestDescription {
                    self_test_id: bytes[0].into(),
                })
            }
            (CommandClass::GetCommand, ParameterId::PresetPlayback) => Ok(Self::GetPresetPlayback),
            (CommandClass::SetCommand, ParameterId::PresetPlayback) => {
                if bytes.len() < 3 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetPresetPlayback {
                    mode: u16::from_be_bytes([bytes[0], bytes[1]]).into(),
                    level: bytes[2],
                })
            }
            (CommandClass::GetCommand, ParameterId::DmxBlockAddress) => {
                Ok(Self::GetDmxBlockAddress)
            }
            (CommandClass::SetCommand, ParameterId::DmxBlockAddress) => {
                if bytes.len() < 2 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetDmxBlockAddress {
                    dmx_block_address: u16::from_be_bytes([bytes[0], bytes[1]]),
                })
            }
            (CommandClass::GetCommand, ParameterId::DmxFailMode) => Ok(Self::GetDmxFailMode),
            (CommandClass::SetCommand, ParameterId::DmxFailMode) => {
                if bytes.len() < 7 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetDmxFailMode {
                    scene_id: u16::from_be_bytes([bytes[0], bytes[1]]).into(),
                    loss_of_signal_delay_time: u16::from_be_bytes([bytes[2], bytes[3]]).into(),
                    hold_time: u16::from_be_bytes([bytes[4], bytes[5]]).into(),
                    level: bytes[6],
                })
            }
            (CommandClass::GetCommand, ParameterId::DmxStartupMode) => Ok(Self::GetDmxStartupMode),
            (CommandClass::SetCommand, ParameterId::DmxStartupMode) => {
                if bytes.len() < 7 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetDmxStartupMode {
                    scene_id: u16::from_be_bytes([bytes[0], bytes[1]]).into(),
                    startup_delay: u16::from_be_bytes([bytes[2], bytes[3]]).into(),
                    hold_time: u16::from_be_bytes([bytes[4], bytes[5]]).into(),
                    level: bytes[6],
                })
            }
            (CommandClass::GetCommand, ParameterId::DimmerInfo) => Ok(Self::GetDimmerInfo),
            (CommandClass::GetCommand, ParameterId::MinimumLevel) => Ok(Self::GetMinimumLevel),
            (CommandClass::SetCommand, ParameterId::MinimumLevel) => {
                if bytes.len() < 5 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetMinimumLevel {
                    minimum_level_increasing: u16::from_be_bytes([bytes[0], bytes[1]]),
                    minimum_level_decreasing: u16::from_be_bytes([bytes[2], bytes[3]]),
                    on_below_minimum: bytes[4] != 0,
                })
            }
            (CommandClass::GetCommand, ParameterId::MaximumLevel) => Ok(Self::GetMaximumLevel),
            (CommandClass::SetCommand, ParameterId::MaximumLevel) => {
                if bytes.len() < 2 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetMaximumLevel {
                    maximum_level: u16::from_be_bytes([bytes[0], bytes[1]]),
                })
            }
            (CommandClass::GetCommand, ParameterId::Curve) => Ok(Self::GetCurve),
            (CommandClass::SetCommand, ParameterId::Curve) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetCurve { curve_id: bytes[0] })
            }
            (CommandClass::GetCommand, ParameterId::CurveDescription) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetCurveDescription { curve_id: bytes[0] })
            }
            (CommandClass::GetCommand, ParameterId::OutputResponseTime) => {
                Ok(Self::GetOutputResponseTime)
            }
            (CommandClass::SetCommand, ParameterId::OutputResponseTime) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetOutputResponseTime {
                    output_response_time_id: bytes[0],
                })
            }
            (CommandClass::GetCommand, ParameterId::OutputResponseTimeDescription) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetOutputResponseTimeDescription {
                    output_response_time_id: bytes[0],
                })
            }
            (CommandClass::GetCommand, ParameterId::ModulationFrequency) => {
                Ok(Self::GetModulationFrequency)
            }
            (CommandClass::SetCommand, ParameterId::ModulationFrequency) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetModulationFrequency {
                    modulation_frequency_id: bytes[0],
                })
            }
            (CommandClass::GetCommand, ParameterId::ModulationFrequencyDescription) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetModulationFrequencyDescription {
                    modulation_frequency_id: bytes[0],
                })
            }
            (CommandClass::GetCommand, ParameterId::PowerOnSelfTest) => {
                Ok(Self::GetPowerOnSelfTest)
            }
            (CommandClass::SetCommand, ParameterId::PowerOnSelfTest) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetPowerOnSelfTest {
                    self_test_id: bytes[0].into(),
                })
            }
            (CommandClass::GetCommand, ParameterId::LockState) => Ok(Self::GetLockState),
            (CommandClass::SetCommand, ParameterId::LockState) => {
                if bytes.len() < 5 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetLockState {
                    pin_code: u16::from_be_bytes([bytes[0], bytes[1]]).try_into()?,
                    lock_state: bytes[4] != 0,
                })
            }
            (CommandClass::GetCommand, ParameterId::LockStateDescription) => {
                Ok(Self::GetLockStateDescription)
            }
            (CommandClass::GetCommand, ParameterId::LockPin) => Ok(Self::GetLockPin),
            (CommandClass::SetCommand, ParameterId::LockPin) => {
                if bytes.len() < 8 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetLockPin {
                    new_pin_code: u16::from_be_bytes([bytes[0], bytes[1]]).try_into()?,
                    current_pin_code: u16::from_be_bytes([bytes[2], bytes[3]]).try_into()?,
                })
            }
            (CommandClass::GetCommand, ParameterId::BurnIn) => Ok(Self::GetBurnIn),
            (CommandClass::SetCommand, ParameterId::BurnIn) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetBurnIn { hours: bytes[0] })
            }
            (CommandClass::GetCommand, ParameterId::IdentifyMode) => Ok(Self::GetIdentifyMode),
            (CommandClass::SetCommand, ParameterId::IdentifyMode) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetIdentifyMode {
                    identify_mode: bytes[0],
                })
            }
            (CommandClass::GetCommand, ParameterId::PresetInfo) => Ok(Self::GetPresetInfo),
            (CommandClass::GetCommand, ParameterId::PresetStatus) => {
                if bytes.len() < 2 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetPresetStatus {
                    scene_id: u16::from_be_bytes([bytes[0], bytes[1]]),
                })
            }
            (CommandClass::SetCommand, ParameterId::PresetStatus) => {
                if bytes.len() < 9 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetPresetStatus {
                    scene_id: u16::from_be_bytes([bytes[0], bytes[1]]),
                    up_fade_time: u16::from_be_bytes([bytes[2], bytes[3]]),
                    down_fade_time: u16::from_be_bytes([bytes[4], bytes[5]]),
                    wait_time: u16::from_be_bytes([bytes[6], bytes[7]]),
                    clear_preset: bytes[8] != 0,
                })
            }
            (CommandClass::GetCommand, ParameterId::PresetMergeMode) => {
                Ok(Self::GetPresetMergeMode)
            }
            (CommandClass::SetCommand, ParameterId::PresetMergeMode) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetPresetMergeMode {
                    merge_mode: bytes[0].try_into()?,
                })
            }
            (CommandClass::GetCommand, ParameterId::ListInterfaces) => Ok(Self::GetListInterfaces),
            (CommandClass::GetCommand, ParameterId::InterfaceLabel) => {
                if bytes.len() < 4 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetInterfaceLabel {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                })
            }
            (CommandClass::GetCommand, ParameterId::InterfaceHardwareAddressType1) => {
                if bytes.len() < 4 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetInterfaceHardwareAddressType1 {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                })
            }
            (CommandClass::GetCommand, ParameterId::IpV4DhcpMode) => {
                if bytes.len() < 4 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetIpV4DhcpMode {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                })
            }
            (CommandClass::SetCommand, ParameterId::IpV4DhcpMode) => {
                if bytes.len() < 5 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetIpV4DhcpMode {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                    dhcp_mode: bytes[4] != 0,
                })
            }
            (CommandClass::GetCommand, ParameterId::IpV4ZeroConfMode) => {
                if bytes.len() < 4 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetIpV4ZeroConfMode {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                })
            }
            (CommandClass::SetCommand, ParameterId::IpV4ZeroConfMode) => {
                if bytes.len() < 5 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetIpV4ZeroConfMode {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                    zero_conf_mode: bytes[4] != 0,
                })
            }
            (CommandClass::GetCommand, ParameterId::IpV4CurrentAddress) => {
                if bytes.len() < 4 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetIpV4CurrentAddress {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                })
            }
            (CommandClass::GetCommand, ParameterId::IpV4StaticAddress) => {
                if bytes.len() < 4 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetIpV4StaticAddress {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                })
            }
            (CommandClass::SetCommand, ParameterId::IpV4StaticAddress) => {
                if bytes.len() < 9 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetIpV4StaticAddress {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                    address: Ipv4Address::from([bytes[4], bytes[5], bytes[6], bytes[7]]),
                    netmask: bytes[8],
                })
            }
            (CommandClass::SetCommand, ParameterId::InterfaceApplyConfiguration) => {
                if bytes.len() < 4 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetInterfaceApplyConfiguration {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                })
            }
            (CommandClass::SetCommand, ParameterId::InterfaceRenewDhcp) => {
                if bytes.len() < 4 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetInterfaceRenewDhcp {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                })
            }
            (CommandClass::SetCommand, ParameterId::InterfaceReleaseDhcp) => {
                if bytes.len() < 4 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetInterfaceReleaseDhcp {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                })
            }
            (CommandClass::GetCommand, ParameterId::IpV4DefaultRoute) => {
                Ok(Self::GetIpV4DefaultRoute)
            }
            (CommandClass::SetCommand, ParameterId::IpV4DefaultRoute) => {
                if bytes.len() < 8 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetIpV4DefaultRoute {
                    interface_id: u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                    ipv4_default_route: Ipv4Route::from([bytes[4], bytes[5], bytes[6], bytes[7]]),
                })
            }
            (CommandClass::GetCommand, ParameterId::DnsIpV4NameServer) => {
                if bytes.is_empty() {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::GetDnsIpV4NameServer {
                    name_server_index: bytes[0],
                })
            }
            (CommandClass::SetCommand, ParameterId::DnsIpV4NameServer) => {
                if bytes.len() < 5 {
                    return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
                }
                Ok(Self::SetDnsIpV4NameServer {
                    name_server_index: bytes[0],
                    name_server_address: Ipv4Address::from([
                        bytes[1], bytes[2], bytes[3], bytes[4],
                    ]),
                })
            }
            (CommandClass::GetCommand, ParameterId::DnsHostName) => Ok(Self::GetDnsHostName),
            (CommandClass::SetCommand, ParameterId::DnsHostName) => Ok(Self::SetDnsHostName {
                host_name: decode_string_bytes(bytes)?,
            }),
            (CommandClass::GetCommand, ParameterId::DnsDomainName) => Ok(Self::GetDnsDomainName),
            (CommandClass::SetCommand, ParameterId::DnsDomainName) => Ok(Self::SetDnsDomainName {
                domain_name: decode_string_bytes(bytes)?,
            }),
            (command_class, parameter_id) => {
                #[cfg(feature = "alloc")]
                let parameter_data = bytes.to_vec();
                #[cfg(not(feature = "alloc"))]
                let parameter_data = Vec::<u8, 231>::from_slice(bytes).unwrap();

                let parameter_id: u16 = parameter_id.into();

                if parameter_id >= 0x8000 {
                    Ok(Self::ManufacturerSpecific {
                        command_class,
                        parameter_id,
                        parameter_data,
                    })
                } else {
                    Ok(Self::Unsupported {
                        command_class,
                        parameter_id,
                        parameter_data,
                    })
                }
            }
        }
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

    pub fn encode(&self) -> EncodedFrame {
        let parameter_data = self.parameter.encode();

        let message_length = 24 + parameter_data.len();

        #[cfg(feature = "alloc")]
        let mut buf = Vec::with_capacity(message_length + 2);
        #[cfg(not(feature = "alloc"))]
        let mut buf = Vec::new();

        #[cfg(feature = "alloc")]
        buf.push(RDM_START_CODE_BYTE);
        #[cfg(not(feature = "alloc"))]
        buf.push(RDM_START_CODE_BYTE).unwrap();

        #[cfg(feature = "alloc")]
        buf.push(RDM_SUB_START_CODE_BYTE);
        #[cfg(not(feature = "alloc"))]
        buf.push(RDM_SUB_START_CODE_BYTE).unwrap();

        #[cfg(feature = "alloc")]
        buf.push(message_length as u8);
        #[cfg(not(feature = "alloc"))]
        buf.push(message_length as u8).unwrap();

        buf.extend(self.destination_uid.manufacturer_id.to_be_bytes());
        buf.extend(self.destination_uid.device_id.to_be_bytes());
        buf.extend(self.source_uid.manufacturer_id.to_be_bytes());
        buf.extend(self.source_uid.device_id.to_be_bytes());

        #[cfg(feature = "alloc")]
        buf.push(self.transaction_number);
        #[cfg(not(feature = "alloc"))]
        buf.push(self.transaction_number).unwrap();

        #[cfg(feature = "alloc")]
        buf.push(self.port_id);
        #[cfg(not(feature = "alloc"))]
        buf.push(self.port_id).unwrap();

        // Message Count shall be set to 0x00 in all controller generated requests
        #[cfg(feature = "alloc")]
        buf.push(0x00);
        #[cfg(not(feature = "alloc"))]
        buf.push(0x00).unwrap();

        buf.extend(u16::from(self.sub_device_id).to_be_bytes());

        #[cfg(feature = "alloc")]
        buf.push(self.parameter.command_class() as u8);
        #[cfg(not(feature = "alloc"))]
        buf.push(self.parameter.command_class() as u8).unwrap();

        buf.extend(u16::from(self.parameter.parameter_id()).to_be_bytes());

        #[cfg(feature = "alloc")]
        buf.push(parameter_data.len() as u8);
        #[cfg(not(feature = "alloc"))]
        buf.push(parameter_data.len() as u8).unwrap();

        buf.extend(parameter_data);
        buf.extend(bsd_16_crc(&buf[..]).to_be_bytes());

        buf
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, RdmError> {
        if bytes.len() < 24 {
            return Err(RdmError::InvalidMessageLength(bytes.len() as u8));
        }

        let destination_uid = DeviceUID::new(
            u16::from_be_bytes([bytes[3], bytes[4]]),
            u32::from_be_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]),
        );

        let source_uid = DeviceUID::new(
            u16::from_be_bytes([bytes[9], bytes[10]]),
            u32::from_be_bytes([bytes[11], bytes[12], bytes[13], bytes[14]]),
        );

        let transaction_number = bytes[15];
        let port_id = bytes[16];
        let sub_device_id = u16::from_be_bytes([bytes[18], bytes[19]]).into();
        let command_class = bytes[20].try_into()?;
        let parameter_id = u16::from_be_bytes([bytes[21], bytes[22]]).into();
        let parameter_data_length = bytes[23] as usize;

        if bytes.len() < 24 + parameter_data_length {
            return Err(RdmError::InvalidFrameLength(bytes.len() as u8));
        }

        let parameter_data = &bytes[24..24 + parameter_data_length];

        let parameter = RequestParameter::decode(command_class, parameter_id, parameter_data)?;

        Ok(Self::new(
            destination_uid,
            source_uid,
            transaction_number,
            port_id,
            sub_device_id,
            parameter,
        ))
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
    fn should_decode_discovery_unique_branch_request() {
        let decoded = RdmRequest::decode(&[
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
        ]).unwrap();
        
        let expected = RdmRequest::new(
            DeviceUID::new(0x0102, 0x03040506),
            DeviceUID::new(0x0605, 0x04030201),
            0x00,
            0x01,
            SubDeviceId::Id(0x01),
            RequestParameter::DiscUniqueBranch {
                lower_bound_uid: DeviceUID::new(0x0000, 0x00000000),
                upper_bound_uid: DeviceUID::new(0xffff, 0xffffffff),
            },
        );

        assert_eq!(decoded, expected);
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
    fn should_decode_valid_rdm_request() {
        let decoded = RdmRequest::decode(&[
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
        ]).unwrap();

        let expected = RdmRequest::new(
            DeviceUID::new(0x0102, 0x03040506),
            DeviceUID::new(0x0605, 0x04030201),
            0x00,
            0x01,
            SubDeviceId::RootDevice,
            RequestParameter::GetIdentifyDevice,
        );

        assert_eq!(decoded, expected);
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

    #[test]
    fn should_decode_manufacturer_specific_rdm_request() {
        let decoded = RdmRequest::decode(&[
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
        ]).unwrap();

        let expected = RdmRequest::new(
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
        );

        assert_eq!(decoded, expected);
    }
}
