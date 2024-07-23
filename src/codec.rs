use crate::{
    bsd_16_crc, bsd_16_crc_bytes_mut,
    device::{DeviceUID, DmxSlot},
    parameter::{
        DisplayInvertMode, LampOnMode, LampState, ManufacturerSpecificParameter, ParameterId,
        PowerState, ProductCategory,
    },
    request::{
        DiscoveryRequestParameterData, GetRequestParameterData, RdmRequestMessage,
        SetRequestParameterData,
    },
    response::{
        DiscoveryResponseParameterData, GetResponseParameterData, RdmFrame,
        SetResponseParameterData,
    },
    sensor::Sensor,
    CommandClass, ProtocolError,
};
use bytes::{BufMut, Bytes, BytesMut};
use std::io;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Copy, Clone, Default)]
pub struct RdmCodec;

impl RdmCodec {
    const SC_RDM: u8 = 0xcc;
    const SC_SUB_MESSAGE: u8 = 0x01;
    const FRAME_HEADER_FOOTER_SIZE: usize = 4;

    pub fn get_request_parameter_data_to_bytes(
        parameter_data: GetRequestParameterData,
    ) -> BytesMut {
        let mut bytes = BytesMut::new();

        match parameter_data {
            GetRequestParameterData::ParameterDescription { parameter_id } => {
                bytes.put_u16(parameter_id)
            }
            GetRequestParameterData::SensorDefinition { sensor_id } => bytes.put_u8(sensor_id),
            GetRequestParameterData::DmxPersonalityDescription { personality } => {
                bytes.put_u8(personality)
            }
            GetRequestParameterData::CurveDescription { curve } => bytes.put_u8(curve),
            GetRequestParameterData::ModulationFrequencyDescription {
                modulation_frequency,
            } => bytes.put_u8(modulation_frequency),
            GetRequestParameterData::OutputResponseTimeDescription {
                output_response_time,
            } => bytes.put_u8(output_response_time),
            GetRequestParameterData::SelfTestDescription { self_test_id } => {
                bytes.put_u8(self_test_id)
            }
            GetRequestParameterData::SlotDescription { slot_id } => bytes.put_u16(slot_id),
        }

        bytes
    }

    pub fn set_request_parameter_data_to_bytes(
        parameter_data: SetRequestParameterData,
    ) -> BytesMut {
        let mut bytes = BytesMut::new();

        match parameter_data {
            SetRequestParameterData::DmxPersonality { personality_id } => {
                bytes.put_u8(personality_id)
            }
            SetRequestParameterData::DeviceLabel { device_label } => {
                bytes.extend(device_label.as_bytes())
            }
            SetRequestParameterData::DmxStartAddress { dmx_start_address } => {
                bytes.put_u16(dmx_start_address)
            }
            SetRequestParameterData::Curve { curve_id } => bytes.put_u8(curve_id),
            SetRequestParameterData::ModulationFrequency {
                modulation_frequency_id,
            } => bytes.put_u8(modulation_frequency_id),
            SetRequestParameterData::OutputResponseTime {
                output_response_time_id,
            } => bytes.put_u8(output_response_time_id),
            SetRequestParameterData::IdentifyDevice { identify } => bytes.put_u8(identify.into()),
        }

        bytes
    }

