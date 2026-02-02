// use dmx512_rdm_protocol::rdm::{
//     CommandClass, DeviceUID, RdmFrame, SubDeviceId,
//     parameter::{
//         ParameterId,
//         e120::{DefaultSlotValue, Iso639_1, ProductDetail, StatusMessage},
//         e137_2::NetworkInterface,
//         e137_7::{EndpointId, EndpointIdValue, EndpointType, GetEndpointLabelRequest},
//     },
//     request::{RdmRequest, RequestParameter},
//     response::ResponseParameterData,
// };
// use heapless::Vec;
// use rdm_derive::rdm_parameter;
// use rdm_core::parameter_traits::{RdmParameterCodec, RdmParameterData};

// #[derive(Clone, Debug, PartialEq)]
// #[rdm_parameter(pid = 0x8001, command_class = 0x01)]
// struct MyStruct {
//     one: u8,
//     data: heapless::Vec<u16, 4>,
// }

fn main() {
    //     let mut buf = [0u8; 20];

    //     let t = MyStruct {
    //         one: 5,
    //         data: Vec::from_slice(&[1, 2]).unwrap(),
    //     };

    //     // dbg!("Size of MyStruct: {}", t.size_of_parameter_data());
    //     // dbg!(t.encode_parameter_data(&mut buf).unwrap());
    //     // dbg!(buf);
    //     // dbg!(MyStruct::decode_parameter_data(&buf).unwrap());
    //     dbg!("Size of MyStruct: {}", t.size_of_parameter_data());
    //     dbg!(t.encode_parameter(&mut buf).unwrap());
    //     dbg!(buf);
    //     dbg!(MyStruct::decode_parameter(&buf).unwrap());

    //     // println!("Size of enum: {}", std::mem::size_of::<RequestParameter>());
    //     // println!(
    //     //     "Size of enum: {}",
    //     //     std::mem::size_of::<ResponseParameterData>()
    //     // );
    //     // println!("Size of enum: {}", std::mem::size_of::<ProductDetail>());
    //     // println!("Size of enum: {}", std::mem::size_of::<Iso639_1>());
    //     // println!("Size of enum: {}", std::mem::size_of::<DefaultSlotValue>());
    //     // println!("Size of enum: {}", std::mem::size_of::<EndpointId>());
    //     // println!("Size of enum: {}", std::mem::size_of::<EndpointType>());
    //     // println!("Size of enum: {}", std::mem::size_of::<Vec<u8, 231>>());
    //     // println!(
    //     //     "Size of enum: {}",
    //     //     std::mem::size_of::<Vec<DeviceUID, 37>>()
    //     // );
    //     // println!(
    //     //     "Size of enum: {}",
    //     //     std::mem::size_of::<Vec<(EndpointId, EndpointType), 75>>()
    //     // );
    //     // println!(
    //     //     "Size of enum: {}",
    //     //     std::mem::size_of::<Vec<(EndpointIdValue, EndpointType), 75>>()
    //     // );
    //     // println!(
    //     //     "Size of enum: {}",
    //     //     std::mem::size_of::<Vec<NetworkInterface, 38>>()
    //     // );
    //     // println!(
    //     //     "Size of enum: {}",
    //     //     std::mem::size_of::<Vec<DefaultSlotValue, 77>>()
    //     // );
    //     // println!(
    //     //     "Size of enum: {}",
    //     //     std::mem::size_of::<Vec<Iso639_1, 115>>()
    //     // );
    //     // println!(
    //     //     "Size of enum: {}",
    //     //     std::mem::size_of::<Vec<ProductDetail, 115>>()
    //     // );
    //     // println!(
    //     //     "Size of enum: {}",
    //     //     std::mem::size_of::<Vec<ProductDetailValue, 115>>()
    //     // );
    //     // println!("Size of enum: {}", std::mem::size_of::<Vec<u16, 115>>());
    //     // println!("Size of enum: {}", std::mem::size_of::<StatusMessage>());
    //     // println!(
    //     //     "Size of enum: {}",
    //     //     std::mem::size_of::<Vec<StatusMessage, 25>>()
    //     // );

    //     // let frame1 = RdmFrame::Request(RdmRequest::new(
    //     //     DeviceUID::broadcast_to_all_devices(),
    //     //     DeviceUID::broadcast_to_all_devices(),
    //     //     0x01,
    //     //     0x01,
    //     //     SubDeviceId::RootDevice,
    //     //     RequestParameter::GetEndpointLabel(GetEndpointLabelRequest {
    //     //         endpoint_id: EndpointId::Device(0x0001),
    //     //     }),
    //     // ));

    //     // let mut buf = [0u8; 257];

    //     // let bytes_written = frame1.encode(&mut buf);

    //     // println!("{}", bytes_written.unwrap());
}
