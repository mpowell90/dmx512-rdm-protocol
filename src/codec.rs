// use crate::{
//     bsd_16_crc, bsd_16_crc_bytes_mut,
//     request::{
//         DiscoveryRequestParameterData, GetRequestParameterData, RdmRequestMessage,
//         SetRequestParameterData,
//     },
//     response::RdmFrame,
//     CommandClass,
// };
// use bytes::{BufMut, BytesMut};
// use std::io;
// use tokio_util::codec::{Decoder, Encoder};

// #[derive(Copy, Clone, Default)]
// pub struct RdmCodec;

// impl RdmCodec {
//     pub fn get_request_parameter_data_to_bytes(
//         parameter_data: GetRequestParameterData,
//     ) -> BytesMut {
//         let mut bytes = BytesMut::new();

//         match parameter_data {
//             GetRequestParameterData::ParameterDescription { parameter_id } => {
//                 bytes.put_u16(parameter_id)
//             }
//             GetRequestParameterData::SensorDefinition { sensor_id } => bytes.put_u8(sensor_id),
//             GetRequestParameterData::DmxPersonalityDescription { personality } => {
//                 bytes.put_u8(personality)
//             }
//             GetRequestParameterData::CurveDescription { curve } => bytes.put_u8(curve),
//             GetRequestParameterData::ModulationFrequencyDescription {
//                 modulation_frequency,
//             } => bytes.put_u8(modulation_frequency),
//             GetRequestParameterData::OutputResponseTimeDescription {
//                 output_response_time,
//             } => bytes.put_u8(output_response_time),
//             GetRequestParameterData::SelfTestDescription { self_test_id } => {
//                 bytes.put_u8(self_test_id)
//             }
//             GetRequestParameterData::SlotDescription { slot_id } => bytes.put_u16(slot_id),
//         }

//         bytes
//     }

//     pub fn set_request_parameter_data_to_bytes(
//         parameter_data: SetRequestParameterData,
//     ) -> BytesMut {
//         let mut bytes = BytesMut::new();

//         match parameter_data {
//             SetRequestParameterData::DmxPersonality { personality_id } => {
//                 bytes.put_u8(personality_id)
//             }
//             SetRequestParameterData::DeviceLabel { device_label } => {
//                 bytes.extend(device_label.as_bytes())
//             }
//             SetRequestParameterData::DmxStartAddress { dmx_start_address } => {
//                 bytes.put_u16(dmx_start_address)
//             }
//             SetRequestParameterData::Curve { curve_id } => bytes.put_u8(curve_id),
//             SetRequestParameterData::ModulationFrequency {
//                 modulation_frequency_id,
//             } => bytes.put_u8(modulation_frequency_id),
//             SetRequestParameterData::OutputResponseTime {
//                 output_response_time_id,
//             } => bytes.put_u8(output_response_time_id),
//             SetRequestParameterData::IdentifyDevice { identify } => bytes.put_u8(identify.into()),
//         }

//         bytes
//     }

//     pub fn discovery_request_parameter_data_to_bytes(
//         parameter_data: DiscoveryRequestParameterData,
//     ) -> BytesMut {
//         let mut bytes = BytesMut::new();

//         match parameter_data {
//             DiscoveryRequestParameterData::DiscUniqueBranch {
//                 lower_bound_uid,
//                 upper_bound_uid,
//             } => {
//                 bytes.put_u16(lower_bound_uid.manufacturer_id);
//                 bytes.put_u32(lower_bound_uid.device_id);
//                 bytes.put_u16(upper_bound_uid.manufacturer_id);
//                 bytes.put_u32(upper_bound_uid.device_id);
//             }
//         }

//         bytes
//     }

//     pub fn is_checksum_valid(packet: &[u8]) -> bool {
//         let packet_checksum =
//             u16::from_be_bytes(packet[packet.len() - 2..packet.len()].try_into().unwrap());

//         let calculated_checksum = bsd_16_crc(&packet[..packet.len() - 2]);

//         packet_checksum == calculated_checksum
//     }
// }

// impl Encoder<RdmRequestMessage> for RdmCodec {
//     type Error = anyhow::Error;

