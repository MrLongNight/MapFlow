//! MapFlow Render - Graphics Abstraction Layer
//!
//! This crate provides the rendering abstraction for MapFlow, including:
//! - wgpu backend implementation
//! - Texture pool management
//! - Shader compilation and hot-reloading
//! - GPU profiling
//! - Effect chain post-processing
//! - Preset system for effect chains

use thiserror::Error;

pub mod backend;
pub mod color_calibration_renderer;
pub mod compositor;
pub mod edge_blend_renderer;
pub mod effect_chain_renderer;
pub mod hot_reload;
mod mesh_buffer_cache;
pub mod mesh_renderer;
pub mod oscillator_renderer;
pub mod paint_texture_cache;
pub mod preset;
pub mod quad;
pub mod shader;
pub mod texture;

pub use backend::{RenderBackend, WgpuBackend};
pub use color_calibration_renderer::ColorCalibrationRenderer;
pub use compositor::Compositor;
pub use edge_blend_renderer::EdgeBlendRenderer;
pub use effect_chain_renderer::{EffectChainRenderer, EffectParams};
pub use hot_reload::{HotReloadIntegration, ShaderChangeEvent, ShaderHotReload, ShaderStatus};
pub use mesh_renderer::MeshRenderer;
pub use oscillator_renderer::OscillatorRenderer;
pub use preset::{EffectPreset, PresetLibrary, PresetMetadata};
pub use quad::QuadRenderer;
pub use shader::{ShaderHandle, ShaderSource};
pub use texture::{TextureDescriptor, TextureHandle, TexturePool};

/// Rendering errors
#[derive(Error, Debug)]
pub enum RenderError {
    #[error("Device error: {0}")]
    DeviceError(String),

    #[error("Shader compilation failed: {0}")]
    ShaderCompilation(String),

    #[error("Texture creation failed: {0}")]
    TextureCreation(String),

    #[error("Device lost")]
    DeviceLost,

    #[error("Surface error: {0}")]
    SurfaceError(String),
}

/// Result type for rendering operations
pub type Result<T> = std::result::Result<T, RenderError>;

/// Re-export commonly used wgpu types
pub use wgpu::{
    BufferUsages, CommandEncoder, CompositeAlphaMode, Device, PresentMode, Queue, Surface,
    SurfaceConfiguration, Texture, TextureFormat, TextureUsages, TextureView,
};
