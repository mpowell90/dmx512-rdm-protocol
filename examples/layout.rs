use dmx512_rdm_protocol::rdm::{
    parameter::{
        e120::{DefaultSlotValue, Iso639_1, ProductDetail, ProductDetailValue, StatusMessage},
        e137_2::NetworkInterface,
        e137_7::{EndpointId, EndpointIdValue, EndpointType},
    },
    request::RequestParameter,
    response::ResponseParameterData,
    DeviceUID,
};
use heapless::Vec;

fn main() {
    println!("Size of enum: {}", std::mem::size_of::<RequestParameter>());
    println!(
        "Size of enum: {}",
        std::mem::size_of::<ResponseParameterData>()
    );
    // println!("Size of enum: {}", std::mem::size_of::<ProductDetail>());
    // println!("Size of enum: {}", std::mem::size_of::<Iso639_1>());
    // println!("Size of enum: {}", std::mem::size_of::<DefaultSlotValue>());
    // println!("Size of enum: {}", std::mem::size_of::<EndpointId>());
    // println!("Size of enum: {}", std::mem::size_of::<EndpointType>());
    // println!("Size of enum: {}", std::mem::size_of::<Vec<u8, 231>>());
    // println!(
    //     "Size of enum: {}",
    //     std::mem::size_of::<Vec<DeviceUID, 37>>()
    // );
    println!(
        "Size of enum: {}",
        std::mem::size_of::<Vec<(EndpointId, EndpointType), 75>>()
    );
    println!(
        "Size of enum: {}",
        std::mem::size_of::<Vec<(EndpointIdValue, EndpointType), 75>>()
    );
    // println!(
    //     "Size of enum: {}",
    //     std::mem::size_of::<Vec<NetworkInterface, 38>>()
    // );
    // println!(
    //     "Size of enum: {}",
    //     std::mem::size_of::<Vec<DefaultSlotValue, 77>>()
    // );
    // println!(
    //     "Size of enum: {}",
    //     std::mem::size_of::<Vec<Iso639_1, 115>>()
    // );
    // println!(
    //     "Size of enum: {}",
    //     std::mem::size_of::<Vec<ProductDetail, 115>>()
    // );
    // println!(
    //     "Size of enum: {}",
    //     std::mem::size_of::<Vec<ProductDetailValue, 115>>()
    // );
    // println!("Size of enum: {}", std::mem::size_of::<Vec<u16, 115>>());
    // println!("Size of enum: {}", std::mem::size_of::<StatusMessage>());
    // println!(
    //     "Size of enum: {}",
    //     std::mem::size_of::<Vec<StatusMessage, 25>>()
    // );
}
