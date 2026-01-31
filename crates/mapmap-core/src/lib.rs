//! MapFlow Core - Domain Model and Data Structures
//!
//! This crate contains the core domain model for MapFlow, including:
//! - Paint/Mapping/Mesh hierarchy
//! - Layer system for compositing
//! - Project file format
//! - Geometry primitives
//! - Transform calculations

#![warn(missing_docs)]

pub use glam::{Mat4, Vec2, Vec3};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Phase 1: Layer system for compositing
pub mod layer;
pub use layer::{BlendMode, Composition, Layer, LayerManager, ResizeMode, Transform};

// Phase 2: Multi-output and projection mapping
pub mod mapping;
pub mod mesh;
pub mod monitor;
pub mod output;
pub mod paint;

// Phase 3: Effects Pipeline
pub mod animation;
pub mod assignment;
pub mod audio;
pub mod audio_media_pipeline;
pub mod audio_reactive;
pub mod codegen;
pub mod diagnostics;
pub mod effect_animation;
pub mod effects;
pub mod logging;
pub mod lut;
pub mod module;
pub mod module_eval;
pub mod oscillator;
pub mod recent_effect_configs;
pub mod shader_graph;
pub mod state;
pub mod trigger_system;
pub mod media_library;

// --- Re-exports grouped by category ---

// Animation
pub use animation::{
    AnimValue, AnimationClip, AnimationPlayer, AnimationTrack, InterpolationMode, Keyframe,
    TimePoint,
};
pub use effect_animation::{
    EffectAnimationId, EffectParameterAnimator, EffectParameterBinding, EffectParameterUpdate,
};

// Assignment & Control
pub use assignment::{Assignment, AssignmentManager, ControlSource, ControlTarget};

// Audio System
pub use audio::{
    AudioAnalysis, AudioAnalyzer, AudioConfig, AudioMappingType, AudioReactiveMapping, AudioSource,
    FrequencyBand,
};
pub use audio_media_pipeline::AudioMediaPipeline;
pub use audio_reactive::{
    AudioAnimationBlendMode, AudioReactiveAnimationSystem, AudioReactiveController,
    AudioReactivePreset, AudioTriggerData,
};

// Effects & Processing
pub use effects::{Effect, EffectChain, EffectType};
pub use lut::{Lut3D, LutError, LutFormat, LutManager, LutPreset};
pub use oscillator::{
    ColorMode, CoordinateMode, OscillatorConfig, PhaseInitMode, RingParams, SimulationResolution,
};
pub use recent_effect_configs::{
    EffectConfig, EffectParamValue, RecentConfigQueue, RecentEffectConfigs, MAX_RECENT_CONFIGS,
};

// Geometry & Meshes
pub use mesh::{keystone, BezierPatch, Mesh, MeshType, MeshVertex, VertexId};
pub use paint::{Paint, PaintId, PaintManager, PaintType};

// Logging & Diagnostics
pub use logging::LogConfig;

// Module System & Evaluation
pub use module_eval::{
    ModuleEvalResult, ModuleEvaluator, RenderOp, SourceCommand, SourceProperties,
};

// Output & Display
pub use mapping::{Mapping, MappingId, MappingManager};
pub use monitor::{MonitorInfo, MonitorTopology};
pub use output::{
    CanvasRegion, ColorCalibration, EdgeBlendConfig, EdgeBlendZone, OutputConfig, OutputId,
    OutputManager,
};

// Shader Graph & Codegen
pub use codegen::{CodegenError, WGSLCodegen};
pub use shader_graph::{
    DataType, GraphId, InputSocket, NodeId, NodeType, OutputSocket, ParameterValue, ShaderGraph,
    ShaderNode,
};

// State & Project
pub use state::{AppSettings, AppState};

/// Core error types
#[derive(Error, Debug)]
pub enum CoreError {
    /// Invalid geometry configuration
    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),

    /// Transform operation failed
    #[error("Transform error: {0}")]
    TransformError(String),
}

/// Result type for core operations
pub type Result<T> = std::result::Result<T, CoreError>;

/// Represents a 2D point with texture coordinates
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vertex {
    /// 2D Position
    pub position: Vec2,
    /// Texture Coordinates (UV)
    pub uv: Vec2,
}

impl Vertex {
    /// Create a new vertex
    pub fn new(x: f32, y: f32, u: f32, v: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            uv: Vec2::new(u, v),
        }
    }
}

/// Represents a quadrilateral mesh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quad {
    /// The four vertices of the quad
    pub vertices: [Vertex; 4],
}

impl Quad {
    /// Create a unit quad (0,0) to (1,1)
    pub fn unit() -> Self {
        Self {
            vertices: [
                Vertex::new(-1.0, -1.0, 0.0, 0.0),
                Vertex::new(1.0, -1.0, 1.0, 0.0),
                Vertex::new(1.0, 1.0, 1.0, 1.0),
                Vertex::new(-1.0, 1.0, 0.0, 1.0),
            ],
        }
    }

    /// Apply a transform matrix to all vertices
    pub fn transform(&mut self, mat: Mat4) {
        for vertex in &mut self.vertices {
            let pos = mat.transform_point3(Vec3::new(vertex.position.x, vertex.position.y, 0.0));
            vertex.position = Vec2::new(pos.x, pos.y);
        }
    }
}

/// Shape trait - represents any mappable geometry
/// (Legacy - will be replaced by Mesh in Phase 2)
pub trait Shape: Send + Sync {
    /// Get vertices
    fn vertices(&self) -> &[Vertex];
    /// Get indices
    fn indices(&self) -> &[u16];
    /// Update logic
    fn update(&mut self, delta_time: f32);
}

/// Legacy shape types (Phase 0/1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShapeType {
    /// Simple quad
    Quad(Quad),
    /// Arbitrary mesh
    Mesh {
        /// Vertices
        vertices: Vec<Vertex>,
        /// Indices
        indices: Vec<u16>,
    },
}

/// Project - top-level container (Phase 2+)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Project name
    pub name: String,
    /// Paint manager containing all sources
    pub paint_manager: PaintManager,
    /// Mapping manager containing all mappings
    pub mapping_manager: MappingManager,
    /// Layer manager containing compositing logic
    pub layer_manager: LayerManager,
}

impl Project {
    /// Create a new project
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            paint_manager: PaintManager::new(),
            mapping_manager: MappingManager::new(),
            layer_manager: LayerManager::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quad_creation() {
        let quad = Quad::unit();
        assert_eq!(quad.vertices.len(), 4);
    }

    #[test]
    fn test_project_creation() {
        let project = Project::new("Test Project");
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.paint_manager.paints().len(), 0);
        assert_eq!(project.mapping_manager.mappings().len(), 0);
    }

    #[test]
    fn test_quad_transform() {
        let mut quad = Quad::unit();
        let scale = Mat4::from_scale(Vec3::new(2.0, 2.0, 1.0));
        quad.transform(scale);

        // Check that vertices were scaled
        assert!((quad.vertices[0].position.x - (-2.0)).abs() < 0.001);
        assert!((quad.vertices[0].position.y - (-2.0)).abs() < 0.001);
    }
}
