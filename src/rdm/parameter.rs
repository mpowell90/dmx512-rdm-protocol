use super::ProtocolError;
use std::{ffi::CStr, fmt::Display};

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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ImplementedCommandClass {
    Get = 0x01,
    Set = 0x02,
    GetSet = 0x03,
}

impl TryFrom<u8> for ImplementedCommandClass {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::Get),
            0x02 => Ok(Self::Set),
            0x03 => Ok(Self::GetSet),
            _ => Err(ProtocolError::InvalidCommandClassImplementation(value)),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ManufacturerSpecificParameter {
    pub parameter_id: u16,
    pub parameter_data_size: Option<u8>, // TODO use enum
    pub data_type: Option<u8>,           // TODO use enum
    pub command_class: Option<ImplementedCommandClass>,
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
pub enum ParameterDataType {
    NotDefined,
    BitField,
    Ascii,
    UnsignedByte,
    SignedByte,
    UnsignedWord,
    SignedWord,
    UnsignedDWord,
    SignedDWord,
    ManufacturerSpecific(u8),
}

impl TryFrom<u8> for ParameterDataType {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, ProtocolError> {
        match value {
            0x00 => Ok(Self::NotDefined),
            0x01 => Ok(Self::BitField),
            0x02 => Ok(Self::Ascii),
            0x03 => Ok(Self::UnsignedByte),
            0x04 => Ok(Self::SignedByte),
            0x05 => Ok(Self::UnsignedWord),
            0x06 => Ok(Self::SignedWord),
            0x07 => Ok(Self::UnsignedDWord),
            0x08 => Ok(Self::SignedDWord),
            n if (0x80..=0xdf).contains(&n) => Ok(Self::ManufacturerSpecific(n)),
            _ => Err(ProtocolError::InvalidParameterDataType(value)),
        }
    }
}

pub enum ConvertedParameterValue {
    BitField(u8),
    Ascii(String),
    UnsignedByte(u8),
    SignedByte(i8),
    UnsignedWord(u16),
    SignedWord(i16),
    UnsignedDWord(u32),
    SignedDWord(i32),
    Raw([u8; 4]),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParameterDescription {
    pub parameter_id: u16,
    pub parameter_data_length: u8,
    pub data_type: ParameterDataType,
    pub command_class: ImplementedCommandClass,
    pub unit_type: SensorUnit,
    pub prefix: SensorUnitPrefix,
    pub raw_minimum_valid_value: [u8; 4],
    pub raw_maximum_valid_value: [u8; 4],
    pub raw_default_value: [u8; 4],
    pub description: String,
}

impl ParameterDescription {
    fn convert_parameter_value(
        parameter_data_type: ParameterDataType,
        value: [u8; 4],
    ) -> Result<ConvertedParameterValue, ProtocolError> {
        match parameter_data_type {
            ParameterDataType::BitField => Ok(ConvertedParameterValue::BitField(
                value[3],
            )),
            ParameterDataType::Ascii => Ok(ConvertedParameterValue::Ascii(
                CStr::from_bytes_with_nul(&value)?
                    .to_string_lossy()
                    .to_string(),
            )),
            ParameterDataType::UnsignedByte => Ok(ConvertedParameterValue::UnsignedByte(
                value[3],
            )),
            ParameterDataType::SignedByte => Ok(ConvertedParameterValue::SignedByte(
                value[3] as i8, // TODO test this
            )),
            ParameterDataType::UnsignedWord => Ok(ConvertedParameterValue::UnsignedWord(
                u16::from_be_bytes([value[2], value[3]]),
            )),
            ParameterDataType::SignedWord => Ok(ConvertedParameterValue::SignedWord(
                i16::from_be_bytes([value[2], value[3]]),
            )),
            ParameterDataType::UnsignedDWord => Ok(ConvertedParameterValue::UnsignedDWord(
                u32::from_be_bytes(value),
            )),
            ParameterDataType::SignedDWord => Ok(ConvertedParameterValue::SignedDWord(
                i32::from_be_bytes(value),
            )),
            ParameterDataType::NotDefined | ParameterDataType::ManufacturerSpecific(..) => {
                Ok(ConvertedParameterValue::Raw(value))
            }
        }
    }

    pub fn minimum_valid_value(&self) -> Result<ConvertedParameterValue, ProtocolError> {
        Self::convert_parameter_value(self.data_type, self.raw_minimum_valid_value)
    }
    pub fn maximum_valid_value(&self) -> Result<ConvertedParameterValue, ProtocolError> {
        Self::convert_parameter_value(self.data_type, self.raw_maximum_valid_value)
    }
    pub fn default_value(&self) -> Result<ConvertedParameterValue, ProtocolError> {
        Self::convert_parameter_value(self.data_type, self.raw_default_value)
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
    LampOff = 0x00,
    LampOn = 0x01,
    LampStrike = 0x02,
    LampStandby = 0x03,
    LampNotPresent = 0x04,
    LampError = 0x05,
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
    OffMode = 0x00,
    DmxMode = 0x01,
    OnMode = 0x02,
    AfterCal = 0x03,
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
    FullOff = 0x00,
    Shutdown = 0x01,
    Standby = 0x02,
    Normal = 0xff,
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
    Off = 0x00,
    On = 0x01,
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
    Off = 0x00,
    On = 0x01,
    Auto = 0x02,
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

#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum StatusMessageIdDefinition {
    CalibrationFailed = 0x0001,
    SensorNotFound = 0x0002,
    SensorAlwaysOn = 0x0003,
    LampDoused = 0x0011,
    LampStrike = 0x0012,
    OverTemperature = 0x0021,
    UnderTemperature = 0x0022,
    SensorOutOfRange = 0x0023,
    OverVoltagePhase = 0x0031,
    UnderVoltagePhase = 0x0032,
    OverCurrent = 0x0033,
    UnderCurrent = 0x0034,
    Phase = 0x0035,
    PhaseError = 0x0036,
    Amps = 0x0037,
    Volts = 0x0038,
    DimSlotOccupied = 0x0041,
    BreakerTrip = 0x0042,
    Watts = 0x0043,
    DimmerFailure = 0x0044,
    DimmerPanic = 0x0045,
    Ready = 0x0050,
    NotReady = 0x0051,
    LowFluid = 0x0052,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StatusMessage {
    pub sub_device_id: u16,
    pub status_type: StatusType,
    pub status_message_id: u16,
    pub data_value1: u16,
    pub data_value2: u16,
    pub description: Option<String>,
}

impl StatusMessage {
    pub fn new(
        sub_device_id: u16,
        status_type: StatusType,
        status_message_id: u16,
        data_value1: u16,
        data_value2: u16,
    ) -> Self {
        let description = if status_message_id < 0x8000 {
            match status_message_id {
                0x0001 => SlotIdDefinition::try_from(data_value1)
                    .ok()
                    .map(|slot_id| format!("{} failed calibration", slot_id)),
                0x0002 => SlotIdDefinition::try_from(data_value1)
                    .ok()
                    .map(|slot_id| format!("{} sensor not found", slot_id)),
                0x0003 => SlotIdDefinition::try_from(data_value1)
                    .ok()
                    .map(|slot_id| format!("{} sensor always on", slot_id)),
                0x0011 => Some("Lamp Doused".to_string()),
                0x0012 => Some("Lamp Strike".to_string()),
                0x0021 => Some(format!(
                    "Sensor {} over temp at {} degrees C",
                    data_value1, data_value2
                )),
                0x0022 => Some(format!(
                    "Sensor {} under temp at {} degrees C",
                    data_value1, data_value2
                )),
                0x0023 => Some(format!("Sensor {} out of range", data_value1)),
                0x0031 => Some(format!(
                    "Phase {} over voltage at {} V",
                    data_value1, data_value2
                )),
                0x0032 => Some(format!(
                    "Phase {} under voltage at {} V",
                    data_value1, data_value2
                )),
                0x0033 => Some(format!(
                    "Phase {} over current at {} A",
                    data_value1, data_value2
                )),
                0x0034 => Some(format!(
                    "Phase {} under current at {} A",
                    data_value1, data_value2
                )),
                0x0035 => Some(format!(
                    "Phase {} is at {} degrees",
                    data_value1, data_value2
                )),
                0x0036 => Some(format!("Phase {} Error", data_value1)),
                0x0037 => Some(format!("{} Amps", data_value1)),
                0x0038 => Some(format!("{} Volts", data_value1)),
                0x0041 => Some("No Dimmer".to_string()),
                0x0042 => Some("Tripped Breaker".to_string()),
                0x0043 => Some(format!("{} Watts", data_value1)),
                0x0044 => Some("Dimmer Failure".to_string()),
                0x0045 => Some("Panic Mode".to_string()),
                0x0050 => SlotIdDefinition::try_from(data_value1)
                    .ok()
                    .map(|slot_id| format!("{} ready", slot_id)),
                0x0051 => SlotIdDefinition::try_from(data_value1)
                    .ok()
                    .map(|slot_id| format!("{} not ready", slot_id)),
                0x0052 => SlotIdDefinition::try_from(data_value1)
                    .ok()
                    .map(|slot_id| format!("{} low fluid", slot_id)),
                _ => None,
            }
        } else {
            None
        };

        StatusMessage {
            sub_device_id,
            status_type,
            status_message_id,
            data_value1,
            data_value2,
            description,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SlotType {
    Primary = 0x00,
    SecondaryFine = 0x01,
    SecondaryTiming = 0x02,
    SecondarySpeed = 0x03,
    SecondaryControl = 0x04,
    SecondaryIndex = 0x05,
    SecondaryRotation = 0x06,
    SecondaryIndexRotate = 0x07,
    SecondaryUndefined = 0xff,
}

impl TryFrom<u8> for SlotType {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, ProtocolError> {
        match value {
            0x00 => Ok(Self::Primary),
            0x01 => Ok(Self::SecondaryFine),
            0x02 => Ok(Self::SecondaryTiming),
            0x03 => Ok(Self::SecondarySpeed),
            0x04 => Ok(Self::SecondaryControl),
            0x05 => Ok(Self::SecondaryIndex),
            0x06 => Ok(Self::SecondaryRotation),
            0x07 => Ok(Self::SecondaryIndexRotate),
            0xff => Ok(Self::SecondaryUndefined),
            _ => Err(ProtocolError::InvalidSlotType(value)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SlotInfo {
    pub id: u16,
    pub r#type: SlotType,
    pub label_id: u16,
}

impl SlotInfo {
    pub fn new(id: u16, r#type: SlotType, label_id: u16) -> Self {
        Self {
            id,
            r#type,
            label_id,
        }
    }
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SlotIdDefinition {
    Intensity = 0x0001,
    IntensityMaster = 0x0002,
    Pan = 0x0101,
    Tilt = 0x0102,
    ColorWheel = 0x0201,
    ColorSubCyan = 0x0202,
    ColorSubYellow = 0x0203,
    ColorSubMagenta = 0x0204,
    ColorAddRed = 0x0205,
    ColorAddGreen = 0x0206,
    ColorAddBlue = 0x0207,
    ColorCorrection = 0x0208,
    ColorScroll = 0x0209,
    ColorSemaphore = 0x0210,
    StaticGoboWheel = 0x0301,
    RotoGoboWheel = 0x0302,
    PrismWheel = 0x0303,
    EffectsWheel = 0x0304,
    BeamSizeIris = 0x0401,
    Edge = 0x0402,
    Frost = 0x0403,
    Strobe = 0x0404,
    Zoom = 0x0405,
    FramingShutter = 0x0406,
    ShutterRotate = 0x0407,
    Douser = 0x0408,
    BarnDoor = 0x0409,
    LampControl = 0x0501,
    FixtureControl = 0x0502,
    FixtureSpeed = 0x0503,
    Macro = 0x0504,
    Undefined = 0xffff,
}

impl TryFrom<u16> for SlotIdDefinition {
    type Error = ProtocolError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0x0001 => Ok(Self::Intensity),
            0x0002 => Ok(Self::IntensityMaster),
            0x0101 => Ok(Self::Pan),
            0x0102 => Ok(Self::Tilt),
            0x0201 => Ok(Self::ColorWheel),
            0x0202 => Ok(Self::ColorSubCyan),
            0x0203 => Ok(Self::ColorSubYellow),
            0x0204 => Ok(Self::ColorSubMagenta),
            0x0205 => Ok(Self::ColorAddRed),
            0x0206 => Ok(Self::ColorAddGreen),
            0x0207 => Ok(Self::ColorAddBlue),
            0x0208 => Ok(Self::ColorCorrection),
            0x0209 => Ok(Self::ColorScroll),
            0x0210 => Ok(Self::ColorSemaphore),
            0x0301 => Ok(Self::StaticGoboWheel),
            0x0302 => Ok(Self::RotoGoboWheel),
            0x0303 => Ok(Self::PrismWheel),
            0x0304 => Ok(Self::EffectsWheel),
            0x0401 => Ok(Self::BeamSizeIris),
            0x0402 => Ok(Self::Edge),
            0x0403 => Ok(Self::Frost),
            0x0404 => Ok(Self::Strobe),
            0x0405 => Ok(Self::Zoom),
            0x0406 => Ok(Self::FramingShutter),
            0x0407 => Ok(Self::ShutterRotate),
            0x0408 => Ok(Self::Douser),
            0x0409 => Ok(Self::BarnDoor),
            0x0501 => Ok(Self::LampControl),
            0x0502 => Ok(Self::FixtureControl),
            0x0503 => Ok(Self::FixtureSpeed),
            0x0504 => Ok(Self::Macro),
            0xffff => Ok(Self::Undefined),
            _ => Err(ProtocolError::UnsupportedSlotIdDefinition(value)),
        }
    }
}

impl Display for SlotIdDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let definition = match self {
            SlotIdDefinition::Intensity => "Intensity",
            SlotIdDefinition::IntensityMaster => "Intensity Master",
            SlotIdDefinition::Pan => "Pan",
            SlotIdDefinition::Tilt => "Tilt",
            SlotIdDefinition::ColorWheel => "Color Wheel",
            SlotIdDefinition::ColorSubCyan => "Color Sub Cyan",
            SlotIdDefinition::ColorSubYellow => "Color Sub Yellow",
            SlotIdDefinition::ColorSubMagenta => "Color Sub Magenta",
            SlotIdDefinition::ColorAddRed => "Color Add Red",
            SlotIdDefinition::ColorAddGreen => "Color Add Green",
            SlotIdDefinition::ColorAddBlue => "Color Add Blue",
            SlotIdDefinition::ColorCorrection => "Color Correction",
            SlotIdDefinition::ColorScroll => "Color Scroll",
            SlotIdDefinition::ColorSemaphore => "Color Semaphore",
            SlotIdDefinition::StaticGoboWheel => "Static Gobo Wheel",
            SlotIdDefinition::RotoGoboWheel => "Roto Gobo Wheel",
            SlotIdDefinition::PrismWheel => "Prism Wheel",
            SlotIdDefinition::EffectsWheel => "Effects Wheel",
            SlotIdDefinition::BeamSizeIris => "Beam Size Iris",
            SlotIdDefinition::Edge => "Edge",
            SlotIdDefinition::Frost => "Frost",
            SlotIdDefinition::Strobe => "Strobe",
            SlotIdDefinition::Zoom => "Zoom",
            SlotIdDefinition::FramingShutter => "Framing Shutter",
            SlotIdDefinition::ShutterRotate => "Shutter Rotate",
            SlotIdDefinition::Douser => "Douser",
            SlotIdDefinition::BarnDoor => "Barn Door",
            SlotIdDefinition::LampControl => "Lamp Control",
            SlotIdDefinition::FixtureControl => "Fixture Control",
            SlotIdDefinition::FixtureSpeed => "Fixture Speed",
            SlotIdDefinition::Macro => "Macro",
            SlotIdDefinition::Undefined => "Undefined",
        };

        f.write_str(definition)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DefaultSlotValue {
    pub id: u16,
    pub value: u8,
}

impl DefaultSlotValue {
    pub fn new(id: u16, value: u8) -> Self {
        Self { id, value }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SensorUnit {
    None = 0x00,
    Centigrade = 0x01,
    VoltsDc = 0x02,
    VoltsAcPeak = 0x03,
    VoltsAcRms = 0x04,
    AmpsDc = 0x05,
    AmpsAcPeak = 0x06,
    AmpsAcRms = 0x07,
    Hertz = 0x08,
    Ohm = 0x09,
    Watt = 0x0a,
    Kilogram = 0x0b,
    Meter = 0x0c,
    SquareMeter = 0x0d,
    CubicMeter = 0x0e,
    KilogramPerCubicMeter = 0x0f,
    MeterPerSecond = 0x10,
    MeterPerSecondSquared = 0x11,
    Newton = 0x12,
    Joule = 0x13,
    Pascal = 0x14,
    Second = 0x15,
    Degree = 0x16,
    Steradian = 0x17,
    Candela = 0x18,
    Lumen = 0x19,
    Lux = 0x1a,
    Ire = 0x1b,
    Byte = 0x1c,
}

impl TryFrom<u8> for SensorUnit {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::None),
            0x01 => Ok(Self::Centigrade),
            0x02 => Ok(Self::VoltsDc),
            0x03 => Ok(Self::VoltsAcPeak),
            0x04 => Ok(Self::VoltsAcRms),
            0x05 => Ok(Self::AmpsDc),
            0x06 => Ok(Self::AmpsAcPeak),
            0x07 => Ok(Self::AmpsAcRms),
            0x08 => Ok(Self::Hertz),
            0x09 => Ok(Self::Ohm),
            0x0a => Ok(Self::Watt),
            0x0b => Ok(Self::Kilogram),
            0x0c => Ok(Self::Meter),
            0x0d => Ok(Self::SquareMeter),
            0x0e => Ok(Self::CubicMeter),
            0x0f => Ok(Self::KilogramPerCubicMeter),
            0x10 => Ok(Self::MeterPerSecond),
            0x11 => Ok(Self::MeterPerSecondSquared),
            0x12 => Ok(Self::Newton),
            0x13 => Ok(Self::Joule),
            0x14 => Ok(Self::Pascal),
            0x15 => Ok(Self::Second),
            0x16 => Ok(Self::Degree),
            0x17 => Ok(Self::Steradian),
            0x18 => Ok(Self::Candela),
            0x19 => Ok(Self::Lumen),
            0x1a => Ok(Self::Lux),
            0x1b => Ok(Self::Ire),
            0x1c => Ok(Self::Byte),
            _ => Err(ProtocolError::InvalidSensorUnit(value)),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SensorUnitPrefix {
    None = 0x00,
    Deci = 0x01,
    Centi = 0x02,
    Milli = 0x03,
    Micro = 0x04,
    Nano = 0x05,
    Pico = 0x06,
    Femto = 0x07,
    Atto = 0x08,
    Zepto = 0x09,
    Yocto = 0x0a,
    Deca = 0x11,
    Hecto = 0x12,
    Kilo = 0x13,
    Mega = 0x14,
    Giga = 0x15,
    Terra = 0x16,
    Peta = 0x17,
    Exa = 0x18,
    Zetta = 0x19,
    Yotta = 0x1a,
}

impl TryFrom<u8> for SensorUnitPrefix {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::None),
            0x01 => Ok(Self::Deci),
            0x02 => Ok(Self::Centi),
            0x03 => Ok(Self::Milli),
            0x04 => Ok(Self::Micro),
            0x05 => Ok(Self::Nano),
            0x06 => Ok(Self::Pico),
            0x07 => Ok(Self::Femto),
            0x08 => Ok(Self::Atto),
            0x09 => Ok(Self::Zepto),
            0x0a => Ok(Self::Yocto),
            0x11 => Ok(Self::Deca),
            0x12 => Ok(Self::Hecto),
            0x13 => Ok(Self::Kilo),
            0x14 => Ok(Self::Mega),
            0x15 => Ok(Self::Giga),
            0x16 => Ok(Self::Terra),
            0x17 => Ok(Self::Peta),
            0x18 => Ok(Self::Exa),
            0x19 => Ok(Self::Zetta),
            0x1a => Ok(Self::Yotta),
            _ => Err(ProtocolError::InvalidSensorUnitPrefix(value)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SensorDefinition {
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
    pub description: String,
}

#[derive(Copy, Clone, Debug, PartialEq)]
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
        recorded_value: i16,
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
