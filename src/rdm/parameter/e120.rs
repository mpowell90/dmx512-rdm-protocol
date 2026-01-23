use super::{RdmError, SubDeviceId};
use crate::{impl_rdm_string, rdm::DeviceUID};
use core::{fmt, str::FromStr};
use heapless::{String, Vec};
use rdm_parameter_derive::{
    RdmDiscoveryResponseParameter, RdmGetRequestParameter, RdmGetResponseParameter,
    RdmSetRequestParameter, RdmSetResponseParameter,
};
use rdm_parameter_traits::{ParameterCodecError, RdmParameterData};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ProtocolVersion {
    major: u8,
    minor: u8,
}

impl ProtocolVersion {
    pub const V1: Self = Self { major: 1, minor: 0 };

    pub fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }
}

impl From<ProtocolVersion> for u16 {
    fn from(value: ProtocolVersion) -> Self {
        u16::from_be_bytes([value.major, value.minor])
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl RdmParameterData for ProtocolVersion {
    fn size_of(&self) -> usize {
        2
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        if buf.len() < 2 {
            return Err(ParameterCodecError::BufferTooSmall {
                provided: buf.len(),
                required: 2,
            });
        }
        buf[0] = self.major;
        buf[1] = self.minor;
        Ok(2)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        if buf.len() < 2 {
            return Err(ParameterCodecError::MalformedData);
        }
        let major = buf[0];
        let minor = buf[1];
        Ok(Self { major, minor })
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ProductDetailValue(pub u16);

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ProductDetail {
    NotDeclared,
    Arc,
    MetalHalide,
    Incandescent,
    Led,
    Fluorescent,
    ColdCathode,
    ElectroLuminescent,
    Laser,
    FlashTube,
    ColorScroller,
    ColorWheel,
    ColorChange,
    IrisDouser,
    DimmingShutter,
    ProfileShutter,
    BarnDoorShutter,
    EffectsDisc,
    GoboRotator,
    Video,
    Slide,
    Film,
    OilWheel,
    LcdGate,
    FoggerGlycol,
    FoggerMineralOil,
    FoggerWater,
    CO2,
    LN2,
    Bubble,
    FlamePropane,
    FlameOther,
    OlefactoryStimulator,
    Snow,
    WaterJet,
    Wind,
    Confetti,
    Hazard,
    PhaseControl,
    ReversePhaseControl,
    Sine,
    Pwm,
    Dc,
    HfBallast,
    HfHvNeonBallast,
    HfHvEl,
    MhrBallast,
    BitangleModulation,
    FrequencyModulation,
    HighFrequency12V,
    RelayMechanical,
    RelayElectronic,
    SwitchElectronic,
    Contactor,
    MirrorBallRotator,
    OtherRotator,
    KabukiDrop,
    Curtain,
    LineSet,
    MotorControl,
    DamperControl,
    Splitter,
    EthernetNode,
    Merge,
    DataPatch,
    WirelessLink,
    ProtocolConvertor,
    AnalogDemultiplex,
    AnalogMultiplex,
    SwitchPanel,
    Router,
    Fader,
    Mixer,
    ChangeOverManual,
    ChangeOverAuto,
    Test,
    GfiRcd,
    Battery,
    ControllableBreaker,
    Other,
    ManufacturerSpecific(u16),
    Unknown(u16),
}

impl From<u16> for ProductDetail {
    fn from(value: u16) -> Self {
        match value {
            0x0000 => Self::NotDeclared,
            0x0001 => Self::Arc,
            0x0002 => Self::MetalHalide,
            0x0003 => Self::Incandescent,
            0x0004 => Self::Led,
            0x0005 => Self::Fluorescent,
            0x0006 => Self::ColdCathode,
            0x0007 => Self::ElectroLuminescent,
            0x0008 => Self::Laser,
            0x0009 => Self::FlashTube,
            0x0100 => Self::ColorScroller,
            0x0101 => Self::ColorWheel,
            0x0102 => Self::ColorChange,
            0x0103 => Self::IrisDouser,
            0x0104 => Self::DimmingShutter,
            0x0105 => Self::ProfileShutter,
            0x0106 => Self::BarnDoorShutter,
            0x0107 => Self::EffectsDisc,
            0x0108 => Self::GoboRotator,
            0x0200 => Self::Video,
            0x0201 => Self::Slide,
            0x0202 => Self::Film,
            0x0203 => Self::OilWheel,
            0x0204 => Self::LcdGate,
            0x0300 => Self::FoggerGlycol,
            0x0301 => Self::FoggerMineralOil,
            0x0302 => Self::FoggerWater,
            0x0303 => Self::CO2,
            0x0304 => Self::LN2,
            0x0305 => Self::Bubble,
            0x0306 => Self::FlamePropane,
            0x0307 => Self::FlameOther,
            0x0308 => Self::OlefactoryStimulator,
            0x0309 => Self::Snow,
            0x030a => Self::WaterJet,
            0x030b => Self::Wind,
            0x030c => Self::Confetti,
            0x030d => Self::Hazard,
            0x0400 => Self::PhaseControl,
            0x0401 => Self::ReversePhaseControl,
            0x0402 => Self::Sine,
            0x0403 => Self::Pwm,
            0x0404 => Self::Dc,
            0x0405 => Self::HfBallast,
            0x0406 => Self::HfHvNeonBallast,
            0x0407 => Self::HfHvEl,
            0x0408 => Self::MhrBallast,
            0x0409 => Self::BitangleModulation,
            0x040a => Self::FrequencyModulation,
            0x040b => Self::HighFrequency12V,
            0x040c => Self::RelayMechanical,
            0x040d => Self::RelayElectronic,
            0x040e => Self::SwitchElectronic,
            0x040f => Self::Contactor,
            0x0500 => Self::MirrorBallRotator,
            0x0501 => Self::OtherRotator,
            0x0502 => Self::KabukiDrop,
            0x0503 => Self::Curtain,
            0x0504 => Self::LineSet,
            0x0505 => Self::MotorControl,
            0x0506 => Self::DamperControl,
            0x0600 => Self::Splitter,
            0x0601 => Self::EthernetNode,
            0x0602 => Self::Merge,
            0x0603 => Self::DataPatch,
            0x0604 => Self::WirelessLink,
            0x0701 => Self::ProtocolConvertor,
            0x0702 => Self::AnalogDemultiplex,
            0x0703 => Self::AnalogMultiplex,
            0x0704 => Self::SwitchPanel,
            0x0800 => Self::Router,
            0x0801 => Self::Fader,
            0x0802 => Self::Mixer,
            0x0900 => Self::ChangeOverManual,
            0x0901 => Self::ChangeOverAuto,
            0x0902 => Self::Test,
            0x0a00 => Self::GfiRcd,
            0x0a01 => Self::Battery,
            0x0a02 => Self::ControllableBreaker,
            0x7fff => Self::Other,
            value if (0x8000..=0xdfff).contains(&value) => Self::ManufacturerSpecific(value),
            value => Self::Unknown(value),
        }
    }
}

impl From<ProductDetail> for u16 {
    fn from(value: ProductDetail) -> Self {
        match value {
            ProductDetail::NotDeclared => 0x0000,
            ProductDetail::Arc => 0x0001,
            ProductDetail::MetalHalide => 0x0002,
            ProductDetail::Incandescent => 0x0003,
            ProductDetail::Led => 0x0004,
            ProductDetail::Fluorescent => 0x0005,
            ProductDetail::ColdCathode => 0x0006,
            ProductDetail::ElectroLuminescent => 0x0007,
            ProductDetail::Laser => 0x0008,
            ProductDetail::FlashTube => 0x0009,
            ProductDetail::ColorScroller => 0x0100,
            ProductDetail::ColorWheel => 0x0101,
            ProductDetail::ColorChange => 0x0102,
            ProductDetail::IrisDouser => 0x0103,
            ProductDetail::DimmingShutter => 0x0104,
            ProductDetail::ProfileShutter => 0x0105,
            ProductDetail::BarnDoorShutter => 0x0106,
            ProductDetail::EffectsDisc => 0x0107,
            ProductDetail::GoboRotator => 0x0108,
            ProductDetail::Video => 0x0200,
            ProductDetail::Slide => 0x0201,
            ProductDetail::Film => 0x0202,
            ProductDetail::OilWheel => 0x0203,
            ProductDetail::LcdGate => 0x0204,
            ProductDetail::FoggerGlycol => 0x0300,
            ProductDetail::FoggerMineralOil => 0x0301,
            ProductDetail::FoggerWater => 0x0302,
            ProductDetail::CO2 => 0x0303,
            ProductDetail::LN2 => 0x0304,
            ProductDetail::Bubble => 0x0305,
            ProductDetail::FlamePropane => 0x0306,
            ProductDetail::FlameOther => 0x0307,
            ProductDetail::OlefactoryStimulator => 0x0308,
            ProductDetail::Snow => 0x0309,
            ProductDetail::WaterJet => 0x030a,
            ProductDetail::Wind => 0x030b,
            ProductDetail::Confetti => 0x030c,
            ProductDetail::Hazard => 0x030d,
            ProductDetail::PhaseControl => 0x0400,
            ProductDetail::ReversePhaseControl => 0x0401,
            ProductDetail::Sine => 0x0402,
            ProductDetail::Pwm => 0x0403,
            ProductDetail::Dc => 0x0404,
            ProductDetail::HfBallast => 0x0405,
            ProductDetail::HfHvNeonBallast => 0x0406,
            ProductDetail::HfHvEl => 0x0407,
            ProductDetail::MhrBallast => 0x0408,
            ProductDetail::BitangleModulation => 0x0409,
            ProductDetail::FrequencyModulation => 0x040a,
            ProductDetail::HighFrequency12V => 0x040b,
            ProductDetail::RelayMechanical => 0x040c,
            ProductDetail::RelayElectronic => 0x040d,
            ProductDetail::SwitchElectronic => 0x040e,
            ProductDetail::Contactor => 0x040f,
            ProductDetail::MirrorBallRotator => 0x0500,
            ProductDetail::OtherRotator => 0x0501,
            ProductDetail::KabukiDrop => 0x0502,
            ProductDetail::Curtain => 0x0503,
            ProductDetail::LineSet => 0x0504,
            ProductDetail::MotorControl => 0x0505,
            ProductDetail::DamperControl => 0x0506,
            ProductDetail::Splitter => 0x0600,
            ProductDetail::EthernetNode => 0x0601,
            ProductDetail::Merge => 0x0602,
            ProductDetail::DataPatch => 0x0603,
            ProductDetail::WirelessLink => 0x0604,
            ProductDetail::ProtocolConvertor => 0x0701,
            ProductDetail::AnalogDemultiplex => 0x0702,
            ProductDetail::AnalogMultiplex => 0x0703,
            ProductDetail::SwitchPanel => 0x0704,
            ProductDetail::Router => 0x0800,
            ProductDetail::Fader => 0x0801,
            ProductDetail::Mixer => 0x0802,
            ProductDetail::ChangeOverManual => 0x0900,
            ProductDetail::ChangeOverAuto => 0x0901,
            ProductDetail::Test => 0x0902,
            ProductDetail::GfiRcd => 0x0a00,
            ProductDetail::Battery => 0x0a01,
            ProductDetail::ControllableBreaker => 0x0a02,
            ProductDetail::Other => 0x7fff,
            ProductDetail::ManufacturerSpecific(value) => value,
            ProductDetail::Unknown(value) => value,
        }
    }
}

impl From<ProductDetailValue> for ProductDetail {
    fn from(value: ProductDetailValue) -> Self {
        value.0.into()
    }
}

impl From<ProductDetail> for ProductDetailValue {
    fn from(value: ProductDetail) -> Self {
        Self(value.into())
    }
}

impl RdmParameterData for ProductDetailValue {
    fn size_of(&self) -> usize {
        2
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        if buf.len() < 2 {
            return Err(ParameterCodecError::BufferTooSmall {
                provided: buf.len(),
                required: 2,
            });
        }
        let bytes = self.0.to_be_bytes();
        buf[0] = bytes[0];
        buf[1] = bytes[1];
        Ok(2)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        if buf.len() < 2 {
            return Err(ParameterCodecError::MalformedData);
        }
        let value = u16::from_be_bytes([buf[0], buf[1]]);
        Ok(Self(value))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ImplementedCommandClass {
    Get = 0x01,
    Set = 0x02,
    GetSet = 0x03,
}

impl TryFrom<u8> for ImplementedCommandClass {
    type Error = RdmError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::Get),
            0x02 => Ok(Self::Set),
            0x03 => Ok(Self::GetSet),
            _ => Err(RdmError::InvalidCommandClassImplementation(value)),
        }
    }
}

impl RdmParameterData for ImplementedCommandClass {
    fn size_of(&self) -> usize {
        1
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        if buf.len() < 1 {
            return Err(ParameterCodecError::BufferTooSmall {
                provided: buf.len(),
                required: 1,
            });
        }
        buf[0] = *self as u8;
        Ok(1)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        if buf.len() < 1 {
            return Err(ParameterCodecError::MalformedData);
        }
        let command_class = ImplementedCommandClass::try_from(buf[0])
            .map_err(|_| ParameterCodecError::MalformedData)?;
        Ok(command_class)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
    type Error = RdmError;

    fn try_from(value: u8) -> Result<Self, RdmError> {
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
            _ => Err(RdmError::InvalidParameterDataType(value)),
        }
    }
}

impl From<ParameterDataType> for u8 {
    fn from(value: ParameterDataType) -> Self {
        match value {
            ParameterDataType::NotDefined => 0x00,
            ParameterDataType::BitField => 0x01,
            ParameterDataType::Ascii => 0x02,
            ParameterDataType::UnsignedByte => 0x03,
            ParameterDataType::SignedByte => 0x04,
            ParameterDataType::UnsignedWord => 0x05,
            ParameterDataType::SignedWord => 0x06,
            ParameterDataType::UnsignedDWord => 0x07,
            ParameterDataType::SignedDWord => 0x08,
            ParameterDataType::ManufacturerSpecific(n) => n,
        }
    }
}

impl ParameterDataType {
    fn convert_parameter_value(&self, value: [u8; 4]) -> Result<ConvertedParameterValue, RdmError> {
        match self {
            ParameterDataType::BitField => {
                Ok(ConvertedParameterValue::BitField(u32::from_be_bytes(value)))
            }
            ParameterDataType::Ascii => Ok(ConvertedParameterValue::Ascii(
                String::<4>::from_str(core::str::from_utf8(&value)?).unwrap(),
            )),
            ParameterDataType::UnsignedByte => Ok(ConvertedParameterValue::UnsignedByte(value[3])),
            ParameterDataType::SignedByte => {
                Ok(ConvertedParameterValue::SignedByte(value[3] as i8))
            }
            ParameterDataType::UnsignedWord => {
                Ok(ConvertedParameterValue::UnsignedWord(u16::from_be_bytes([
                    value[2], value[3],
                ])))
            }
            ParameterDataType::SignedWord => {
                Ok(ConvertedParameterValue::SignedWord(i16::from_be_bytes([
                    value[2], value[3],
                ])))
            }
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

    pub fn minimum_valid_value(
        &self,
        raw_minimum_valid_value: [u8; 4],
    ) -> Result<ConvertedParameterValue, RdmError> {
        self.convert_parameter_value(raw_minimum_valid_value)
    }
    pub fn maximum_valid_value(
        &self,
        raw_maximum_valid_value: [u8; 4],
    ) -> Result<ConvertedParameterValue, RdmError> {
        self.convert_parameter_value(raw_maximum_valid_value)
    }
    pub fn default_value(
        &self,
        raw_default_value: [u8; 4],
    ) -> Result<ConvertedParameterValue, RdmError> {
        self.convert_parameter_value(raw_default_value)
    }
}

impl RdmParameterData for ParameterDataType {
    fn size_of(&self) -> usize {
        1
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        if buf.len() < 1 {
            return Err(ParameterCodecError::BufferTooSmall {
                provided: buf.len(),
                required: 1,
            });
        }
        buf[0] = (*self).into();
        Ok(1)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        if buf.len() < 1 {
            return Err(ParameterCodecError::MalformedData);
        }
        let data_type =
            ParameterDataType::try_from(buf[0]).map_err(|_| ParameterCodecError::MalformedData)?;
        Ok(data_type)
    }
}

pub enum ConvertedParameterValue {
    BitField(u32),
    Ascii(String<4>),
    UnsignedByte(u8),
    SignedByte(i8),
    UnsignedWord(u16),
    SignedWord(i16),
    UnsignedDWord(u32),
    SignedDWord(i32),
    Raw([u8; 4]),
}

pub const PARAMETER_DESCRIPTION_LABEL_MAX_LENGTH: usize = 32;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ParameterDescriptionLabel(String<PARAMETER_DESCRIPTION_LABEL_MAX_LENGTH>);
impl_rdm_string!(
    ParameterDescriptionLabel,
    PARAMETER_DESCRIPTION_LABEL_MAX_LENGTH
);

pub const DMX_PERSONALITY_DESCRIPTION_MAX_LENGTH: usize = 32;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DmxPersonalityDescription(String<DMX_PERSONALITY_DESCRIPTION_MAX_LENGTH>);

impl_rdm_string!(
    DmxPersonalityDescription,
    DMX_PERSONALITY_DESCRIPTION_MAX_LENGTH
);

pub const SLOT_DESCRIPTION_MAX_LENGTH: usize = 32;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SlotDescription(String<SLOT_DESCRIPTION_MAX_LENGTH>);

impl_rdm_string!(SlotDescription, SLOT_DESCRIPTION_MAX_LENGTH);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeviceLabel(String<{ DeviceLabel::MAX_LENGTH }>);

impl_rdm_string!(DeviceLabel, 32);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeviceModelDescription(String<{ DeviceModelDescription::MAX_LENGTH }>);

impl_rdm_string!(DeviceModelDescription, 32);

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ManufacturerLabel(String<{ ManufacturerLabel::MAX_LENGTH }>);

impl_rdm_string!(ManufacturerLabel, 32);

pub const SOFTWARE_VERSION_LABEL_MAX_LENGTH: usize = 32;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SoftwareVersionLabel(String<SOFTWARE_VERSION_LABEL_MAX_LENGTH>);
impl_rdm_string!(SoftwareVersionLabel, SOFTWARE_VERSION_LABEL_MAX_LENGTH);

pub const BOOT_SOFTWARE_VERSION_LABEL_MAX_LENGTH: usize = 32;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BootSoftwareVersionLabel(String<BOOT_SOFTWARE_VERSION_LABEL_MAX_LENGTH>);
impl_rdm_string!(
    BootSoftwareVersionLabel,
    BOOT_SOFTWARE_VERSION_LABEL_MAX_LENGTH
);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
    type Error = RdmError;

    fn try_from(value: u8) -> Result<Self, RdmError> {
        match value {
            0x00 => Ok(Self::None),
            0x01 => Ok(Self::GetLastMessage),
            0x02 => Ok(Self::Advisory),
            0x03 => Ok(Self::Warning),
            0x04 => Ok(Self::Error),
            0x12 => Ok(Self::AdvisoryCleared),
            0x13 => Ok(Self::WarningCleared),
            0x14 => Ok(Self::ErrorCleared),
            _ => Err(RdmError::InvalidStatusType(value)),
        }
    }
}

impl RdmParameterData for StatusType {
    fn size_of(&self) -> usize {
        1
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        buf[0] = *self as u8;
        Ok(1)
    }

    fn decode_rdm_parameter_data(
        buf: &[u8],
    ) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
        let status_type = StatusType::from_be_bytes([buf[0]]);
        Ok(status_type)
    }
}

impl StatusType {
    pub fn from_be_bytes(bytes: [u8; 1]) -> Self {
        Self::try_from(bytes[0]).unwrap() // TODO consider error handling
    }

    pub fn to_be_bytes(&self) -> [u8; 1] {
        [*self as u8]
    }
}

// Product Categories - Page 105 RDM Spec
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u16)]
pub enum ProductCategory {
    NotDeclared,
    Fixture,
    FixtureFixed,
    FixtureMovingYoke,
    FixtureMovingMirror,
    FixtureOther,
    FixtureAccessory,
    FixtureAccessoryColor,
    FixtureAccessoryYoke,
    FixtureAccessoryMirror,
    FixtureAccessoryEffect,
    FixtureAccessoryBeam,
    AccessoryOther,
    Projector,
    ProjectorFixed,
    ProjectorMovingYoke,
    ProjectorMovingMirror,
    ProjectorOther,
    Atmospheric,
    AtmosphericEffect,
    AtmosphericPyro,
    AtmosphericOther,
    Dimmer,
    DimmerACIncandescent,
    DimmerACFlourescent,
    DimmerACColdCathode,
    DimmerACNonDimModule,
    DimmerACLowVoltage,
    DimmerControllableAC,
    DimmerDCLevelOutput,
    DimmerDCPWMOutput,
    DimmerSpecialisedLED,
    DimmerOther,
    Power,
    PowerControl,
    PowerSource,
    PowerOther,
    Scenic,
    ScenicDrive,
    ScenicOther,
    Data,
    DataDistribution,
    DataConversion,
    DataOther,
    AV,
    AVAudio,
    AVVideo,
    AVOther,
    Monitor,
    MonitorACLinePower,
    MonitorDCPower,
    MonitorEnvironmental,
    MonitorOther,
    Control,
    ControlController,
    ControlBackupDevice,
    ControlOther,
    Test,
    TestEquipment,
    TestEquipmentOther,
    Other,
    ManufacturerSpecific(u16),
    Unknown(u16),
}

impl From<u16> for ProductCategory {
    fn from(value: u16) -> Self {
        match value {
            0x0000 => Self::NotDeclared,
            0x0100 => Self::Fixture,
            0x0101 => Self::FixtureFixed,
            0x0102 => Self::FixtureMovingYoke,
            0x0103 => Self::FixtureMovingMirror,
            0x01ff => Self::FixtureOther,
            0x0200 => Self::FixtureAccessory,
            0x0201 => Self::FixtureAccessoryColor,
            0x0202 => Self::FixtureAccessoryYoke,
            0x0203 => Self::FixtureAccessoryMirror,
            0x0204 => Self::FixtureAccessoryEffect,
            0x0205 => Self::FixtureAccessoryBeam,
            0x02ff => Self::AccessoryOther,
            0x0300 => Self::Projector,
            0x0301 => Self::ProjectorFixed,
            0x0302 => Self::ProjectorMovingYoke,
            0x0303 => Self::ProjectorMovingMirror,
            0x03ff => Self::ProjectorOther,
            0x0400 => Self::Atmospheric,
            0x0401 => Self::AtmosphericEffect,
            0x0402 => Self::AtmosphericPyro,
            0x04ff => Self::AtmosphericOther,
            0x0500 => Self::Dimmer,
            0x0501 => Self::DimmerACIncandescent,
            0x0502 => Self::DimmerACFlourescent,
            0x0503 => Self::DimmerACColdCathode,
            0x0504 => Self::DimmerACNonDimModule,
            0x0505 => Self::DimmerACLowVoltage,
            0x0506 => Self::DimmerControllableAC,
            0x0507 => Self::DimmerDCLevelOutput,
            0x0508 => Self::DimmerDCPWMOutput,
            0x0509 => Self::DimmerSpecialisedLED,
            0x05ff => Self::DimmerOther,
            0x0600 => Self::Power,
            0x0601 => Self::PowerControl,
            0x0602 => Self::PowerSource,
            0x06ff => Self::PowerOther,
            0x0700 => Self::Scenic,
            0x0701 => Self::ScenicDrive,
            0x07ff => Self::ScenicOther,
            0x0800 => Self::Data,
            0x0801 => Self::DataDistribution,
            0x0802 => Self::DataConversion,
            0x08ff => Self::DataOther,
            0x0900 => Self::AV,
            0x0901 => Self::AVAudio,
            0x0902 => Self::AVVideo,
            0x09ff => Self::AVOther,
            0x0a00 => Self::Monitor,
            0x0a01 => Self::MonitorACLinePower,
            0x0a02 => Self::MonitorDCPower,
            0x0a03 => Self::MonitorEnvironmental,
            0x0aff => Self::MonitorOther,
            0x7000 => Self::Control,
            0x7001 => Self::ControlController,
            0x7002 => Self::ControlBackupDevice,
            0x70ff => Self::ControlOther,
            0x7100 => Self::Test,
            0x7101 => Self::TestEquipment,
            0x71ff => Self::TestEquipmentOther,
            0x7fff => Self::Other,
            value if (0x8000..=0xdfff).contains(&value) => Self::ManufacturerSpecific(value),
            value => Self::Unknown(value),
        }
    }
}

impl From<ProductCategory> for u16 {
    fn from(value: ProductCategory) -> Self {
        match value {
            ProductCategory::NotDeclared => 0x0000,
            ProductCategory::Fixture => 0x0100,
            ProductCategory::FixtureFixed => 0x0101,
            ProductCategory::FixtureMovingYoke => 0x0102,
            ProductCategory::FixtureMovingMirror => 0x0103,
            ProductCategory::FixtureOther => 0x01ff,
            ProductCategory::FixtureAccessory => 0x0200,
            ProductCategory::FixtureAccessoryColor => 0x0201,
            ProductCategory::FixtureAccessoryYoke => 0x0202,
            ProductCategory::FixtureAccessoryMirror => 0x0203,
            ProductCategory::FixtureAccessoryEffect => 0x0204,
            ProductCategory::FixtureAccessoryBeam => 0x0205,
            ProductCategory::AccessoryOther => 0x02ff,
            ProductCategory::Projector => 0x0300,
            ProductCategory::ProjectorFixed => 0x0301,
            ProductCategory::ProjectorMovingYoke => 0x0302,
            ProductCategory::ProjectorMovingMirror => 0x0303,
            ProductCategory::ProjectorOther => 0x03ff,
            ProductCategory::Atmospheric => 0x0400,
            ProductCategory::AtmosphericEffect => 0x0401,
            ProductCategory::AtmosphericPyro => 0x0402,
            ProductCategory::AtmosphericOther => 0x04ff,
            ProductCategory::Dimmer => 0x0500,
            ProductCategory::DimmerACIncandescent => 0x0501,
            ProductCategory::DimmerACFlourescent => 0x0502,
            ProductCategory::DimmerACColdCathode => 0x0503,
            ProductCategory::DimmerACNonDimModule => 0x0504,
            ProductCategory::DimmerACLowVoltage => 0x0505,
            ProductCategory::DimmerControllableAC => 0x0506,
            ProductCategory::DimmerDCLevelOutput => 0x0507,
            ProductCategory::DimmerDCPWMOutput => 0x0508,
            ProductCategory::DimmerSpecialisedLED => 0x0509,
            ProductCategory::DimmerOther => 0x05ff,
            ProductCategory::Power => 0x0600,
            ProductCategory::PowerControl => 0x0601,
            ProductCategory::PowerSource => 0x0602,
            ProductCategory::PowerOther => 0x06ff,
            ProductCategory::Scenic => 0x0700,
            ProductCategory::ScenicDrive => 0x0701,
            ProductCategory::ScenicOther => 0x07ff,
            ProductCategory::Data => 0x0800,
            ProductCategory::DataDistribution => 0x0801,
            ProductCategory::DataConversion => 0x0802,
            ProductCategory::DataOther => 0x08ff,
            ProductCategory::AV => 0x0900,
            ProductCategory::AVAudio => 0x0901,
            ProductCategory::AVVideo => 0x0902,
            ProductCategory::AVOther => 0x09ff,
            ProductCategory::Monitor => 0x0a00,
            ProductCategory::MonitorACLinePower => 0x0a01,
            ProductCategory::MonitorDCPower => 0x0a02,
            ProductCategory::MonitorEnvironmental => 0x0a03,
            ProductCategory::MonitorOther => 0x0aff,
            ProductCategory::Control => 0x7000,
            ProductCategory::ControlController => 0x7001,
            ProductCategory::ControlBackupDevice => 0x7002,
            ProductCategory::ControlOther => 0x70ff,
            ProductCategory::Test => 0x7100,
            ProductCategory::TestEquipment => 0x7101,
            ProductCategory::TestEquipmentOther => 0x71ff,
            ProductCategory::Other => 0x7fff,
            ProductCategory::ManufacturerSpecific(value) => value,
            ProductCategory::Unknown(value) => value,
        }
    }
}

impl RdmParameterData for ProductCategory {
    fn size_of(&self) -> usize {
        2
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        let bytes = u16::from(*self).to_be_bytes();
        buf[0] = bytes[0];
        buf[1] = bytes[1];
        Ok(2)
    }

