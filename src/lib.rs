pub mod codec;
pub mod device;
pub mod sensor;

use crate::{
    device::{DeviceUID, DmxSlot},
    sensor::Sensor,
};
use bytes::BytesMut;
use std::{cmp::PartialEq, collections::HashMap};
use thiserror::Error;

pub const MIN_PACKET_LEN: usize = 26;

pub const SC_RDM: u8 = 0xcc;
pub const SC_SUB_MESSAGE: u8 = 0x01;

pub const BROADCAST_ALL_DEVICES_ID: u64 = 0xffffffffffff;
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
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

impl TryFrom<u16> for ParameterId {   
    type Error = String;
    
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x0001 => Ok(ParameterId::DiscUniqueBranch),
            0x0002 => Ok(ParameterId::DiscMute),
            0x0003 => Ok(ParameterId::DiscUnMute),
            0x0010 => Ok(ParameterId::ProxiedDevices),
            0x0011 => Ok(ParameterId::ProxiedDeviceCount),
            0x0015 => Ok(ParameterId::CommsStatus),
            0x0020 => Ok(ParameterId::QueuedMessage),
            0x0030 => Ok(ParameterId::StatusMessages),
            0x0031 => Ok(ParameterId::StatusIdDescription),
            0x0032 => Ok(ParameterId::ClearStatusId),
            0x0033 => Ok(ParameterId::SubDeviceStatusReportThreshold),
            0x0050 => Ok(ParameterId::SupportedParameters),
            0x0051 => Ok(ParameterId::ParameterDescription),
            0x0060 => Ok(ParameterId::DeviceInfo),
            0x0070 => Ok(ParameterId::ProductDetailIdList),
            0x0080 => Ok(ParameterId::DeviceModelDescription),
            0x0081 => Ok(ParameterId::ManufacturerLabel),
            0x0082 => Ok(ParameterId::DeviceLabel),
            0x0090 => Ok(ParameterId::FactoryDefaults),
            0x00a0 => Ok(ParameterId::LanguageCapabilities),
            0x00b0 => Ok(ParameterId::Language),
            0x00c0 => Ok(ParameterId::SoftwareVersionLabel),
            0x00c1 => Ok(ParameterId::BootSoftwareVersionId),
            0x00c2 => Ok(ParameterId::BootSoftwareVersionLabel),
            0x00e0 => Ok(ParameterId::DmxPersonality),
            0x00e1 => Ok(ParameterId::DmxPersonalityDescription),
            0x00f0 => Ok(ParameterId::DmxStartAddress),
            0x0120 => Ok(ParameterId::SlotInfo),
            0x0121 => Ok(ParameterId::SlotDescription),
            0x0122 => Ok(ParameterId::DefaultSlotValue),
            0x0200 => Ok(ParameterId::SensorDefinition),
            0x0201 => Ok(ParameterId::SensorValue),
            0x0202 => Ok(ParameterId::RecordSensors),
            0x0340 => Ok(ParameterId::DimmerInfo),
            0x0341 => Ok(ParameterId::MinimumLevel),
            0x0342 => Ok(ParameterId::MaximumLevel),
            0x0343 => Ok(ParameterId::Curve),
            0x0344 => Ok(ParameterId::CurveDescription),
            0x0345 => Ok(ParameterId::OutputResponseTime),
            0x0346 => Ok(ParameterId::OutputResponseTimeDescription),
            0x0347 => Ok(ParameterId::ModulationFrequency),
            0x0348 => Ok(ParameterId::ModulationFrequencyDescription),
            0x0400 => Ok(ParameterId::DeviceHours),
            0x0401 => Ok(ParameterId::LampHours),
            0x0402 => Ok(ParameterId::LampStrikes),
            0x0403 => Ok(ParameterId::LampState),
            0x0404 => Ok(ParameterId::LampOnMode),
            0x0405 => Ok(ParameterId::DevicePowerCycles),
            0x0500 => Ok(ParameterId::DisplayInvert),
            0x0501 => Ok(ParameterId::DisplayLevel),
            0x0600 => Ok(ParameterId::PanInvert),
            0x0601 => Ok(ParameterId::TiltInvert),
            0x0602 => Ok(ParameterId::PanTiltSwap),
            0x0603 => Ok(ParameterId::RealTimeClock),
            0x1000 => Ok(ParameterId::IdentifyDevice),
            0x1001 => Ok(ParameterId::ResetDevice),
            0x1010 => Ok(ParameterId::PowerState),
            0x1020 => Ok(ParameterId::PerformSelfTest),
            0x1021 => Ok(ParameterId::SelfTestDescription),
            0x1030 => Ok(ParameterId::CapturePreset),
            0x1031 => Ok(ParameterId::PresetPlayback),
            _ => Err(format!("Unsupported parameter id: 0x{:04X?}", value)),
        }
    }
}

pub const REQUIRED_PARAMETERS: [ParameterId; 4] = [
    ParameterId::DeviceInfo,
    ParameterId::SupportedParameters,
    ParameterId::SoftwareVersionLabel,
    ParameterId::IdentifyDevice,
];

