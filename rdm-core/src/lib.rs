#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

// Allows using std types when desired, while keeping the crate `no_std`.
#[cfg(feature = "std")]
extern crate std;

pub mod error;
pub mod parameter_traits;
pub mod request;
pub mod response;
pub mod utils;

use crate::{error::ParameterDataError, parameter_traits::RdmParameterData};
use core::time::Duration;

pub const DISCOVERY_COMMAND: u8 = 0x10;
pub const DISCOVERY_RESPONSE_COMMAND: u8 = 0x11;
pub const GET_COMMAND: u8 = 0x20;
pub const GET_RESPONSE_COMMAND: u8 = 0x21;
pub const SET_COMMAND: u8 = 0x30;
pub const SET_RESPONSE_COMMAND: u8 = 0x31;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RdmFrameKind {
    Request,
    Response,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CommandClass {
    Discovery = 0x10,
    DiscoveryResponse = 0x11,
    Get = 0x20,
    GetResponse = 0x21,
    Set = 0x30,
    SetResponse = 0x31,
}

impl CommandClass {
    pub fn rdm_frame_kind(&self) -> RdmFrameKind {
        match self {
            CommandClass::Discovery | CommandClass::Get | CommandClass::Set => {
                RdmFrameKind::Request
            }
            CommandClass::DiscoveryResponse
            | CommandClass::GetResponse
            | CommandClass::SetResponse => RdmFrameKind::Response,
        }
    }
}

impl TryFrom<u8> for CommandClass {
    type Error = ParameterDataError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x10 => Ok(Self::Discovery),
            0x11 => Ok(Self::DiscoveryResponse),
            0x20 => Ok(Self::Get),
            0x21 => Ok(Self::GetResponse),
            0x30 => Ok(Self::Set),
            0x31 => Ok(Self::SetResponse),
            _ => Err(ParameterDataError::MalformedData), // TODO consider a better error type
        }
    }
}