    pub fn get_response_bytes_to_parameter_data(
        parameter_id: ParameterId,
        bytes: Bytes,
    ) -> GetResponseParameterData {
        match parameter_id {
            ParameterId::ProxiedDeviceCount => GetResponseParameterData::ProxiedDeviceCount {
                device_count: u16::from_be_bytes(bytes[..=1].try_into().unwrap()),
                list_change: bytes[2] != 0,
            },
            ParameterId::ProxiedDevices => GetResponseParameterData::ProxiedDevices {
                device_uids: bytes.chunks(6).map(DeviceUID::from).collect(),
            },
            ParameterId::ParameterDescription => GetResponseParameterData::ParameterDescription {
                parameter_id: u16::from_be_bytes(bytes[0..=1].try_into().unwrap()),
                parameter_data_size: bytes[2],
                data_type: bytes[3],
                command_class: CommandClass::try_from(bytes[4]).unwrap(),
                prefix: bytes[5],
                minimum_valid_value: u32::from_be_bytes(bytes[8..=11].try_into().unwrap()),
                maximum_valid_value: u32::from_be_bytes(bytes[12..=15].try_into().unwrap()),
                default_value: u32::from_be_bytes(bytes[16..=19].try_into().unwrap()),
                description: String::from_utf8_lossy(&bytes[20..])
                    .trim_end_matches('\0')
                    .to_string(),
            },
            ParameterId::DeviceLabel => GetResponseParameterData::DeviceLabel {
                device_label: String::from_utf8_lossy(&bytes)
                    .trim_end_matches('\0')
                    .to_string(),
            },
            ParameterId::DeviceInfo => GetResponseParameterData::DeviceInfo {
                protocol_version: format!("{}.{}", bytes[0], bytes[1]),
                model_id: u16::from_be_bytes(bytes[2..=3].try_into().unwrap()),
                product_category: ProductCategory::from(&bytes[4..=5]),
                software_version_id: u32::from_be_bytes(bytes[6..=9].try_into().unwrap()),
                footprint: u16::from_be_bytes(bytes[10..=11].try_into().unwrap()),
                current_personality: bytes[12],
                personality_count: bytes[13],
                start_address: u16::from_be_bytes(bytes[14..=15].try_into().unwrap()),
                sub_device_count: u16::from_be_bytes(bytes[16..=17].try_into().unwrap()),
                sensor_count: u8::from_be(bytes[18]),
            },
            ParameterId::SoftwareVersionLabel => GetResponseParameterData::SoftwareVersionLabel {
                software_version_label: String::from_utf8_lossy(&bytes)
                    .trim_end_matches('\0')
                    .to_string(),
            },
            ParameterId::SupportedParameters => {
                let parameters = bytes
                    .chunks(2)
                    .map(|chunk| u16::from_be_bytes(chunk.try_into().unwrap()));

                GetResponseParameterData::SupportedParameters {
                    standard_parameters: parameters
                        .clone()
                        .filter(|parameter_id| {
                            // TODO consider if we should filter parameters here or before we add to the queue
                            let parameter_id = *parameter_id;
                            (0x0060_u16..0x8000_u16).contains(&parameter_id)
                        })
                        .map(ParameterId::try_from)
                        .collect::<Result<Vec<ParameterId>, ProtocolError>>()
                        .unwrap(), // TODO handle this error properly
                    manufacturer_specific_parameters: parameters
                        .filter(|parameter_id| *parameter_id >= 0x8000_u16)
                        .map(|parameter_id| {
                            (
                                parameter_id,
                                ManufacturerSpecificParameter {
                                    parameter_id,
                                    ..Default::default()
                                },
                            )
                        })
                        .collect(),
                }
            }
            ParameterId::SensorDefinition => GetResponseParameterData::SensorDefinition {
                sensor: Sensor {
                    id: bytes[0],
                    kind: bytes[1].into(),
                    unit: bytes[2],
                    prefix: bytes[3],
                    range_minimum_value: i16::from_be_bytes(bytes[4..=5].try_into().unwrap()),
                    range_maximum_value: i16::from_be_bytes(bytes[6..=7].try_into().unwrap()),
                    normal_minimum_value: i16::from_be_bytes(bytes[8..=9].try_into().unwrap()),
                    normal_maximum_value: i16::from_be_bytes(bytes[10..=11].try_into().unwrap()),
                    recorded_value_support: bytes[12],
                    description: String::from_utf8_lossy(&bytes[13..])
                        .trim_end_matches('\0')
                        .to_string(),
                },
            },
            ParameterId::IdentifyDevice => GetResponseParameterData::IdentifyDevice {
                is_identifying: bytes[0] != 0,
            },
            ParameterId::ManufacturerLabel => GetResponseParameterData::ManufacturerLabel {
                manufacturer_label: String::from_utf8_lossy(&bytes)
                    .trim_end_matches('\0')
                    .to_string(),
            },
            ParameterId::FactoryDefaults => GetResponseParameterData::FactoryDefaults {
                factory_default: bytes[0] != 0,
            },
            ParameterId::DeviceModelDescription => {
                GetResponseParameterData::DeviceModelDescription {
                    device_model_description: String::from_utf8_lossy(&bytes)
                        .trim_end_matches('\0')
                        .to_string(),
                }
            }
            ParameterId::ProductDetailIdList => GetResponseParameterData::ProductDetailIdList {
                product_detail_id_list: bytes
                    .chunks(2)
                    .map(|id| u16::from_be_bytes(id.try_into().unwrap()))
                    .collect(),
            },
            ParameterId::DmxPersonality => GetResponseParameterData::DmxPersonality {
                current_personality: bytes[0],
                personality_count: bytes[1],
            },
            ParameterId::DmxPersonalityDescription => {
                GetResponseParameterData::DmxPersonalityDescription {
                    id: bytes[0],
                    dmx_slots_required: u16::from_be_bytes(bytes[1..=2].try_into().unwrap()),
                    description: String::from_utf8_lossy(&bytes[3..])
                        .trim_end_matches('\0')
                        .to_string(),
                }
            }
            ParameterId::DmxStartAddress => GetResponseParameterData::DmxStartAddress {
                dmx_start_address: u16::from_be_bytes(bytes[0..=1].try_into().unwrap()),
            },
            ParameterId::SlotInfo => {
                let dmx_slots = if bytes.len() >= 5 {
                    Some(bytes.chunks(5).map(DmxSlot::from).collect())
                } else {
                    None
                };

                GetResponseParameterData::SlotInfo { dmx_slots }
            }
            ParameterId::SlotDescription => GetResponseParameterData::SlotDescription {
                slot_id: u16::from_be_bytes(bytes[0..=1].try_into().unwrap()),
                description: String::from_utf8_lossy(&bytes[2..])
                    .trim_end_matches('\0')
                    .to_string(),
            },
            ParameterId::DeviceHours => GetResponseParameterData::DeviceHours {
                device_hours: u32::from_be_bytes(bytes[0..=3].try_into().unwrap()),
            },
            ParameterId::LampHours => GetResponseParameterData::LampHours {
                lamp_hours: u32::from_be_bytes(bytes[0..=3].try_into().unwrap()),
            },
            ParameterId::LampStrikes => GetResponseParameterData::LampStrikes {
                lamp_strikes: u32::from_be_bytes(bytes[0..=3].try_into().unwrap()),
            },
            ParameterId::LampState => GetResponseParameterData::LampState {
                lamp_state: LampState::from(bytes[0]),
            },
            ParameterId::LampOnMode => GetResponseParameterData::LampOnMode {
                lamp_on_mode: LampOnMode::from(bytes[0]),
            },
            ParameterId::DevicePowerCycles => GetResponseParameterData::DevicePowerCycles {
                power_cycle_count: u32::from_be_bytes(bytes[0..=3].try_into().unwrap()),
            },
            ParameterId::DisplayInvert => GetResponseParameterData::DisplayInvert {
                display_invert_mode: DisplayInvertMode::from(bytes[0]),
            },
            ParameterId::Curve => GetResponseParameterData::Curve {
                current_curve: bytes[0],
                curve_count: bytes[1],
            },
            ParameterId::CurveDescription => GetResponseParameterData::CurveDescription {
                id: bytes[0],
                description: String::from_utf8_lossy(&bytes[1..])
                    .trim_end_matches('\0')
                    .to_string(),
            },
            ParameterId::ModulationFrequency => GetResponseParameterData::ModulationFrequency {
                current_modulation_frequency: bytes[0],
                modulation_frequency_count: bytes[1],
            },
            ParameterId::ModulationFrequencyDescription => {
                GetResponseParameterData::ModulationFrequencyDescription {
                    id: bytes[0],
                    frequency: u32::from_be_bytes(bytes[1..=4].try_into().unwrap()),
                    description: String::from_utf8_lossy(&bytes[5..])
                        .trim_end_matches('\0')
                        .to_string(),
                }
            }
            ParameterId::DimmerInfo => GetResponseParameterData::DimmerInfo {
                minimum_level_lower_limit: u16::from_be_bytes(bytes[0..=1].try_into().unwrap()),
                minimum_level_upper_limit: u16::from_be_bytes(bytes[2..=3].try_into().unwrap()),
                maximum_level_lower_limit: u16::from_be_bytes(bytes[4..=5].try_into().unwrap()),
                maximum_level_upper_limit: u16::from_be_bytes(bytes[6..=7].try_into().unwrap()),
                num_of_supported_curves: bytes[8],
                levels_resolution: bytes[9],
                minimum_levels_split_levels_supports: bytes[10], // TODO could be bool
            },
            ParameterId::MinimumLevel => GetResponseParameterData::MinimumLevel {
                minimum_level_increasing: u16::from_be_bytes(bytes[0..=1].try_into().unwrap()),
                minimum_level_decreasing: u16::from_be_bytes(bytes[2..=3].try_into().unwrap()),
                on_below_minimum: bytes[4],
            },
            ParameterId::MaximumLevel => GetResponseParameterData::MaximumLevel {
                maximum_level: u16::from_be_bytes(bytes[0..=1].try_into().unwrap()),
            },
            ParameterId::OutputResponseTime => GetResponseParameterData::OutputResponseTime {
                current_output_response_time: bytes[0],
                output_response_time_count: bytes[1],
            },
            ParameterId::OutputResponseTimeDescription => {
                GetResponseParameterData::OutputResponseTimeDescription {
                    id: bytes[0],
                    description: String::from_utf8_lossy(&bytes[1..])
                        .trim_end_matches('\0')
                        .to_string(),
                }
            }
            ParameterId::PowerState => GetResponseParameterData::PowerState {
                power_state: PowerState::from(bytes[0]),
            },
            ParameterId::PerformSelfTest => GetResponseParameterData::PerformSelfTest {
                is_active: bytes[0] != 0,
            },
            ParameterId::SelfTestDescription => GetResponseParameterData::SelfTestDescription {
                self_test_id: bytes[0],
                description: String::from_utf8_lossy(&bytes[1..])
                    .trim_end_matches('\0')
                    .to_string(),
            },
            ParameterId::PresetPlayback => GetResponseParameterData::PresetPlayback {
                mode: u16::from_be_bytes(bytes[..=1].try_into().unwrap()),
                level: bytes[2],
            },
            _ => panic!("unsupported parameter"),
        }
    }

