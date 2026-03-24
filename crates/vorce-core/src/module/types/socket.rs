//!
//! Socket definitions.
//!
//! # Socket Naming & Semantic Standards
//!
//! To ensure graph consistency, backward compatibility, and an understandable UI:
//!
//! - **IDs**: Standardize IDs for common paths:
//!   - `"media_in"` / `"media_out"`: Primary `Media` connector (Texture/Geometry flow).
//!   - `"trigger_in"` / `"trigger_out"`: General automation and trigger paths (`Trigger` connector).
//!   - `"layer_in"` / `"layer_out"`: Pipeline composition nodes (`Layer` connector).
//!   - `"mask_in"`: Secondary modifier path for Masks.
//!
//! - **Directions**: Sockets distinctly follow `ModuleSocketDirection::Input` or `Output`.
//!   Outputs connect to Inputs of identical `ModuleSocketType`s.
//!
//! - **Primary Path**: Nodes passing a main sequence through should mark the main Input
//!   and Output via `.primary()`.
//!

use serde::{Deserialize, Serialize};

/// Direction of a module socket.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ModuleSocketDirection {
    /// Input socket.
    #[default]
    Input,
    /// Output socket.
    Output,
}

/// A connection point on a node
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleSocket {
    /// Stable schema ID for this socket.
    #[serde(default)]
    pub id: String,
    /// Label for the socket
    pub name: String,
    /// Data type accepted/provided
    pub socket_type: ModuleSocketType,
    /// Whether this socket is an input or output.
    #[serde(default)]
    pub direction: ModuleSocketDirection,
    /// Whether the inspector may map trigger automation onto this input.
    #[serde(default)]
    pub supports_trigger_mapping: bool,
    /// Whether this socket is the primary path for the render chain.
    #[serde(default)]
    pub is_primary: bool,
    /// Whether multiple connections may target this input.
    #[serde(default)]
    pub accepts_multiple_connections: bool,
}

impl ModuleSocket {
    /// Create a new input socket.
    pub fn input(
        id: impl Into<String>,
        name: impl Into<String>,
        socket_type: ModuleSocketType,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            socket_type,
            direction: ModuleSocketDirection::Input,
            supports_trigger_mapping: false,
            is_primary: false,
            accepts_multiple_connections: false,
        }
    }

    // Standard Socket Builders to prevent drift

    /// Create a standard visual/media primary input.
    pub fn standard_media_in() -> Self {
        Self::input("media_in", "Media In", ModuleSocketType::Media).primary()
    }

    /// Create a standard visual/media primary output.
    pub fn standard_media_out() -> Self {
        Self::output("media_out", "Media Out", ModuleSocketType::Media).primary()
    }

    /// Create a standard legacy trigger input.
    pub fn standard_trigger_in() -> Self {
        Self::input_mappable("trigger_in", "Trigger In", ModuleSocketType::Trigger)
    }

    /// Create a standard legacy trigger output.
    pub fn standard_trigger_out() -> Self {
        Self::output("trigger_out", "Trigger Out", ModuleSocketType::Trigger)
    }

    /// Create a standard layer primary input.
    pub fn standard_layer_in() -> Self {
        Self::input("layer_in", "Layer In", ModuleSocketType::Layer).primary()
    }

    /// Create a standard layer primary output.
    pub fn standard_layer_out() -> Self {
        Self::output("layer_out", "Layer Out", ModuleSocketType::Layer).primary()
    }

    /// Create a standard mask input socket.
    pub fn standard_mask_in() -> Self {
        Self::input("mask_in", "Mask In", ModuleSocketType::Media)
    }

    /// Create a new input socket that supports trigger mapping.
    pub fn input_mappable(
        id: impl Into<String>,
        name: impl Into<String>,
        socket_type: ModuleSocketType,
    ) -> Self {
        Self {
            supports_trigger_mapping: true,
            ..Self::input(id, name, socket_type)
        }
    }

    /// Create a new output socket.
    pub fn output(
        id: impl Into<String>,
        name: impl Into<String>,
        socket_type: ModuleSocketType,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            socket_type,
            direction: ModuleSocketDirection::Output,
            supports_trigger_mapping: false,
            is_primary: false,
            accepts_multiple_connections: false,
        }
    }

    /// Mark a socket as primary.
    pub fn primary(mut self) -> Self {
        self.is_primary = true;
        self
    }

    /// Allow multiple connections on this socket.
    pub fn multi_input(mut self) -> Self {
        self.accepts_multiple_connections = true;
        self
    }

    /// Check whether a source socket may connect into a target socket.
    pub fn is_compatible_with(&self, target: &Self) -> bool {
        self.direction == ModuleSocketDirection::Output
            && target.direction == ModuleSocketDirection::Input
            && self.socket_type == target.socket_type
    }
}

/// Type of data carried by a connection
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModuleSocketType {
    /// Event-based trigger node.
    Trigger,
    /// Enumeration variant.
    Media,
    /// Enumeration variant.
    Effect,
    /// A compositing layer within a scene.
    Layer,
    /// Enumeration variant.
    Output,
    /// Enumeration variant.
    Link,
}

impl ModuleSocketType {
    /// Human-readable display name.
    pub fn name(&self) -> &'static str {
        match self {
            ModuleSocketType::Trigger => "Trigger",
            ModuleSocketType::Media => "Media",
            ModuleSocketType::Effect => "Effect",
            ModuleSocketType::Layer => "Layer",
            ModuleSocketType::Output => "Output",
            ModuleSocketType::Link => "Link",
        }
    }
}

/// Blend mode types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BlendModeType {
    /// Enumeration variant.
    Normal,
    /// Enumeration variant.
    Add,
    /// Enumeration variant.
    Multiply,
    /// Enumeration variant.
    Screen,
    /// Enumeration variant.
    Overlay,
    /// Enumeration variant.
    Difference,
    /// Enumeration variant.
    Exclusion,
}

impl BlendModeType {
    /// Associated function.
    pub fn all() -> &'static [BlendModeType] {
        &[
            BlendModeType::Normal,
            BlendModeType::Add,
            BlendModeType::Multiply,
            BlendModeType::Screen,
            BlendModeType::Overlay,
            BlendModeType::Difference,
            BlendModeType::Exclusion,
        ]
    }

    /// Human-readable display name.
    pub fn name(&self) -> &'static str {
        match self {
            BlendModeType::Normal => "Normal",
            BlendModeType::Add => "Add",
            BlendModeType::Multiply => "Multiply",
            BlendModeType::Screen => "Screen",
            BlendModeType::Overlay => "Overlay",
            BlendModeType::Difference => "Difference",
            BlendModeType::Exclusion => "Exclusion",
        }
    }
}