impl From<CommandClass> for u8 {
    fn from(command_class: CommandClass) -> Self {
        command_class as u8
    }
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ParameterId {
    // E1.20
    DiscUniqueBranch,
    DiscMute,
    DiscUnMute,
    ProxiedDevices,
    ProxiedDeviceCount,
    CommsStatus,
    QueuedMessage,
    StatusMessages,
    StatusIdDescription,
    ClearStatusId,
    SubDeviceIdStatusReportThreshold,
    SupportedParameters,
    ParameterDescription,
    DeviceInfo,
    ProductDetailIdList,
    DeviceModelDescription,
    ManufacturerLabel,
    DeviceLabel,
    FactoryDefaults,
    LanguageCapabilities,
    Language,
    SoftwareVersionLabel,
    BootSoftwareVersionId,
    BootSoftwareVersionLabel,
    DmxPersonality,
    DmxPersonalityDescription,
    DmxStartAddress,
    SlotInfo,
    SlotDescription,
    DefaultSlotValue,
    SensorDefinition,
    SensorValue,
    RecordSensors,
    DeviceHours,
    LampHours,
    LampStrikes,
    LampState,
    LampOnMode,
    DevicePowerCycles,
    DisplayInvert,
    DisplayLevel,
    PanInvert,
    TiltInvert,
    PanTiltSwap,
    RealTimeClock,
    IdentifyDevice,
    ResetDevice,
    PowerState,
    PerformSelfTest,
    SelfTestDescription,
    CapturePreset,
    PresetPlayback,
    // E1.37-1
    DmxBlockAddress,
    DmxFailMode,
    DmxStartupMode,
    DimmerInfo,
    MinimumLevel,
    MaximumLevel,
    Curve,
    CurveDescription,
    OutputResponseTime,
    OutputResponseTimeDescription,
    ModulationFrequency,
    ModulationFrequencyDescription,
    BurnIn,
    LockPin,
    LockState,
    LockStateDescription,
    IdentifyMode,
    PresetInfo,
    PresetStatus,
    PresetMergeMode,
    PowerOnSelfTest,
    // E1.37-2
    ListInterfaces,
    InterfaceLabel,
    InterfaceHardwareAddressType1,
    IpV4DhcpMode,
    IpV4ZeroConfMode,
    IpV4CurrentAddress,
    IpV4StaticAddress,
    InterfaceRenewDhcp,
    InterfaceReleaseDhcp,
    InterfaceApplyConfiguration,
    IpV4DefaultRoute,
    DnsIpV4NameServer,
    DnsHostName,
    DnsDomainName,
    // E1.37-7
    EndpointList,
    EndpointListChange,
    IdentifyEndpoint,
    EndpointToUniverse,
    EndpointMode,
    EndpointLabel,
    RdmTrafficEnable,
    DiscoveryState,
    BackgroundDiscovery,
    EndpointTiming,
    EndpointTimingDescription,
    EndpointResponders,
    EndpointResponderListChange,
    BindingControlFields,
    BackgroundQueuedStatusPolicy,
    BackgroundQueuedStatusPolicyDescription,
    // E1.33
    ComponentScope,
    SearchDomain,
    TcpCommsStatus,
    BrokerStatus,
    // use for unsupported standard and manufacturer specific parameters
    Custom(u16),
}

impl From<u16> for ParameterId {
    fn from(value: u16) -> Self {
        match value {
            // E1.20
            0x0001 => Self::DiscUniqueBranch,
            0x0002 => Self::DiscMute,
            0x0003 => Self::DiscUnMute,
            0x0010 => Self::ProxiedDevices,
            0x0011 => Self::ProxiedDeviceCount,
            0x0015 => Self::CommsStatus,
            0x0020 => Self::QueuedMessage,
            0x0030 => Self::StatusMessages,
            0x0031 => Self::StatusIdDescription,
            0x0032 => Self::ClearStatusId,
            0x0033 => Self::SubDeviceIdStatusReportThreshold,
            0x0050 => Self::SupportedParameters,
            0x0051 => Self::ParameterDescription,
            0x0060 => Self::DeviceInfo,
            0x0070 => Self::ProductDetailIdList,
            0x0080 => Self::DeviceModelDescription,
            0x0081 => Self::ManufacturerLabel,
            0x0082 => Self::DeviceLabel,
            0x0090 => Self::FactoryDefaults,
            0x00a0 => Self::LanguageCapabilities,
            0x00b0 => Self::Language,
            0x00c0 => Self::SoftwareVersionLabel,
            0x00c1 => Self::BootSoftwareVersionId,
            0x00c2 => Self::BootSoftwareVersionLabel,
            0x00e0 => Self::DmxPersonality,
            0x00e1 => Self::DmxPersonalityDescription,
            0x00f0 => Self::DmxStartAddress,
            0x0120 => Self::SlotInfo,
            0x0121 => Self::SlotDescription,
            0x0122 => Self::DefaultSlotValue,
            0x0200 => Self::SensorDefinition,
            0x0201 => Self::SensorValue,
            0x0202 => Self::RecordSensors,
            0x0400 => Self::DeviceHours,
            0x0401 => Self::LampHours,
            0x0402 => Self::LampStrikes,
            0x0403 => Self::LampState,
            0x0404 => Self::LampOnMode,
            0x0405 => Self::DevicePowerCycles,
            0x0500 => Self::DisplayInvert,
            0x0501 => Self::DisplayLevel,
            0x0600 => Self::PanInvert,
            0x0601 => Self::TiltInvert,
            0x0602 => Self::PanTiltSwap,
            0x0603 => Self::RealTimeClock,
            0x1000 => Self::IdentifyDevice,
            0x1001 => Self::ResetDevice,
            0x1010 => Self::PowerState,
            0x1020 => Self::PerformSelfTest,
            0x1021 => Self::SelfTestDescription,
            0x1030 => Self::CapturePreset,
            0x1031 => Self::PresetPlayback,
            // E1.37-1
            0x0140 => Self::DmxBlockAddress,
            0x0141 => Self::DmxFailMode,
            0x0142 => Self::DmxStartupMode,
            0x0340 => Self::DimmerInfo,
            0x0341 => Self::MinimumLevel,
            0x0342 => Self::MaximumLevel,
            0x0343 => Self::Curve,
            0x0344 => Self::CurveDescription,
            0x0345 => Self::OutputResponseTime,
            0x0346 => Self::OutputResponseTimeDescription,
            0x0347 => Self::ModulationFrequency,
            0x0348 => Self::ModulationFrequencyDescription,
            0x0440 => Self::BurnIn,
            0x0640 => Self::LockPin,
            0x0641 => Self::LockState,
            0x0642 => Self::LockStateDescription,
            0x1040 => Self::IdentifyMode,
            0x1041 => Self::PresetInfo,
            0x1042 => Self::PresetStatus,
            0x1043 => Self::PresetMergeMode,
            0x1044 => Self::PowerOnSelfTest,
            // E1.37-2
            0x0700 => Self::ListInterfaces,
            0x0701 => Self::InterfaceLabel,
            0x0702 => Self::InterfaceHardwareAddressType1,
            0x0703 => Self::IpV4DhcpMode,
            0x0704 => Self::IpV4ZeroConfMode,
            0x0705 => Self::IpV4CurrentAddress,
            0x0706 => Self::IpV4StaticAddress,
            0x0707 => Self::InterfaceRenewDhcp,
            0x0708 => Self::InterfaceReleaseDhcp,
            0x0709 => Self::InterfaceApplyConfiguration,
            0x070a => Self::IpV4DefaultRoute,
            0x070b => Self::DnsIpV4NameServer,
            0x070c => Self::DnsHostName,
            0x070d => Self::DnsDomainName,
            // E1.37-7
            0x0900 => Self::EndpointList,
            0x0901 => Self::EndpointListChange,
            0x0902 => Self::IdentifyEndpoint,
            0x0903 => Self::EndpointToUniverse,
            0x0904 => Self::EndpointMode,
            0x0905 => Self::EndpointLabel,
            0x0906 => Self::RdmTrafficEnable,
            0x0907 => Self::DiscoveryState,
            0x0908 => Self::BackgroundDiscovery,
            0x0909 => Self::EndpointTiming,
            0x090a => Self::EndpointTimingDescription,
            0x090b => Self::EndpointResponders,
            0x090c => Self::EndpointResponderListChange,
            0x090d => Self::BindingControlFields,
            0x090e => Self::BackgroundQueuedStatusPolicy,
            0x090f => Self::BackgroundQueuedStatusPolicyDescription,
            // E1.33
            0x8000 => Self::ComponentScope,
            0x8001 => Self::SearchDomain,
            0x8002 => Self::TcpCommsStatus,
            0x8003 => Self::BrokerStatus,
            pid => Self::Custom(pid),
        }
    }
}

impl From<ParameterId> for u16 {
    fn from(value: ParameterId) -> Self {
        match value {
            // E1.20
            ParameterId::DiscUniqueBranch => 0x0001,
            ParameterId::DiscMute => 0x0002,
            ParameterId::DiscUnMute => 0x0003,
            ParameterId::ProxiedDevices => 0x0010,
            ParameterId::ProxiedDeviceCount => 0x0011,
            ParameterId::CommsStatus => 0x0015,
            ParameterId::QueuedMessage => 0x0020,
            ParameterId::StatusMessages => 0x0030,
            ParameterId::StatusIdDescription => 0x0031,
            ParameterId::ClearStatusId => 0x0032,
            ParameterId::SubDeviceIdStatusReportThreshold => 0x0033,
            ParameterId::SupportedParameters => 0x0050,
            ParameterId::ParameterDescription => 0x0051,
            ParameterId::DeviceInfo => 0x0060,
            ParameterId::ProductDetailIdList => 0x0070,
            ParameterId::DeviceModelDescription => 0x0080,
            ParameterId::ManufacturerLabel => 0x0081,
            ParameterId::DeviceLabel => 0x0082,
            ParameterId::FactoryDefaults => 0x0090,
            ParameterId::LanguageCapabilities => 0x00a0,
            ParameterId::Language => 0x00b0,
            ParameterId::SoftwareVersionLabel => 0x00c0,
            ParameterId::BootSoftwareVersionId => 0x00c1,
            ParameterId::BootSoftwareVersionLabel => 0x00c2,
            ParameterId::DmxPersonality => 0x00e0,
            ParameterId::DmxPersonalityDescription => 0x00e1,
            ParameterId::DmxStartAddress => 0x00f0,
            ParameterId::SlotInfo => 0x0120,
            ParameterId::SlotDescription => 0x0121,
            ParameterId::DefaultSlotValue => 0x0122,
            ParameterId::SensorDefinition => 0x0200,
            ParameterId::SensorValue => 0x0201,
            ParameterId::RecordSensors => 0x0202,
            ParameterId::DeviceHours => 0x0400,
            ParameterId::LampHours => 0x0401,
            ParameterId::LampStrikes => 0x0402,
            ParameterId::LampState => 0x0403,
            ParameterId::LampOnMode => 0x0404,
            ParameterId::DevicePowerCycles => 0x0405,
            ParameterId::DisplayInvert => 0x0500,
            ParameterId::DisplayLevel => 0x0501,
            ParameterId::PanInvert => 0x0600,
            ParameterId::TiltInvert => 0x0601,
            ParameterId::PanTiltSwap => 0x0602,
            ParameterId::RealTimeClock => 0x0603,
            ParameterId::IdentifyDevice => 0x1000,
            ParameterId::ResetDevice => 0x1001,
            ParameterId::PowerState => 0x1010,
            ParameterId::PerformSelfTest => 0x1020,
            ParameterId::SelfTestDescription => 0x1021,
            ParameterId::CapturePreset => 0x1030,
            ParameterId::PresetPlayback => 0x1031,
            // E1.37-1
            ParameterId::DmxBlockAddress => 0x0140,
            ParameterId::DmxFailMode => 0x0141,
            ParameterId::DmxStartupMode => 0x0142,
            ParameterId::DimmerInfo => 0x0340,
            ParameterId::MinimumLevel => 0x0341,
            ParameterId::MaximumLevel => 0x0342,
            ParameterId::Curve => 0x0343,
            ParameterId::CurveDescription => 0x0344,
            ParameterId::OutputResponseTime => 0x0345,
            ParameterId::OutputResponseTimeDescription => 0x0346,
            ParameterId::ModulationFrequency => 0x0347,
            ParameterId::ModulationFrequencyDescription => 0x0348,
            ParameterId::BurnIn => 0x0440,
            ParameterId::LockPin => 0x0640,
            ParameterId::LockState => 0x0641,
            ParameterId::LockStateDescription => 0x0642,
            ParameterId::IdentifyMode => 0x1040,
            ParameterId::PresetInfo => 0x1041,
            ParameterId::PresetStatus => 0x1042,
            ParameterId::PresetMergeMode => 0x1043,
            ParameterId::PowerOnSelfTest => 0x1044,
            // E1.37-2
            ParameterId::ListInterfaces => 0x0700,
            ParameterId::InterfaceLabel => 0x0701,
            ParameterId::InterfaceHardwareAddressType1 => 0x0702,
            ParameterId::IpV4DhcpMode => 0x0703,
            ParameterId::IpV4ZeroConfMode => 0x0704,
            ParameterId::IpV4CurrentAddress => 0x0705,
            ParameterId::IpV4StaticAddress => 0x0706,
            ParameterId::InterfaceRenewDhcp => 0x0707,
            ParameterId::InterfaceReleaseDhcp => 0x0708,
            ParameterId::InterfaceApplyConfiguration => 0x0709,
            ParameterId::IpV4DefaultRoute => 0x070a,
            ParameterId::DnsIpV4NameServer => 0x070b,
            ParameterId::DnsHostName => 0x070c,
            ParameterId::DnsDomainName => 0x070d,
            // E1.37-7
            ParameterId::EndpointList => 0x0900,
            ParameterId::EndpointListChange => 0x0901,
            ParameterId::IdentifyEndpoint => 0x0902,
            ParameterId::EndpointToUniverse => 0x0903,
            ParameterId::EndpointMode => 0x0904,
            ParameterId::EndpointLabel => 0x0905,
            ParameterId::RdmTrafficEnable => 0x0906,
            ParameterId::DiscoveryState => 0x0907,
            ParameterId::BackgroundDiscovery => 0x0908,
            ParameterId::EndpointTiming => 0x0909,
            ParameterId::EndpointTimingDescription => 0x090a,
            ParameterId::EndpointResponders => 0x090b,
            ParameterId::EndpointResponderListChange => 0x090c,
            ParameterId::BindingControlFields => 0x090d,
            ParameterId::BackgroundQueuedStatusPolicy => 0x090e,
            ParameterId::BackgroundQueuedStatusPolicyDescription => 0x090f,
            // E1.33
            ParameterId::ComponentScope => 0x0800,
            ParameterId::SearchDomain => 0x0801,
            ParameterId::TcpCommsStatus => 0x0802,
            ParameterId::BrokerStatus => 0x0803,
            // Use for unsupported standard or manufacturer specific PID's
            ParameterId::Custom(pid) => pid,
        }
    }
}

impl RdmParameterData for ParameterId {
    fn size_of(&self) -> usize {
        2
    }

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterDataError> {
        if buf.len() < 2 {
            return Err(ParameterDataError::BufferTooSmall {
                provided: buf.len(),
                required: 2,
            });
        }

        buf[0..2].copy_from_slice(&u16::from(*self).to_be_bytes());

        Ok(2)
    }

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterDataError> {
        if buf.len() < 2 {
            return Err(ParameterDataError::BufferTooSmall {
                provided: buf.len(),
                required: 2,
            });
        }

        Ok(u16::from_be_bytes([buf[0], buf[1]]).into())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct DeviceUID {
    pub manufacturer_id: u16,
    pub device_id: u32,
}

impl DeviceUID {
    pub const ALL_MANUFACTURERS_ID: u16 = 0xffff;
    pub const ALL_DEVICES_ID: u32 = 0xffffffff;

    pub const fn new(manufacturer_id: u16, device_id: u32) -> Self {
        Self {
            manufacturer_id,
            device_id,
        }
    }

    pub const fn new_dynamic(mut manufacturer_id: u16, device_id: u32) -> Self {
        manufacturer_id |= 0x8000;

        Self {
            manufacturer_id,
            device_id,
        }
    }

    pub const fn broadcast_to_devices_with_manufacturer_id(manufacturer_id: u16) -> Self {
        Self {
            manufacturer_id,
            device_id: Self::ALL_DEVICES_ID,
        }
    }

    pub const fn broadcast_to_devices_with_manufacturer_id_dynamic(
        mut manufacturer_id: u16,
        device_id: u32,
    ) -> Self {
        manufacturer_id |= 0x8000;

        Self {
            manufacturer_id,
            device_id,
        }
    }

    pub const fn broadcast_to_all_devices() -> Self {
        Self {
            manufacturer_id: Self::ALL_MANUFACTURERS_ID,
            device_id: Self::ALL_DEVICES_ID,
        }
    }

    pub fn is_dynamic(&self) -> bool {
        self.manufacturer_id & 0x8000 != 0
    }
}

impl From<[u8; 6]> for DeviceUID {
    fn from(bytes: [u8; 6]) -> Self {
        let manufacturer_id = u16::from_be_bytes([bytes[0], bytes[1]]);
        let device_id = u32::from_be_bytes([bytes[2], bytes[3], bytes[4], bytes[5]]);

        DeviceUID {
            manufacturer_id,
            device_id,
        }
    }
}

impl From<DeviceUID> for [u8; 6] {
    fn from(uid: DeviceUID) -> Self {
        let manufacturer_id_bytes = uid.manufacturer_id.to_be_bytes();
        let device_id_bytes = uid.device_id.to_be_bytes();

        [
            manufacturer_id_bytes[0],
            manufacturer_id_bytes[1],
            device_id_bytes[0],
            device_id_bytes[1],
            device_id_bytes[2],
            device_id_bytes[3],
        ]
    }
}

impl RdmParameterData for DeviceUID {
    fn size_of(&self) -> usize {
        6
    }

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterDataError> {
        if buf.len() < 6 {
            return Err(ParameterDataError::BufferTooSmall {
                provided: buf.len(),
                required: 6,
            });
        }

        buf[0..6].copy_from_slice(&(<[u8; 6]>::from(*self)));

        Ok(6)
    }

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterDataError> {
        if buf.len() < 6 {
            return Err(ParameterDataError::BufferTooSmall {
                provided: buf.len(),
                required: 6,
            });
        }

        Ok(DeviceUID {
            manufacturer_id: u16::from_be_bytes([buf[0], buf[1]]),
            device_id: u32::from_be_bytes([buf[2], buf[3], buf[4], buf[5]]),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SubDeviceId {
    RootDevice,
    Id(u16),
    AllDevices,
}

impl From<u16> for SubDeviceId {
    fn from(value: u16) -> SubDeviceId {
        match value {
            0x0000 => SubDeviceId::RootDevice,
            0xffff => SubDeviceId::AllDevices,
            _ => SubDeviceId::Id(value),
        }
    }
}

impl From<SubDeviceId> for u16 {
    fn from(sub_device: SubDeviceId) -> u16 {
        match sub_device {
            SubDeviceId::RootDevice => 0x0000,
            SubDeviceId::AllDevices => 0xffff,
            SubDeviceId::Id(id) => id,
        }
    }
}

impl RdmParameterData for SubDeviceId {
    fn size_of(&self) -> usize {
        2
    }

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterDataError> {
        let value: u16 = (*self).into();
        buf[0..2].copy_from_slice(&value.to_be_bytes());
        Ok(2)
    }

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterDataError> {
        if buf.len() < 2 {
            return Err(ParameterDataError::MalformedData);
        }
        let value = u16::from_be_bytes([buf[0], buf[1]]);
        Ok(SubDeviceId::from(value))
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u16)]
pub enum NackReasonCode {
    UnknownPid,
    FormatError,
    HardwareFault,
    ProxyReject,
    WriteProtect,
    UnsupportedCommandClass,
    DataOutOfRange,
    BufferFull,
    PacketSizeUnsupported,
    SubDeviceIdOutOfRange,
    ProxyBufferFull,
    ActionNotSupported,
    EndpointNumberInvalid,
    InvalidEndpointMode,
    UnknownUid,
    Unsupported(u16),
}

impl From<u16> for NackReasonCode {
    fn from(value: u16) -> Self {
        match value {
            0x0000 => Self::UnknownPid,
            0x0001 => Self::FormatError,
            0x0002 => Self::HardwareFault,
            0x0003 => Self::ProxyReject,
            0x0004 => Self::WriteProtect,
            0x0005 => Self::UnsupportedCommandClass,
            0x0006 => Self::DataOutOfRange,
            0x0007 => Self::BufferFull,
            0x0008 => Self::PacketSizeUnsupported,
            0x0009 => Self::SubDeviceIdOutOfRange,
            0x000a => Self::ProxyBufferFull,
            0x000b => Self::ActionNotSupported,
            0x000c => Self::EndpointNumberInvalid,
            0x000d => Self::InvalidEndpointMode,
            0x000e => Self::UnknownUid,
            value => Self::Unsupported(value),
        }
    }
}

impl From<NackReasonCode> for u16 {
    fn from(value: NackReasonCode) -> Self {
        match value {
            NackReasonCode::UnknownPid => 0x0000,
            NackReasonCode::FormatError => 0x0001,
            NackReasonCode::HardwareFault => 0x0002,
            NackReasonCode::ProxyReject => 0x0003,
            NackReasonCode::WriteProtect => 0x0004,
            NackReasonCode::UnsupportedCommandClass => 0x0005,
            NackReasonCode::DataOutOfRange => 0x0006,
            NackReasonCode::BufferFull => 0x0007,
            NackReasonCode::PacketSizeUnsupported => 0x0008,
            NackReasonCode::SubDeviceIdOutOfRange => 0x0009,
            NackReasonCode::ProxyBufferFull => 0x000a,
            NackReasonCode::ActionNotSupported => 0x000b,
            NackReasonCode::EndpointNumberInvalid => 0x000c,
            NackReasonCode::InvalidEndpointMode => 0x000d,
            NackReasonCode::UnknownUid => 0x000e,
            NackReasonCode::Unsupported(code) => code,
        }
    }
}

impl core::fmt::Display for NackReasonCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Self::UnknownPid => {
                "The responder cannot comply with request because the message is not implemented in responder."
            }
            Self::FormatError => {
                "The responder cannot interpret request as controller data was not formatted correctly."
            }
            Self::HardwareFault => "The responder cannot comply due to an internal hardware fault.",
            Self::ProxyReject => "Proxy is not the RDM line master and cannot comply with message.",
            Self::WriteProtect => "Command normally allowed but being blocked currently.",
            Self::UnsupportedCommandClass => {
                "Not valid for Command Class attempted. May be used where GET allowed but SET is not supported."
            }
            Self::DataOutOfRange => {
                "Value for given Parameter out of allowable range or not supported."
            }
            Self::BufferFull => "Buffer or Queue space currently has no free space to store data.",
            Self::PacketSizeUnsupported => "Incoming message exceeds buffer capacity.",
            Self::SubDeviceIdOutOfRange => "Sub-Device is out of range or unknown.",
            Self::ProxyBufferFull => {
                "The proxy buffer is full and can not store any more Queued Message or Status Message responses."
            }
            Self::ActionNotSupported => {
                "The parameter data is valid but the SET operation cannot be performed with the current configuration."
            }
            Self::EndpointNumberInvalid => "The Endpoint Number is invalid.",
            Self::InvalidEndpointMode => "The Endpoint Mode is invalid.",
            Self::UnknownUid => "The UID is not known to the responder.",
            Self::Unsupported(code) => {
                return write!(
                    f,
                    "An unknown NACK reason code was received: 0x{:04x}",
                    code
                );
            }
        };

        f.write_str(message)
    }
}

impl RdmParameterData for NackReasonCode {
    fn size_of(&self) -> usize {
        2
    }

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterDataError> {
        buf[0..2].copy_from_slice(&u16::from(*self).to_be_bytes());
        Ok(2)
    }

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterDataError> {
        if buf.len() < 2 {
            return Err(ParameterDataError::BufferTooSmall {
                provided: buf.len(),
                required: 2,
            });
        }

        Ok(NackReasonCode::from(u16::from_be_bytes([buf[0], buf[1]])))
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u8)]
pub enum ResponseType {
    Ack = 0x00,
    AckTimer = 0x01,
    Nack = 0x02,
    AckOverflow = 0x03,
}

impl TryFrom<u8> for ResponseType {
    type Error = ParameterDataError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Ack),
            0x01 => Ok(Self::AckTimer),
            0x02 => Ok(Self::Nack),
            0x03 => Ok(Self::AckOverflow),
            _ => Err(ParameterDataError::MalformedData),
        }
    }
}

impl RdmParameterData for Duration {
    fn size_of(&self) -> usize {
        2
    }