    pub fn set_response_bytes_to_parameter_data(
        parameter_id: ParameterId,
        bytes: Bytes,
    ) -> SetResponseParameterData {
        match parameter_id {
            ParameterId::DmxPersonality => SetResponseParameterData::DmxPersonality,
            ParameterId::IdentifyDevice => SetResponseParameterData::IdentifyDevice,
            ParameterId::DeviceLabel => SetResponseParameterData::DeviceLabel,
            ParameterId::DmxStartAddress => SetResponseParameterData::DmxStartAddress,
            ParameterId::Curve => SetResponseParameterData::Curve,
            ParameterId::ModulationFrequency => SetResponseParameterData::ModulationFrequency,
            ParameterId::OutputResponseTime => SetResponseParameterData::OutputResponseTime,
            _ => panic!("unsupported parameter"),
        }
    }

    pub fn discovery_request_parameter_data_to_bytes(
        parameter_data: DiscoveryRequestParameterData,
    ) -> BytesMut {
        let mut bytes = BytesMut::new();

        match parameter_data {
            DiscoveryRequestParameterData::DiscUniqueBranch {
                lower_bound_uid,
                upper_bound_uid,
            } => {
                bytes.put_u16(lower_bound_uid.manufacturer_id);
                bytes.put_u32(lower_bound_uid.device_id);
                bytes.put_u16(upper_bound_uid.manufacturer_id);
                bytes.put_u32(upper_bound_uid.device_id);
            }
        }

        bytes
    }

