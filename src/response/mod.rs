pub mod discovery;
pub mod get;
pub mod set;

use bytes::{Buf, BytesMut};
use discovery::DiscoveryResponseParameterData;
use get::GetResponseParameterData;
use set::SetResponseParameterData;

use crate::{
    bsd_16_crc, device::DeviceUID, parameter::ParameterId, CommandClass, ProtocolError,
    DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE, DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE, SC_RDM,
    SC_SUB_MESSAGE,
};

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResponseType {
    Ack = 0x00,
    AckTimer = 0x01,
    NackReason = 0x02,
    AckOverflow = 0x03,
}

impl TryFrom<u8> for ResponseType {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Ack),
            0x01 => Ok(Self::AckTimer),
            0x02 => Ok(Self::NackReason),
            0x03 => Ok(Self::AckOverflow),
            _ => Err(ProtocolError::InvalidResponseType(value)),
        }
    }
}

// TODO the following is a quick and dirty test
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PacketResponseType {
    SuccessResponse = 0x05,
    NullResponse = 0x0c,
}

impl TryFrom<u8> for PacketResponseType {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x05 => Ok(Self::SuccessResponse),
            0x0c => Ok(Self::NullResponse),
            _ => Err(ProtocolError::InvalidPacketResponseType(value)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RdmResponseParameterData {
    GetResponse(GetResponseParameterData),
    SetResponse(SetResponseParameterData),
    DiscoveryResponse(DiscoveryResponseParameterData),
}

impl RdmResponseParameterData {
    pub fn parse(
        command_class: CommandClass,
        parameter_id: ParameterId,
        bytes: &[u8],
    ) -> Result<Option<Self>, ProtocolError> {
        match command_class {
            CommandClass::GetCommandResponse => {
                Ok(GetResponseParameterData::parse(parameter_id, bytes)?.map(Self::GetResponse))
            }
            CommandClass::SetCommandResponse => {
                Ok(SetResponseParameterData::parse(parameter_id, bytes)?.map(Self::SetResponse))
            }
            CommandClass::DiscoveryCommandResponse => {
                Ok(DiscoveryResponseParameterData::parse(parameter_id, bytes)?
                    .map(Self::DiscoveryResponse))
            }
            _ => Err(ProtocolError::InvalidCommandClass(command_class as u8)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RdmResponse {
    pub destination_uid: DeviceUID,
    pub source_uid: DeviceUID,
    pub transaction_number: u8,
    pub response_type: ResponseType,
    pub message_count: u8,
    pub sub_device_id: u16,
    pub command_class: CommandClass,
    pub parameter_id: ParameterId,
    pub parameter_data: Option<RdmResponseParameterData>,
}

impl RdmResponse {
    pub fn parse(bytes: &mut BytesMut) -> Result<Self, ProtocolError> {
        let message_length = bytes[2];

        if message_length < 24 {
            return Err(ProtocolError::InvalidMessageLength(message_length));
        }

        let packet_checksum = u16::from_be_bytes(
            bytes[message_length as usize..=message_length as usize + 1]
                .try_into()
                .unwrap(),
        );

        let decoded_checksum = bsd_16_crc(&bytes[..message_length as usize - 1]);

        if decoded_checksum != packet_checksum {
            return Err(ProtocolError::InvalidChecksum(
                decoded_checksum,
                packet_checksum,
            ));
        }

        let destination_manufacturer_id = u16::from_be_bytes(bytes[3..=4].try_into().unwrap());
        let destination_device_id = u32::from_be_bytes(bytes[5..=8].try_into().unwrap());
        let destination_uid = DeviceUID::new(destination_manufacturer_id, destination_device_id);

        let source_manufacturer_id = u16::from_be_bytes(bytes[9..=10].try_into().unwrap());
        let source_device_id = u32::from_be_bytes(bytes[11..=14].try_into().unwrap());
        let source_uid = DeviceUID::new(source_manufacturer_id, source_device_id);

        let transaction_number = bytes[15];

        let response_type = ResponseType::try_from(bytes[16])?;

        let message_count = bytes[17];

        let sub_device_id = u16::from_be_bytes(bytes[18..=19].try_into().unwrap());

        let command_class = CommandClass::try_from(bytes[20])?;

        let parameter_id =
            ParameterId::try_from(u16::from_be_bytes(bytes[21..=22].try_into().unwrap()))?;

        let parameter_data_length = bytes[23];

        if parameter_data_length > 231 {
            return Err(ProtocolError::InvalidParameterDataLength(
                parameter_data_length,
            ));
        }

        let parameter_data = if parameter_data_length > 0 {
            RdmResponseParameterData::parse(
                command_class,
                parameter_id,
                &bytes[25..=message_length as usize + 1],
            )?
        } else {
            None
        };

        bytes.advance(message_length as usize + 2);

        Ok(Self {
            destination_uid,
            source_uid,
            transaction_number,
            response_type,
            message_count,
            sub_device_id,
            command_class,
            parameter_id,
            parameter_data,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveryUniqueBranchResponse(DeviceUID);

impl DiscoveryUniqueBranchResponse {
    pub fn parse(bytes: &mut BytesMut) -> Result<Self, ProtocolError> {
        let Some(frame_start_index) = bytes.iter().position(|&x| x == 0xaa) else {
            return Err(ProtocolError::InvalidDiscoveryUniqueBranchPreamble);
        };

        let euid = &bytes[(frame_start_index + 1)..=(frame_start_index + 12)];

        let ecs = &bytes[(frame_start_index + 13)..=(frame_start_index + 16)];

        let decoded_checksum = bsd_16_crc(euid);

        let checksum = u16::from_be_bytes([ecs[0] & ecs[1], ecs[2] & ecs[3]]);

        if checksum != decoded_checksum {
            return Err(ProtocolError::InvalidChecksum(decoded_checksum, checksum));
        }

        let manufacturer_id = u16::from_be_bytes([euid[0] & euid[1], euid[2] & euid[3]]);

        let device_id = u32::from_be_bytes([
            euid[4] & euid[5],
            euid[6] & euid[7],
            euid[8] & euid[9],
            euid[10] & euid[11],
        ]);

        bytes.advance(frame_start_index + 17);

        Ok(Self(DeviceUID::new(manufacturer_id, device_id)))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RdmFrame {
    Rdm(RdmResponse),
    DiscoveryUniqueBranch(DiscoveryUniqueBranchResponse),
}

impl RdmFrame {
    pub fn parse(bytes: &mut BytesMut) -> Result<Option<Self>, ProtocolError> {
        if bytes[0] == SC_RDM && bytes[1] == SC_SUB_MESSAGE {
            if bytes.len() < 25 {
                return Ok(None);
            }

            match RdmResponse::parse(bytes) {
                Ok(frame) => {
                    return Ok(Some(RdmFrame::Rdm(frame)));
                }
                Err(e) => {
                    bytes.advance(1);
                    return Err(e);
                }
            }
        }

        if bytes[0] == DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE
            || bytes[0] == DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE
        {
            if bytes.len() < 17 {
                return Ok(None);
            }

            match DiscoveryUniqueBranchResponse::parse(bytes) {
                Ok(frame) => {
                    return Ok(Some(RdmFrame::DiscoveryUniqueBranch(frame)));
                }
                Err(e) => {
                    bytes.advance(1);
                    return Err(e);
                }
            }
        }

        bytes.advance(1);

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BufMut;

    #[test]
    fn should_take_first_byte_when_first_bytes_do_not_match_frame_header() {
        let mut bytes = BytesMut::zeroed(16);

        assert_eq!(RdmFrame::parse(&mut bytes), Ok(None));

        let bytes_check = [0u8; 15];

        assert_eq!(bytes.len(), bytes_check.len());
    }

    #[test]
    fn should_defer_parsing_rdm_response_when_packet_length_is_short() {
        let mut bytes = BytesMut::zeroed(24);
        bytes[0] = SC_RDM;
        bytes[1] = SC_SUB_MESSAGE;

        assert_eq!(RdmFrame::parse(&mut bytes), Ok(None));

        let mut bytes_check = BytesMut::zeroed(24);
        bytes_check[0] = SC_RDM;
        bytes_check[1] = SC_SUB_MESSAGE;

        assert_eq!(bytes, bytes_check);
    }

    #[test]
    fn should_defer_parsing_discovery_unique_branch_response_when_packet_length_is_short() {
        let mut bytes = BytesMut::zeroed(16);
        bytes[0] = DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE;
        bytes[1] = DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE;

        assert_eq!(RdmFrame::parse(&mut bytes), Ok(None));

        let mut bytes_check = BytesMut::zeroed(16);
        bytes_check[0] = DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE;
        bytes_check[1] = DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE;

        assert_eq!(bytes, bytes_check);
    }

    #[test]
    fn should_parse_valid_rdm_response() {
        let mut bytes = BytesMut::with_capacity(27);
        bytes.put(
            &[
                SC_RDM,
                SC_SUB_MESSAGE,
                25,   // message length
                0x01, // destination uid
                0x02,
                0x03,
                0x04,
                0x05,
                0x06,
                0x06, // source uid
                0x05,
                0x04,
                0x03,
                0x02,
                0x01,
                0x00, // transaction number
                0x00, // response type = Ack
                0x01, // message count
                0x00, // sub device id = root device
                0x00,
                0x21, // command class = get command response
                0x10, // parameter id = identify device
                0x00,
                0x01, // parameter data length
                0x01, // identifying = true
                0x01,
                0x43, // checksum
            ][..],
        );

        assert_eq!(
            RdmFrame::parse(&mut bytes),
            Ok(Some(RdmFrame::Rdm(RdmResponse {
                destination_uid: DeviceUID::new(0x0102, 0x03040506),
                source_uid: DeviceUID::new(0x0605, 0x04030201),
                transaction_number: 0x00,
                response_type: ResponseType::Ack,
                message_count: 0x01,
                sub_device_id: 0x0000,
                command_class: CommandClass::GetCommandResponse,
                parameter_id: ParameterId::IdentifyDevice,
                parameter_data: Some(RdmResponseParameterData::GetResponse(
                    GetResponseParameterData::IdentifyDevice {
                        is_identifying: true,
                    }
                )),
            })))
        );

        assert_eq!(bytes.len(), 0);
    }

    #[test]
    fn should_parse_valid_discovery_unique_branch_response() {
        // includes preamble bytes
        let mut bytes = BytesMut::with_capacity(27);

        // TODO dedupe bytes creation test code
        bytes.put(
            &[
                DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
                DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
                DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
                DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
                DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
                DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
                DISCOVERY_UNIQUE_BRANCH_PREAMBLE_BYTE,
                DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE,
                0xab, // euid 11 = manufacturer id 1 (MSB)
                0x55, // euid 10 = manufacturer id 1 (MSB)
                0xaa, // euid 9 = manufacturer id 0 (LSB)
                0x57, // euid 8 = manufacturer id 0 (LSB)
                0xab, // euid 7 = device id 3 (MSB)
                0x57, // euid 6 = device id 3 (MSB)
                0xae, // euid 5 = device id 2
                0x55, // euid 4 = device id 2
                0xaf, // euid 3 = device id 1
                0x55, // euid 2 = device id 1
                0xae, // euid 1 = device id 0 (LSB)
                0x57, // euid 0 = device id 0 (LSB)
                0xae, // ecs 3 = Checksum1 (MSB)
                0x57, // ecs 2 = Checksum1 (MSB)
                0xaf, // ecs 1 = Checksum0 (LSB)
                0x5f, // ecs 0 = Checksum0 (LSB)
            ][..],
        );

        assert_eq!(
            RdmFrame::parse(&mut bytes),
            Ok(Some(RdmFrame::DiscoveryUniqueBranch(
                DiscoveryUniqueBranchResponse(DeviceUID::new(0x0102, 0x03040506))
            )))
        );

        assert_eq!(bytes.len(), 0);

        // does not include preamble bytes
        let mut bytes = BytesMut::with_capacity(27);
        bytes.put(
            &[
                DISCOVERY_UNIQUE_BRANCH_PREAMBLE_SEPARATOR_BYTE,
                0xab, // euid 11 = manufacturer id 1 (MSB)
                0x55, // euid 10 = manufacturer id 1 (MSB)
                0xaa, // euid 9 = manufacturer id 0 (LSB)
                0x57, // euid 8 = manufacturer id 0 (LSB)
                0xab, // euid 7 = device id 3 (MSB)
                0x57, // euid 6 = device id 3 (MSB)
                0xae, // euid 5 = device id 2
                0x55, // euid 4 = device id 2
                0xaf, // euid 3 = device id 1
                0x55, // euid 2 = device id 1
                0xae, // euid 1 = device id 0 (LSB)
                0x57, // euid 0 = device id 0 (LSB)
                0xae, // ecs 3 = Checksum1 (MSB)
                0x57, // ecs 2 = Checksum1 (MSB)
                0xaf, // ecs 1 = Checksum0 (LSB)
                0x5f, // ecs 0 = Checksum0 (LSB)
            ][..],
        );

        assert_eq!(
            RdmFrame::parse(&mut bytes),
            Ok(Some(RdmFrame::DiscoveryUniqueBranch(
                DiscoveryUniqueBranchResponse(DeviceUID::new(0x0102, 0x03040506))
            )))
        );

        assert_eq!(bytes.len(), 0);
    }
}
