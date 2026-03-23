//! OSC (Open Sound Control) system
//!
//! This module provides OSC server and client functionality for remote control of MapFlow.
//!
//! ## OSC Address Space
//!
//! ```text
//! /mapflow/layer/{id}/opacity       [f32: 0.0-1.0]
//! /mapflow/layer/{id}/position      [f32, f32: x, y]
//! /mapflow/layer/{id}/rotation      [f32: degrees]
//! /mapflow/layer/{id}/scale         [f32: scale]
//! /mapflow/layer/{id}/visibility    [bool]
//! /mapflow/paint/{id}/parameter/{name}  [varies]
//! /mapflow/effect/{id}/parameter/{name} [varies]
//! /mapflow/playback/speed           [f32: speed multiplier]
//! /mapflow/playback/position        [f32: 0.0-1.0]
//! /mapflow/output/{id}/brightness   [f32: 0.0-1.0]
//! ```
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use mapmap_control::osc::{OscServer, OscClient};
//! use mapmap_control::{ControlTarget, ControlValue};
//!
//! # #[cfg(feature = "osc")]
//! # fn main() -> mapmap_control::Result<()> {
//! // Create server
//! let server = OscServer::new(8000)?;
//!
//! // Create client for sending state updates
//! let client = OscClient::new("192.168.1.100:8001")?;
//!
//! // Poll for packets
//! while let Some(_packet) = server.poll_packet() {
//!     // Handle the OSC packet (Message or Bundle)
//!     println!("Received OSC packet");
//! }
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "osc"))]
//! # fn main() {}
//! ```

pub mod address;
pub mod client;
pub mod mapping;
pub mod server;
pub mod types;

pub use address::{control_target_to_address, parse_osc_address};
pub use client::OscClient;
pub use mapping::OscMapping;
pub use server::OscServer;

#[cfg(feature = "osc")]
pub use types::{control_value_to_osc, osc_to_control_value, osc_to_vec2, osc_to_vec3};
