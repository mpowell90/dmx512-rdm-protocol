//! DMX512 and Remote Device Management (RDM) protocol written in Rust
//!
//! ## About the project
//!
//! DMX512 is a unidirectional packet based communication protocol commonly used to control lighting and effects.
//!
//! Remote Device Management (RDM) is a backward-compatible extension of DMX512, enabling bi-directional communication with compatible devices for the purpose of discovery, identification, reporting, and configuration.
//!
//! ### Included
//!
//! Data-types and functionality for encoding and decoding DMX512 and RDM packets.
//!
//! ### Not included
//!
//! Driver implementations: DMX512 / RDM packets are transmitted over an RS485 bus but interface devices like the Enttec DMX USB PRO exist to communicate with devices via USB. These interface devices usually require extra packet framing ontop of the standard DMX512 / RDM packets, which is out of scope of this project.
//!
//! ### Implemented specifications
//!
//! - ANSI E1.11 (2008): Asynchronous Serial Digital Data Transmission Standard for Controlling Lighting Equipment and Accessories
//! - ANSI E1.20 (2010): RDM Remote Device Management Over DMX512 Networks
//!
//! ## Usage
//!
//! ### DmxUniverse
//!
//! ```rust
//! use dmx512_rdm_protocol::dmx::DmxUniverse;
//!
//! // Create a 512 channel universe
//! let dmx_universe = DmxUniverse::default();
//! // or create a smaller universe
//! let mut dmx_universe = DmxUniverse::new(4).unwrap();
//!
//! dmx_universe.set_channel_value(0, 64).unwrap();
//! dmx_universe.set_channel_values(1, &[128, 192, 255]).unwrap();
//!
//! assert_eq!(dmx_universe.get_channel_value(0).unwrap(), 64);
//! assert_eq!(dmx_universe.get_channel_values(1..=2).unwrap(), &[128, 192]);
//! assert_eq!(dmx_universe.as_bytes(), &[64, 128, 192, 255]);
//! assert_eq!(dmx_universe.encode(), &[0, 64, 128, 192, 255]);
//! ```
//!
//! ### RdmRequest
//!
//! ```rust
//! use dmx512_rdm_protocol::rdm::{
//!     request::{RdmRequest, RequestParameter},
//!     DeviceUID, SubDeviceId,
//! };
//!
//! let encoded = RdmRequest::new(
//!     DeviceUID::new(0x0102, 0x03040506),
//!     DeviceUID::new(0x0605, 0x04030201),
//!     0x00,
//!     0x01,
//!     SubDeviceId::RootDevice,
//!     RequestParameter::GetIdentifyDevice,
//! )
//! .encode();
//!
//! let expected = &[
//!     0xcc, // Start Code
//!     0x01, // Sub Start Code
//!     0x18, // Message Length
//!     0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // Destination UID
//!     0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // Source UID
//!     0x00, // Transaction Number
//!     0x01, // Port ID
//!     0x00, // Message Count
//!     0x00, 0x00, // Sub-Device ID = Root Device
//!     0x20, // Command Class = GetCommand
//!     0x10, 0x00, // Parameter ID = Identify Device
//!     0x00, // PDL
//!     0x01, 0x40, // Checksum
//! ];
//!
//! assert_eq!(encoded, expected);
//! ```
//!
//! ### RdmResponse
//!
//! ```rust
//! use dmx512_rdm_protocol::rdm::{
//!     parameter::ParameterId,
//!     response::{
//!         RdmFrameResponse, RdmResponse, ResponseData, ResponseParameterData, ResponseType,
//!     },
//!     CommandClass, DeviceUID, SubDeviceId,
//! };
//! 
//! let decoded = RdmResponse::decode(&[
//!     0xcc, // Start Code
//!     0x01, // Sub Start Code
//!     0x19, // Message Length
//!     0x01, 0x02, 0x03, 0x04, 0x05, 0x06, // Destination UID
//!     0x06, 0x05, 0x04, 0x03, 0x02, 0x01, // Source UID
//!     0x00, // Transaction Number
//!     0x00, // Response Type = Ack
//!     0x00, // Message Count
//!     0x00, 0x00, // Sub-Device ID = Root Device
//!     0x21, // Command Class = GetCommandResponse
//!     0x10, 0x00, // Parameter ID = Identify Device
//!     0x01, // PDL
//!     0x01, // Identifying = true
//!     0x01, 0x43, // Checksum
//! ]);
//!
//! let expected = Ok(RdmResponse::RdmFrame(RdmFrameResponse {
//!     destination_uid: DeviceUID::new(0x0102, 0x03040506),
//!     source_uid: DeviceUID::new(0x0605, 0x04030201),
//!     transaction_number: 0x00,
//!     response_type: ResponseType::Ack,
//!     message_count: 0x00,
//!     sub_device_id: SubDeviceId::RootDevice,
//!     command_class: CommandClass::GetCommandResponse,
//!     parameter_id: ParameterId::IdentifyDevice,
//!     parameter_data: ResponseData::ParameterData(Some(
//!         ResponseParameterData::GetIdentifyDevice(true),
//!     )),
//! }));
//!
//! assert_eq!(decoded, expected);
//! ```

/// DMX512 Data types and functionality
pub mod dmx;

/// RDM Data types and functionality
#[cfg(feature = "rdm")]
pub mod rdm;
