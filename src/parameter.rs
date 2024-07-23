use thiserror::Error;

use crate::ProtocolError;

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
    type Error = ProtocolError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x0001 => Ok(Self::DiscUniqueBranch),
            0x0002 => Ok(Self::DiscMute),
            0x0003 => Ok(Self::DiscUnMute),
            0x0010 => Ok(Self::ProxiedDevices),
            0x0011 => Ok(Self::ProxiedDeviceCount),
            0x0015 => Ok(Self::CommsStatus),
            0x0020 => Ok(Self::QueuedMessage),
            0x0030 => Ok(Self::StatusMessages),
            0x0031 => Ok(Self::StatusIdDescription),
            0x0032 => Ok(Self::ClearStatusId),
            0x0033 => Ok(Self::SubDeviceStatusReportThreshold),
            0x0050 => Ok(Self::SupportedParameters),
            0x0051 => Ok(Self::ParameterDescription),
            0x0060 => Ok(Self::DeviceInfo),
            0x0070 => Ok(Self::ProductDetailIdList),
            0x0080 => Ok(Self::DeviceModelDescription),
            0x0081 => Ok(Self::ManufacturerLabel),
            0x0082 => Ok(Self::DeviceLabel),
            0x0090 => Ok(Self::FactoryDefaults),
            0x00a0 => Ok(Self::LanguageCapabilities),
            0x00b0 => Ok(Self::Language),
            0x00c0 => Ok(Self::SoftwareVersionLabel),
            0x00c1 => Ok(Self::BootSoftwareVersionId),
            0x00c2 => Ok(Self::BootSoftwareVersionLabel),
            0x00e0 => Ok(Self::DmxPersonality),
            0x00e1 => Ok(Self::DmxPersonalityDescription),
            0x00f0 => Ok(Self::DmxStartAddress),
            0x0120 => Ok(Self::SlotInfo),
            0x0121 => Ok(Self::SlotDescription),
            0x0122 => Ok(Self::DefaultSlotValue),
            0x0200 => Ok(Self::SensorDefinition),
            0x0201 => Ok(Self::SensorValue),
            0x0202 => Ok(Self::RecordSensors),
            0x0340 => Ok(Self::DimmerInfo),
            0x0341 => Ok(Self::MinimumLevel),
            0x0342 => Ok(Self::MaximumLevel),
            0x0343 => Ok(Self::Curve),
            0x0344 => Ok(Self::CurveDescription),
            0x0345 => Ok(Self::OutputResponseTime),
            0x0346 => Ok(Self::OutputResponseTimeDescription),
            0x0347 => Ok(Self::ModulationFrequency),
            0x0348 => Ok(Self::ModulationFrequencyDescription),
            0x0400 => Ok(Self::DeviceHours),
            0x0401 => Ok(Self::LampHours),
            0x0402 => Ok(Self::LampStrikes),
            0x0403 => Ok(Self::LampState),
            0x0404 => Ok(Self::LampOnMode),
            0x0405 => Ok(Self::DevicePowerCycles),
            0x0500 => Ok(Self::DisplayInvert),
            0x0501 => Ok(Self::DisplayLevel),
            0x0600 => Ok(Self::PanInvert),
            0x0601 => Ok(Self::TiltInvert),
            0x0602 => Ok(Self::PanTiltSwap),
            0x0603 => Ok(Self::RealTimeClock),
            0x1000 => Ok(Self::IdentifyDevice),
            0x1001 => Ok(Self::ResetDevice),
            0x1010 => Ok(Self::PowerState),
            0x1020 => Ok(Self::PerformSelfTest),
            0x1021 => Ok(Self::SelfTestDescription),
            0x1030 => Ok(Self::CapturePreset),
            0x1031 => Ok(Self::PresetPlayback),
            _ => Err(ProtocolError::UnsupportedParameterId(value)),
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

#[derive(Debug, Error)]
pub enum SupportedCommandClassError {
    #[error("Invalid command class: {0}")]
    InvalidCommandClass(u8),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SupportedCommandClass {
    Get = 0x01,
    Set = 0x02,
    GetSet = 0x03,
}

impl TryFrom<u8> for SupportedCommandClass {
    type Error = SupportedCommandClassError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::Get),
            0x02 => Ok(Self::Set),
            0x03 => Ok(Self::GetSet),
            _ => Err(SupportedCommandClassError::InvalidCommandClass(value)),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ManufacturerSpecificParameter {
    pub parameter_id: u16,
    pub parameter_data_size: Option<u8>, // TODO use enum
    pub data_type: Option<u8>,           // TODO use enum
    pub command_class: Option<SupportedCommandClass>,
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

