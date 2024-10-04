use core::{array::TryFromSliceError, error::Error, fmt, str::Utf8Error};

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum RdmError {
    InvalidStartCode,
    InvalidFrameLength(u8),
    InvalidMessageLength(u8),
    InvalidChecksum(u16, u16),
    InvalidResponseType(u8),
    InvalidNackReasonCode(u16),
    InvalidStatusType(u8),
    InvalidCommandClass(u8),
    InvalidCommandClassImplementation(u8),
    UnsupportedParameter(u8, u16),
    InvalidParameterDataLength(u8),
    InvalidParameterDataType(u8),
    InvalidSensorUnit(u8),
    InvalidSensorUnitPrefix(u8),
    InvalidDiscoveryUniqueBranchPreamble,
    Utf8Error { source: core::str::Utf8Error },
    TryFromSliceError,
    InvalidLampState(u8),
    InvalidLampOnMode(u8),
    InvalidPowerState(u8),
    InvalidOnOffStates(u8),
    InvalidDisplayInvertMode(u8),
    InvalidResetDeviceMode(u8),
    InvalidSensorType(u8),
    InvalidIdentifyMode(u8),
    InvalidMergeMode(u8),
    InvalidPresetProgrammed(u8),
    InvalidPinCode(u16),
    InvalidDhcpMode(u8),
    InvalidStaticConfigType(u8),
    InvalidBrokerState(u8),
    MalformedPacket,
}

impl From<TryFromSliceError> for RdmError {
    fn from(_: TryFromSliceError) -> Self {
        RdmError::TryFromSliceError
    }
}

impl From<Utf8Error> for RdmError {
    fn from(source: Utf8Error) -> Self {
        RdmError::Utf8Error { source }
    }
}

impl fmt::Display for RdmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidStartCode => write!(f, "Invalid start code"),
            Self::InvalidFrameLength(length) => write!(f, "Invalid frame length: {}", length),
            Self::InvalidMessageLength(length) => write!(
                f,
                "Invalid message length: {}, must be >= 24 and <= 255",
                length
            ),
            Self::InvalidChecksum(checksum, expected) => {
                write!(f, "Invalid checksum: {}, expected: {}", checksum, expected)
            }
            Self::InvalidResponseType(response_type) => {
                write!(f, "Invalid ResponseType: {}", response_type)
            }
            Self::InvalidNackReasonCode(reason_code) => {
                write!(f, "Invalid NackReasonCode: {}", reason_code)
            }
            Self::InvalidStatusType(status_type) => {
                write!(f, "Invalid StatusType: {}", status_type)
            }
            Self::InvalidCommandClass(command_class) => {
                write!(f, "Invalid CommandClass: {}", command_class)
            }
            Self::InvalidCommandClassImplementation(implementation) => {
                write!(f, "Invalid CommandClass Implementation: {}", implementation)
            }
            Self::UnsupportedParameter(command_class, parameter_id) => write!(
                f,
                "Unsupported Parameter, CommandClass: {}, ParameterId: {}",
                command_class, parameter_id
            ),
            Self::InvalidParameterDataLength(length) => write!(
                f,
                "Invalid parameter data length: {}, must be >= 0 and <= 231",
                length
            ),
            Self::InvalidParameterDataType(data_type) => {
                write!(f, "Invalid ParameterDataType: {}", data_type)
            }
            Self::InvalidSensorUnit(sensor_unit) => {
                write!(f, "Invalid SensorUnit: {}", sensor_unit)
            }
            Self::InvalidSensorUnitPrefix(prefix) => {
                write!(f, "Invalid SensorUnitPrefix: {}", prefix)
            }
            Self::InvalidDiscoveryUniqueBranchPreamble => {
                write!(f, "Invalid discovery unique branch preamble")
            }
            Self::Utf8Error { source } => write!(f, "Invalid utf-8 sequence: {}", source),
            Self::TryFromSliceError => write!(f, "Could not convert slice to array"),
            Self::InvalidLampState(state) => write!(f, "Invalid LampState: {}", state),
            Self::InvalidLampOnMode(mode) => write!(f, "Invalid LampOnMode: {}", mode),
            Self::InvalidPowerState(state) => write!(f, "Invalid PowerState: {}", state),
            Self::InvalidOnOffStates(states) => write!(f, "Invalid OnOffStates: {}", states),
            Self::InvalidDisplayInvertMode(mode) => {
                write!(f, "Invalid DisplayInvertMode: {}", mode)
            }
            Self::InvalidResetDeviceMode(mode) => write!(f, "Invalid ResetDeviceMode: {}", mode),
            Self::InvalidSensorType(sensor_type) => {
                write!(f, "Invalid SensorType: {}", sensor_type)
            }
            Self::InvalidIdentifyMode(identify_mode) => {
                write!(f, "Invalid IdentifyMode: {}", identify_mode)
            }
            Self::InvalidMergeMode(merge_mode) => {
                write!(f, "Invalid MergeMode: {}", merge_mode)
            }
            Self::InvalidPresetProgrammed(preset_programmed) => {
                write!(f, "Invalid PresetProgrammed: {}", preset_programmed)
            }
            Self::InvalidPinCode(pin_code) => {
                write!(f, "Invalid PinCode: {}", pin_code)
            }
            Self::InvalidDhcpMode(dhcp_mode) => {
                write!(f, "Invalid DhcpMode: {}", dhcp_mode)
            }
            Self::InvalidStaticConfigType(static_config_type) => {
                write!(f, "Invalid StaticConfigType: {}", static_config_type)
            }
            Self::InvalidBrokerState(broker_state) => write!(f, "Invalid BrokerState: {}", broker_state),
            Self::MalformedPacket => write!(f, "Malformed packet"),
        }
    }
}

impl Error for RdmError {}