    pub fn discovery_response_bytes_to_parameter_data(
        parameter_id: ParameterId,
        bytes: Bytes,
    ) -> DiscoveryResponseParameterData {
        match parameter_id {
            ParameterId::DiscMute => {
                // TODO could deduplicate the code here
                let binding_uid = if bytes.len() > 2 {
                    Some(DeviceUID::from(&bytes[2..]))
                } else {
                    None
                };
                DiscoveryResponseParameterData::DiscMute {
                    control_field: u16::from_be_bytes(bytes[..=1].try_into().unwrap()),
                    binding_uid,
                }
            }
            ParameterId::DiscUnMute => {
                let binding_uid = if bytes.len() > 2 {
                    Some(DeviceUID::from(&bytes[2..]))
                } else {
                    None
                };
                DiscoveryResponseParameterData::DiscMute {
                    control_field: u16::from_be_bytes(bytes[..=1].try_into().unwrap()),
                    binding_uid,
                }
            }
            _ => panic!("unsupported parameter"),
        }
    }

    pub fn is_checksum_valid(packet: &[u8]) -> bool {
        let packet_checksum =
            u16::from_be_bytes(packet[packet.len() - 2..packet.len()].try_into().unwrap());

        let calculated_checksum = bsd_16_crc(&packet[..packet.len() - 2]);

        packet_checksum == calculated_checksum
    }
}