    fn decode_rdm_parameter_data(
        buf: &[u8],
    ) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
        let category = ProductCategory::from(u16::from_be_bytes([buf[0], buf[1]]));
        Ok(category)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LampState {
    LampOff,
    LampOn,
    LampStrike,
    LampStandby,
    LampNotPresent,
    LampError,
    ManufacturerSpecific(u8),
}

impl TryFrom<u8> for LampState {
    type Error = RdmError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::LampOff),
            0x01 => Ok(Self::LampOn),
            0x02 => Ok(Self::LampStrike),
            0x03 => Ok(Self::LampStandby),
            0x04 => Ok(Self::LampNotPresent),
            0x05 => Ok(Self::LampError),
            n if (0x80..=0xdf).contains(&n) => Ok(Self::ManufacturerSpecific(n)),
            _ => Err(RdmError::InvalidLampState(value)),
        }
    }
}

impl From<LampState> for u8 {
    fn from(value: LampState) -> u8 {
        match value {
            LampState::LampOff => 0x00,
            LampState::LampOn => 0x01,
            LampState::LampStrike => 0x02,
            LampState::LampStandby => 0x03,
            LampState::LampNotPresent => 0x04,
            LampState::LampError => 0x05,
            LampState::ManufacturerSpecific(n) => n,
        }
    }
}

