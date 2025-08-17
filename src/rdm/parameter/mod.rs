pub mod e120;
pub mod e133;
pub mod e137_1;
pub mod e137_2;
pub mod e137_7;

use super::{RdmError, SubDeviceId};

#[cfg(not(feature = "alloc"))]
use core::str::FromStr;
#[cfg(not(feature = "alloc"))]
use heapless::{String, Vec};

#[cfg(feature = "alloc")]
pub fn decode_string_bytes(bytes: &[u8]) -> Result<String, RdmError> {
    let utf8 = String::from_utf8_lossy(bytes);

    if utf8.contains(char::from(0)) {
        Ok(utf8.split_once(char::from(0)).unwrap().0.to_string())
    } else {
        Ok(utf8.to_string())
    }
}

#[cfg(not(feature = "alloc"))]
pub fn decode_string_bytes<const N: usize>(bytes: &[u8]) -> Result<String<N>, RdmError> {
    let utf8 = String::<N>::from_utf8(Vec::<u8, N>::from_slice(bytes).unwrap())?;

    if utf8.contains(char::from(0)) {
        Ok(String::<N>::from_str(utf8.split_once(char::from(0)).unwrap().0).unwrap())
    } else {
        Ok(utf8)
    }
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq)]
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
    RawParameterId(u16),
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
            n => Self::RawParameterId(n),
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
            ParameterId::RawParameterId(pid) => pid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "alloc")]
    fn should_decode_string_bytes() {
        assert_eq!(
            decode_string_bytes(&b"null terminated string\0"[..]).unwrap(),
            "null terminated string".to_string()
        );
        assert_eq!(
            decode_string_bytes(&b"not null terminated string"[..]).unwrap(),
            "not null terminated string".to_string()
        );
        assert_eq!(
            decode_string_bytes(&b"early terminated\0string"[..]).unwrap(),
            "early terminated".to_string()
        );
    }

    #[test]
    #[cfg(not(feature = "alloc"))]
    fn should_decode_string_bytes() {
        assert_eq!(
            decode_string_bytes::<32>(&b"null terminated string\0"[..]).unwrap(),
            String::from_utf8(Vec::<u8, 32>::from_slice(b"null terminated string").unwrap())
                .unwrap()
        );
        assert_eq!(
            decode_string_bytes::<32>(&b"not null terminated string"[..]).unwrap(),
            String::from_utf8(Vec::<u8, 32>::from_slice(b"not null terminated string").unwrap())
                .unwrap()
        );
        assert_eq!(
            decode_string_bytes::<32>(&b"early terminated\0string"[..]).unwrap(),
            String::from_utf8(Vec::<u8, 32>::from_slice(b"early terminated").unwrap()).unwrap()
        );
    }
}
