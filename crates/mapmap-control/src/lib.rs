//! MapFlow Control - Professional Control System Integration
//!
//! This crate provides comprehensive control system integration for MapFlow including:
//! - **MIDI**: Input/output, learn mode, controller profiles, clock sync
//! - **OSC**: Server/client for TouchOSC, Lemur, and custom apps
//! - **DMX**: Art-Net and sACN output for lighting control
//! - **Web API**: REST API and WebSocket for remote control
//! - **Cue System**: Automated shows with crossfades and triggers
//!
//! ## Feature Flags
//!
//! - `midi`: Enable MIDI support (requires `midir`)
//! - `osc`: Enable OSC support (requires `rosc`)
//! - `http-api`: Enable web API (requires `axum`, `tokio`)
//! - `full`: Enable all features
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use mapmap_control::{ControlTarget, ControlValue};
//!
//! // Define a control target
//! let target = ControlTarget::LayerOpacity(0);
//! let value = ControlValue::Float(0.75);
//! ```
//!
//! ## Modules
//!
//! - [`midi`]: MIDI input/output system
//! - [`osc`]: OSC server and client
//! - [`dmx`]: DMX output via Art-Net and sACN
//! - [`web`]: Web API and WebSocket
//! - [`cue`]: Cue system for show automation
//! - [`shortcuts`]: Keyboard shortcuts and macros
//! - [`target`]: Control target abstraction
//! - [`error`]: Error types

// Core modules
pub mod error;
pub mod manager;
pub mod target;

// Control system modules
#[cfg(feature = "midi")]
pub mod midi;

pub mod dmx;
pub mod hue;

#[cfg(feature = "osc")]
pub mod osc;

#[cfg(feature = "link")]
pub mod link;

#[cfg(feature = "http-api")]
pub mod web;

pub mod cue;
pub mod shortcuts;

// Re-exports
pub use error::{ControlError, Result};
pub use manager::ControlManager;
pub use target::{ControlTarget, ControlValue, EdgeSide};

#[cfg(feature = "midi")]
pub use midi::MidiMessage;

pub use dmx::{ArtNetSender, ChannelAssignment, DmxChannel, Fixture, FixtureProfile, SacnSender};
pub use hue::controller::HueController;
pub use hue::models::HueConfig;

#[cfg(feature = "osc")]
pub use osc::{OscClient, OscMapping, OscServer};

#[cfg(feature = "link")]
pub use link::AbletonLinkHandle;

#[cfg(feature = "http-api")]
pub use web::{WebServer, WebServerConfig};

pub use cue::{Cue, CueList, FadeCurve, LayerState};
pub use shortcuts::{
    Action, Key, KeyBindings, Macro, MacroPlayer, MacroRecorder, Modifiers, Shortcut,
    ShortcutContext,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_value_creation() {
        let _float_val = ControlValue::Float(0.5);
        let _int_val = ControlValue::Int(42);
        let _bool_val = ControlValue::Bool(true);
    }

    #[test]
    fn test_control_target_creation() {
        let _layer_opacity = ControlTarget::LayerOpacity(0);
        // let _playback_speed = ControlTarget::PlaybackSpeed(None);
    }
}
