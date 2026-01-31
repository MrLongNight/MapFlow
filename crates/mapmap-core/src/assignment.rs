//! Assignment - Control Signal Routing
//!
//! This module manages the mapping between control sources (MIDI, OSC, DMX)
//! and control targets (Layer Opacity, Effect Parameters, etc.).
//!
//! # Features
//!
//! - **ControlSource**: Defines where a signal comes from (e.g., MIDI CC, OSC Address).
//! - **ControlTarget**: Defines what the signal controls (e.g., Layer Opacity).
//! - **Assignment**: Connects a Source to a Target.
//! - **AssignmentManager**: Manages the collection of all assignments.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents the source of a control signal.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ControlSource {
    /// MIDI message (Note or CC)
    Midi {
        /// MIDI Channel (0-15)
        channel: u8,
        /// Note or CC number
        note: u8,
    },
    /// OSC message
    Osc {
        /// OSC Address pattern (e.g., "/fader/1")
        address: String,
    },
    /// DMX channel value
    Dmx {
        /// DMX Universe (0-65535)
        universe: u16,
        /// DMX Channel (1-512)
        channel: u16,
    },
}

/// Represents a potential target for a control signal.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ControlTarget {
    /// Controls the opacity of a layer
    LayerOpacity {
        /// ID of the target layer
        layer_id: u64,
    },
    /// Controls a float parameter of an effect
    EffectParamF32 {
        /// ID of the layer containing the effect
        layer_id: u64,
        /// ID of the effect instance
        effect_id: Uuid,
        /// Name of the parameter
        param_name: String,
    },
    // Add other target types here...
}

/// A single mapping from a ControlSource to a ControlTarget.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assignment {
    /// Unique identifier for this assignment
    pub id: Uuid,
    /// The input source
    pub source: ControlSource,
    /// The output target
    pub target: ControlTarget,
    /// Whether this assignment is active
    pub enabled: bool,
    // Add mapping/scaling parameters here, e.g., range, curve...
}

impl Assignment {
    /// Create a new assignment
    pub fn new(source: ControlSource, target: ControlTarget) -> Self {
        Self {
            id: Uuid::new_v4(),
            source,
            target,
            enabled: true,
        }
    }
}

/// Manages all control assignments in the project.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AssignmentManager {
    assignments: Vec<Assignment>,
}

impl AssignmentManager {
    /// Create a new assignment manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new assignment
    pub fn add(&mut self, assignment: Assignment) {
        self.assignments.push(assignment);
    }

    /// Remove an assignment by ID
    pub fn remove(&mut self, id: Uuid) {
        self.assignments.retain(|a| a.id != id);
    }

    /// Get all assignments
    pub fn assignments(&self) -> &[Assignment] {
        &self.assignments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_source_creation() {
        let midi = ControlSource::Midi {
            channel: 0,
            note: 60,
        };
        let osc = ControlSource::Osc {
            address: "/test".to_string(),
        };
        let dmx = ControlSource::Dmx {
            universe: 1,
            channel: 1,
        };

        assert!(matches!(midi, ControlSource::Midi { .. }));
        assert!(matches!(osc, ControlSource::Osc { .. }));
        assert!(matches!(dmx, ControlSource::Dmx { .. }));
    }

    #[test]
    fn test_assignment_new() {
        let source = ControlSource::Midi {
            channel: 1,
            note: 10,
        };
        let target = ControlTarget::LayerOpacity { layer_id: 100 };

        let assignment = Assignment::new(source.clone(), target.clone());

        assert!(!assignment.id.is_nil());
        assert_eq!(assignment.source, source);
        assert_eq!(assignment.target, target);
        assert!(assignment.enabled);
    }

    #[test]
    fn test_manager_add_remove() {
        let mut manager = AssignmentManager::new();
        assert!(manager.assignments().is_empty());

        let a1 = Assignment::new(
            ControlSource::Midi {
                channel: 0,
                note: 0,
            },
            ControlTarget::LayerOpacity { layer_id: 1 },
        );
        let id1 = a1.id;

        manager.add(a1);
        assert_eq!(manager.assignments().len(), 1);
        assert_eq!(manager.assignments()[0].id, id1);

        let a2 = Assignment::new(
            ControlSource::Osc {
                address: "/test".into(),
            },
            ControlTarget::LayerOpacity { layer_id: 2 },
        );
        let id2 = a2.id;
        manager.add(a2);
        assert_eq!(manager.assignments().len(), 2);

        // Remove first
        manager.remove(id1);
        assert_eq!(manager.assignments().len(), 1);
        assert_eq!(manager.assignments()[0].id, id2);

        // Remove non-existent (should be safe)
        manager.remove(id1);
        assert_eq!(manager.assignments().len(), 1);

        // Remove second
        manager.remove(id2);
        assert!(manager.assignments().is_empty());
    }

    #[test]
    fn test_serialization() {
        let source = ControlSource::Midi {
            channel: 5,
            note: 64,
        };
        let target = ControlTarget::LayerOpacity { layer_id: 42 };
        let original = Assignment::new(source, target);

        let json = serde_json::to_string(&original).expect("Failed to serialize");
        let deserialized: Assignment = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(original, deserialized);
        assert_eq!(original.id, deserialized.id);
    }
}
