use super::ProtocolError;
use thiserror::Error;

// TODO add remaining parameter ids
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq)]
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

#[derive(Copy, Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, Default, PartialEq)]
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StatusType {
    None = 0x00,
    GetLastMessage = 0x01,
    Advisory = 0x02,
    Warning = 0x03,
    Error = 0x04,
    AdvisoryCleared = 0x12,
    WarningCleared = 0x13,
    ErrorCleared = 0x14,
}

impl TryFrom<u8> for StatusType {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, ProtocolError> {
        match value {
            0x00 => Ok(Self::None),
            0x01 => Ok(Self::GetLastMessage),
            0x02 => Ok(Self::Advisory),
            0x03 => Ok(Self::Warning),
            0x04 => Ok(Self::Error),
            0x12 => Ok(Self::AdvisoryCleared),
            0x13 => Ok(Self::WarningCleared),
            0x14 => Ok(Self::ErrorCleared),
            _ => Err(ProtocolError::InvalidStatusType(value)),
        }
    }
}

// Product Categories - Page 105 RDM Spec
#[derive(Copy, Clone, Debug, PartialEq)]
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

impl TryFrom<u16> for ProductCategory {
    type Error = ProtocolError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x0000 => Ok(Self::NotDeclared),
            0x0100 => Ok(Self::Fixture),
            0x0101 => Ok(Self::FixtureFixed),
            0x0102 => Ok(Self::FixtureMovingYoke),
            0x0103 => Ok(Self::FixtureMovingMirror),
            0x01ff => Ok(Self::FixtureOther),
            0x0200 => Ok(Self::FixtureAccessory),
            0x0201 => Ok(Self::FixtureAccessoryColor),
            0x0202 => Ok(Self::FixtureAccessoryYoke),
            0x0203 => Ok(Self::FixtureAccessoryMirror),
            0x0204 => Ok(Self::FixtureAccessoryEffect),
            0x0205 => Ok(Self::FixtureAccessoryBeam),
            0x02ff => Ok(Self::AccessoryOther),
            0x0300 => Ok(Self::Projector),
            0x0301 => Ok(Self::ProjectorFixed),
            0x0302 => Ok(Self::ProjectorMovingYoke),
            0x0303 => Ok(Self::ProjectorMovingMirror),
            0x03ff => Ok(Self::ProjectorOther),
            0x0400 => Ok(Self::Atmospheric),
            0x0401 => Ok(Self::AtmosphericEffect),
            0x0402 => Ok(Self::AtmosphericPyro),
            0x04ff => Ok(Self::AtmosphericOther),
            0x0500 => Ok(Self::Dimmer),
            0x0501 => Ok(Self::DimmerACIncandescent),
            0x0502 => Ok(Self::DimmerACFlourescent),
            0x0503 => Ok(Self::DimmerACColdCathode),
            0x0504 => Ok(Self::DimmerACNonDimModule),
            0x0505 => Ok(Self::DimmerACLowVoltage),
            0x0506 => Ok(Self::DimmerControllableAC),
            0x0507 => Ok(Self::DimmerDCLevelOutput),
            0x0508 => Ok(Self::DimmerDCPWMOutput),
            0x0509 => Ok(Self::DimmerSpecialisedLED),
            0x05ff => Ok(Self::DimmerOther),
            0x0600 => Ok(Self::Power),
            0x0601 => Ok(Self::PowerControl),
            0x0602 => Ok(Self::PowerSource),
            0x06ff => Ok(Self::PowerOther),
            0x0700 => Ok(Self::Scenic),
            0x0701 => Ok(Self::ScenicDrive),
            0x07ff => Ok(Self::ScenicOther),
            0x0800 => Ok(Self::Data),
            0x0801 => Ok(Self::DataDistribution),
            0x0802 => Ok(Self::DataConversion),
            0x08ff => Ok(Self::DataOther),
            0x0900 => Ok(Self::AV),
            0x0901 => Ok(Self::AVAudio),
            0x0902 => Ok(Self::AVVideo),
            0x09ff => Ok(Self::AVOther),
            0x0a00 => Ok(Self::Monitor),
            0x0a01 => Ok(Self::MonitorACLinePower),
            0x0a02 => Ok(Self::MonitorDCPower),
            0x0a03 => Ok(Self::MonitorEnvironmental),
            0x0aff => Ok(Self::MonitorOther),
            0x7000 => Ok(Self::Control),
            0x7001 => Ok(Self::ControlController),
            0x7002 => Ok(Self::ControlBackupDevice),
            0x70ff => Ok(Self::ControlOther),
            0x7100 => Ok(Self::Test),
            0x7101 => Ok(Self::TestEquipment),
            0x71ff => Ok(Self::TestEquipmentOther),
            0x7fff => Ok(Self::Other),
            _ => Err(ProtocolError::InvalidProductCategory(value)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LampState {
    LampOff = 0x00,        // 0x00 = "Lamp Off",
    LampOn = 0x01,         // 0x01 = "Lamp On",
    LampStrike = 0x02,     // 0x02 = "Lamp Strike",
    LampStandby = 0x03,    // 0x03 = "Lamp Standby",
    LampNotPresent = 0x04, // 0x04 = "Lamp Not Present",
    LampError = 0x05,      // 0x05 = "Lamp Error",
}

impl TryFrom<u8> for LampState {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::LampOff),
            0x01 => Ok(Self::LampOn),
            0x02 => Ok(Self::LampStrike),
            0x03 => Ok(Self::LampStandby),
            0x04 => Ok(Self::LampNotPresent),
            0x05 => Ok(Self::LampError),
            _ => Err(ProtocolError::InvalidLampState(value)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LampOnMode {
    OffMode = 0x00,  // 0x00 = "Off Mode",
    DmxMode = 0x01,  // 0x01 = "DMX Mode",
    OnMode = 0x02,   // 0x02 = "On Mode",
    AfterCal = 0x03, // 0x03 = "After Cal",
}

impl TryFrom<u8> for LampOnMode {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::OffMode),
            0x01 => Ok(Self::DmxMode),
            0x02 => Ok(Self::OnMode),
            0x03 => Ok(Self::AfterCal),
            _ => Err(ProtocolError::InvalidLampOnMode(value)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PowerState {
    FullOff = 0x00,  // 0x00 = "Full Off",
    Shutdown = 0x01, // 0x01 = "Shutdown",
    Standby = 0x02,  // 0x02 = "Standby",
    Normal = 0xff,   // 0xff = "Normal",
}

impl TryFrom<u8> for PowerState {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::FullOff),
            0x01 => Ok(Self::Shutdown),
            0x02 => Ok(Self::Standby),
            0x03 => Ok(Self::Normal),
            _ => Err(ProtocolError::InvalidPowerState(value)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OnOffStates {
    Off = 0x00, // 0x00 = "Off",
    On = 0x01,  // 0x01 = "On",
}

impl TryFrom<u8> for OnOffStates {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Off),
            0x01 => Ok(Self::On),
            _ => Err(ProtocolError::InvalidOnOffStates(value)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum DisplayInvertMode {
    Off = 0x00,  // 0x00 = "Off",
    On = 0x01,   // 0x01 = "On",
    Auto = 0x02, // 0x02 = "Auto",
}

impl TryFrom<u8> for DisplayInvertMode {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Off),
            0x01 => Ok(Self::On),
            0x02 => Ok(Self::Auto),
            _ => Err(ProtocolError::InvalidDisplayInvertMode(value)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ResetDeviceMode {
    Warm = 0x01,
    Cold = 0xff,
}

impl TryFrom<u8> for ResetDeviceMode {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::Warm),
            0xff => Ok(Self::Cold),
            _ => Err(ProtocolError::InvalidResetDeviceMode(value)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PresetPlaybackMode {
    Off,
    All,
    Scene(u16),
}

impl From<u16> for PresetPlaybackMode {
    fn from(value: u16) -> Self {
        match value {
            0x0000 => Self::Off,
            0xffff => Self::All,
            value => Self::Scene(value),
        }
    }
}

impl From<PresetPlaybackMode> for u16 {
    fn from(value: PresetPlaybackMode) -> u16 {
        match value {
            PresetPlaybackMode::Off => 0x0000,
            PresetPlaybackMode::All => 0xffff,
            PresetPlaybackMode::Scene(value) => value,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FadeTimes {
    pub up_fade_time: u16,
    pub down_fade_time: u16,
    pub wait_time: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StatusMessage {
    pub sub_device_id: u16,
    pub status_type: StatusType,
    pub status_message_id: u16, // TODO reference appendix B for status message IDs
    pub data_value1: u16,
    pub data_value2: u16,
}

impl StatusMessage {
    pub fn new(
        sub_device_id: u16,
        status_type: StatusType,
        status_message_id: u16,
        data_value1: u16,
        data_value2: u16,
    ) -> Self {
        StatusMessage {
            sub_device_id,
            status_type,
            status_message_id,
            data_value1,
            data_value2,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SlotInfo {
    pub id: u16,
    pub kind: u8, // TODO use enum
    pub label_id: u16,
}

impl SlotInfo {
    pub fn new(id: u16, kind: u8, label_id: u16) -> Self {
        Self { id, kind, label_id }
    }
}

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DefaultSlotValue {
    pub id: u16,
    pub value: u8,
}

impl DefaultSlotValue {
    pub fn new(id: u16, value: u8) -> Self {
        Self { id, value }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SensorType {
    Temperature = 0x00,
    Voltage = 0x01,
    Current = 0x02,
    Frequency = 0x03,
    Resistance = 0x04,
    Power = 0x05,
    Mass = 0x06,
    Length = 0x07,
    Area = 0x08,
    Volume = 0x09,
    Density = 0x0a,
    Velocity = 0x0b,
    Acceleration = 0x0c,
    Force = 0x0d,
    Energy = 0x0e,
    Pressure = 0x0f,
    Time = 0x10,
    Angle = 0x11,
    PositionX = 0x12,
    PositionY = 0x13,
    PositionZ = 0x14,
    AngularVelocity = 0x15,
    LuminousIntensity = 0x16,
    LuminousFlux = 0x17,
    Illuminance = 0x18,
    ChrominanceRed = 0x19,
    ChrominanceGreen = 0x1a,
    ChrominanceBlue = 0x1b,
    Contacts = 0x1c,
    Memory = 0x1d,
    Items = 0x1e,
    Humidity = 0x1f,
    Counter16Bit = 0x20,
    Other = 0x7f,
}

impl TryFrom<u8> for SensorType {
    type Error = ProtocolError;
    fn try_from(value: u8) -> Result<Self, ProtocolError> {
        match value {
            0x00 => Ok(Self::Temperature),
            0x01 => Ok(Self::Voltage),
            0x02 => Ok(Self::Current),
            0x03 => Ok(Self::Frequency),
            0x04 => Ok(Self::Resistance),
            0x05 => Ok(Self::Power),
            0x06 => Ok(Self::Mass),
            0x07 => Ok(Self::Length),
            0x08 => Ok(Self::Area),
            0x09 => Ok(Self::Volume),
            0x0a => Ok(Self::Density),
            0x0b => Ok(Self::Velocity),
            0x0c => Ok(Self::Acceleration),
            0x0d => Ok(Self::Force),
            0x0e => Ok(Self::Energy),
            0x0f => Ok(Self::Pressure),
            0x10 => Ok(Self::Time),
            0x11 => Ok(Self::Angle),
            0x12 => Ok(Self::PositionX),
            0x13 => Ok(Self::PositionY),
            0x14 => Ok(Self::PositionZ),
            0x15 => Ok(Self::AngularVelocity),
            0x16 => Ok(Self::LuminousIntensity),
            0x17 => Ok(Self::LuminousFlux),
            0x18 => Ok(Self::Illuminance),
            0x19 => Ok(Self::ChrominanceRed),
            0x1a => Ok(Self::ChrominanceGreen),
            0x1b => Ok(Self::ChrominanceBlue),
            0x1c => Ok(Self::Contacts),
            0x1d => Ok(Self::Memory),
            0x1e => Ok(Self::Items),
            0x1f => Ok(Self::Humidity),
            0x20 => Ok(Self::Counter16Bit),
            0x7f => Ok(Self::Other),
            _ => Err(ProtocolError::InvalidSensorType(value)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SensorValue {
    pub sensor_id: u8,
    pub current_value: i16,
    pub lowest_detected_value: i16,
    pub highest_detected_value: i16,
    pub recorded_value: i16,
}

impl SensorValue {
    pub fn new(
        sensor_id: u8,
        current_value: i16,
        lowest_detected_value: i16,
        highest_detected_value: i16,
        recorded_value: i16
    ) -> Self {
        Self {
            sensor_id,
            current_value,
            lowest_detected_value,
            highest_detected_value,
            recorded_value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sensor {
    pub id: u8,
    pub kind: SensorType,
    pub unit: u8,
    pub prefix: u8,
    pub range_minimum_value: i16,
    pub range_maximum_value: i16,
    pub normal_minimum_value: i16,
    pub normal_maximum_value: i16,
    pub recorded_value_support: u8,
    pub description: String,
}