//     fn encode(&mut self, item: RdmRequestMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
//         let (
//             command_class,
//             destination_uid,
//             source_uid,
//             transaction_number,
//             port_id,
//             sub_device,
//             parameter_id,
//             parameter_data,
//         ) = match item {
//             RdmRequestMessage::DiscoveryRequest(message) => (
//                 CommandClass::DiscoveryCommand,
//                 message.destination_uid,
//                 message.source_uid,
//                 message.transaction_number,
//                 message.port_id,
//                 message.sub_device_id,
//                 message.parameter_id,
//                 message
//                     .parameter_data
//                     .map(Self::discovery_request_parameter_data_to_bytes),
//             ),
//             RdmRequestMessage::GetRequest(message) => (
//                 CommandClass::GetCommand,
//                 message.destination_uid,
//                 message.source_uid,
//                 message.transaction_number,
//                 message.port_id,
//                 message.sub_device_id,
//                 message.parameter_id,
//                 message
//                     .parameter_data
//                     .map(Self::get_request_parameter_data_to_bytes),
//             ),
//             RdmRequestMessage::SetRequest(message) => (
//                 CommandClass::SetCommand,
//                 message.destination_uid,
//                 message.source_uid,
//                 message.transaction_number,
//                 message.port_id,
//                 message.sub_device_id,
//                 message.parameter_id,
//                 message
//                     .parameter_data
//                     .map(Self::set_request_parameter_data_to_bytes),
//             ), // _ => panic!("Unknown RdmRequestMessage type"),
//         };

//         let parameter_data_length = if let Some(parameter_data) = parameter_data.clone() {
//             parameter_data.len()
//         } else {
//             0
//         };

//         dst.reserve(parameter_data_length + 26); // TODO double check length

//         dst.put_u8(super::SC_RDM); // Start Code
//         dst.put_u8(super::SC_SUB_MESSAGE); // Sub Start Code
//         dst.put_u8(parameter_data_length as u8 + 24); // Message Length: Range 24 to 255 excluding the checksum
//         dst.put_u16(destination_uid.manufacturer_id);
//         dst.put_u32(destination_uid.device_id);
//         dst.put_u16(source_uid.manufacturer_id);
//         dst.put_u32(source_uid.device_id);
//         dst.put_u8(transaction_number);
//         dst.put_u8(port_id);
//         dst.put_u8(0x00); // Message Count, should always be set to 0x00 for all controller created requests
//         dst.put_u16(sub_device); // Sub Device;
//         dst.put_u8(command_class as u8);
//         dst.put_u16(parameter_id);

//         dst.put_u8(parameter_data_length as u8);

//         if let Some(bytes) = parameter_data {
//             dst.put(bytes);
//         }

//         dst.put_u16(bsd_16_crc_bytes_mut(&mut dst.clone()));
//         Ok(())
//     }
// }

// impl Decoder for RdmCodec {
//     type Item = RdmFrame;
//     type Error = io::Error;

//     fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
//         RdmFrame::parse(src).map_err(|err| io::Error::new(io::ErrorKind::Other, err))

//         // let len = src.len();

//         // let rdm_packet_type =
//         //     PacketType::try_from(u16::from_be_bytes(src[0..=1].try_into().unwrap())).unwrap();

//         // let frame = match rdm_packet_type {
//         //     PacketType::DiscoveryUniqueBranchResponse => {
//         //         let euid_start_index = src.iter().position(|x| *x == 0xaa).unwrap();

//         //         let euid = Vec::from(&src[(euid_start_index + 1)..=(euid_start_index + 12)]);

//         //         let ecs = Vec::from(&src[(euid_start_index + 13)..=(euid_start_index + 16)]);

//         //         let decoded_checksum = bsd_16_crc(&euid);

//         //         let checksum = u16::from_be_bytes([ecs[0] & ecs[1], ecs[2] & ecs[3]]);

//         //         if checksum != decoded_checksum {
//         //             return Err(anyhow::anyhow!("decoded checksum incorrect",));
//         //             // return Err(io::Error::new(
//         //             //     io::ErrorKind::Other,
//         //             //     "decoded checksum incorrect",
//         //             // ));
//         //         }

//         //         let manufacturer_id = u16::from_be_bytes([euid[0] & euid[1], euid[2] & euid[3]]);

//         //         let device_id = u32::from_be_bytes([
//         //             euid[4] & euid[5],
//         //             euid[6] & euid[7],
//         //             euid[8] & euid[9],
//         //             euid[10] & euid[11],
//         //         ]);

//         //         RdmResponseMessage::DiscoveryUniqueBranchResponse(DeviceUID::new(
//         //             manufacturer_id,
//         //             device_id,
//         //         ))
//         //     }
//         //     PacketType::RdmResponse => {
//         //         // if let Some(start_byte) = src.iter().position(|b| *b == Self::SC_RDM) {
//         //         // We can safely ignore any bytes before and including the START_BYTE
//         //         // let _ = src.split_to(start_byte);

//         //         // TODO should be checking if checksum is valid

//         //         if len < Self::FRAME_HEADER_FOOTER_SIZE {
//         //             return Ok(None);
//         //         }

//         //         // let packet_length = src[2] as usize;

//         //         // if len < Self::FRAME_HEADER_FOOTER_SIZE + 3 {
//         //         //     return Ok(None);
//         //         // }

//         //         let frame = src
//         //             .split_to(len)
//         //             // .split_to(packet_length + Self::FRAME_HEADER_FOOTER_SIZE)
//         //             .freeze();

