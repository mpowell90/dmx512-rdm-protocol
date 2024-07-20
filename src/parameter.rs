use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParameterError {
    #[error("Unsupported parameter id: {0}")]
    UnsupportedParameter(u16),
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
    type Error = ParameterError;

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
            _ => Err(ParameterError::UnsupportedParameter(value)),
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

#[derive(Clone, Debug, Default)]
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
