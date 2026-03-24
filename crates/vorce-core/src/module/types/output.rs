//!
//! Output variations.
//!

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of final outputs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutputType {
    /// Enumeration variant.
    Projector {
        /// Unique identifier for this entity.
        id: u64,
        /// Human-readable display name.
        name: String,
        #[serde(default)]
        /// Component property or field.
        hide_cursor: bool,
        #[serde(default)]
        /// Component property or field.
        target_screen: u8,
        #[serde(default = "crate::module::config::default_true")]
        /// Component property or field.
        show_in_preview_panel: bool,
        #[serde(default)]
        /// Component property or field.
        extra_preview_window: bool,
        #[serde(default)]
        /// Component property or field.
        output_width: u32,
        #[serde(default)]
        /// Component property or field.
        output_height: u32,
        #[serde(default = "crate::module::config::default_output_fps")]
        /// Component property or field.
        output_fps: f32,
        #[serde(default)]
        /// Component property or field.
        ndi_enabled: bool,
        #[serde(default)]
        /// Display name.
        ndi_stream_name: String,
    },
    /// Enumeration variant.
    NdiOutput {
        /// Human-readable display name.
        name: String,
    },
    #[cfg(target_os = "windows")]
    /// Enumeration variant.
    Spout {
        /// Human-readable display name.
        name: String,
    },
    /// Hue shift in degrees.
    Hue {
        /// Component property or field.
        bridge_ip: String,
        /// Display name.
        username: String,
        /// Component property or field.
        client_key: String,
        /// Component property or field.
        entertainment_area: String,
        /// Component property or field.
        lamp_positions: HashMap<String, (f32, f32)>,
        /// Component property or field.
        mapping_mode: HueMappingMode,
    },
}

/// Mapping mode for Hue Entertainment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HueMappingMode {
    /// Enumeration variant.
    Ambient,
    /// Enumeration variant.
    Spatial,
    /// Event-based trigger node.
    Trigger,
}