//         //         // TODO handle unwraps properly
//         //         let command_class = CommandClass::try_from(frame[20]).unwrap();
//         //         let destination_uid = DeviceUID::from(&frame[3..=8]);
//         //         let source_uid = DeviceUID::from(&frame[9..=14]);
//         //         let transaction_number = frame[15];
//         //         let response_type = ResponseType::try_from(frame[16]).unwrap();
//         //         let message_count = frame[17];
//         //         let sub_device_id = u16::from_be_bytes(frame[18..=19].try_into().unwrap());
//         //         let parameter_id =
//         //             ParameterId::try_from(u16::from_be_bytes(frame[21..=22].try_into().unwrap()))
//         //                 .unwrap();

//         //         let parameter_data_length = frame[23];
//         //         let parameter_data: Option<Bytes> = if parameter_data_length > 0 {
//         //             Some(frame.slice(24..frame.len() - 2))
//         //         } else {
//         //             None
//         //         };

//         //         match command_class {
//         //             CommandClass::GetCommandResponse => {
//         //                 RdmResponseMessage::GetResponse(GetResponse {
//         //                     destination_uid,
//         //                     source_uid,
//         //                     transaction_number,
//         //                     response_type,
//         //                     message_count,
//         //                     command_class,
//         //                     sub_device_id,
//         //                     parameter_id,
//         //                     parameter_data: parameter_data.map(|parameter_data| {
//         //                         Self::get_response_bytes_to_parameter_data(
//         //                             parameter_id,
//         //                             parameter_data,
//         //                         )
//         //                     }),
//         //                 })
//         //             }
//         //             CommandClass::SetCommandResponse => {
//         //                 RdmResponseMessage::SetResponse(SetResponse {
//         //                     destination_uid,
//         //                     source_uid,
//         //                     transaction_number,
//         //                     response_type,
//         //                     message_count,
//         //                     command_class,
//         //                     sub_device_id,
//         //                     parameter_id,
//         //                     parameter_data: parameter_data.map(|parameter_data| {
//         //                         Self::set_response_bytes_to_parameter_data(
//         //                             parameter_id,
//         //                             parameter_data,
//         //                         )
//         //                     }),
//         //                 })
//         //             }
//         //             CommandClass::DiscoveryCommandResponse => {
//         //                 RdmResponseMessage::DiscoveryResponse(DiscoveryResponse {
//         //                     destination_uid,
//         //                     source_uid,
//         //                     transaction_number,
//         //                     response_type,
//         //                     message_count,
//         //                     command_class,
//         //                     sub_device_id,
//         //                     parameter_id,
//         //                     parameter_data: parameter_data.map(|parameter_data| {
//         //                         Self::discovery_response_bytes_to_parameter_data(
//         //                             parameter_id,
//         //                             parameter_data,
//         //                         )
//         //                     }),
//         //                 })
//         //             }
//         //             _ => todo!("Unknown CommandClass"),
//         //         }

//         // } else {
//         //     println!("packet length");
//         //     // TODO might need to return Err() here
//         //     return Ok(None);
//         // }
//         //     }
//         // };

//         // Ok(Some(frame))
//     }
// }

// // #[cfg(test)]
// // pub mod tests {
// //     use super::*;
// //     use crate::{device::DeviceUID, request::DiscoveryRequest};
// //     use bytes::BytesMut;

// //     #[test]
// //     fn test_encode_discovery_request() {
// //         let mut codec = RdmCodec;

// //         let message = RdmRequestMessage::DiscoveryRequest(DiscoveryRequest {
// //             destination_uid: DeviceUID::new(0x6574, 0x00000001),
// //             source_uid: DeviceUID::new(0x6574, 0x00000002),
// //             transaction_number: 0x01,
// //             port_id: 0x01,
// //             sub_device_id: 0x0001,
// //             parameter_id: ParameterId::DiscUniqueBranch as u16,
// //             parameter_data: Some(DiscoveryRequestParameterData::DiscUniqueBranch {
// //                 lower_bound_uid: DeviceUID::new(0x6574, 0x00000001),
// //                 upper_bound_uid: DeviceUID::new(0x6574, 0x00000002),
// //             }),
// //         });

// //         let mut bytes = BytesMut::new();

// //         codec.encode(message, &mut bytes).unwrap();

// //         let expected = vec![
// //             0xcc, 0x01, 0x1c, 0x65, 0x74, 0x00, 0x00, 0x00, 0x01, 0x65, 0x74, 0x00, 0x00, 0x00,
// //             0x02, 0x01, 0x01, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x65, 0x74, 0x00,
// //             0x00, 0x00, 0x01, 0x65, 0x74, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x01,
// //         ];

// //         assert_eq!(bytes.to_vec(), expected);
// //     }
// // }
