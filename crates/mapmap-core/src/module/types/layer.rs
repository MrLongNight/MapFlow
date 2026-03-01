use crate::module::types::mesh::MeshType;
use crate::module::types::socket::BlendModeType;
use serde::{Deserialize, Serialize};

/// Types of compositing layers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LayerType {
    /// Enumeration variant.
    Single {
        /// Unique identifier for this entity.
        id: u64,
        /// Human-readable display name.
        name: String,
        /// Opacity value (0.0 to 1.0).
        opacity: f32,
        /// Blending mode used for rendering.
        blend_mode: Option<BlendModeType>,
        #[serde(default = "crate::module::config::default_mesh_quad")]
        /// Component property or field.
        mesh: MeshType,
        #[serde(default)]
        /// Component property or field.
        mapping_mode: bool,
    },
    /// Enumeration variant.
    Group {
        /// Human-readable display name.
        name: String,
        /// Opacity value (0.0 to 1.0).
        opacity: f32,
        /// Blending mode used for rendering.
        blend_mode: Option<BlendModeType>,
        #[serde(default = "crate::module::config::default_mesh_quad")]
        /// Component property or field.
        mesh: MeshType,
        #[serde(default)]
        /// Component property or field.
        mapping_mode: bool,
    },
    /// Enumeration variant.
    All {
        /// Opacity value (0.0 to 1.0).
        opacity: f32,
        /// Blending mode used for rendering.
        blend_mode: Option<BlendModeType>,
    },
}