impl Encoder<RdmRequestMessage> for RdmCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: RdmRequestMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let (
            command_class,
            destination_uid,
            source_uid,
            transaction_number,
            port_id,
            sub_device,
            parameter_id,
            parameter_data,
        ) = match item {
            RdmRequestMessage::DiscoveryRequest(message) => (
                CommandClass::DiscoveryCommand,
                message.destination_uid,
                message.source_uid,
                message.transaction_number,
                message.port_id,
                message.sub_device_id,
                message.parameter_id,
                message
                    .parameter_data
                    .map(Self::discovery_request_parameter_data_to_bytes),
            ),
            RdmRequestMessage::GetRequest(message) => (
                CommandClass::GetCommand,
                message.destination_uid,
                message.source_uid,
                message.transaction_number,
                message.port_id,
                message.sub_device_id,
                message.parameter_id,
                message
                    .parameter_data
                    .map(Self::get_request_parameter_data_to_bytes),
            ),
            RdmRequestMessage::SetRequest(message) => (
                CommandClass::SetCommand,
                message.destination_uid,
                message.source_uid,
                message.transaction_number,
                message.port_id,
                message.sub_device_id,
                message.parameter_id,
                message
                    .parameter_data
                    .map(Self::set_request_parameter_data_to_bytes),
            ), // _ => panic!("Unknown RdmRequestMessage type"),
        };

        let parameter_data_length = if let Some(parameter_data) = parameter_data.clone() {
            parameter_data.len()
        } else {
            0
        };

        dst.reserve(parameter_data_length + 26); // TODO double check length

        dst.put_u8(Self::SC_RDM); // Start Code
        dst.put_u8(Self::SC_SUB_MESSAGE); // Sub Start Code
        dst.put_u8(parameter_data_length as u8 + 24); // Message Length: Range 24 to 255 excluding the checksum
        dst.put_u16(destination_uid.manufacturer_id);
        dst.put_u32(destination_uid.device_id);
        dst.put_u16(source_uid.manufacturer_id);
        dst.put_u32(source_uid.device_id);
        dst.put_u8(transaction_number);
        dst.put_u8(port_id);
        dst.put_u8(0x00); // Message Count, should always be set to 0x00 for all controller created requests
        dst.put_u16(sub_device); // Sub Device;
        dst.put_u8(command_class as u8);
        dst.put_u16(parameter_id);

        dst.put_u8(parameter_data_length as u8);

        if let Some(bytes) = parameter_data {
            dst.put(bytes);
        }

        dst.put_u16(bsd_16_crc_bytes_mut(&mut dst.clone()));
        Ok(())
    }
}

