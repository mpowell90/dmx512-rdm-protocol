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
//! See module documentation or test suite for examples.

#![cfg_attr(not(feature = "alloc"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod dmx;

#[cfg(feature = "rdm")]
pub mod rdm;