impl RdmParameterData for LampState {
    fn size_of(&self) -> usize {
        1
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        buf[0] = (*self).into();
        Ok(1)
    }

    fn decode_rdm_parameter_data(
        buf: &[u8],
    ) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
        let lamp_state = Self::try_from(buf[0]).map_err(|_| ParameterCodecError::MalformedData)?;
        Ok(lamp_state)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LampOnMode {
    OffMode,
    DmxMode,
    OnMode,
    AfterCal,
    ManufacturerSpecific(u8),
}

impl TryFrom<u8> for LampOnMode {
    type Error = RdmError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::OffMode),
            0x01 => Ok(Self::DmxMode),
            0x02 => Ok(Self::OnMode),
            0x03 => Ok(Self::AfterCal),
            n if (0x80..=0xdf).contains(&n) => Ok(Self::ManufacturerSpecific(n)),
            _ => Err(RdmError::InvalidLampOnMode(value)),
        }
    }
}

impl From<LampOnMode> for u8 {
    fn from(value: LampOnMode) -> u8 {
        match value {
            LampOnMode::OffMode => 0x00,
            LampOnMode::DmxMode => 0x01,
            LampOnMode::OnMode => 0x02,
            LampOnMode::AfterCal => 0x03,
            LampOnMode::ManufacturerSpecific(n) => n,
        }
    }
}

impl RdmParameterData for LampOnMode {
    fn size_of(&self) -> usize {
        1
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        buf[0] = (*self).into();
        Ok(1)
    }

    fn decode_rdm_parameter_data(
        buf: &[u8],
    ) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
        let lamp_on_mode =
            Self::try_from(buf[0]).map_err(|_| ParameterCodecError::MalformedData)?;
        Ok(lamp_on_mode)
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
    type Error = RdmError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::FullOff),
            0x01 => Ok(Self::Shutdown),
            0x02 => Ok(Self::Standby),
            0x03 => Ok(Self::Normal),
            _ => Err(RdmError::InvalidPowerState(value)),
        }
    }
}

impl RdmParameterData for PowerState {
    fn size_of(&self) -> usize {
        1
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        buf[0] = *self as u8;
        Ok(1)
    }

    fn decode_rdm_parameter_data(
        buf: &[u8],
    ) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
        let power_state = Self::try_from(buf[0]).map_err(|_| ParameterCodecError::MalformedData)?;
        Ok(power_state)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OnOffStates {
    Off = 0x00,
    On = 0x01,
}

impl TryFrom<u8> for OnOffStates {
    type Error = RdmError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Off),
            0x01 => Ok(Self::On),
            _ => Err(RdmError::InvalidOnOffStates(value)),
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
    type Error = RdmError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Off),
            0x01 => Ok(Self::On),
            0x02 => Ok(Self::Auto),
            _ => Err(RdmError::InvalidDisplayInvertMode(value)),
        }
    }
}

impl RdmParameterData for DisplayInvertMode {
    fn size_of(&self) -> usize {
        1
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        buf[0] = *self as u8;
        Ok(1)
    }

    fn decode_rdm_parameter_data(
        buf: &[u8],
    ) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
        let display_invert_mode =
            Self::try_from(buf[0]).map_err(|_| ParameterCodecError::MalformedData)?;
        Ok(display_invert_mode)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ResetDeviceMode {
    Warm = 0x01,
    Cold = 0xff,
}

impl TryFrom<u8> for ResetDeviceMode {
    type Error = RdmError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::Warm),
            0xff => Ok(Self::Cold),
            _ => Err(RdmError::InvalidResetDeviceMode(value)),
        }
    }
}

impl RdmParameterData for ResetDeviceMode {
    fn size_of(&self) -> usize {
        1
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        buf[0] = *self as u8;
        Ok(1)
    }