impl Decoder for RdmCodec {
    type Item = RdmFrame;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        RdmFrame::parse(src).map_err(|err| io::Error::new(io::ErrorKind::Other, err))

        // let len = src.len();

        // let rdm_packet_type =
        //     PacketType::try_from(u16::from_be_bytes(src[0..=1].try_into().unwrap())).unwrap();

        // let frame = match rdm_packet_type {
        //     PacketType::DiscoveryUniqueBranchResponse => {
        //         let euid_start_index = src.iter().position(|x| *x == 0xaa).unwrap();

        //         let euid = Vec::from(&src[(euid_start_index + 1)..=(euid_start_index + 12)]);

        //         let ecs = Vec::from(&src[(euid_start_index + 13)..=(euid_start_index + 16)]);

        //         let decoded_checksum = bsd_16_crc(&euid);

        //         let checksum = u16::from_be_bytes([ecs[0] & ecs[1], ecs[2] & ecs[3]]);

        //         if checksum != decoded_checksum {
        //             return Err(anyhow::anyhow!("decoded checksum incorrect",));
        //             // return Err(io::Error::new(
        //             //     io::ErrorKind::Other,
        //             //     "decoded checksum incorrect",
        //             // ));
        //         }

        //         let manufacturer_id = u16::from_be_bytes([euid[0] & euid[1], euid[2] & euid[3]]);

        //         let device_id = u32::from_be_bytes([
        //             euid[4] & euid[5],
        //             euid[6] & euid[7],
        //             euid[8] & euid[9],
        //             euid[10] & euid[11],
        //         ]);

        //         RdmResponseMessage::DiscoveryUniqueBranchResponse(DeviceUID::new(
        //             manufacturer_id,
        //             device_id,
        //         ))
        //     }
        //     PacketType::RdmResponse => {
        //         // if let Some(start_byte) = src.iter().position(|b| *b == Self::SC_RDM) {
        //         // We can safely ignore any bytes before and including the START_BYTE
        //         // let _ = src.split_to(start_byte);

        //         // TODO should be checking if checksum is valid

        //         if len < Self::FRAME_HEADER_FOOTER_SIZE {
        //             return Ok(None);
        //         }

        //         // let packet_length = src[2] as usize;

        //         // if len < Self::FRAME_HEADER_FOOTER_SIZE + 3 {
        //         //     return Ok(None);
        //         // }

        //         let frame = src
        //             .split_to(len)
        //             // .split_to(packet_length + Self::FRAME_HEADER_FOOTER_SIZE)
        //             .freeze();

        //         // TODO handle unwraps properly
        //         let command_class = CommandClass::try_from(frame[20]).unwrap();
        //         let destination_uid = DeviceUID::from(&frame[3..=8]);
        //         let source_uid = DeviceUID::from(&frame[9..=14]);
        //         let transaction_number = frame[15];
        //         let response_type = ResponseType::try_from(frame[16]).unwrap();
        //         let message_count = frame[17];
        //         let sub_device_id = u16::from_be_bytes(frame[18..=19].try_into().unwrap());
        //         let parameter_id =
        //             ParameterId::try_from(u16::from_be_bytes(frame[21..=22].try_into().unwrap()))
        //                 .unwrap();