    fn encode_parameter_data(&self, buf: &mut [u8]) -> Result<usize, ParameterDataError> {
        buf[0..2].copy_from_slice(&(self.as_millis().saturating_div(100) as u16).to_be_bytes());
        Ok(2)
    }

    fn decode_parameter_data(buf: &[u8]) -> Result<Self, ParameterDataError> {
        let estimated_response_time = u16::from_be_bytes([buf[0], buf[1]]);
        Ok(Duration::from_millis(
            (estimated_response_time as u64).saturating_mul(100),
        ))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResponseResult<T: RdmParameterData> {
    Ack(T),
    AckOverflow(T),
    AckTimer(Duration),
    Nack(NackReasonCode),
}

impl<T: RdmParameterData> ResponseResult<T> {
    pub fn response_type(&self) -> ResponseType {
        match self {
            ResponseResult::Ack(_) => ResponseType::Ack,
            ResponseResult::AckOverflow(_) => ResponseType::AckOverflow,
            ResponseResult::AckTimer(_) => ResponseType::AckTimer,
            ResponseResult::Nack(_) => ResponseType::Nack,
        }
    }

    pub fn size_of(&self) -> usize {
        match self {
            Self::Ack(data) => data.size_of(),
            Self::AckOverflow(data) => data.size_of(),
            Self::AckTimer(_) => 2,
            Self::Nack(_) => 2,
        }
    }

    pub fn encode(&self, buf: &mut [u8]) -> Result<usize, ParameterDataError> {
        match self {
            Self::Ack(data) | Self::AckOverflow(data) => data.encode_parameter_data(buf),
            Self::AckTimer(data) => data.encode_parameter_data(buf),
            Self::Nack(data) => data.encode_parameter_data(buf),
        }
    }

    pub fn decode(response_type: ResponseType, buf: &[u8]) -> Result<Self, ParameterDataError> {
        // TODO consider how we handle parameter data length checks here
        // if buf.len() < 8 {
        //     return Err(ParameterDataError::BufferTooSmall {
        //         provided: buf.len(),
        //         required: 8,
        //     });
        // }

        // if buf.len() < 8 + parameter_data_length as usize {
        //     return Err(ParameterDataError::BufferTooSmall {
        //         provided: buf.len(),
        //         required: 8 + parameter_data_length as usize,
        //     });
        // }

        let res = match response_type {
            ResponseType::Ack => ResponseResult::Ack(T::decode_parameter_data(buf)?),
            ResponseType::AckOverflow => {
                ResponseResult::AckOverflow(T::decode_parameter_data(buf)?)
            }
            ResponseType::AckTimer => {
                ResponseResult::AckTimer(Duration::decode_parameter_data(buf)?)
            }
            ResponseType::Nack => ResponseResult::Nack(NackReasonCode::decode_parameter_data(buf)?),
        };

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_static_device_uid() {
        let device_uid = DeviceUID::new(0x1234, 0x56789abc);

        assert_eq!(
            device_uid,
            DeviceUID {
                manufacturer_id: 0x1234,
                device_id: 0x56789abc
            }
        );
        assert!(!device_uid.is_dynamic());
    }

    #[test]
    fn should_create_dynamic_device_uid() {
        let device_uid = DeviceUID::new_dynamic(0x1234, 0x56789abc);

        assert_eq!(
            device_uid,
            DeviceUID {
                manufacturer_id: 0x9234,
                device_id: 0x56789abc
            }
        );
        assert!(device_uid.is_dynamic());
    }

    #[test]
    fn should_array_to_convert_device_uid() {
        assert_eq!(
            DeviceUID::from([0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc]),
            DeviceUID::new(0x1234, 0x56789abc)
        );
    }

    #[test]
    fn should_convert_device_uid_to_array() {
        assert_eq!(
            <[u8; 6]>::from(DeviceUID::new(0x1234, 0x56789abc)),
            [0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc]
        );
    }
}
