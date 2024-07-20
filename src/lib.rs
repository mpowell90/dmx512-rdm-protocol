pub mod codec;
pub mod device;
pub mod parameter;
pub mod request;
pub mod response;
pub mod sensor;

use bytes::BytesMut;

pub const MIN_PACKET_LEN: usize = 26;

pub const SC_RDM: u8 = 0xcc;
pub const SC_SUB_MESSAGE: u8 = 0x01;

pub const BROADCAST_ALL_DEVICES_ID: u64 = 0xffffffffffff;
pub const SUB_DEVICE_ALL_CALL: u16 = 0xffff;
pub const ROOT_DEVICE: u16 = 0x0000;

#[derive(Debug, PartialEq)]
pub enum PacketType {
    RdmResponse = 0xcc01,
    DiscoveryUniqueBranchResponse = 0xfefe,
}

impl TryFrom<u16> for PacketType {
    type Error = &'static str;

    fn try_from(byte: u16) -> Result<Self, Self::Error> {
        let packet_type = match byte {
            0xcc01 => PacketType::RdmResponse,
            0xfefe => PacketType::DiscoveryUniqueBranchResponse,
            _ => return Err("Invalid value for PacketRequestType"),
        };
        Ok(packet_type)
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

pub fn bsd_16_crc_bytes_mut(packet: &mut BytesMut) -> u16 {
    packet
        .iter()
        .fold(0_u16, |sum, byte| (sum.overflowing_add(*byte as u16).0))
}

pub fn bsd_16_crc(packet: &[u8]) -> u16 {
    packet
        .iter()
        .fold(0_u16, |sum, byte| (sum.overflowing_add(*byte as u16).0))
}

pub fn is_checksum_valid(packet: &[u8]) -> bool {
    let packet_checksum =
        u16::from_be_bytes(packet[packet.len() - 2..packet.len()].try_into().unwrap());

    let calculated_checksum = bsd_16_crc(&packet[..packet.len() - 2]);

    packet_checksum == calculated_checksum
}