        //         let parameter_data_length = frame[23];
        //         let parameter_data: Option<Bytes> = if parameter_data_length > 0 {
        //             Some(frame.slice(24..frame.len() - 2))
        //         } else {
        //             None
        //         };

        //         match command_class {
        //             CommandClass::GetCommandResponse => {
        //                 RdmResponseMessage::GetResponse(GetResponse {
        //                     destination_uid,
        //                     source_uid,
        //                     transaction_number,
        //                     response_type,
        //                     message_count,
        //                     command_class,
        //                     sub_device_id,
        //                     parameter_id,
        //                     parameter_data: parameter_data.map(|parameter_data| {
        //                         Self::get_response_bytes_to_parameter_data(
        //                             parameter_id,
        //                             parameter_data,
        //                         )
        //                     }),
        //                 })
        //             }
        //             CommandClass::SetCommandResponse => {
        //                 RdmResponseMessage::SetResponse(SetResponse {
        //                     destination_uid,
        //                     source_uid,
        //                     transaction_number,
        //                     response_type,
        //                     message_count,
        //                     command_class,
        //                     sub_device_id,
        //                     parameter_id,
        //                     parameter_data: parameter_data.map(|parameter_data| {
        //                         Self::set_response_bytes_to_parameter_data(
        //                             parameter_id,
        //                             parameter_data,
        //                         )
        //                     }),
        //                 })
        //             }
        //             CommandClass::DiscoveryCommandResponse => {
        //                 RdmResponseMessage::DiscoveryResponse(DiscoveryResponse {
        //                     destination_uid,
        //                     source_uid,
        //                     transaction_number,
        //                     response_type,
        //                     message_count,
        //                     command_class,
        //                     sub_device_id,
        //                     parameter_id,
        //                     parameter_data: parameter_data.map(|parameter_data| {
        //                         Self::discovery_response_bytes_to_parameter_data(
        //                             parameter_id,
        //                             parameter_data,
        //                         )
        //                     }),
        //                 })
        //             }
        //             _ => todo!("Unknown CommandClass"),
        //         }

        // } else {
        //     println!("packet length");
        //     // TODO might need to return Err() here
        //     return Ok(None);
        // }
        //     }
        // };

        // Ok(Some(frame))
    }
}

// #[cfg(test)]
// pub mod tests {
//     use super::*;
//     use crate::{device::DeviceUID, request::DiscoveryRequest};
//     use bytes::BytesMut;

//     #[test]
//     fn test_encode_discovery_request() {
//         let mut codec = RdmCodec;

//         let message = RdmRequestMessage::DiscoveryRequest(DiscoveryRequest {
//             destination_uid: DeviceUID::new(0x6574, 0x00000001),
//             source_uid: DeviceUID::new(0x6574, 0x00000002),
//             transaction_number: 0x01,
//             port_id: 0x01,
//             sub_device_id: 0x0001,
//             parameter_id: ParameterId::DiscUniqueBranch as u16,
//             parameter_data: Some(DiscoveryRequestParameterData::DiscUniqueBranch {
//                 lower_bound_uid: DeviceUID::new(0x6574, 0x00000001),
//                 upper_bound_uid: DeviceUID::new(0x6574, 0x00000002),
//             }),
//         });

//         let mut bytes = BytesMut::new();

//         codec.encode(message, &mut bytes).unwrap();

//         let expected = vec![
//             0xcc, 0x01, 0x1c, 0x65, 0x74, 0x00, 0x00, 0x00, 0x01, 0x65, 0x74, 0x00, 0x00, 0x00,
//             0x02, 0x01, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x65, 0x74, 0x00,
//             0x00, 0x00, 0x01, 0x65, 0x74, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01,
//         ];

//         assert_eq!(bytes.to_vec(), expected);
//     }
// }