    fn decode_rdm_parameter_data(
        buf: &[u8],
    ) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
        let reset_device_mode =
            Self::try_from(buf[0]).map_err(|_| ParameterCodecError::MalformedData)?;
        Ok(reset_device_mode)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SelfTest {
    Off,
    All,
    ManufacturerId(u8),
}

impl From<u8> for SelfTest {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::Off,
            0xff => Self::All,
            value => Self::ManufacturerId(value),
        }
    }
}

impl From<SelfTest> for u8 {
    fn from(value: SelfTest) -> u8 {
        match value {
            SelfTest::Off => 0x00,
            SelfTest::All => 0xff,
            SelfTest::ManufacturerId(value) => value,
        }
    }
}

impl RdmParameterData for SelfTest {
    fn size_of(&self) -> usize {
        1
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        buf[0] = (*self).into();
        Ok(1)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        let self_test = SelfTest::from_be_bytes([buf[0]]);
        Ok(self_test)
    }
}

impl SelfTest {
    pub fn from_be_bytes(bytes: [u8; 1]) -> Self {
        Self::from(bytes[0])
    }

    pub fn to_be_bytes(&self) -> [u8; 1] {
        [(*self).into()]
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

impl RdmParameterData for PresetPlaybackMode {
    fn size_of(&self) -> usize {
        2
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        let bytes = u16::from(*self).to_be_bytes();
        buf[0] = bytes[0];
        buf[1] = bytes[1];
        Ok(2)
    }

    fn decode_rdm_parameter_data(
        buf: &[u8],
    ) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
        let mode = PresetPlaybackMode::from(u16::from_be_bytes([buf[0], buf[1]]));
        Ok(mode)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FadeTimes {
    pub up_fade_time: u16,
    pub down_fade_time: u16,
    pub wait_time: u16,
}

impl RdmParameterData for FadeTimes {
    fn size_of(&self) -> usize {
        6
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        buf[0..2].copy_from_slice(&self.up_fade_time.to_be_bytes());
        buf[2..4].copy_from_slice(&self.down_fade_time.to_be_bytes());
        buf[4..6].copy_from_slice(&self.wait_time.to_be_bytes());
        Ok(6)
    }

    fn decode_rdm_parameter_data(
        buf: &[u8],
    ) -> Result<Self, rdm_parameter_traits::ParameterCodecError> {
        Ok(FadeTimes {
            up_fade_time: u16::from_be_bytes([buf[0], buf[1]]),
            down_fade_time: u16::from_be_bytes([buf[2], buf[3]]),
            wait_time: u16::from_be_bytes([buf[4], buf[5]]),
        })
    }
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

pub const STATUS_ID_DESCRIPTION_MAX_LENGTH: usize = 32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusIdDescription(String<STATUS_ID_DESCRIPTION_MAX_LENGTH>);
impl_rdm_string!(StatusIdDescription, STATUS_ID_DESCRIPTION_MAX_LENGTH);

pub const STATUS_MESSAGE_DESCRIPTION_MAX_LENGTH: usize = 32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusMessageDescription(String<STATUS_MESSAGE_DESCRIPTION_MAX_LENGTH>);
impl_rdm_string!(
    StatusMessageDescription,
    STATUS_MESSAGE_DESCRIPTION_MAX_LENGTH
);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusMessage {
    pub sub_device_id: SubDeviceId,
    pub status_type: StatusType,
    pub status_message_id: u16,
    pub data_value1: u16,
    pub data_value2: u16,
}

impl StatusMessage {
    pub fn description(&self) -> Option<StatusMessageDescription> {
        if self.status_message_id < 0x8000 {
            let message = match self.status_message_id {
                0x0001 => format_args!(
                    "{} failed calibration",
                    SlotIdDefinition::from(self.data_value1)
                )
                .as_str(),
                0x0002 => format_args!(
                    "{} sensor not found",
                    SlotIdDefinition::from(self.data_value1)
                )
                .as_str(),
                0x0003 => format_args!(
                    "{} sensor always on",
                    SlotIdDefinition::from(self.data_value1)
                )
                .as_str(),
                0x0011 => Some("Lamp Doused"),
                0x0012 => Some("Lamp Strike"),
                0x0021 => format_args!(
                    "Sensor {} over temp at {} degrees C",
                    self.data_value1, self.data_value2
                )
                .as_str(),
                0x0022 => format_args!(
                    "Sensor {} under temp at {} degrees C",
                    self.data_value1, self.data_value2
                )
                .as_str(),
                0x0023 => format_args!("Sensor {} out of range", self.data_value1).as_str(),
                0x0031 => format_args!(
                    "Phase {} over voltage at {} V",
                    self.data_value1, self.data_value2
                )
                .as_str(),
                0x0032 => format_args!(
                    "Phase {} under voltage at {} V",
                    self.data_value1, self.data_value2
                )
                .as_str(),
                0x0033 => format_args!(
                    "Phase {} over current at {} A",
                    self.data_value1, self.data_value2
                )
                .as_str(),
                0x0034 => format_args!(
                    "Phase {} under current at {} A",
                    self.data_value1, self.data_value2
                )
                .as_str(),
                0x0035 => format_args!(
                    "Phase {} is at {} degrees",
                    self.data_value1, self.data_value2
                )
                .as_str(),
                0x0036 => format_args!("Phase {} Error", self.data_value1).as_str(),
                0x0037 => format_args!("{} Amps", self.data_value1).as_str(),
                0x0038 => format_args!("{} Volts", self.data_value1).as_str(),
                0x0041 => Some("No Dimmer"),
                0x0042 => Some("Tripped Breaker"),
                0x0043 => format_args!("{} Watts", self.data_value1).as_str(),
                0x0044 => Some("Dimmer Failure"),
                0x0045 => Some("Panic Mode"),
                0x0050 => {
                    format_args!("{} ready", SlotIdDefinition::from(self.data_value1)).as_str()
                }
                0x0051 => {
                    format_args!("{} not ready", SlotIdDefinition::from(self.data_value1)).as_str()
                }
                0x0052 => {
                    format_args!("{} low fluid", SlotIdDefinition::from(self.data_value1)).as_str()
                }
                _ => None,
            };

            message.and_then(|s| StatusMessageDescription::from_str(s).ok())
        } else {
            None
        }
    }
}

impl RdmParameterData for StatusMessage {
    fn size_of(&self) -> usize {
        8
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        self.sub_device_id
            .encode_rdm_parameter_data(&mut buf[0..2])?;
        buf[2] = self.status_type as u8;
        buf[3..5].copy_from_slice(&self.status_message_id.to_be_bytes());
        buf[5..7].copy_from_slice(&self.data_value1.to_be_bytes());
        buf[7..9].copy_from_slice(&self.data_value2.to_be_bytes());
        Ok(8)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        Ok(StatusMessage {
            sub_device_id: SubDeviceId::decode_rdm_parameter_data(buf)?,
            status_type: StatusType::decode_rdm_parameter_data(buf)?,
            status_message_id: u16::from_be_bytes([buf[3], buf[4]]),
            data_value1: u16::from_be_bytes([buf[5], buf[6]]),
            data_value2: u16::from_be_bytes([buf[7], buf[8]]),
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SlotType {
    Primary,
    SecondaryFine,
    SecondaryTiming,
    SecondarySpeed,
    SecondaryControl,
    SecondaryIndex,
    SecondaryRotation,
    SecondaryIndexRotate,
    SecondaryUndefined,
    Unknown(u8),
}

impl From<u8> for SlotType {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::Primary,
            0x01 => Self::SecondaryFine,
            0x02 => Self::SecondaryTiming,
            0x03 => Self::SecondarySpeed,
            0x04 => Self::SecondaryControl,
            0x05 => Self::SecondaryIndex,
            0x06 => Self::SecondaryRotation,
            0x07 => Self::SecondaryIndexRotate,
            0xff => Self::SecondaryUndefined,
            value => Self::Unknown(value),
        }
    }
}

impl From<SlotType> for u8 {
    fn from(value: SlotType) -> Self {
        match value {
            SlotType::Primary => 0x00,
            SlotType::SecondaryFine => 0x01,
            SlotType::SecondaryTiming => 0x02,
            SlotType::SecondarySpeed => 0x03,
            SlotType::SecondaryControl => 0x04,
            SlotType::SecondaryIndex => 0x05,
            SlotType::SecondaryRotation => 0x06,
            SlotType::SecondaryIndexRotate => 0x07,
            SlotType::SecondaryUndefined => 0xff,
            SlotType::Unknown(value) => value,
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

impl RdmParameterData for SlotInfo {
    fn size_of(&self) -> usize {
        5
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        buf[0..2].copy_from_slice(&self.id.to_be_bytes());
        buf[2] = self.r#type.into();
        buf[3..5].copy_from_slice(&self.label_id.to_be_bytes());
        Ok(5)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        Ok(SlotInfo {
            id: u16::from_be_bytes([buf[0], buf[1]]),
            r#type: SlotType::from(buf[2]),
            label_id: u16::from_be_bytes([buf[3], buf[4]]),
        })
    }
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum SlotIdDefinition {
    Intensity,
    IntensityMaster,
    Pan,
    Tilt,
    ColorWheel,
    ColorSubCyan,
    ColorSubYellow,
    ColorSubMagenta,
    ColorAddRed,
    ColorAddGreen,
    ColorAddBlue,
    ColorCorrection,
    ColorScroll,
    ColorSemaphore,
    StaticGoboWheel,
    RotoGoboWheel,
    PrismWheel,
    EffectsWheel,
    BeamSizeIris,
    Edge,
    Frost,
    Strobe,
    Zoom,
    FramingShutter,
    ShutterRotate,
    Douser,
    BarnDoor,
    LampControl,
    FixtureControl,
    FixtureSpeed,
    Macro,
    Undefined,
    ManufacturerSpecific(u16),
    Unknown(u16),
}

impl From<u16> for SlotIdDefinition {
    fn from(value: u16) -> Self {
        match value {
            0x0001 => Self::Intensity,
            0x0002 => Self::IntensityMaster,
            0x0101 => Self::Pan,
            0x0102 => Self::Tilt,
            0x0201 => Self::ColorWheel,
            0x0202 => Self::ColorSubCyan,
            0x0203 => Self::ColorSubYellow,
            0x0204 => Self::ColorSubMagenta,
            0x0205 => Self::ColorAddRed,
            0x0206 => Self::ColorAddGreen,
            0x0207 => Self::ColorAddBlue,
            0x0208 => Self::ColorCorrection,
            0x0209 => Self::ColorScroll,
            0x0210 => Self::ColorSemaphore,
            0x0301 => Self::StaticGoboWheel,
            0x0302 => Self::RotoGoboWheel,
            0x0303 => Self::PrismWheel,
            0x0304 => Self::EffectsWheel,
            0x0401 => Self::BeamSizeIris,
            0x0402 => Self::Edge,
            0x0403 => Self::Frost,
            0x0404 => Self::Strobe,
            0x0405 => Self::Zoom,
            0x0406 => Self::FramingShutter,
            0x0407 => Self::ShutterRotate,
            0x0408 => Self::Douser,
            0x0409 => Self::BarnDoor,
            0x0501 => Self::LampControl,
            0x0502 => Self::FixtureControl,
            0x0503 => Self::FixtureSpeed,
            0x0504 => Self::Macro,
            0xffff => Self::Undefined,
            value if (0x8000..=0xffdf).contains(&value) => Self::ManufacturerSpecific(value),
            value => Self::Unknown(value),
        }
    }
}

impl From<SlotIdDefinition> for u16 {
    fn from(value: SlotIdDefinition) -> Self {
        match value {
            SlotIdDefinition::Intensity => 0x0001,
            SlotIdDefinition::IntensityMaster => 0x0002,
            SlotIdDefinition::Pan => 0x0101,
            SlotIdDefinition::Tilt => 0x0102,
            SlotIdDefinition::ColorWheel => 0x0201,
            SlotIdDefinition::ColorSubCyan => 0x0202,
            SlotIdDefinition::ColorSubYellow => 0x0203,
            SlotIdDefinition::ColorSubMagenta => 0x0204,
            SlotIdDefinition::ColorAddRed => 0x0205,
            SlotIdDefinition::ColorAddGreen => 0x0206,
            SlotIdDefinition::ColorAddBlue => 0x0207,
            SlotIdDefinition::ColorCorrection => 0x0208,
            SlotIdDefinition::ColorScroll => 0x0209,
            SlotIdDefinition::ColorSemaphore => 0x0210,
            SlotIdDefinition::StaticGoboWheel => 0x0301,
            SlotIdDefinition::RotoGoboWheel => 0x0302,
            SlotIdDefinition::PrismWheel => 0x0303,
            SlotIdDefinition::EffectsWheel => 0x0304,
            SlotIdDefinition::BeamSizeIris => 0x0401,
            SlotIdDefinition::Edge => 0x0402,
            SlotIdDefinition::Frost => 0x0403,
            SlotIdDefinition::Strobe => 0x0404,
            SlotIdDefinition::Zoom => 0x0405,
            SlotIdDefinition::FramingShutter => 0x0406,
            SlotIdDefinition::ShutterRotate => 0x0407,
            SlotIdDefinition::Douser => 0x0408,
            SlotIdDefinition::BarnDoor => 0x0409,
            SlotIdDefinition::LampControl => 0x0501,
            SlotIdDefinition::FixtureControl => 0x0502,
            SlotIdDefinition::FixtureSpeed => 0x0503,
            SlotIdDefinition::Macro => 0x0504,
            SlotIdDefinition::Undefined => 0xffff,
            SlotIdDefinition::ManufacturerSpecific(value) => value,
            SlotIdDefinition::Unknown(value) => value,
        }
    }
}

impl core::fmt::Display for SlotIdDefinition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let definition = match self {
            Self::Intensity => "Intensity",
            Self::IntensityMaster => "Intensity Master",
            Self::Pan => "Pan",
            Self::Tilt => "Tilt",
            Self::ColorWheel => "Color Wheel",
            Self::ColorSubCyan => "Color Sub Cyan",
            Self::ColorSubYellow => "Color Sub Yellow",
            Self::ColorSubMagenta => "Color Sub Magenta",
            Self::ColorAddRed => "Color Add Red",
            Self::ColorAddGreen => "Color Add Green",
            Self::ColorAddBlue => "Color Add Blue",
            Self::ColorCorrection => "Color Correction",
            Self::ColorScroll => "Color Scroll",
            Self::ColorSemaphore => "Color Semaphore",
            Self::StaticGoboWheel => "Static Gobo Wheel",
            Self::RotoGoboWheel => "Roto Gobo Wheel",
            Self::PrismWheel => "Prism Wheel",
            Self::EffectsWheel => "Effects Wheel",
            Self::BeamSizeIris => "Beam Size Iris",
            Self::Edge => "Edge",
            Self::Frost => "Frost",
            Self::Strobe => "Strobe",
            Self::Zoom => "Zoom",
            Self::FramingShutter => "Framing Shutter",
            Self::ShutterRotate => "Shutter Rotate",
            Self::Douser => "Douser",
            Self::BarnDoor => "Barn Door",
            Self::LampControl => "Lamp Control",
            Self::FixtureControl => "Fixture Control",
            Self::FixtureSpeed => "Fixture Speed",
            Self::Macro => "Macro",
            Self::Undefined => "Undefined",
            Self::ManufacturerSpecific(value) => {
                return write!(f, "Manufacturer Specific: {value}");
            }
            Self::Unknown(value) => return write!(f, "Unknown Slot Id Definition: {value}"),
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

impl RdmParameterData for DefaultSlotValue {
    fn size_of(&self) -> usize {
        3
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        buf[0..2].copy_from_slice(&self.id.to_be_bytes());
        buf[2] = self.value;
        Ok(3)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        Ok(DefaultSlotValue {
            id: u16::from_be_bytes([buf[0], buf[1]]),
            value: buf[2],
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SensorType {
    Temperature,
    Voltage,
    Current,
    Frequency,
    Resistance,
    Power,
    Mass,
    Length,
    Area,
    Volume,
    Density,
    Velocity,
    Acceleration,
    Force,
    Energy,
    Pressure,
    Time,
    Angle,
    PositionX,
    PositionY,
    PositionZ,
    AngularVelocity,
    LuminousIntensity,
    LuminousFlux,
    Illuminance,
    ChrominanceRed,
    ChrominanceGreen,
    ChrominanceBlue,
    Contacts,
    Memory,
    Items,
    Humidity,
    Counter16Bit,
    Other,
    ManufacturerSpecific(u8),
}

impl TryFrom<u8> for SensorType {
    type Error = RdmError;
    fn try_from(value: u8) -> Result<Self, RdmError> {
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
            value if (0x80..=0xff).contains(&value) => Ok(Self::ManufacturerSpecific(value)),
            _ => Err(RdmError::InvalidSensorType(value)),
        }
    }
}

impl From<SensorType> for u8 {
    fn from(value: SensorType) -> Self {
        match value {
            SensorType::Temperature => 0x00,
            SensorType::Voltage => 0x01,
            SensorType::Current => 0x02,
            SensorType::Frequency => 0x03,
            SensorType::Resistance => 0x04,
            SensorType::Power => 0x05,
            SensorType::Mass => 0x06,
            SensorType::Length => 0x07,
            SensorType::Area => 0x08,
            SensorType::Volume => 0x09,
            SensorType::Density => 0x0a,
            SensorType::Velocity => 0x0b,
            SensorType::Acceleration => 0x0c,
            SensorType::Force => 0x0d,
            SensorType::Energy => 0x0e,
            SensorType::Pressure => 0x0f,
            SensorType::Time => 0x10,
            SensorType::Angle => 0x11,
            SensorType::PositionX => 0x12,
            SensorType::PositionY => 0x13,
            SensorType::PositionZ => 0x14,
            SensorType::AngularVelocity => 0x15,
            SensorType::LuminousIntensity => 0x16,
            SensorType::LuminousFlux => 0x17,
            SensorType::Illuminance => 0x18,
            SensorType::ChrominanceRed => 0x19,
            SensorType::ChrominanceGreen => 0x1a,
            SensorType::ChrominanceBlue => 0x1b,
            SensorType::Contacts => 0x1c,
            SensorType::Memory => 0x1d,
            SensorType::Items => 0x1e,
            SensorType::Humidity => 0x1f,
            SensorType::Counter16Bit => 0x20,
            SensorType::Other => 0x7f,
            SensorType::ManufacturerSpecific(value) => value,
        }
    }
}

impl RdmParameterData for SensorType {
    fn size_of(&self) -> usize {
        1
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        buf[0] = (*self).into();
        Ok(1)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        let sensor_type = Self::try_from(buf[0]).map_err(|_| ParameterCodecError::MalformedData)?;
        Ok(sensor_type)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SensorUnit {
    None,
    Centigrade,
    VoltsDc,
    VoltsAcPeak,
    VoltsAcRms,
    AmpsDc,
    AmpsAcPeak,
    AmpsAcRms,
    Hertz,
    Ohm,
    Watt,
    Kilogram,
    Meter,
    SquareMeter,
    CubicMeter,
    KilogramPerCubicMeter,
    MeterPerSecond,
    MeterPerSecondSquared,
    Newton,
    Joule,
    Pascal,
    Second,
    Degree,
    Steradian,
    Candela,
    Lumen,
    Lux,
    Ire,
    Byte,
    ManufacturerSpecific(u8),
}

impl TryFrom<u8> for SensorUnit {
    type Error = RdmError;

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
            value if (0x80..=0xff).contains(&value) => Ok(Self::ManufacturerSpecific(value)),
            _ => Err(RdmError::InvalidSensorUnit(value)),
        }
    }
}

impl From<SensorUnit> for u8 {
    fn from(value: SensorUnit) -> Self {
        match value {
            SensorUnit::None => 0x00,
            SensorUnit::Centigrade => 0x01,
            SensorUnit::VoltsDc => 0x02,
            SensorUnit::VoltsAcPeak => 0x03,
            SensorUnit::VoltsAcRms => 0x04,
            SensorUnit::AmpsDc => 0x05,
            SensorUnit::AmpsAcPeak => 0x06,
            SensorUnit::AmpsAcRms => 0x07,
            SensorUnit::Hertz => 0x08,
            SensorUnit::Ohm => 0x09,
            SensorUnit::Watt => 0x0a,
            SensorUnit::Kilogram => 0x0b,
            SensorUnit::Meter => 0x0c,
            SensorUnit::SquareMeter => 0x0d,
            SensorUnit::CubicMeter => 0x0e,
            SensorUnit::KilogramPerCubicMeter => 0x0f,
            SensorUnit::MeterPerSecond => 0x10,
            SensorUnit::MeterPerSecondSquared => 0x11,
            SensorUnit::Newton => 0x12,
            SensorUnit::Joule => 0x13,
            SensorUnit::Pascal => 0x14,
            SensorUnit::Second => 0x15,
            SensorUnit::Degree => 0x16,
            SensorUnit::Steradian => 0x17,
            SensorUnit::Candela => 0x18,
            SensorUnit::Lumen => 0x19,
            SensorUnit::Lux => 0x1a,
            SensorUnit::Ire => 0x1b,
            SensorUnit::Byte => 0x1c,
            SensorUnit::ManufacturerSpecific(value) => value,
        }
    }
}

impl RdmParameterData for SensorUnit {
    fn size_of(&self) -> usize {
        1
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        buf[0] = (*self).into();
        Ok(1)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        let sensor_unit = Self::try_from(buf[0]).map_err(|_| ParameterCodecError::MalformedData)?;
        Ok(sensor_unit)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
    type Error = RdmError;

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
            _ => Err(RdmError::InvalidSensorUnitPrefix(value)),
        }
    }
}

impl RdmParameterData for SensorUnitPrefix {
    fn size_of(&self) -> usize {
        1
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        buf[0] = (*self).clone() as u8;
        Ok(1)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        let sensor_unit_prefix =
            Self::try_from(buf[0]).map_err(|_| ParameterCodecError::MalformedData)?;
        Ok(sensor_unit_prefix)
    }
}

pub const SENSOR_DEFINITION_DESCRIPTION_MAX_LENGTH: usize = 32;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SensorDefinitionDescription(String<SENSOR_DEFINITION_DESCRIPTION_MAX_LENGTH>);

impl_rdm_string!(
    SensorDefinitionDescription,
    SENSOR_DEFINITION_DESCRIPTION_MAX_LENGTH
);

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

// ISO 639-1 Language Codes copied from https://github.com/AlbanMinassian/iso639
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Iso639_1 {
    Aa, // Afar
    Ab, // Abkhaz
    Ae, // Avestan
    Af, // Afrikaans
    Ak, // Akan
    Am, // Amharic
    An, // Aragonese
    Ar, // Arabic
    As, // Assamese
    Av, // Avaric
    Ay, // Aymara
    Az, // Azerbaijani
    Ba, // Bashkir
    Be, // Belarusian
    Bg, // Bulgarian
    Bh, // Bihari
    Bi, // Bislama
    Bm, // Bambara
    Bn, // Bengali, Bangla
    Bo, // Tibetan Standard, Tibetan, Central
    Br, // Breton
    Bs, // Bosnian
    Ca, // Catalan
    Ce, // Chechen
    Ch, // Chamorro
    Co, // Corsican
    Cr, // Cree
    Cs, // Czech
    Cu, // Old Church Slavonic, Church Slavonic, Old Bulgarian
    Cv, // Chuvash
    Cy, // Welsh
    Da, // Danish
    De, // German
    Dv, // Divehi, Dhivehi, Maldivian
    Dz, // Dzongkha
    Ee, // Ewe
    El, // Greek
    En, // English
    Eo, // Esperanto
    Es, // Spanish
    Et, // Estonian
    Eu, // Basque
    Fa, // Persian
    Ff, // Fula, Fulah, Pulaar, Pular
    Fi, // Finnish
    Fj, // Fijian
    Fo, // Faroese
    Fr, // French
    Fy, // Western Frisian
    Ga, // Irish
    Gd, // Scottish Gaelic, Gaelic
    Gl, // Galician
    Gn, // Guaraní
    Gu, // Gujarati
    Gv, // Manx
    Ha, // Hausa
    He, // Hebrew
    Hi, // Hindi
    Ho, // Hiri Motu
    Hr, // Croatian
    Ht, // Haitian, Haitian Creole
    Hu, // Hungarian
    Hy, // Armenian
    Hz, // Herero
    Ia, // Interlingua
    Id, // Indonesian
    Ie, // Interlingue
    Ig, // Igbo
    Ii, // Nuosu
    Ik, // Inupiaq
    Io, // Ido
    Is, // Icelandic
    It, // Italian
    Iu, // Inuktitut
    Ja, // Japanese
    Jv, // Javanese
    Ka, // Georgian
    Kg, // Kongo
    Ki, // Kikuyu, Gikuyu
    Kj, // Kwanyama, Kuanyama
    Kk, // Kazakh
    Kl, // Kalaallisut, Greenlandic
    Km, // Khmer
    Kn, // Kannada
    Ko, // Korean
    Kr, // Kanuri
    Ks, // Kashmiri
    Ku, // Kurdish
    Kv, // Komi
    Kw, // Cornish
    Ky, // Kyrgyz
    La, // Latin
    Lb, // Luxembourgish, Letzeburgesch
    Lg, // Ganda
    Li, // Limburgish, Limburgan, Limburger
    Ln, // Lingala
    Lo, // Lao
    Lt, // Lithuanian
    Lu, // Luba-Katanga
    Lv, // Latvian
    Mg, // Malagasy
    Mh, // Marshallese
    Mi, // Māori
    Mk, // Macedonian
    Ml, // Malayalam
    Mn, // Mongolian
    Mr, // Marathi
    Ms, // Malay
    Mt, // Maltese
    My, // Burmese
    Na, // Nauruan
    Nb, // Norwegian Bokmål
    Nd, // Northern Ndebele
    Ne, // Nepali
    Ng, // Ndonga
    Nl, // Dutch
    Nn, // Norwegian Nynorsk
    No, // Norwegian
    Nr, // Southern Ndebele
    Nv, // Navajo, Navaho
    Ny, // Chichewa, Chewa, Nyanja
    Oc, // Occitan
    Oj, // Ojibwe, Ojibwa
    Om, // Oromo
    Or, // Oriya
    Os, // Ossetian, Ossetic
    Pa, // Eastern Punjab
    Pi, // Pāli
    Pl, // Polish
    Ps, // Pashto, Pushto
    Pt, // Portuguese
    Qu, // Quechua
    Rm, // Romansh
    Rn, // Kirundi
    Ro, // Romanian
    Ru, // Russian
    Rw, // Kinyarwanda
    Sa, // Sanskrit
    Sc, // Sardinian
    Sd, // Sindhi
    Se, // Northern Sami
    Sg, // Sango
    Si, // Sinhalese, Sinhala
    Sk, // Slovak
    Sl, // Slovene
    Sm, // Samoan
    Sn, // Shona
    So, // Somali
    Sq, // Albanian
    Sr, // Serbian
    Ss, // Swati
    St, // Southern Sotho
    Su, // Sundanese
    Sv, // Swedish
    Sw, // Swahili
    Ta, // Tamil
    Te, // Telugu
    Tg, // Tajik
    Th, // Thai
    Ti, // Tigrinya
    Tk, // Turkmen
    Tl, // Tagalog
    Tn, // Tswana
    To, // Tonga
    Tr, // Turkish
    Ts, // Tsonga
    Tt, // Tatar
    Tw, // Twi
    Ty, // Tahitian
    Ug, // Uyghur
    Uk, // Ukrainian
    Ur, // Urdu
    Uz, // Uzbek
    Ve, // Venda
    Vi, // Vietnamese
    Vo, // Volapük
    Wa, // Walloon
    Wo, // Wolof
    Xh, // Xhosa
    Yi, // Yiddish
    Yo, // Yoruba
    Za, // Zhuang, Chuang
    Zh, // Chinese
    Zu, // Zulu
    Unsupported([u8; 2]),
}

impl Iso639_1 {
    pub const LENGTH: usize = 2;

    pub fn as_str(&self) -> &str {
        match self {
            Iso639_1::Aa => "aa",
            Iso639_1::Ab => "ab",
            Iso639_1::Ae => "ae",
            Iso639_1::Af => "af",
            Iso639_1::Ak => "ak",
            Iso639_1::Am => "am",
            Iso639_1::An => "an",
            Iso639_1::Ar => "ar",
            Iso639_1::As => "as",
            Iso639_1::Av => "av",
            Iso639_1::Ay => "ay",
            Iso639_1::Az => "az",
            Iso639_1::Ba => "ba",
            Iso639_1::Be => "be",
            Iso639_1::Bg => "bg",
            Iso639_1::Bh => "bh",
            Iso639_1::Bi => "bi",
            Iso639_1::Bm => "bm",
            Iso639_1::Bn => "bn",
            Iso639_1::Bo => "bo",
            Iso639_1::Br => "br",
            Iso639_1::Bs => "bs",
            Iso639_1::Ca => "ca",
            Iso639_1::Ce => "ce",
            Iso639_1::Ch => "ch",
            Iso639_1::Co => "co",
            Iso639_1::Cr => "cr",
            Iso639_1::Cs => "cs",
            Iso639_1::Cu => "cu",
            Iso639_1::Cv => "cv",
            Iso639_1::Cy => "cy",
            Iso639_1::Da => "da",
            Iso639_1::De => "de",
            Iso639_1::Dv => "dv",
            Iso639_1::Dz => "dz",
            Iso639_1::Ee => "ee",
            Iso639_1::El => "el",
            Iso639_1::En => "en",
            Iso639_1::Eo => "eo",
            Iso639_1::Es => "es",
            Iso639_1::Et => "et",
            Iso639_1::Eu => "eu",
            Iso639_1::Fa => "fa",
            Iso639_1::Ff => "ff",
            Iso639_1::Fi => "fi",
            Iso639_1::Fj => "fj",
            Iso639_1::Fo => "fo",
            Iso639_1::Fr => "fr",
            Iso639_1::Fy => "fy",
            Iso639_1::Ga => "ga",
            Iso639_1::Gd => "gd",
            Iso639_1::Gl => "gl",
            Iso639_1::Gn => "gn",
            Iso639_1::Gu => "gu",
            Iso639_1::Gv => "gv",
            Iso639_1::Ha => "ha",
            Iso639_1::He => "he",
            Iso639_1::Hi => "hi",
            Iso639_1::Ho => "ho",
            Iso639_1::Hr => "hr",
            Iso639_1::Ht => "ht",
            Iso639_1::Hu => "hu",
            Iso639_1::Hy => "hy",
            Iso639_1::Hz => "hz",
            Iso639_1::Ia => "ia",
            Iso639_1::Id => "id",
            Iso639_1::Ie => "ie",
            Iso639_1::Ig => "ig",
            Iso639_1::Ii => "ii",
            Iso639_1::Ik => "ik",
            Iso639_1::Io => "io",
            Iso639_1::Is => "is",
            Iso639_1::It => "it",
            Iso639_1::Iu => "iu",
            Iso639_1::Ja => "ja",
            Iso639_1::Jv => "jv",
            Iso639_1::Ka => "ka",
            Iso639_1::Kg => "kg",
            Iso639_1::Ki => "ki",
            Iso639_1::Kj => "kj",
            Iso639_1::Kk => "kk",
            Iso639_1::Kl => "kl",
            Iso639_1::Km => "km",
            Iso639_1::Kn => "kn",
            Iso639_1::Ko => "ko",
            Iso639_1::Kr => "kr",
            Iso639_1::Ks => "ks",
            Iso639_1::Ku => "ku",
            Iso639_1::Kv => "kv",
            Iso639_1::Kw => "kw",
            Iso639_1::Ky => "ky",
            Iso639_1::La => "la",
            Iso639_1::Lb => "lb",
            Iso639_1::Lg => "lg",
            Iso639_1::Li => "li",
            Iso639_1::Ln => "ln",
            Iso639_1::Lo => "lo",
            Iso639_1::Lt => "lt",
            Iso639_1::Lu => "lu",
            Iso639_1::Lv => "lv",
            Iso639_1::Mg => "mg",
            Iso639_1::Mh => "mh",
            Iso639_1::Mi => "mi",
            Iso639_1::Mk => "mk",
            Iso639_1::Ml => "ml",
            Iso639_1::Mn => "mn",
            Iso639_1::Mr => "mr",
            Iso639_1::Ms => "ms",
            Iso639_1::Mt => "mt",
            Iso639_1::My => "my",
            Iso639_1::Na => "na",
            Iso639_1::Nb => "nb",
            Iso639_1::Nd => "nd",
            Iso639_1::Ne => "ne",
            Iso639_1::Ng => "ng",
            Iso639_1::Nl => "nl",
            Iso639_1::Nn => "nn",
            Iso639_1::No => "no",
            Iso639_1::Nr => "nr",
            Iso639_1::Nv => "nv",
            Iso639_1::Ny => "ny",
            Iso639_1::Oc => "oc",
            Iso639_1::Oj => "oj",
            Iso639_1::Om => "om",
            Iso639_1::Or => "or",
            Iso639_1::Os => "os",
            Iso639_1::Pa => "pa",
            Iso639_1::Pi => "pi",
            Iso639_1::Pl => "pl",
            Iso639_1::Ps => "ps",
            Iso639_1::Pt => "pt",
            Iso639_1::Qu => "qu",
            Iso639_1::Rm => "rm",
            Iso639_1::Rn => "rn",
            Iso639_1::Ro => "ro",
            Iso639_1::Ru => "ru",
            Iso639_1::Rw => "rw",
            Iso639_1::Sa => "sa",
            Iso639_1::Sc => "sc",
            Iso639_1::Sd => "sd",
            Iso639_1::Se => "se",
            Iso639_1::Sg => "sg",
            Iso639_1::Si => "si",
            Iso639_1::Sk => "sk",
            Iso639_1::Sl => "sl",
            Iso639_1::Sm => "sm",
            Iso639_1::Sn => "sn",
            Iso639_1::So => "so",
            Iso639_1::Sq => "sq",
            Iso639_1::Sr => "sr",
            Iso639_1::Ss => "ss",
            Iso639_1::St => "st",
            Iso639_1::Su => "su",
            Iso639_1::Sv => "sv",
            Iso639_1::Sw => "sw",
            Iso639_1::Ta => "ta",
            Iso639_1::Te => "te",
            Iso639_1::Tg => "tg",
            Iso639_1::Th => "th",
            Iso639_1::Ti => "ti",
            Iso639_1::Tk => "tk",
            Iso639_1::Tl => "tl",
            Iso639_1::Tn => "tn",
            Iso639_1::To => "to",
            Iso639_1::Tr => "tr",
            Iso639_1::Ts => "ts",
            Iso639_1::Tt => "tt",
            Iso639_1::Tw => "tw",
            Iso639_1::Ty => "ty",
            Iso639_1::Ug => "ug",
            Iso639_1::Uk => "uk",
            Iso639_1::Ur => "ur",
            Iso639_1::Uz => "uz",
            Iso639_1::Ve => "ve",
            Iso639_1::Vi => "vi",
            Iso639_1::Vo => "vo",
            Iso639_1::Wa => "wa",
            Iso639_1::Wo => "wo",
            Iso639_1::Xh => "xh",
            Iso639_1::Yi => "yi",
            Iso639_1::Yo => "yo",
            Iso639_1::Za => "za",
            Iso639_1::Zh => "zh",
            Iso639_1::Zu => "zu",
            Iso639_1::Unsupported(value) => core::str::from_utf8(value).unwrap(),
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(code: &str) -> Self {
        match code {
            "aa" => Self::Aa,
            "ab" => Self::Ab,
            "ae" => Self::Ae,
            "af" => Self::Af,
            "ak" => Self::Ak,
            "am" => Self::Am,
            "an" => Self::An,
            "ar" => Self::Ar,
            "as" => Self::As,
            "av" => Self::Av,
            "ay" => Self::Ay,
            "az" => Self::Az,
            "ba" => Self::Ba,
            "be" => Self::Be,
            "bg" => Self::Bg,
            "bh" => Self::Bh,
            "bi" => Self::Bi,
            "bm" => Self::Bm,
            "bn" => Self::Bn,
            "bo" => Self::Bo,
            "br" => Self::Br,
            "bs" => Self::Bs,
            "ca" => Self::Ca,
            "ce" => Self::Ce,
            "ch" => Self::Ch,
            "co" => Self::Co,
            "cr" => Self::Cr,
            "cs" => Self::Cs,
            "cu" => Self::Cu,
            "cv" => Self::Cv,
            "cy" => Self::Cy,
            "da" => Self::Da,
            "de" => Self::De,
            "dv" => Self::Dv,
            "dz" => Self::Dz,
            "ee" => Self::Ee,
            "el" => Self::El,
            "en" => Self::En,
            "eo" => Self::Eo,
            "es" => Self::Es,
            "et" => Self::Et,
            "eu" => Self::Eu,
            "fa" => Self::Fa,
            "ff" => Self::Ff,
            "fi" => Self::Fi,
            "fj" => Self::Fj,
            "fo" => Self::Fo,
            "fr" => Self::Fr,
            "fy" => Self::Fy,
            "ga" => Self::Ga,
            "gd" => Self::Gd,
            "gl" => Self::Gl,
            "gn" => Self::Gn,
            "gu" => Self::Gu,
            "gv" => Self::Gv,
            "ha" => Self::Ha,
            "he" => Self::He,
            "hi" => Self::Hi,
            "ho" => Self::Ho,
            "hr" => Self::Hr,
            "ht" => Self::Ht,
            "hu" => Self::Hu,
            "hy" => Self::Hy,
            "hz" => Self::Hz,
            "ia" => Self::Ia,
            "id" => Self::Id,
            "ie" => Self::Ie,
            "ig" => Self::Ig,
            "ii" => Self::Ii,
            "ik" => Self::Ik,
            "io" => Self::Io,
            "is" => Self::Is,
            "it" => Self::It,
            "iu" => Self::Iu,
            "ja" => Self::Ja,
            "jv" => Self::Jv,
            "ka" => Self::Ka,
            "kg" => Self::Kg,
            "ki" => Self::Ki,
            "kj" => Self::Kj,
            "kk" => Self::Kk,
            "kl" => Self::Kl,
            "km" => Self::Km,
            "kn" => Self::Kn,
            "ko" => Self::Ko,
            "kr" => Self::Kr,
            "ks" => Self::Ks,
            "ku" => Self::Ku,
            "kv" => Self::Kv,
            "kw" => Self::Kw,
            "ky" => Self::Ky,
            "la" => Self::La,
            "lb" => Self::Lb,
            "lg" => Self::Lg,
            "li" => Self::Li,
            "ln" => Self::Ln,
            "lo" => Self::Lo,
            "lt" => Self::Lt,
            "lu" => Self::Lu,
            "lv" => Self::Lv,
            "mg" => Self::Mg,
            "mh" => Self::Mh,
            "mi" => Self::Mi,
            "mk" => Self::Mk,
            "ml" => Self::Ml,
            "mn" => Self::Mn,
            "mr" => Self::Mr,
            "ms" => Self::Ms,
            "mt" => Self::Mt,
            "my" => Self::My,
            "na" => Self::Na,
            "nb" => Self::Nb,
            "nd" => Self::Nd,
            "ne" => Self::Ne,
            "ng" => Self::Ng,
            "nl" => Self::Nl,
            "nn" => Self::Nn,
            "no" => Self::No,
            "nr" => Self::Nr,
            "nv" => Self::Nv,
            "ny" => Self::Ny,
            "oc" => Self::Oc,
            "oj" => Self::Oj,
            "om" => Self::Om,
            "or" => Self::Or,
            "os" => Self::Os,
            "pa" => Self::Pa,
            "pi" => Self::Pi,
            "pl" => Self::Pl,
            "ps" => Self::Ps,
            "pt" => Self::Pt,
            "qu" => Self::Qu,
            "rm" => Self::Rm,
            "rn" => Self::Rn,
            "ro" => Self::Ro,
            "ru" => Self::Ru,
            "rw" => Self::Rw,
            "sa" => Self::Sa,
            "sc" => Self::Sc,
            "sd" => Self::Sd,
            "se" => Self::Se,
            "sg" => Self::Sg,
            "si" => Self::Si,
            "sk" => Self::Sk,
            "sl" => Self::Sl,
            "sm" => Self::Sm,
            "sn" => Self::Sn,
            "so" => Self::So,
            "sq" => Self::Sq,
            "sr" => Self::Sr,
            "ss" => Self::Ss,
            "st" => Self::St,
            "su" => Self::Su,
            "sv" => Self::Sv,
            "sw" => Self::Sw,
            "ta" => Self::Ta,
            "te" => Self::Te,
            "tg" => Self::Tg,
            "th" => Self::Th,
            "ti" => Self::Ti,
            "tk" => Self::Tk,
            "tl" => Self::Tl,
            "tn" => Self::Tn,
            "to" => Self::To,
            "tr" => Self::Tr,
            "ts" => Self::Ts,
            "tt" => Self::Tt,
            "tw" => Self::Tw,
            "ty" => Self::Ty,
            "ug" => Self::Ug,
            "uk" => Self::Uk,
            "ur" => Self::Ur,
            "uz" => Self::Uz,
            "ve" => Self::Ve,
            "vi" => Self::Vi,
            "vo" => Self::Vo,
            "wa" => Self::Wa,
            "wo" => Self::Wo,
            "xh" => Self::Xh,
            "yi" => Self::Yi,
            "yo" => Self::Yo,
            "za" => Self::Za,
            "zh" => Self::Zh,
            "zu" => Self::Zu,
            value => {
                let value = value.as_bytes();
                Self::Unsupported([value[0], value[1]])
            }
        }
    }
}

impl RdmParameterData for Iso639_1 {
    fn size_of(&self) -> usize {
        Self::LENGTH
    }

    fn encode_rdm_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterCodecError> {
        if buf.len() < Self::LENGTH {
            return Err(ParameterCodecError::BufferTooSmall {
                provided: buf.len(),
                required: Self::LENGTH,
            });
        }

        buf[0..Self::LENGTH].copy_from_slice(self.as_str().as_bytes());

        Ok(Self::LENGTH)
    }

    fn decode_rdm_parameter_data(buf: &[u8]) -> Result<Self, ParameterCodecError> {
        if buf.len() < Self::LENGTH {
            return Err(ParameterCodecError::BufferTooSmall {
                provided: buf.len(),
                required: Self::LENGTH,
            });
        }

        let iso639_1 = core::str::from_utf8(&buf[0..Self::LENGTH])
            .map_err(|_| ParameterCodecError::MalformedData)?;
        Ok(Iso639_1::from_str(iso639_1))
    }
}

pub const SELF_TEST_DESCRIPTION_MAX_LENGTH: usize = 32;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SelfTestDescription(String<SELF_TEST_DESCRIPTION_MAX_LENGTH>);
impl_rdm_string!(SelfTestDescription, SELF_TEST_DESCRIPTION_MAX_LENGTH);

pub const LOCK_STATE_DESCRIPTION_MAX_LENGTH: usize = 32;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LockStateDescription(String<LOCK_STATE_DESCRIPTION_MAX_LENGTH>);
impl_rdm_string!(LockStateDescription, LOCK_STATE_DESCRIPTION_MAX_LENGTH);

pub const CURVE_DESCRIPTION_MAX_LENGTH: usize = 32;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CurveDescription(String<CURVE_DESCRIPTION_MAX_LENGTH>);
impl_rdm_string!(CurveDescription, CURVE_DESCRIPTION_MAX_LENGTH);

pub const OUTPUT_RESPONSE_TIME_DESCRIPTION_MAX_LENGTH: usize = 32;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OutputResponseTimeDescription(String<OUTPUT_RESPONSE_TIME_DESCRIPTION_MAX_LENGTH>);
impl_rdm_string!(
    OutputResponseTimeDescription,
    OUTPUT_RESPONSE_TIME_DESCRIPTION_MAX_LENGTH
);

pub const MODULATION_FREQUENCY_DESCRIPTION_MAX_LENGTH: usize = 32;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModulationFrequencyDescription(String<MODULATION_FREQUENCY_DESCRIPTION_MAX_LENGTH>);
impl_rdm_string!(
    ModulationFrequencyDescription,
    MODULATION_FREQUENCY_DESCRIPTION_MAX_LENGTH
);

#[derive(Clone, Debug, PartialEq, RdmDiscoveryResponseParameter)]
#[repr(C)]
pub struct DiscMuteResponse {
    pub control_field: u16,
    pub binding_uid: Option<DeviceUID>,
}

#[derive(Clone, Debug, PartialEq, RdmDiscoveryResponseParameter)]
#[repr(C)]
pub struct DiscUnMuteResponse {
    pub control_field: u16,
    pub binding_uid: Option<DeviceUID>,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmGetRequestParameter)]
#[repr(C, packed)]
pub struct GetQueuedMessageRequest {
    pub status_type: StatusType,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetProxiedDeviceCountResponse {
    pub device_count: u16,
    pub list_change: bool,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetProxiedDevicesResponse {
    pub device_uids: Vec<DeviceUID, 38>,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetCommsStatusResponse {
    pub short_message: u16,
    pub length_mismatch: u16,
    pub checksum_fail: u16,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmGetRequestParameter)]
#[repr(C, packed)]
pub struct GetStatusMessagesRequest {
    pub status_type: StatusType,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetStatusMessagesResponse {
    pub messages: Vec<StatusMessage, 25>,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmGetRequestParameter)]
#[repr(C)]
pub struct GetStatusIdDescriptionRequest {
    pub status_id: u16,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetStatusIdDescriptionResponse {
    pub status_id_description: StatusIdDescription,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetSubDeviceIdStatusReportThresholdResponse {
    pub status_type: StatusType,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C, packed)]
pub struct SetSubDeviceIdStatusReportThresholdRequest {
    pub status_type: StatusType,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetSupportedParametersResponse {
    pub supported_parameters: Vec<u16, 115>,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmGetRequestParameter)]
#[repr(C)]
pub struct GetParameterDescriptionRequest {
    pub parameter_id: u16,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetParameterDescriptionResponse {
    pub parameter_id: u16,
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

#[derive(Copy, Clone, Debug, PartialEq, RdmGetResponseParameter)]
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

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetProductDetailIdListResponse {
    pub product_detail_ids: Vec<ProductDetailValue, 115>,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetDeviceModelDescriptionResponse {
    pub device_model_description: DeviceModelDescription,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetManufacturerLabelResponse {
    pub manufacturer_label: ManufacturerLabel,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetDeviceLabelResponse {
    pub device_label: DeviceLabel,
}

#[derive(Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C)]
pub struct SetDeviceLabelRequest {
    pub device_label: DeviceLabel,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetFactoryDefaultsResponse {
    pub factory_reset: bool,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetLanguageResponse {
    pub language: Iso639_1,
}

#[derive(Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C)]
pub struct SetLanguageRequest {
    pub language: Iso639_1,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetLanguageCapabilitiesResponse {
    pub supported_languages: Vec<Iso639_1, 115>,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetSoftwareVersionLabelResponse {
    pub software_version_label: SoftwareVersionLabel,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetBootSoftwareVersionIdResponse {
    pub boot_software_version_id: u32,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetBootSoftwareVersionLabelResponse {
    pub boot_software_version_label: BootSoftwareVersionLabel,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetDmxPersonalityResponse {
    pub current_personality: u8,
    pub personality_count: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C, packed)]
pub struct SetDmxPersonalityRequest {
    pub personality_id: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmGetRequestParameter)]
#[repr(C, packed)]
pub struct GetDmxPersonalityDescriptionRequest {
    pub personality_id: u8,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetDmxPersonalityDescriptionResponse {
    pub id: u8,
    pub dmx_slots_required: u16,
    pub description: DmxPersonalityDescription,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetDmxStartAddressResponse {
    pub dmx_start_address: u16,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C)]
pub struct SetDmxStartAddressRequest {
    pub dmx_start_address: u16,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetSlotInfoResponse {
    pub slot_info: Vec<SlotInfo, 46>,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmGetRequestParameter)]
#[repr(C)]
pub struct GetSlotDescriptionRequest {
    pub slot_id: u16,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetSlotDescriptionResponse {
    pub slot_id: u16,
    pub description: SlotDescription,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetDefaultSlotValueResponse {
    pub default_values: Vec<DefaultSlotValue, 77>
}

#[derive(Copy, Clone, Debug, PartialEq, RdmGetRequestParameter)]
#[repr(C, packed)]
pub struct GetSensorDefinitionRequest {
    pub sensor_id: u8,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
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

#[derive(Copy, Clone, Debug, PartialEq, RdmGetRequestParameter)]
#[repr(C, packed)]
pub struct GetSensorValueRequest {
    pub sensor_id: u8,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetSensorValueResponse {
    pub sensor_id: u8,
    pub current_value: i16,
    pub lowest_detected_value: i16,
    pub highest_detected_value: i16,
    pub recorded_value: i16,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C, packed)]
pub struct SetSensorValueRequest {
    pub sensor_id: u8,
}

#[derive(Clone, Debug, PartialEq, RdmSetResponseParameter)]
#[repr(C)]
pub struct SetSensorValueResponse {
    pub sensor_id: u8,
    pub current_value: i16,
    pub lowest_detected_value: i16,
    pub highest_detected_value: i16,
    pub recorded_value: i16,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C, packed)]
pub struct SetRecordSensorsRequest {
    pub sensor_id: u8,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetDeviceHoursResponse {
    pub device_hours: u32
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C)]
pub struct SetDeviceHoursRequest {
    pub device_hours: u32,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetLampHoursResponse {
    pub lamp_hours: u32
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C)]
pub struct SetLampHoursRequest {
    pub lamp_hours: u32,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetLampStrikesResponse {
    pub lamp_strikes: u32
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C)]
pub struct SetLampStrikesRequest {
    pub lamp_strikes: u32,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetLampStateResponse {
    pub lamp_state: LampState,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C, packed)]
pub struct SetLampStateRequest {
    pub lamp_state: LampState,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetLampOnModeResponse {
    pub lamp_on_mode: LampOnMode,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C, packed)]
pub struct SetLampOnModeRequest {
    pub lamp_on_mode: LampOnMode,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetDevicePowerCyclesResponse {
    pub device_power_cycles: u32,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetDisplayInvertResponse {
    pub display_invert_mode: DisplayInvertMode,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetDisplayLevelResponse {
    pub display_level: u8,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetPanInvertResponse {
    pub pan_invert: bool,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetTiltInvertResponse {
    pub tilt_invert: bool,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetPanTiltSwapResponse {
    pub pan_tilt_swap: bool,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetIdentifyDeviceResponse {
    pub identify: bool,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetPowerStateResponse {
    pub power_state: PowerState,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C, packed)]
pub struct SetPowerStateRequest {
    pub power_state: PowerState,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetPerformSelfTestResponse {
    pub perform_self_test: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C, packed)]
pub struct SetPerformSelfTestRequest {
    pub self_test_id: SelfTest,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C)]
pub struct SetCapturePresetRequest {
    pub scene_id: u16,
    pub fade_times: Option<FadeTimes>,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmGetRequestParameter)]
#[repr(C, packed)]
pub struct GetSelfTestDescriptionRequest {
    pub self_test_id: SelfTest,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C)]
pub struct SetPresetPlaybackRequest {
    pub mode: PresetPlaybackMode,
    pub level: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C)]
pub struct SetDevicePowerCyclesRequest {
    pub device_power_cycles: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C)]
pub struct SetDisplayInvertModeRequest {
    pub display_invert_mode: DisplayInvertMode,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C, packed)]
pub struct SetDisplayLevelRequest {
    pub display_level: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C, packed)]
pub struct SetPanInvertRequest {
    pub pan_invert: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C, packed)]
pub struct SetTiltInvertRequest {
    pub tilt_invert: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C, packed)]
pub struct SetPanTiltSwapRequest {
    pub pan_tilt_swap: bool,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetRealTimeClock {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C)]
pub struct SetRealTimeClockRequest {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetSelfTestDescriptionResponse {
    pub self_test_id: SelfTest,
    pub description: SelfTestDescription,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C, packed)]
pub struct SetIdentifyDeviceRequest {
    pub identify: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, RdmSetRequestParameter)]
#[repr(C)]
pub struct SetResetDeviceRequest {
    pub reset_device: ResetDeviceMode,
}

#[derive(Clone, Debug, PartialEq, RdmGetResponseParameter)]
#[repr(C)]
pub struct GetPresetPlaybackResponse {
    pub mode: PresetPlaybackMode,
    pub level: u8,
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;
    use rdm_parameter_traits::{
        RdmDiscoveryResponseParameterCodec, RdmGetResponseParameterCodec,
        RdmSetRequestParameterCodec,
    };

    #[test]
    fn disc_mute_response() {
        use crate::rdm::DeviceUID;

        let param = super::DiscMuteResponse {
            control_field: 0x1234,
            binding_uid: None,
        };

        let mut buf = [0u8; 0x02];

        let encoded = param.discovery_response_encode_data(&mut buf).unwrap();
        assert_eq!(encoded, 2);
        assert_eq!(buf, [0x12, 0x34]);

        let decoded =
            super::DiscMuteResponse::discovery_response_decode_data(&buf[..encoded]).unwrap();
        assert_eq!(param, decoded);

        let param = super::DiscMuteResponse {
            control_field: 0x1234,
            binding_uid: Some(DeviceUID::from([0x01, 0x02, 0x03, 0x04, 0x05, 0x06])),
        };

        let mut buf = [0u8; 0x08];

        let encoded = param.discovery_response_encode_data(&mut buf).unwrap();
        assert_eq!(encoded, 8);
        assert_eq!(buf, [0x12, 0x34, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);

        let decoded =
            super::DiscMuteResponse::discovery_response_decode_data(&buf[..encoded]).unwrap();
        assert_eq!(param, decoded);
    }

    #[test]
    fn get_device_info_response() {
        use crate::rdm::parameter::e120::{ProductCategory, ProtocolVersion};

        let param = super::GetDeviceInfoResponse {
            protocol_version: ProtocolVersion::V1,
            device_model_id: 0x0002,
            product_category: ProductCategory::Dimmer,
            software_version_id: 0x00000004,
            dmx512_footprint: 0x0005,
            current_personality: 0x06,
            personality_count: 0x07,
            dmx512_start_address: 0x0008,
            sub_device_count: 0x0009,
            sensor_count: 0x0a,
        };

        let mut buf = [0u8; 0x13];

        let encoded = param.get_response_encode_data(&mut buf).unwrap();
        assert_eq!(encoded, 19);
        assert_eq!(
            buf,
            [1, 0, 0, 2, 5, 0, 0, 0, 0, 4, 0, 5, 6, 7, 0, 8, 0, 9, 0x0a]
        );

        let decoded = super::GetDeviceInfoResponse::get_response_decode_data(&buf).unwrap();
        assert_eq!(param, decoded);
    }

    #[test]
    fn set_device_label_request() {
        use crate::rdm::parameter::e120::DeviceLabel;

        let param = super::SetDeviceLabelRequest {
            device_label: DeviceLabel::from_str("Test").unwrap(),
        };

        let mut buf = [0u8; 4];

        let encoded = param.set_request_encode_data(&mut buf).unwrap();
        assert_eq!(encoded, 4);
        assert_eq!(buf, [b'T', b'e', b's', b't']);

        let decoded = super::SetDeviceLabelRequest::set_request_decode_data(&buf).unwrap();
        assert_eq!(param, decoded);
    }
}
