//!
//! Hue settings and properties.
//!

use serde::{Deserialize, Serialize};

/// Types of Philips Hue Nodes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HueNodeType {
    /// Enumeration variant.
    SingleLamp {
        /// Unique identifier for this entity.
        id: String,
        /// Human-readable display name.
        name: String,
        #[serde(default = "crate::module::config::default_opacity")]
        /// Brightness factor.
        brightness: f32,
        #[serde(default = "crate::module::config::default_hue_color")]
        /// RGBA color value.
        color: [f32; 3],
        #[serde(default)]
        /// Component property or field.
        effect: Option<String>,
        #[serde(default)]
        /// Component property or field.
        effect_active: bool,
    },
    /// Enumeration variant.
    MultiLamp {
        /// Component property or field.
        ids: Vec<String>,
        /// Human-readable display name.
        name: String,
        #[serde(default = "crate::module::config::default_opacity")]
        /// Brightness factor.
        brightness: f32,
        #[serde(default = "crate::module::config::default_hue_color")]
        /// RGBA color value.
        color: [f32; 3],
        #[serde(default)]
        /// Component property or field.
        effect: Option<String>,
        #[serde(default)]
        /// Component property or field.
        effect_active: bool,
    },
    /// Enumeration variant.
    EntertainmentGroup {
        /// Human-readable display name.
        name: String,
        #[serde(default = "crate::module::config::default_opacity")]
        /// Brightness factor.
        brightness: f32,
        #[serde(default = "crate::module::config::default_hue_color")]
        /// RGBA color value.
        color: [f32; 3],
        #[serde(default)]
        /// Component property or field.
        effect: Option<String>,
        #[serde(default)]
        /// Component property or field.
        effect_active: bool,
    },
}