pub const GET_PARAMETERS: [ParameterId; 46] = [
    ParameterId::ProxiedDevices,
    ParameterId::ProxiedDeviceCount,
    ParameterId::CommsStatus,
    ParameterId::QueuedMessage,
    ParameterId::StatusMessages,
    ParameterId::StatusIdDescription,
    ParameterId::SubDeviceStatusReportThreshold,
    ParameterId::SupportedParameters,
    ParameterId::ParameterDescription,
    ParameterId::DeviceInfo,
    ParameterId::ProductDetailIdList,
    ParameterId::DeviceModelDescription,
    ParameterId::ManufacturerLabel,
    ParameterId::DeviceLabel,
    ParameterId::FactoryDefaults,
    ParameterId::LanguageCapabilities,
    ParameterId::Language,
    ParameterId::SoftwareVersionLabel,
    ParameterId::BootSoftwareVersionId,
    ParameterId::BootSoftwareVersionLabel,
    ParameterId::DmxPersonality,
    // ParameterId::DmxPersonalityDescription,
    ParameterId::DmxStartAddress,
    ParameterId::SlotInfo,
    // ParameterId::SlotDescription,
    ParameterId::DefaultSlotValue,
    // ParameterId::SensorDefinition,
    // ParameterId::SensorValue,
    ParameterId::DimmerInfo,
    ParameterId::MinimumLevel,
    ParameterId::MaximumLevel,
    ParameterId::Curve,
    // ParameterId::CurveDescription,
    ParameterId::OutputResponseTime,
    // ParameterId::OutputResponseTimeDescription,
    ParameterId::ModulationFrequency,
    // ParameterId::ModulationFrequencyDescription,
    ParameterId::DeviceHours,
    ParameterId::LampHours,
    ParameterId::LampStrikes,
    ParameterId::LampState,
    ParameterId::LampOnMode,
    ParameterId::DevicePowerCycles,
    ParameterId::DisplayInvert,
    ParameterId::DisplayLevel,
    ParameterId::PanInvert,
    ParameterId::TiltInvert,
    ParameterId::PanTiltSwap,
    ParameterId::RealTimeClock,
    ParameterId::IdentifyDevice,
    ParameterId::PowerState,
    ParameterId::PerformSelfTest,
    // ParameterId::SelfTestDescription,
    ParameterId::PresetPlayback,
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

pub fn bsd_16_crc(packet: &[u8]) -> u16 {
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
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

pub fn is_checksum_valid(packet: &[u8]) -> bool {
    let packet_checksum =
        u16::from_be_bytes(packet[packet.len() - 2..packet.len()].try_into().unwrap());

    let calculated_checksum = bsd_16_crc(&packet[..packet.len() - 2]);

    packet_checksum == calculated_checksum
}

pub enum DiscoveryRequestParameterData {
    DiscUniqueBranch {
        lower_bound_uid: DeviceUID,
        upper_bound_uid: DeviceUID,
    },
}

pub struct DiscoveryRequest {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub port_id: u8,
    pub sub_device_id: u16,
    pub parameter_id: u16,
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
    SlotDescription { slot_id: u16 },
}

pub struct GetRequest {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub port_id: u8,
    pub sub_device_id: u16,
    pub parameter_id: u16,
    pub parameter_data: Option<GetRequestParameterData>,
}

pub enum SetRequestParameterData {
    DeviceLabel { device_label: String },
    DmxPersonality { personality_id: u8 },
    DmxStartAddress { dmx_start_address: u16 },
    Curve { curve_id: u8 },
    ModulationFrequency { modulation_frequency_id: u8 },
    OutputResponseTime { output_response_time_id: u8 },
    IdentifyDevice { identify: bool },
}

pub struct SetRequest {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub port_id: u8,
    pub sub_device_id: u16,
    pub parameter_id: u16,
    pub parameter_data: Option<SetRequestParameterData>,
}

pub enum RdmRequestMessage {
    DiscoveryRequest(DiscoveryRequest),
    GetRequest(GetRequest),
    SetRequest(SetRequest),
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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
    DmxStartAddress {
        dmx_start_address: u16,
    },
    SlotInfo {
        dmx_slots: Option<Vec<DmxSlot>>,
    },
    SlotDescription {
        slot_id: u16,
        description: String,
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

#[derive(Clone, Debug)]
pub enum SetResponseParameterData {
    DeviceLabel,
    DmxPersonality,
    DmxStartAddress,
    Curve,
    ModulationFrequency,
    OutputResponseTime,
    IdentifyDevice,
}

#[derive(Clone, Debug)]
pub struct GetResponse {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub response_type: ResponseType,
    pub message_count: u8,
    pub sub_device_id: u16,
    pub command_class: CommandClass,
    pub parameter_id: ParameterId,
    pub parameter_data: Option<GetResponseParameterData>,
}

#[derive(Clone, Debug)]
pub struct SetResponse {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub response_type: ResponseType,
    pub message_count: u8,
    pub sub_device_id: u16,
    pub command_class: CommandClass,
    pub parameter_id: ParameterId,
    pub parameter_data: Option<SetResponseParameterData>,
}

#[derive(Clone, Debug)]
pub struct DiscoveryResponse {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub response_type: ResponseType,
    pub message_count: u8,
    pub sub_device_id: u16,
    pub command_class: CommandClass,
    pub parameter_id: ParameterId,
    pub parameter_data: Option<DiscoveryResponseParameterData>,
}

#[derive(Clone, Debug)]
pub enum RdmResponseMessage {
    DiscoveryUniqueBranchResponse(DeviceUID),
    DiscoveryResponse(DiscoveryResponse),
    GetResponse(GetResponse),
    SetResponse(SetResponse),
}
