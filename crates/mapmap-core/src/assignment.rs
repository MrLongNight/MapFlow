use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents the source of a control signal.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ControlSource {
    Midi { channel: u8, note: u8 },
    Osc { address: String },
    Dmx { universe: u16, channel: u16 },
}

/// Represents a potential target for a control signal.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ControlTarget {
    LayerOpacity {
        layer_id: u64,
    },
    EffectParamF32 {
        layer_id: u64,
        effect_id: Uuid,
        param_name: String,
    },
    // Add other target types here...
}

/// A single mapping from a ControlSource to a ControlTarget.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assignment {
    pub id: Uuid,
    pub source: ControlSource,
    pub target: ControlTarget,
    pub enabled: bool,
    // Add mapping/scaling parameters here, e.g., range, curve...
}

impl Assignment {
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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, assignment: Assignment) {
        self.assignments.push(assignment);
    }

    pub fn remove(&mut self, id: Uuid) {
        self.assignments.retain(|a| a.id != id);
    }

    pub fn assignments(&self) -> &[Assignment] {
        &self.assignments
    }
}
