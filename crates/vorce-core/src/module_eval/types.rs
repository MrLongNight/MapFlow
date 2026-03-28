//! Module Graph Evaluator
//!
//! Traverses the module graph and computes output values.
//! This handles the full pipeline: Trigger -> Source -> Mask -> Effect -> Layer(Mesh) -> Output.

use crate::module::{ModulePartId, ModulePartType, VorceModule};
use std::collections::HashMap;

/// State for individual trigger nodes, stored in the evaluator
#[derive(Debug, Clone, Default)]
pub enum TriggerState {
    #[default]
    /// No state
    None,
    /// Random trigger state
    Random {
        /// The timestamp (in ms since start) when the next trigger is scheduled.
        next_fire_time_ms: u64,
    },
}

/// Source-specific rendering properties (from MediaFile)
#[derive(Debug, Clone, Default)]
pub struct SourceProperties {
    /// Source opacity (multiplied with layer opacity)
    pub opacity: f32,
    /// Color correction: Brightness (-1.0 to 1.0)
    pub brightness: f32,
    /// Color correction: Contrast (0.0 to 2.0, 1.0 = normal)
    pub contrast: f32,
    /// Color correction: Saturation (0.0 to 2.0, 1.0 = normal)
    pub saturation: f32,
    /// Color correction: Hue shift (-180 to 180 degrees)
    pub hue_shift: f32,
    /// Transform: Scale X
    pub scale_x: f32,
    /// Transform: Scale Y
    pub scale_y: f32,
    /// Transform: Rotation in degrees
    pub rotation: f32,
    /// Transform: Offset X
    pub offset_x: f32,
    /// Transform: Offset Y
    pub offset_y: f32,
    /// Flip horizontally
    pub flip_horizontal: bool,
    /// Flip vertically
    pub flip_vertical: bool,
}

impl SourceProperties {
    /// Default source properties (no modifications)
    pub fn default_identity() -> Self {
        Self {
            opacity: 1.0,
            brightness: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            hue_shift: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
            flip_horizontal: false,
            flip_vertical: false,
        }
    }
}

/// Cached indices for a specific module graph revision
#[derive(Debug, Clone)]
pub struct ModuleGraphIndices {
    /// Cached map from Part ID to index in `module.parts`
    pub part_index_cache: HashMap<ModulePartId, usize>,
    /// Cached map from Part ID to list of indices in `module.connections` (incoming connections)
    pub conn_index_cache: HashMap<ModulePartId, Vec<usize>>,
    /// The graph revision this cache corresponds to
    pub last_revision: u64,
}

/// Get the primary render connection index for a module part.
pub fn primary_render_connection_idx(
    module: &VorceModule,
    indices: &ModuleGraphIndices,
    part_id: ModulePartId,
) -> Option<usize> {
    // 1. Determine target socket name for the part type
    let target_id = if let Some(&part_idx) = indices.part_index_cache.get(&part_id) {
        let part = &module.parts[part_idx];
        match &part.part_type {
            ModulePartType::Output(_) => "layer_in",
            ModulePartType::Layer(_) => "media_in",
            ModulePartType::Modulizer(_) => "media_in",
            ModulePartType::Mask(_) => "media_in",
            ModulePartType::Mesh(_) => "vertex_in",
            _ => "0", // Fallback for legacy / unknown
        }
    } else {
        return None;
    };

    // 2. Find connection targeting that socket
    indices
        .conn_index_cache
        .get(&part_id)?
        .iter()
        .copied()
        .find(|&conn_idx| module.connections[conn_idx].to_socket == target_id)
}
