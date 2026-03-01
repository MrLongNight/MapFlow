//!
//! Node linking types.
//!

use serde::{Deserialize, Serialize};

/// Configuration for the Link System (Master/Slave nodes)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeLinkData {
    /// Link mode (Off, Master, Slave)
    pub mode: LinkMode,
    /// Behavior when linked
    pub behavior: LinkBehavior,
    /// Whether the Trigger Input socket is enabled
    pub trigger_input_enabled: bool,
}

impl Default for NodeLinkData {
    fn default() -> Self {
        Self {
            mode: LinkMode::Off,
            behavior: LinkBehavior::SameAsMaster,
            trigger_input_enabled: false,
        }
    }
}

/// Link mode for a node
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum LinkMode {
    #[default]
    /// Enumeration variant.
    Off,
    /// Enumeration variant.
    Master,
    /// Enumeration variant.
    Slave,
}

/// Behavior of a slave node relative to its master
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum LinkBehavior {
    #[default]
    /// Enumeration variant.
    SameAsMaster,
    /// Enumeration variant.
    Inverted,
}
