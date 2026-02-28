use serde::{Deserialize, Serialize};

/// A connection point on a node
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModuleSocket {
    /// Label for the socket
    pub name: String,
    /// Data type accepted/provided
    pub socket_type: ModuleSocketType,
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
