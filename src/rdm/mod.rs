#![allow(unused)]
pub mod device;

use anyhow::anyhow;
use bytes::{BufMut, Bytes, BytesMut};
use core::panic;
use std::{cmp::PartialEq, collections::HashMap};
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};
use ux::u48;

use self::device::{DeviceUID, DmxSlot, Sensor};

pub const MIN_PACKET_LEN: usize = 26;

pub const SC_RDM: u8 = 0xcc;
pub const SC_SUB_MESSAGE: u8 = 0x01;

pub const BROADCAST_ALL_DEVICES_ID: u48 = u48::new(0xffffffffffff);
pub const SUB_DEVICE_ALL_CALL: u16 = 0xffff;
pub const ROOT_DEVICE: u16 = 0x0000;

#[derive(Debug, Error)]
pub enum ParameterError {
    #[error("unsupported parameter {0}")]
    UnsupportedParameter(String),
    // #[error("unknown parameter error")]
    // Unknown,
}

// TODO add remaining parameter ids
#[derive(Copy, Clone, Debug)]
pub enum StandardParameterId {
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
impl From<&[u8]> for StandardParameterId {
    fn from(bytes: &[u8]) -> Self {
        match u16::from_be_bytes(bytes.try_into().unwrap()) {
            0x0001 => StandardParameterId::DiscUniqueBranch,
            0x0002 => StandardParameterId::DiscMute,
            0x0003 => StandardParameterId::DiscUnMute,
            0x0010 => StandardParameterId::ProxiedDevices,
            0x0011 => StandardParameterId::ProxiedDeviceCount,
            0x0015 => StandardParameterId::CommsStatus,
            0x0020 => StandardParameterId::QueuedMessage,
            0x0030 => StandardParameterId::StatusMessages,
            0x0031 => StandardParameterId::StatusIdDescription,
            0x0032 => StandardParameterId::ClearStatusId,
            0x0033 => StandardParameterId::SubDeviceStatusReportThreshold,
            0x0050 => StandardParameterId::SupportedParameters,
            0x0051 => StandardParameterId::ParameterDescription,
            0x0060 => StandardParameterId::DeviceInfo,
            0x0070 => StandardParameterId::ProductDetailIdList,
            0x0080 => StandardParameterId::DeviceModelDescription,
            0x0081 => StandardParameterId::ManufacturerLabel,
            0x0082 => StandardParameterId::DeviceLabel,
            0x0090 => StandardParameterId::FactoryDefaults,
            0x00a0 => StandardParameterId::LanguageCapabilities,
            0x00b0 => StandardParameterId::Language,
            0x00c0 => StandardParameterId::SoftwareVersionLabel,
            0x00c1 => StandardParameterId::BootSoftwareVersionId,
            0x00c2 => StandardParameterId::BootSoftwareVersionLabel,
            0x00e0 => StandardParameterId::DmxPersonality,
            0x00e1 => StandardParameterId::DmxPersonalityDescription,
            0x00f0 => StandardParameterId::DmxStartAddress,
            0x0120 => StandardParameterId::SlotInfo,
            0x0121 => StandardParameterId::SlotDescription,
            0x0122 => StandardParameterId::DefaultSlotValue,
            0x0200 => StandardParameterId::SensorDefinition,
            0x0201 => StandardParameterId::SensorValue,
            0x0202 => StandardParameterId::RecordSensors,
            0x0340 => StandardParameterId::DimmerInfo,
            0x0341 => StandardParameterId::MinimumLevel,
            0x0342 => StandardParameterId::MaximumLevel,
            0x0343 => StandardParameterId::Curve,
            0x0344 => StandardParameterId::CurveDescription,
            0x0345 => StandardParameterId::OutputResponseTime,
            0x0346 => StandardParameterId::OutputResponseTimeDescription,
            0x0347 => StandardParameterId::ModulationFrequency,
            0x0348 => StandardParameterId::ModulationFrequencyDescription,
            0x0400 => StandardParameterId::DeviceHours,
            0x0401 => StandardParameterId::LampHours,
            0x0402 => StandardParameterId::LampStrikes,
            0x0403 => StandardParameterId::LampState,
            0x0404 => StandardParameterId::LampOnMode,
            0x0405 => StandardParameterId::DevicePowerCycles,
            0x0500 => StandardParameterId::DisplayInvert,
            0x0501 => StandardParameterId::DisplayLevel,
            0x0600 => StandardParameterId::PanInvert,
            0x0601 => StandardParameterId::TiltInvert,
            0x0602 => StandardParameterId::PanTiltSwap,
            0x0603 => StandardParameterId::RealTimeClock,
            0x1000 => StandardParameterId::IdentifyDevice,
            0x1001 => StandardParameterId::ResetDevice,
            0x1010 => StandardParameterId::PowerState,
            0x1020 => StandardParameterId::PerformSelfTest,
            0x1021 => StandardParameterId::SelfTestDescription,
            0x1030 => StandardParameterId::CapturePreset,
            0x1031 => StandardParameterId::PresetPlayback,
            _ => panic!("Invalid value for StandardParameterId: 0x{:04X?}", bytes),
        }
    }
}

// TODO this could use try_from and return a result rather than panic
impl From<u16> for StandardParameterId {
    fn from(parameter_id: u16) -> Self {
        match parameter_id {
            0x0001 => StandardParameterId::DiscUniqueBranch,
            0x0002 => StandardParameterId::DiscMute,
            0x0003 => StandardParameterId::DiscUnMute,
            0x0010 => StandardParameterId::ProxiedDevices,
            0x0011 => StandardParameterId::ProxiedDeviceCount,
            0x0015 => StandardParameterId::CommsStatus,
            0x0020 => StandardParameterId::QueuedMessage,
            0x0030 => StandardParameterId::StatusMessages,
            0x0031 => StandardParameterId::StatusIdDescription,
            0x0032 => StandardParameterId::ClearStatusId,
            0x0033 => StandardParameterId::SubDeviceStatusReportThreshold,
            0x0050 => StandardParameterId::SupportedParameters,
            0x0051 => StandardParameterId::ParameterDescription,
            0x0060 => StandardParameterId::DeviceInfo,
            0x0070 => StandardParameterId::ProductDetailIdList,
            0x0080 => StandardParameterId::DeviceModelDescription,
            0x0081 => StandardParameterId::ManufacturerLabel,
            0x0082 => StandardParameterId::DeviceLabel,
            0x0090 => StandardParameterId::FactoryDefaults,
            0x00a0 => StandardParameterId::LanguageCapabilities,
            0x00b0 => StandardParameterId::Language,
            0x00c0 => StandardParameterId::SoftwareVersionLabel,
            0x00c1 => StandardParameterId::BootSoftwareVersionId,
            0x00c2 => StandardParameterId::BootSoftwareVersionLabel,
            0x00e0 => StandardParameterId::DmxPersonality,
            0x00e1 => StandardParameterId::DmxPersonalityDescription,
            0x00f0 => StandardParameterId::DmxStartAddress,
            0x0120 => StandardParameterId::SlotInfo,
            0x0121 => StandardParameterId::SlotDescription,
            0x0122 => StandardParameterId::DefaultSlotValue,
            0x0200 => StandardParameterId::SensorDefinition,
            0x0201 => StandardParameterId::SensorValue,
            0x0202 => StandardParameterId::RecordSensors,
            0x0340 => StandardParameterId::DimmerInfo,
            0x0341 => StandardParameterId::MinimumLevel,
            0x0342 => StandardParameterId::MaximumLevel,
            0x0343 => StandardParameterId::Curve,
            0x0344 => StandardParameterId::CurveDescription,
            0x0345 => StandardParameterId::OutputResponseTime,
            0x0346 => StandardParameterId::OutputResponseTimeDescription,
            0x0347 => StandardParameterId::ModulationFrequency,
            0x0348 => StandardParameterId::ModulationFrequencyDescription,
            0x0400 => StandardParameterId::DeviceHours,
            0x0401 => StandardParameterId::LampHours,
            0x0402 => StandardParameterId::LampStrikes,
            0x0403 => StandardParameterId::LampState,
            0x0404 => StandardParameterId::LampOnMode,
            0x0405 => StandardParameterId::DevicePowerCycles,
            0x0500 => StandardParameterId::DisplayInvert,
            0x0501 => StandardParameterId::DisplayLevel,
            0x0600 => StandardParameterId::PanInvert,
            0x0601 => StandardParameterId::TiltInvert,
            0x0602 => StandardParameterId::PanTiltSwap,
            0x0603 => StandardParameterId::RealTimeClock,
            0x1000 => StandardParameterId::IdentifyDevice,
            0x1001 => StandardParameterId::ResetDevice,
            0x1010 => StandardParameterId::PowerState,
            0x1020 => StandardParameterId::PerformSelfTest,
            0x1021 => StandardParameterId::SelfTestDescription,
            0x1030 => StandardParameterId::CapturePreset,
            0x1031 => StandardParameterId::PresetPlayback,
            _ => panic!(
                "Invalid value for StandardParameterId: 0x{:04X?}",
                parameter_id
            ),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ParameterId {
    StandardParameter(StandardParameterId),
    ManufacturerSpecificParameter(u16),
}

pub const REQUIRED_PARAMETERS: [ParameterId; 4] = [
    ParameterId::StandardParameter(StandardParameterId::DeviceInfo),
    ParameterId::StandardParameter(StandardParameterId::SupportedParameters),
    ParameterId::StandardParameter(StandardParameterId::SoftwareVersionLabel),
    ParameterId::StandardParameter(StandardParameterId::IdentifyDevice),
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

#[derive(Debug, PartialEq)]
pub enum PacketType {
    RdmResponse = 0xcc01,
    DiscoveryUniqueBranchResponse = 0xfefe,
}

impl TryFrom<u16> for PacketType {
    type Error = &'static str;

    fn try_from(byte: u16) -> Result<Self, Self::Error> {
        let packet_type = match byte {
            0xcc01 => PacketType::RdmResponse,
            0xfefe => PacketType::DiscoveryUniqueBranchResponse,
            _ => return Err("Invalid value for PacketRequestType"),
        };
        Ok(packet_type)
    }
}

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
pub enum SupportedCommandClasses {
    Get = 0x01,
    Set = 0x02,
    GetSet = 0x03,
}

impl TryFrom<u8> for SupportedCommandClasses {
    type Error = &'static str;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        let command_class = match byte {
            0x01 => SupportedCommandClasses::Get,
            0x02 => SupportedCommandClasses::Set,
            0x03 => SupportedCommandClasses::GetSet,
            _ => return Err("Invalid value for CommandClass"),
        };
        Ok(command_class)
    }
}

pub fn bsd_16_crc_bytes_mut(packet: &mut BytesMut) -> u16 {
    packet
        .iter()
        .fold(0_u16, |sum, byte| (sum.overflowing_add(*byte as u16).0))
}

pub fn bsd_16_crc(packet: &Vec<u8>) -> u16 {
    packet
        .iter()
        .fold(0_u16, |sum, byte| (sum.overflowing_add(*byte as u16).0))
}

#[derive(Clone, Debug)]
pub enum ResponseType {
    Ack = 0x00,
    AckTimer = 0x01,
    NackReason = 0x02,
    AckOverflow = 0x03,
}

impl TryFrom<u8> for ResponseType {
    type Error = &'static str;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        let response_type = match byte {
            0x00 => ResponseType::Ack,
            0x01 => ResponseType::AckTimer,
            0x02 => ResponseType::NackReason,
            0x03 => ResponseType::AckOverflow,
            _ => return Err("Invalid value for ResponseType"),
        };
        Ok(response_type)
    }
}

pub enum ResponseNackReasonCode {
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

// Product Categories - Page 105 RDM Spec
#[derive(Copy, Clone, Debug)]
pub enum ProductCategory {
    NotDeclared = 0x0000,
    Fixture = 0x0100,
    FixtureFixed = 0x0101,
    FixtureMovingYoke = 0x0102,
    FixtureMovingMirror = 0x0103,
    FixtureOther = 0x01ff,
    FixtureAccessory = 0x0200,
    FixtureAccessoryColor = 0x0201,
    FixtureAccessoryYoke = 0x0202,
    FixtureAccessoryMirror = 0x0203,
    FixtureAccessoryEffect = 0x0204,
    FixtureAccessoryBeam = 0x0205,
    AccessoryOther = 0x02ff,
    Projector = 0x0300,
    ProjectorFixed = 0x0301,
    ProjectorMovingYoke = 0x0302,
    ProjectorMovingMirror = 0x0303,
    ProjectorOther = 0x03ff,
    Atmospheric = 0x0400,
    AtmosphericEffect = 0x0401,
    AtmosphericPyro = 0x0402,
    AtmosphericOther = 0x04ff,
    Dimmer = 0x0500,
    DimmerACIncandescent = 0x0501,
    DimmerACFlourescent = 0x0502,
    DimmerACColdCathode = 0x0503,
    DimmerACNonDimModule = 0x0504,
    DimmerACLowVoltage = 0x0505,
    DimmerControllableAC = 0x0506,
    DimmerDCLevelOutput = 0x0507,
    DimmerDCPWMOutput = 0x0508,
    DimmerSpecialisedLED = 0x0509,
    DimmerOther = 0x05ff,
    Power = 0x0600,
    PowerControl = 0x0601,
    PowerSource = 0x0602,
    PowerOther = 0x06ff,
    Scenic = 0x0700,
    ScenicDrive = 0x0701,
    ScenicOther = 0x07ff,
    Data = 0x0800,
    DataDistribution = 0x0801,
    DataConversion = 0x0802,
    DataOther = 0x08ff,
    AV = 0x0900,
    AVAudio = 0x0901,
    AVVideo = 0x0902,
    AVOther = 0x09ff,
    Monitor = 0x0a00,
    MonitorACLinePower = 0x0a01,
    MonitorDCPower = 0x0a02,
    MonitorEnvironmental = 0x0a03,
    MonitorOther = 0x0aff,
    Control = 0x7000,
    ControlController = 0x7001,
    ControlBackupDevice = 0x7002,
    ControlOther = 0x70ff,
    Test = 0x7100,
    TestEquipment = 0x7101,
    TestEquipmentOther = 0x71ff,
    Other = 0x7fff,
}

impl From<&[u8]> for ProductCategory {
    fn from(bytes: &[u8]) -> Self {
        match u16::from_be_bytes(bytes.try_into().unwrap()) {
            0x0000 => ProductCategory::NotDeclared,
            0x0100 => ProductCategory::Fixture,
            0x0101 => ProductCategory::FixtureFixed,
            0x0102 => ProductCategory::FixtureMovingYoke,
            0x0103 => ProductCategory::FixtureMovingMirror,
            0x01ff => ProductCategory::FixtureOther,
            0x0200 => ProductCategory::FixtureAccessory,
            0x0201 => ProductCategory::FixtureAccessoryColor,
            0x0202 => ProductCategory::FixtureAccessoryYoke,
            0x0203 => ProductCategory::FixtureAccessoryMirror,
            0x0204 => ProductCategory::FixtureAccessoryEffect,
            0x0205 => ProductCategory::FixtureAccessoryBeam,
            0x02ff => ProductCategory::AccessoryOther,
            0x0300 => ProductCategory::Projector,
            0x0301 => ProductCategory::ProjectorFixed,
            0x0302 => ProductCategory::ProjectorMovingYoke,
            0x0303 => ProductCategory::ProjectorMovingMirror,
            0x03ff => ProductCategory::ProjectorOther,
            0x0400 => ProductCategory::Atmospheric,
            0x0401 => ProductCategory::AtmosphericEffect,
            0x0402 => ProductCategory::AtmosphericPyro,
            0x04ff => ProductCategory::AtmosphericOther,
            0x0500 => ProductCategory::Dimmer,
            0x0501 => ProductCategory::DimmerACIncandescent,
            0x0502 => ProductCategory::DimmerACFlourescent,
            0x0503 => ProductCategory::DimmerACColdCathode,
            0x0504 => ProductCategory::DimmerACNonDimModule,
            0x0505 => ProductCategory::DimmerACLowVoltage,
            0x0506 => ProductCategory::DimmerControllableAC,
            0x0507 => ProductCategory::DimmerDCLevelOutput,
            0x0508 => ProductCategory::DimmerDCPWMOutput,
            0x0509 => ProductCategory::DimmerSpecialisedLED,
            0x05ff => ProductCategory::DimmerOther,
            0x0600 => ProductCategory::Power,
            0x0601 => ProductCategory::PowerControl,
            0x0602 => ProductCategory::PowerSource,
            0x06ff => ProductCategory::PowerOther,
            0x0700 => ProductCategory::Scenic,
            0x0701 => ProductCategory::ScenicDrive,
            0x07ff => ProductCategory::ScenicOther,
            0x0800 => ProductCategory::Data,
            0x0801 => ProductCategory::DataDistribution,
            0x0802 => ProductCategory::DataConversion,
            0x08ff => ProductCategory::DataOther,
            0x0900 => ProductCategory::AV,
            0x0901 => ProductCategory::AVAudio,
            0x0902 => ProductCategory::AVVideo,
            0x09ff => ProductCategory::AVOther,
            0x0a00 => ProductCategory::Monitor,
            0x0a01 => ProductCategory::MonitorACLinePower,
            0x0a02 => ProductCategory::MonitorDCPower,
            0x0a03 => ProductCategory::MonitorEnvironmental,
            0x0aff => ProductCategory::MonitorOther,
            0x7000 => ProductCategory::Control,
            0x7001 => ProductCategory::ControlController,
            0x7002 => ProductCategory::ControlBackupDevice,
            0x70ff => ProductCategory::ControlOther,
            0x7100 => ProductCategory::Test,
            0x7101 => ProductCategory::TestEquipment,
            0x71ff => ProductCategory::TestEquipmentOther,
            0x7fff => ProductCategory::Other,
            _ => panic!("Invalid value for ProductCategory: {:?}", bytes),
        }
    }
}

#[derive(Clone, Debug)]
pub enum LampState {
    LampOff = 0x00,        // 0x00 = "Lamp Off",
    LampOn = 0x01,         // 0x01 = "Lamp On",
    LampStrike = 0x02,     // 0x02 = "Lamp Strike",
    LampStandby = 0x03,    // 0x03 = "Lamp Standby",
    LampNotPresent = 0x04, // 0x04 = "Lamp Not Present",
    LampError = 0x05,      // 0x05 = "Lamp Error",
}

impl From<u8> for LampState {
    fn from(byte: u8) -> Self {
        match byte {
            0x00 => LampState::LampOff,
            0x01 => LampState::LampOn,
            0x02 => LampState::LampStrike,
            0x03 => LampState::LampStandby,
            0x04 => LampState::LampNotPresent,
            0x05 => LampState::LampError,
            _ => panic!("Invalid value for LampState: {:?}", byte),
        }
    }
}

#[derive(Clone, Debug)]
pub enum LampOnMode {
    OffMode = 0x00,  // 0x00 = "Off Mode",
    DmxMode = 0x01,  // 0x01 = "DMX Mode",
    OnMode = 0x02,   // 0x02 = "On Mode",
    AfterCal = 0x03, // 0x03 = "After Cal",
}

impl From<u8> for LampOnMode {
    fn from(byte: u8) -> Self {
        match byte {
            0x00 => LampOnMode::OffMode,
            0x01 => LampOnMode::DmxMode,
            0x02 => LampOnMode::OnMode,
            0x03 => LampOnMode::AfterCal,
            _ => panic!("Invalid value for LampOnMode: {:?}", byte),
        }
    }
}

#[derive(Clone, Debug)]
pub enum PowerState {
    FullOff = 0x00,  // 0x00 = "Full Off",
    Shutdown = 0x01, // 0x01 = "Shutdown",
    Standby = 0x02,  // 0x02 = "Standby",
    Normal = 0xff,   // 0xff = "Normal",
}

impl From<u8> for PowerState {
    fn from(byte: u8) -> Self {
        match byte {
            0x00 => PowerState::FullOff,
            0x01 => PowerState::Shutdown,
            0x02 => PowerState::Standby,
            0x03 => PowerState::Normal,
            _ => panic!("Invalid value for PowerState: {:?}", byte),
        }
    }
}

#[derive(Clone, Debug)]
pub enum OnOffStates {
    Off = 0x00, // 0x00 = "Off",
    On = 0x01,  // 0x01 = "On",
}

impl From<u8> for OnOffStates {
    fn from(byte: u8) -> Self {
        match byte {
            0x00 => OnOffStates::Off,
            0x01 => OnOffStates::On,
            _ => panic!("Invalid value for OnOffStates: {:?}", byte),
        }
    }
}

#[derive(Clone, Debug)]
pub enum DisplayInvertMode {
    Off = 0x00,  // 0x00 = "Off",
    On = 0x01,   // 0x01 = "On",
    Auto = 0x02, // 0x02 = "Auto",
}

impl From<u8> for DisplayInvertMode {
    fn from(byte: u8) -> Self {
        match byte {
            0x00 => DisplayInvertMode::Off,
            0x01 => DisplayInvertMode::On,
            0x02 => DisplayInvertMode::Auto,
            _ => panic!("Invalid value for DisplayInvertMode: {:?}", byte),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ResetType {
    Warm = 0x01,
    Cold = 0xff,
}

impl From<u8> for ResetType {
    fn from(byte: u8) -> Self {
        match byte {
            0x01 => ResetType::Warm,
            0xff => ResetType::Cold,
            _ => panic!("Invalid value for ResetType: {:?}", byte),
        }
    }
}

#[derive(Debug, Error)]
pub enum DriverError {
    #[error("invalid data length")]
    InvalidDataLength,
    #[error("invalid start byte")]
    InvalidStartByte,
    #[error("invalid stop byte")]
    InvalidStopByte,
    #[error("invalid packet type")]
    UnsupportedPacketType,
    #[error("malformed packet")]
    MalformedPacket,
    #[error("unknown driver error")]
    Unknown,
}

// TODO the following is a quick and dirty test
#[derive(Debug, PartialEq)]
pub enum PacketResponseType {
    SuccessResponse = 0x05,
    NullResponse = 0x0c,
}

impl TryFrom<u8> for PacketResponseType {
    type Error = DriverError;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        let packet_type = match byte {
            0x05 => PacketResponseType::SuccessResponse,
            0x0c => PacketResponseType::NullResponse,
            _ => return Err(DriverError::UnsupportedPacketType),
        };
        Ok(packet_type)
    }
}

pub fn is_checksum_valid(packet: &Vec<u8>) -> bool {
    let packet_checksum =
        u16::from_be_bytes(packet[packet.len() - 2..packet.len()].try_into().unwrap());

    let calculated_checksum = bsd_16_crc(&packet[..packet.len() - 2].try_into().unwrap());

    packet_checksum == calculated_checksum
}

pub enum DiscoveryRequestParameterData {
    DiscUniqueBranch {
        lower_bound_uid: u48,
        upper_bound_uid: u48,
    },
}

pub struct DiscoveryRequest {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub port_id: u8,
    pub sub_device: u16,
    pub parameter_id: ParameterId,
    pub parameter_data: Option<DiscoveryRequestParameterData>,
}

pub enum GetRequestParameterData {
    ParameterDescription { parameter_id: u16 },
    SensorDefinition { sensor_id: u8 },
    DmxPersonalityDescription { personality: u8 },
    CurveDescription { curve: u8 },
    ModulationFrequencyDescription { modulation_frequency: u8 },
    OutputResponseTimeDescription { output_response_time: u8 },
    SelfTestDescription { self_test_id: u8 },
}

pub struct GetRequest {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub port_id: u8,
    pub sub_device: u16,
    pub parameter_id: ParameterId,
    pub parameter_data: Option<GetRequestParameterData>,
}

pub enum RdmRequestMessage {
    DiscoveryRequest(DiscoveryRequest),
    GetRequest(GetRequest),
    // SetCommand(SetCommandMessage)
}

impl From<RdmRequestMessage> for BytesMut {
    fn from(value: RdmRequestMessage) -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub enum DiscoveryResponseParameterData {
    DiscMute {
        control_field: u16,
        binding_uid: Option<DeviceUID>,
    },
    DiscUnmute {
        control_field: u16,
        binding_uid: Option<DeviceUID>,
    },
}

#[derive(Debug)]
pub enum GetResponseParameterData {
    ProxiedDeviceCount {
        device_count: u16,
        list_change: bool,
    },
    ProxiedDevices {
        device_uids: Vec<DeviceUID>,
    },
    ParameterDescription {
        parameter_id: u16,
        parameter_data_size: u8,
        data_type: u8,
        command_class: SupportedCommandClasses,
        prefix: u8,
        minimum_valid_value: u32,
        maximum_valid_value: u32,
        default_value: u32,
        description: String,
    },
    DeviceLabel {
        device_label: String,
    },
    DeviceInfo {
        protocol_version: String,
        model_id: u16,
        product_category: ProductCategory,
        software_version_id: u32,
        footprint: u16,
        current_personality: u8,
        personality_count: u8,
        start_address: u16,
        sub_device_count: u16,
        sensor_count: u8,
    },
    SoftwareVersionLabel {
        software_version_label: String,
    },
    SupportedParameters {
        standard_parameters: Vec<ParameterId>,
        manufacturer_specific_parameters: HashMap<u16, ManufacturerSpecificParameter>,
    },
    SensorDefinition {
        sensor: Sensor,
    },
    IdentifyDevice {
        is_identifying: bool,
    },
    ManufacturerLabel {
        manufacturer_label: String,
    },
    FactoryDefaults {
        factory_default: bool,
    },
    DeviceModelDescription {
        device_model_description: String,
    },
    ProductDetailIdList {
        product_detail_id_list: Vec<u16>,
    },
    DmxPersonality {
        current_personality: u8,
        personality_count: u8,
    },
    DmxPersonalityDescription {
        id: u8,
        dmx_slots_required: u16,
        description: String,
    },
    SlotInfo {
        dmx_slots: Vec<DmxSlot>,
    },
    DeviceHours {
        device_hours: u32,
    },
    LampHours {
        lamp_hours: u32,
    },
    LampStrikes {
        lamp_strikes: u32,
    },
    LampState {
        lamp_state: LampState,
    },
    LampOnMode {
        lamp_on_mode: LampOnMode,
    },
    DevicePowerCycles {
        power_cycle_count: u32,
    },
    DisplayInvert {
        display_invert_mode: DisplayInvertMode,
    },
    Curve {
        current_curve: u8,
        curve_count: u8,
    },
    CurveDescription {
        id: u8,
        description: String,
    },
    ModulationFrequency {
        current_modulation_frequency: u8,
        modulation_frequency_count: u8,
    },
    ModulationFrequencyDescription {
        id: u8,
        frequency: u32,
        description: String,
    },
    DimmerInfo {
        minimum_level_lower_limit: u16,
        minimum_level_upper_limit: u16,
        maximum_level_lower_limit: u16,
        maximum_level_upper_limit: u16,
        num_of_supported_curves: u8,
        levels_resolution: u8,
        minimum_levels_split_levels_supports: u8,
    },
    MinimumLevel {
        minimum_level_increasing: u16,
        minimum_level_decreasing: u16,
        on_below_minimum: u8, // TODO could be bool
    },
    MaximumLevel {
        maximum_level: u16,
    },
    OutputResponseTime {
        current_output_response_time: u8,
        output_response_time_count: u8,
    },
    OutputResponseTimeDescription {
        id: u8,
        description: String,
    },
    PowerState {
        power_state: PowerState,
    },
    PerformSelfTest {
        is_active: bool,
    },
    SelfTestDescription {
        self_test_id: u8,
        description: String,
    },
    PresetPlayback {
        mode: u16,
        level: u8,
    },
}

#[derive(Debug)]
pub struct GetResponse {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub response_type: ResponseType,
    pub message_count: u8,
    pub sub_device: u16,
    pub command_class: CommandClass,
    pub parameter_id: ParameterId,
    pub parameter_data: Option<GetResponseParameterData>,
}

#[derive(Debug)]
pub struct DiscoveryResponse {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub response_type: ResponseType,
    pub message_count: u8,
    pub sub_device: u16,
    pub command_class: CommandClass,
    pub parameter_id: ParameterId,
    pub parameter_data: Option<DiscoveryResponseParameterData>,
}

#[derive(Debug)]
pub enum RdmResponseMessage {
    DiscoveryUniqueBranchResponse(DeviceUID),
    DiscoveryResponse(DiscoveryResponse),
    GetResponse(GetResponse),
    // SetCommand(SetCommandMessage)
}

#[derive(Copy, Clone, Default)]
pub struct RdmCodec;

impl RdmCodec {
    const SC_RDM: u8 = 0xcc;
    const SC_SUB_MESSAGE: u8 = 0x01;
    const FRAME_HEADER_FOOTER_SIZE: usize = 4;

    pub fn get_request_parameter_data_to_bytes(
        parameter_data: GetRequestParameterData,
    ) -> BytesMut {
        let mut bytes = BytesMut::new();

        match parameter_data {
            GetRequestParameterData::ParameterDescription { parameter_id } => {
                bytes.put_u16(parameter_id)
            }
            GetRequestParameterData::SensorDefinition { sensor_id } => bytes.put_u8(sensor_id),
            GetRequestParameterData::DmxPersonalityDescription { personality } => {
                bytes.put_u8(personality)
            }
            GetRequestParameterData::CurveDescription { curve } => bytes.put_u8(curve),
            GetRequestParameterData::ModulationFrequencyDescription {
                modulation_frequency,
            } => bytes.put_u8(modulation_frequency),
            GetRequestParameterData::OutputResponseTimeDescription {
                output_response_time,
            } => bytes.put_u8(output_response_time),
            GetRequestParameterData::SelfTestDescription { self_test_id } => {
                bytes.put_u8(self_test_id)
            }
        }

        bytes
    }

    pub fn get_response_bytes_to_parameter_data(
        parameter_id: StandardParameterId,
        bytes: Bytes,
    ) -> GetResponseParameterData {
        match parameter_id {
            StandardParameterId::ProxiedDeviceCount => {
                GetResponseParameterData::ProxiedDeviceCount {
                    device_count: u16::from_be_bytes(bytes[..=1].try_into().unwrap()),
                    list_change: bytes[2] != 0,
                }
            }
            StandardParameterId::ProxiedDevices => GetResponseParameterData::ProxiedDevices {
                device_uids: bytes.chunks(6).map(DeviceUID::from).collect(),
            },
            StandardParameterId::ParameterDescription => {
                GetResponseParameterData::ParameterDescription {
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
            StandardParameterId::DeviceLabel => GetResponseParameterData::DeviceLabel {
                device_label: String::from_utf8_lossy(&bytes)
                    .trim_end_matches("\0")
                    .to_string(),
            },
            StandardParameterId::DeviceInfo => GetResponseParameterData::DeviceInfo {
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
            },
            StandardParameterId::SoftwareVersionLabel => {
                GetResponseParameterData::SoftwareVersionLabel {
                    software_version_label: String::from_utf8_lossy(&bytes)
                        .trim_end_matches("\0")
                        .to_string(),
                }
            }
            StandardParameterId::SupportedParameters => {
                let parameters = bytes
                    .chunks(2)
                    .map(|chunk| u16::from_be_bytes(chunk.try_into().unwrap()));
                GetResponseParameterData::SupportedParameters {
                    standard_parameters: parameters
                        .clone()
                        .filter(|parameter_id| {
                            // TODO consider if we should filter parameters here or before we add to the queue
                            let parameter_id = *parameter_id;
                            parameter_id >= 0x0060_u16 && parameter_id < 0x8000_u16
                        })
                        .map(|pid| ParameterId::StandardParameter(StandardParameterId::from(pid)))
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
            StandardParameterId::SensorDefinition => GetResponseParameterData::SensorDefinition {
                sensor: Sensor {
                    id: bytes[0],
                    kind: bytes[1],
                    unit: bytes[2],
                    prefix: bytes[3],
                    range_minimum_value: u16::from_be_bytes(bytes[4..=5].try_into().unwrap()),
                    range_maximum_value: u16::from_be_bytes(bytes[6..=7].try_into().unwrap()),
                    normal_minimum_value: u16::from_be_bytes(bytes[8..=9].try_into().unwrap()),
                    normal_maximum_value: u16::from_be_bytes(bytes[10..=11].try_into().unwrap()),
                    recorded_value_support: bytes[12],
                    description: String::from_utf8_lossy(&bytes[13..])
                        .trim_end_matches("\0")
                        .to_string(),
                },
            },
            StandardParameterId::IdentifyDevice => GetResponseParameterData::IdentifyDevice {
                is_identifying: bytes[0] != 0,
            },
            StandardParameterId::ManufacturerLabel => GetResponseParameterData::ManufacturerLabel {
                manufacturer_label: String::from_utf8_lossy(&bytes)
                    .trim_end_matches("\0")
                    .to_string(),
            },
            StandardParameterId::FactoryDefaults => GetResponseParameterData::FactoryDefaults {
                factory_default: bytes[0] != 0,
            },
            StandardParameterId::DeviceModelDescription => {
                GetResponseParameterData::DeviceModelDescription {
                    device_model_description: String::from_utf8_lossy(&bytes)
                        .trim_end_matches("\0")
                        .to_string(),
                }
            }
            StandardParameterId::ProductDetailIdList => {
                GetResponseParameterData::ProductDetailIdList {
                    product_detail_id_list: bytes
                        .chunks(2)
                        .map(|id| u16::from_be_bytes(id.try_into().unwrap()))
                        .collect(),
                }
            }
            StandardParameterId::DmxPersonality => GetResponseParameterData::DmxPersonality {
                current_personality: bytes[0],
                personality_count: bytes[1],
            },
            StandardParameterId::DmxPersonalityDescription => {
                GetResponseParameterData::DmxPersonalityDescription {
                    id: bytes[0],
                    dmx_slots_required: u16::from_be_bytes(bytes[1..=2].try_into().unwrap()),
                    description: String::from_utf8_lossy(&bytes[3..])
                        .trim_end_matches("\0")
                        .to_string(),
                }
            }
            StandardParameterId::SlotInfo => GetResponseParameterData::SlotInfo {
                dmx_slots: bytes.chunks(5).map(DmxSlot::from).collect(),
            },
            StandardParameterId::DeviceHours => GetResponseParameterData::DeviceHours {
                device_hours: u32::from_be_bytes(bytes[0..=3].try_into().unwrap()),
            },
            StandardParameterId::LampHours => GetResponseParameterData::LampHours {
                lamp_hours: u32::from_be_bytes(bytes[0..=3].try_into().unwrap()),
            },
            StandardParameterId::LampStrikes => GetResponseParameterData::LampStrikes {
                lamp_strikes: u32::from_be_bytes(bytes[0..=3].try_into().unwrap()),
            },
            StandardParameterId::LampState => GetResponseParameterData::LampState {
                lamp_state: LampState::from(bytes[0]),
            },
            StandardParameterId::LampOnMode => GetResponseParameterData::LampOnMode {
                lamp_on_mode: LampOnMode::from(bytes[0]),
            },
            StandardParameterId::DevicePowerCycles => GetResponseParameterData::DevicePowerCycles {
                power_cycle_count: u32::from_be_bytes(bytes[0..=3].try_into().unwrap()),
            },
            StandardParameterId::DisplayInvert => GetResponseParameterData::DisplayInvert {
                display_invert_mode: DisplayInvertMode::from(bytes[0]),
            },
            StandardParameterId::Curve => GetResponseParameterData::Curve {
                current_curve: bytes[0],
                curve_count: bytes[1],
            },
            StandardParameterId::CurveDescription => GetResponseParameterData::CurveDescription {
                id: bytes[0],
                description: String::from_utf8_lossy(&bytes[1..])
                    .trim_end_matches("\0")
                    .to_string(),
            },
            StandardParameterId::ModulationFrequency => {
                GetResponseParameterData::ModulationFrequency {
                    current_modulation_frequency: bytes[0],
                    modulation_frequency_count: bytes[1],
                }
            }
            StandardParameterId::ModulationFrequencyDescription => {
                GetResponseParameterData::ModulationFrequencyDescription {
                    id: bytes[0],
                    frequency: u32::from_be_bytes(bytes[1..=4].try_into().unwrap()),
                    description: String::from_utf8_lossy(&bytes[5..])
                        .trim_end_matches("\0")
                        .to_string(),
                }
            }
            StandardParameterId::DimmerInfo => GetResponseParameterData::DimmerInfo {
                minimum_level_lower_limit: u16::from_be_bytes(bytes[0..=1].try_into().unwrap()),
                minimum_level_upper_limit: u16::from_be_bytes(bytes[2..=3].try_into().unwrap()),
                maximum_level_lower_limit: u16::from_be_bytes(bytes[4..=5].try_into().unwrap()),
                maximum_level_upper_limit: u16::from_be_bytes(bytes[6..=7].try_into().unwrap()),
                num_of_supported_curves: bytes[8],
                levels_resolution: bytes[9],
                minimum_levels_split_levels_supports: bytes[10], // TODO could be bool
            },
            StandardParameterId::MinimumLevel => GetResponseParameterData::MinimumLevel {
                minimum_level_increasing: u16::from_be_bytes(bytes[0..=1].try_into().unwrap()),
                minimum_level_decreasing: u16::from_be_bytes(bytes[2..=3].try_into().unwrap()),
                on_below_minimum: bytes[4],
            },
            StandardParameterId::MaximumLevel => GetResponseParameterData::MaximumLevel {
                maximum_level: u16::from_be_bytes(bytes[0..=1].try_into().unwrap()),
            },
            StandardParameterId::OutputResponseTime => {
                GetResponseParameterData::OutputResponseTime {
                    current_output_response_time: bytes[0],
                    output_response_time_count: bytes[1],
                }
            }
            StandardParameterId::OutputResponseTimeDescription => {
                GetResponseParameterData::OutputResponseTimeDescription {
                    id: bytes[0],
                    description: String::from_utf8_lossy(&bytes[1..])
                        .trim_end_matches("\0")
                        .to_string(),
                }
            }
            StandardParameterId::PowerState => GetResponseParameterData::PowerState {
                power_state: PowerState::from(bytes[0]),
            },
            StandardParameterId::PerformSelfTest => GetResponseParameterData::PerformSelfTest {
                is_active: bytes[0] != 0,
            },
            StandardParameterId::SelfTestDescription => {
                GetResponseParameterData::SelfTestDescription {
                    self_test_id: bytes[0],
                    description: String::from_utf8_lossy(&bytes[1..])
                        .trim_end_matches("\0")
                        .to_string(),
                }
            }
            StandardParameterId::PresetPlayback => GetResponseParameterData::PresetPlayback {
                mode: u16::from_be_bytes(bytes[..=1].try_into().unwrap()),
                level: bytes[2],
            },
            _ => panic!("unsupported parameter"),
        }
    }

    pub fn discovery_response_bytes_to_parameter_data(
        parameter_id: StandardParameterId,
        bytes: Bytes,
    ) -> DiscoveryResponseParameterData {
        match parameter_id {
            StandardParameterId::DiscMute => {
                // TODO could deduplicate the code here
                let binding_uid = if bytes.len() > 2 {
                    Some(DeviceUID::from(&bytes[2..]))
                } else {
                    None
                };
                DiscoveryResponseParameterData::DiscMute {
                    control_field: u16::from_be_bytes(bytes[..=1].try_into().unwrap()),
                    binding_uid,
                }
            }
            StandardParameterId::DiscUnMute => {
                let binding_uid = if bytes.len() > 2 {
                    Some(DeviceUID::from(&bytes[2..]))
                } else {
                    None
                };
                DiscoveryResponseParameterData::DiscMute {
                    control_field: u16::from_be_bytes(bytes[..=1].try_into().unwrap()),
                    binding_uid,
                }
            }
            _ => panic!("unsupported parameter"),
        }
    }

    pub fn is_checksum_valid(packet: &Vec<u8>) -> bool {
        let packet_checksum =
            u16::from_be_bytes(packet[packet.len() - 2..packet.len()].try_into().unwrap());

        let calculated_checksum = bsd_16_crc(&packet[..packet.len() - 2].try_into().unwrap());

        packet_checksum == calculated_checksum
    }
}

impl Encoder<RdmRequestMessage> for RdmCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: RdmRequestMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let (
            command_class,
            destination_uid,
            source_uid,
            transaction_number,
            port_id,
            sub_device,
            parameter_id,
            parameter_data,
        ) = match item {
            RdmRequestMessage::DiscoveryRequest(message) => (
                CommandClass::DiscoveryCommand,
                message.destination_uid,
                message.source_uid,
                message.transaction_number,
                message.port_id,
                message.sub_device,
                message.parameter_id,
                None,
            ),
            RdmRequestMessage::GetRequest(message) => (
                CommandClass::GetCommand,
                message.destination_uid,
                message.source_uid,
                message.transaction_number,
                message.port_id,
                message.sub_device,
                message.parameter_id,
                message
                    .parameter_data
                    .map(Self::get_request_parameter_data_to_bytes),
            ),
            _ => panic!("Unknown RdmRequestMessage type"),
        };

        let parameter_data_length = if let Some(parameter_data) = parameter_data.clone() {
            parameter_data.len()
        } else {
            0
        };

        dst.reserve(parameter_data_length + 26); // TODO double check length

        dst.put_u8(Self::SC_RDM); // Start Code
        dst.put_u8(Self::SC_SUB_MESSAGE); // Sub Start Code
        dst.put_u8(parameter_data_length as u8 + 24); // Message Length: Range 24 to 255 excluding the checksum
        dst.put_u16(destination_uid.manufacturer_id);
        dst.put_u32(destination_uid.device_id);
        dst.put_u16(source_uid.manufacturer_id); // Transaction Number; // Port Id / Response Type; // Message Count; // Sub Device;
        dst.put_u32(source_uid.device_id); // Transaction Number; // Port Id / Response Type; // Message Count; // Sub Device;

        match parameter_id {
            ParameterId::StandardParameter(parameter_id) => dst.put_u16(parameter_id as u16),
            ParameterId::ManufacturerSpecificParameter(parameter_id) => dst.put_u16(parameter_id),
        }

        dst.put_u8(parameter_data_length as u8);

        if let Some(bytes) = parameter_data {
            dst.put(bytes);
        }

        dst.put_u16(bsd_16_crc_bytes_mut(&mut dst.clone()));
        Ok(())
    }
}

impl Decoder for RdmCodec {
    type Item = RdmResponseMessage;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let len = src.len();

        let rdm_packet_type =
            PacketType::try_from(u16::from_be_bytes(src[0..=1].try_into().unwrap())).unwrap();

        let frame = match rdm_packet_type {
            PacketType::DiscoveryUniqueBranchResponse => {
                let euid_start_index = src.iter().position(|x| *x == 0xaa).unwrap();

                let euid = Vec::from(&src[(euid_start_index + 1)..=(euid_start_index + 12)]);

                let ecs = Vec::from(&src[(euid_start_index + 13)..=(euid_start_index + 16)]);

                let decoded_checksum = bsd_16_crc(&euid);

                let checksum = u16::from_be_bytes([ecs[0] & ecs[1], ecs[2] & ecs[3]]);

                if checksum != decoded_checksum {
                    return Err(anyhow!("decoded checksum incorrect",));
                    // return Err(io::Error::new(
                    //     io::ErrorKind::Other,
                    //     "decoded checksum incorrect",
                    // ));
                }

                let manufacturer_id = u16::from_be_bytes([euid[0] & euid[1], euid[2] & euid[3]]);

                let device_id = u32::from_be_bytes([
                    euid[4] & euid[5],
                    euid[6] & euid[7],
                    euid[8] & euid[9],
                    euid[10] & euid[11],
                ]);

                RdmResponseMessage::DiscoveryUniqueBranchResponse(DeviceUID::new(
                    manufacturer_id,
                    device_id,
                ))
            }
            PacketType::RdmResponse => {
                // if let Some(start_byte) = src.iter().position(|b| *b == Self::SC_RDM) {
                // We can safely ignore any bytes before and including the START_BYTE
                // let _ = src.split_to(start_byte);

                // TODO should be checking if checksum is valid

                if len < Self::FRAME_HEADER_FOOTER_SIZE {
                    return Ok(None);
                }

                let packet_length = src[2] as usize;

                // if len < Self::FRAME_HEADER_FOOTER_SIZE + 3 {
                //     return Ok(None);
                // }

                let frame = src
                    .split_to(len)
                    // .split_to(packet_length + Self::FRAME_HEADER_FOOTER_SIZE)
                    .freeze();

                let command_class = CommandClass::try_from(frame[20]).unwrap();
                let destination_uid = DeviceUID::from(&frame[3..=8]);
                let source_uid = DeviceUID::from(&frame[9..=14]);
                let transaction_number = frame[15];
                let response_type = ResponseType::try_from(frame[16]).unwrap();
                let message_count = frame[17];
                let sub_device = u16::from_be_bytes(frame[18..=19].try_into().unwrap());
                let parameter_id = StandardParameterId::from(&frame[21..=22]);

                let parameter_data_length = frame[23];
                let parameter_data: Option<Bytes> = if parameter_data_length > 0 {
                    Some(frame.slice(24..frame.len() - 2))
                } else {
                    None
                };

                let frame = match command_class {
                    CommandClass::GetCommandResponse => {
                        RdmResponseMessage::GetResponse(GetResponse {
                            destination_uid,
                            source_uid,
                            transaction_number,
                            response_type,
                            message_count,
                            command_class,
                            sub_device,
                            parameter_id: ParameterId::StandardParameter(parameter_id),
                            parameter_data: parameter_data.map(|parameter_data| {
                                Self::get_response_bytes_to_parameter_data(
                                    parameter_id,
                                    parameter_data,
                                )
                            }),
                        })
                    }
                    CommandClass::DiscoveryCommandResponse => {
                        RdmResponseMessage::DiscoveryResponse(DiscoveryResponse {
                            destination_uid,
                            source_uid,
                            transaction_number,
                            response_type,
                            message_count,
                            command_class,
                            sub_device,
                            parameter_id: ParameterId::StandardParameter(parameter_id),
                            parameter_data: parameter_data.map(|parameter_data| {
                                Self::discovery_response_bytes_to_parameter_data(
                                    parameter_id,
                                    parameter_data,
                                )
                            }),
                        })
                    }
                    _ => todo!("Unknown CommandClass"),
                };

                frame

                // } else {
                //     println!("packet length");
                //     // TODO might need to return Err() here
                //     return Ok(None);
                // }
            }
        };

        Ok(Some(frame))
    }
}