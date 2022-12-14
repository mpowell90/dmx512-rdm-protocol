// #![allow(unused)]
pub mod device;
pub mod parameter;

use std::mem;

use byteorder::{BigEndian, WriteBytesExt};
use ux::u48;

use self::device::DeviceUID;

pub const MIN_PACKET_LEN: usize = 26;

pub const SC_RDM: u8 = 0xcc;
pub const SC_SUB_MESSAGE: u8 = 0x01;

pub const BROADCAST_ALL_DEVICES_ID: u48 = u48::new(0xffffffffffff);
pub const SUB_DEVICE_ALL_CALL: u16 = 0xffff;
pub const ROOT_DEVICE: u8 = 0x00;

#[derive(Clone, Debug, Default)]
pub struct ManufacturerSpecificParameter {
    parameter_id: u16,
    parameter_data_size: Option<u8>, // TODO use enum
    data_type: Option<u8>,           // TODO use enum
    command_class: Option<SupportedCommandClasses>,
    prefix: Option<u8>, // TODO use enum
    minimum_valid_value: Option<u32>,
    maximum_valid_value: Option<u32>,
    default_value: Option<u32>,
    description: Option<String>,
}

impl From<u16> for ManufacturerSpecificParameter {
    fn from(parameter_id: u16) -> Self {
        ManufacturerSpecificParameter {
            parameter_id,
            ..Default::default()
        }
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

// TODO add remaining parameter ids
#[derive(Copy, Clone, Debug)]
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
    Curve = 0x0343,
    CurveDescription = 0x0344,
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
    ManfSpec1 = 0x8020,
    ManfSpec2 = 0x8038,
    ManfSpec3 = 0xFFDF,
}

// TODO this could use try_from and return a result rather than panic
impl From<&[u8]> for ParameterId {
    fn from(bytes: &[u8]) -> Self {
        match u16::from_be_bytes(bytes.try_into().unwrap()) {
            0x0001 => ParameterId::DiscUniqueBranch,
            0x0002 => ParameterId::DiscMute,
            0x0003 => ParameterId::DiscUnMute,
            0x0010 => ParameterId::ProxiedDevices,
            0x0011 => ParameterId::ProxiedDeviceCount,
            0x0015 => ParameterId::CommsStatus,
            0x0020 => ParameterId::QueuedMessage,
            0x0030 => ParameterId::StatusMessages,
            0x0031 => ParameterId::StatusIdDescription,
            0x0032 => ParameterId::ClearStatusId,
            0x0033 => ParameterId::SubDeviceStatusReportThreshold,
            0x0050 => ParameterId::SupportedParameters,
            0x0051 => ParameterId::ParameterDescription,
            0x0060 => ParameterId::DeviceInfo,
            0x0070 => ParameterId::ProductDetailIdList,
            0x0080 => ParameterId::DeviceModelDescription,
            0x0081 => ParameterId::ManufacturerLabel,
            0x0082 => ParameterId::DeviceLabel,
            0x0090 => ParameterId::FactoryDefaults,
            0x00a0 => ParameterId::LanguageCapabilities,
            0x00b0 => ParameterId::Language,
            0x00c0 => ParameterId::SoftwareVersionLabel,
            0x00c1 => ParameterId::BootSoftwareVersionId,
            0x00c2 => ParameterId::BootSoftwareVersionLabel,
            0x00e0 => ParameterId::DmxPersonality,
            0x00e1 => ParameterId::DmxPersonalityDescription,
            0x00f0 => ParameterId::DmxStartAddress,
            0x0120 => ParameterId::SlotInfo,
            0x0121 => ParameterId::SlotDescription,
            0x0122 => ParameterId::DefaultSlotValue,
            0x0200 => ParameterId::SensorDefinition,
            0x0201 => ParameterId::SensorValue,
            0x0202 => ParameterId::RecordSensors,
            0x0343 => ParameterId::Curve,
            0x0344 => ParameterId::CurveDescription,
            0x0347 => ParameterId::ModulationFrequency,
            0x0348 => ParameterId::ModulationFrequencyDescription,
            0x0400 => ParameterId::DeviceHours,
            0x0401 => ParameterId::LampHours,
            0x0402 => ParameterId::LampStrikes,
            0x0403 => ParameterId::LampState,
            0x0404 => ParameterId::LampOnMode,
            0x0405 => ParameterId::DevicePowerCycles,
            0x0500 => ParameterId::DisplayInvert,
            0x0501 => ParameterId::DisplayLevel,
            0x0600 => ParameterId::PanInvert,
            0x0601 => ParameterId::TiltInvert,
            0x0602 => ParameterId::PanTiltSwap,
            0x0603 => ParameterId::RealTimeClock,
            0x1000 => ParameterId::IdentifyDevice,
            0x1001 => ParameterId::ResetDevice,
            0x1010 => ParameterId::PowerState,
            0x1020 => ParameterId::PerformSelfTest,
            0x1021 => ParameterId::SelfTestDescription,
            0x1030 => ParameterId::CapturePreset,
            0x1031 => ParameterId::PresetPlayback,
            0x8020 => ParameterId::ManfSpec1, // TODO
            0x8038 => ParameterId::ManfSpec2, // TODO
            0xFFDF => ParameterId::ManfSpec3, // TODO
            _ => panic!("Invalid value for ParameterId: {:02X?}", bytes),
        }
    }
}

// TODO this could use try_from and return a result rather than panic
impl From<u16> for ParameterId {
    fn from(parameter_id: u16) -> Self {
        match parameter_id {
            0x0001 => ParameterId::DiscUniqueBranch,
            0x0002 => ParameterId::DiscMute,
            0x0003 => ParameterId::DiscUnMute,
            0x0010 => ParameterId::ProxiedDevices,
            0x0011 => ParameterId::ProxiedDeviceCount,
            0x0015 => ParameterId::CommsStatus,
            0x0020 => ParameterId::QueuedMessage,
            0x0030 => ParameterId::StatusMessages,
            0x0031 => ParameterId::StatusIdDescription,
            0x0032 => ParameterId::ClearStatusId,
            0x0033 => ParameterId::SubDeviceStatusReportThreshold,
            0x0050 => ParameterId::SupportedParameters,
            0x0051 => ParameterId::ParameterDescription,
            0x0060 => ParameterId::DeviceInfo,
            0x0070 => ParameterId::ProductDetailIdList,
            0x0080 => ParameterId::DeviceModelDescription,
            0x0081 => ParameterId::ManufacturerLabel,
            0x0082 => ParameterId::DeviceLabel,
            0x0090 => ParameterId::FactoryDefaults,
            0x00a0 => ParameterId::LanguageCapabilities,
            0x00b0 => ParameterId::Language,
            0x00c0 => ParameterId::SoftwareVersionLabel,
            0x00c1 => ParameterId::BootSoftwareVersionId,
            0x00c2 => ParameterId::BootSoftwareVersionLabel,
            0x00e0 => ParameterId::DmxPersonality,
            0x00e1 => ParameterId::DmxPersonalityDescription,
            0x00f0 => ParameterId::DmxStartAddress,
            0x0120 => ParameterId::SlotInfo,
            0x0121 => ParameterId::SlotDescription,
            0x0122 => ParameterId::DefaultSlotValue,
            0x0200 => ParameterId::SensorDefinition,
            0x0201 => ParameterId::SensorValue,
            0x0202 => ParameterId::RecordSensors,
            0x0343 => ParameterId::Curve,
            0x0344 => ParameterId::CurveDescription,
            0x0347 => ParameterId::ModulationFrequency,
            0x0348 => ParameterId::ModulationFrequencyDescription,
            0x0400 => ParameterId::DeviceHours,
            0x0401 => ParameterId::LampHours,
            0x0402 => ParameterId::LampStrikes,
            0x0403 => ParameterId::LampState,
            0x0404 => ParameterId::LampOnMode,
            0x0405 => ParameterId::DevicePowerCycles,
            0x0500 => ParameterId::DisplayInvert,
            0x0501 => ParameterId::DisplayLevel,
            0x0600 => ParameterId::PanInvert,
            0x0601 => ParameterId::TiltInvert,
            0x0602 => ParameterId::PanTiltSwap,
            0x0603 => ParameterId::RealTimeClock,
            0x1000 => ParameterId::IdentifyDevice,
            0x1001 => ParameterId::ResetDevice,
            0x1010 => ParameterId::PowerState,
            0x1020 => ParameterId::PerformSelfTest,
            0x1021 => ParameterId::SelfTestDescription,
            0x1030 => ParameterId::CapturePreset,
            0x1031 => ParameterId::PresetPlayback,
            0x8020 => ParameterId::ManfSpec1, // TODO
            0x8038 => ParameterId::ManfSpec2, // TODO
            0xFFDF => ParameterId::ManfSpec3, // TODO
            _ => panic!("Invalid value for ParameterId: {:02X?}", parameter_id),
        }
    }
}

fn bsd_16_crc(packet: &Vec<u8>) -> u16 {
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

pub enum LampState {
    LampOff = 0x00,
    LampOn = 0x01,
    LampStrike = 0x02,
    LampStandby = 0x03,
    LampNotPresent = 0x04,
    LampError = 0x05, // 0x00 = "Lamp Off",
                      // 0x01 = "Lamp On",
                      // 0x02 = "Lamp Strike",
                      // 0x03 = "Lamp Standby",
                      // 0x04 = "Lamp Not Present",
                      // 0x05 = "Lamp Error",
}

pub enum LampOnMode {
    OffMode = 0x00,
    DmxMode = 0x01,
    OnMode = 0x02,
    AfterCal = 0x03,
    // 0x00 = "Off Mode",
    // 0x01 = "DMX Mode",
    // 0x02 = "On Mode",
    // 0x03 = "After Cal",
}

pub enum PowerStates {
    FullOff = 0x00,
    Shutdown = 0x01,
    Standby = 0x02,
    Normal = 0xff, // 0x00 = "Full Off",
                   // 0x01 = "Shutdown",
                   // 0x02 = "Standby",
                   // 0xff = "Normal",
}

pub enum CommandClasses {
    Get = 0x01,
    Set = 0x02,
    GetSet = 0x03,
    // 0x01 = "Get",
    // 0x02 = "Set",
    // 0x03 = "Get / Set",
}

pub enum OnOffStates {
    Off = 0x00,
    On = 0x01,
    // 0x00 = "Off",
    // 0x01 = "On",
}

pub enum DisplayInvertModes {
    Off = 0x00,
    On = 0x01,
    Auto = 0x02,
    // 0x00 = "Off",
    // 0x01 = "On",
    // 0x02 = "Auto",
}

#[derive(Clone, Debug)]
pub struct Request<T> {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub port_id: u8,
    pub sub_device: u16,
    pub command_class: CommandClass,
    pub parameter_id: ParameterId,
    pub parameter_data: Option<T>,
}

impl<'a, T> From<Request<T>> for Vec<u8>
where
    Vec<u8>: From<T>,
    // &'a [u8]: From<T>,
{
    fn from(request: Request<T>) -> Vec<u8> {
        let (parameter_data, parameter_data_len) = if let Some(data) = request.parameter_data {
            let data: Vec<u8> = data.try_into().unwrap();
            let len = data.len() as u8;
            (data, len as u8)
        } else {
            (Vec::new(), 0)
        };

        // let parameter_data: Vec<u8> = request.parameter_data.try_into().unwrap();
        // let parameter_data_len = parameter_data.len() as u8;

        let mut packet = Vec::new();
        packet.write_u8(SC_RDM).unwrap(); // Start Code
        packet.write_u8(SC_SUB_MESSAGE).unwrap(); // Sub Start Code
        packet.write_u8(24_u8 + parameter_data_len).unwrap(); // Message Length: Range 24 to 255 excluding the checksum
        packet
            .write_u48::<BigEndian>(request.destination_uid.into())
            .unwrap();
        packet
            .write_u48::<BigEndian>(request.source_uid.into())
            .unwrap();
        packet.write_u8(request.transaction_number).unwrap(); // Transaction Number
        packet.write_u8(request.port_id).unwrap(); // Port Id / Response Type
        packet.write_u8(0x00).unwrap(); // Message Count
        packet.write_u16::<BigEndian>(request.sub_device).unwrap(); // Sub Device
        packet.write_u8(request.command_class as u8).unwrap();
        packet
            .write_u16::<BigEndian>(request.parameter_id as u16)
            .unwrap();

        packet.write_u8(parameter_data_len).unwrap();

        if parameter_data_len > 0 {
            packet.extend(parameter_data);
        }

        packet.write_u16::<BigEndian>(bsd_16_crc(&packet)).unwrap();
        packet
    }
}

#[derive(Clone, Debug)]
pub struct Response<T> {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub response_type: ResponseType,
    pub message_count: u8,
    pub sub_device: u16,
    pub command_class: CommandClass,
    pub parameter_id: ParameterId,
    pub parameter_data_length: u8,
    pub parameter_data: Option<T>,
}

// impl From<Vec<u8>> for Response<T> {

// }

impl<T> Response<T> {
    // Packet Format
    // [0] Start Code = 1 byte
    // [1] Sub Start Code = 1 byte
    // [2] Message Length = 1 byte
    // [3-8] Destination UID = 6 bytes (48 bit)
    // [9-14] Source UID = 6 bytes (48 bit)
    // [15] Transaction Number (TN) = 1 byte
    // [16] Port ID / Response Type = 1 byte
    // [17] Message Count = 1 byte
    // [18-19] Sub-Device = 2 bytes
    // [20] Command-Class = 1 byte
    // [21-22] Parameter ID = 2 bytes
    // [23] Parameter Data Length = 1 byte
    // [24-N] Parameter Data = Variable Length
    // [N-N+2] Checksum = 2 bytes
    fn parse_packet(packet: &[u8], parameter_data: Option<T>) -> Response<T> {
        Response {
            destination_uid: DeviceUID::from(&packet[3..=8]),
            source_uid: DeviceUID::from(&packet[9..=14]),
            transaction_number: u8::from_be(packet[15]),
            response_type: ResponseType::try_from(packet[16]).unwrap(),
            message_count: u8::from_be(packet[17]),
            sub_device: u16::from_be_bytes(packet[18..=19].try_into().unwrap()),
            command_class: CommandClass::try_from(packet[20]).unwrap(),
            parameter_id: ParameterId::from(&packet[21..=22]),
            parameter_data_length: u8::from_be(packet[23]),
            parameter_data: parameter_data,
        }
    }
}

pub trait Protocol
where
    Self: Sized,
{
    fn parameter_id() -> ParameterId;

    fn create_request(
        destination_uid: DeviceUID,
        source_uid: DeviceUID,
        transaction_number: u8,
        port_id: u8,
        command_class: CommandClass,
        sub_device: u16,
        parameter_data: Self,
    ) -> Request<Self> {
        let parameter_data = if mem::size_of::<Self>() > 0 {
            Some(parameter_data)
        } else {
            None
        };
        Request {
            destination_uid,
            source_uid,
            transaction_number,
            port_id,
            sub_device,
            command_class,
            parameter_id: Self::parameter_id(),
            parameter_data,
        }
    }

    // TODO how to best handle errors
    fn parse_response(packet: Vec<u8>) -> Result<Response<Self>, &'static str>
    where
        Self: From<Vec<u8>>,
    {
        let parameter_data_length = packet[23];
        let parameter_data: Option<Self> = if parameter_data_length > 0 {

            let data = packet[24..packet.len()-3].to_vec();
            // let data = packet[24..(parameter_data_length as usize - 3)].to_vec();
            println!("DATA: {:02X?}", data);
            Some(data.into())
        } else {
            None
        };

        Ok(Response {
            destination_uid: DeviceUID::from(&packet[3..=8]),
            source_uid: DeviceUID::from(&packet[9..=14]),
            transaction_number: u8::from_be(packet[15]),
            response_type: ResponseType::try_from(packet[16]).unwrap(),
            message_count: u8::from_be(packet[17]),
            sub_device: u16::from_be_bytes(packet[18..=19].try_into().unwrap()),
            command_class: CommandClass::try_from(packet[20]).unwrap(),
            parameter_id: ParameterId::from(&packet[21..=22]),
            parameter_data_length,
            parameter_data, // parameter_data: packet[24..(24 + packet[23] as usize)]
                            //     .to_vec()
                            //     .try_into()
                            //     .ok(),
        })
    }
}

pub trait GetRequest
where
    Self: Protocol,
{
    fn get_request(
        self,
        destination_uid: DeviceUID,
        source_uid: DeviceUID,
        transaction_number: u8,
        port_id: u8,
        sub_device: u16,
    ) -> Request<Self> {
        Self::create_request(
            destination_uid,
            source_uid,
            transaction_number,
            port_id,
            CommandClass::GetCommand,
            sub_device,
            self,
        )
    }
}

pub trait SetRequest
where
    Self: Protocol,
{
    fn set_request(
        self,
        destination_uid: DeviceUID,
        source_uid: DeviceUID,
        transaction_number: u8,
        port_id: u8,
        sub_device: u16,
    ) -> Request<Self> {
        Self::create_request(
            destination_uid,
            source_uid,
            transaction_number,
            port_id,
            CommandClass::SetCommand,
            sub_device,
            self,
        )
    }
}

pub trait DiscoveryRequest
where
    Self: Protocol,
{
    fn discovery_request(
        self,
        destination_uid: DeviceUID,
        source_uid: DeviceUID,
        transaction_number: u8,
        port_id: u8,
        sub_device: u16,
    ) -> Request<Self> {
        Self::create_request(
            destination_uid,
            source_uid,
            transaction_number,
            port_id,
            CommandClass::DiscoveryCommand,
            sub_device,
            self,
        )
    }
}
