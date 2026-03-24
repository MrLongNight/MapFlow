use crate::module::types::part::{ModulePart, ModulePartType};
use crate::module::types::socket::ModuleSocket;

/// High-level node category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleNodeKind {
    /// Trigger node.
    Trigger,
    /// Source node.
    Source,
    /// Mask node.
    Mask,
    /// Modulator node.
    Modulizer,
    /// Layer node.
    Layer,
    /// Mesh node.
    Mesh,
    /// Hue node.
    Hue,
    /// Output node.
    Output,
}

impl From<&ModulePartType> for ModuleNodeKind {
    fn from(value: &ModulePartType) -> Self {
        match value {
            ModulePartType::Trigger(_) => Self::Trigger,
            ModulePartType::Source(_) => Self::Source,
            ModulePartType::Mask(_) => Self::Mask,
            ModulePartType::Modulizer(_) => Self::Modulizer,
            ModulePartType::Layer(_) => Self::Layer,
            ModulePartType::Mesh(_) => Self::Mesh,
            ModulePartType::Hue(_) => Self::Hue,
            ModulePartType::Output(_) => Self::Output,
        }
    }
}

/// Inspector capabilities derived from the core schema.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModuleInspectorSchema {
    /// Input indices that may be configured as trigger targets.
    pub mappable_input_indices: Vec<usize>,
    /// Whether this part exposes link configuration.
    pub supports_link_config: bool,
}

impl ModuleInspectorSchema {
    /// Whether the part exposes any trigger-mappable inputs.
    pub fn has_trigger_mapping(&self) -> bool {
        !self.mappable_input_indices.is_empty()
    }
}

/// Consolidated node schema used by UI and validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModulePartSchema {
    /// High-level node kind.
    pub kind: ModuleNodeKind,
    /// Input sockets derived from the part configuration.
    pub inputs: Vec<ModuleSocket>,
    /// Output sockets derived from the part configuration.
    pub outputs: Vec<ModuleSocket>,
    /// Inspector capabilities for this node.
    pub inspector: ModuleInspectorSchema,
}

impl ModulePartSchema {
    /// Returns true if the schema exposes trigger mapping.
    pub fn has_trigger_mapping(&self) -> bool {
        self.inspector.has_trigger_mapping()
    }
}

impl ModulePart {
    /// Build a consolidated runtime schema for this part.
    pub fn schema(&self) -> ModulePartSchema {
        let (inputs, outputs) = self.compute_sockets();
        let mappable_input_indices = inputs
            .iter()
            .enumerate()
            .filter_map(|(idx, socket)| socket.supports_trigger_mapping.then_some(idx))
            .collect();

        ModulePartSchema {
            kind: ModuleNodeKind::from(&self.part_type),
            inputs,
            outputs,
            inspector: ModuleInspectorSchema {
                mappable_input_indices,
                supports_link_config: true,
            },
        }
    }
}
