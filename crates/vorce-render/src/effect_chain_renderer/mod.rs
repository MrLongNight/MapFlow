//! Effect Chain Renderer
//!
//! Multi-pass post-processing effect pipeline with ping-pong buffers.
//! Applies a chain of effects to an input texture and outputs to a target.
//!
//! Phase 3: Effects Pipeline
//! - Shader-Graph integration
//! - Multi-pass rendering
//! - Parameter uniforms
//! - Hot-reload support (via shader recompilation)

use crate::{pipeline::UniformBufferAllocator, QuadRenderer, Result};
use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use std::sync::{Arc, Weak};
use tracing::{debug, info, warn};
use vorce_core::EffectType;
use wgpu::util::DeviceExt;

mod apply;
mod pipeline;
pub mod types;

pub use types::EffectParams;
pub(crate) use types::PingPongBuffer;

/// Effect chain renderer
pub struct EffectChainRenderer {
    pub(crate) device: Arc<wgpu::Device>,
    pub(crate) queue: Arc<wgpu::Queue>,
    pub(crate) target_format: wgpu::TextureFormat,

    // Render pipeline for each effect type
    pub(crate) pipelines: HashMap<EffectType, wgpu::RenderPipeline>,

    // Bind group layout for effects
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) uniform_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) lut_bind_group_layout: wgpu::BindGroupLayout,

    // Sampler for textures
    pub(crate) sampler: wgpu::Sampler,

    // Ping-pong buffers (lazily created)
    pub(crate) ping_pong: Option<PingPongBuffer>,
    pub(crate) current_size: (u32, u32),

    // Fullscreen quad vertices
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) index_buffer: wgpu::Buffer,

    // Passthrough renderer
    pub(crate) quad_renderer: QuadRenderer,

    // Uniform buffer allocator
    pub(crate) allocator: UniformBufferAllocator,

    // Caches
    pub(crate) uniform_bg_cache: HashMap<(usize, u64, u64), Arc<wgpu::BindGroup>>,
    pub(crate) texture_bg_cache: HashMap<usize, (Weak<wgpu::TextureView>, Arc<wgpu::BindGroup>)>,
    pub(crate) lut_cache: HashMap<String, Option<(f32, wgpu::TextureView, Arc<wgpu::BindGroup>)>>,
    pub(crate) lut_last_used: HashMap<String, u64>,
    pub(crate) frame_count: u64,
}

impl EffectChainRenderer {
    /// Create a new effect chain renderer
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        target_format: wgpu::TextureFormat,
    ) -> Result<Self> {
        debug!("Creating EffectChainRenderer");

        let quad_renderer = QuadRenderer::new(&device, target_format)?;

        // Create bind group layout for texture sampling
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Effect Chain Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create bind group layout for uniforms
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Effect Chain Uniform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // Create bind group layout for LUTs
        let lut_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Effect Chain LUT Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        // Create sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Effect Chain Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Create fullscreen quad vertices
        #[repr(C)]
        #[derive(Copy, Clone, Debug, Pod, Zeroable)]
        struct Vertex {
            position: [f32; 2],
            uv: [f32; 2],
        }

        let vertices = [
            Vertex {
                position: [-1.0, -1.0],
                uv: [0.0, 1.0],
            },
            Vertex {
                position: [1.0, -1.0],
                uv: [1.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0],
                uv: [1.0, 0.0],
            },
            Vertex {
                position: [-1.0, 1.0],
                uv: [0.0, 0.0],
            },
        ];

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Effect Chain Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Effect Chain Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create pipelines for each effect type
        let mut pipelines = HashMap::new();

        // Create built-in effect pipelines
        let effect_types = [
            EffectType::LoadLUT {
                path: String::new(),
            },
            EffectType::ColorAdjust,
            EffectType::Blur,
            EffectType::ChromaticAberration,
            EffectType::EdgeDetect,
            EffectType::Invert,
            EffectType::Pixelate,
            EffectType::Vignette,
            EffectType::FilmGrain,
            EffectType::Wave,
            EffectType::Glitch,
            EffectType::RgbSplit,
            EffectType::Mirror,
            EffectType::HueShift,
            EffectType::Kaleidoscope,
        ];

        for effect_type in effect_types {
            if let Ok(pipeline) = Self::create_effect_pipeline(
                &device,
                &bind_group_layout,
                &uniform_bind_group_layout,
                &lut_bind_group_layout,
                target_format,
                &effect_type,
            ) {
                pipelines.insert(effect_type, pipeline);
            } else {
                warn!("Failed to create pipeline for effect: {:?}", effect_type);
            }
        }

        let allocator = UniformBufferAllocator::new(device.clone(), "EffectChain");

        Ok(Self {
            device,
            queue,
            target_format,
            pipelines,
            bind_group_layout,
            uniform_bind_group_layout,
            lut_bind_group_layout,
            sampler,
            ping_pong: None,
            current_size: (0, 0),
            vertex_buffer,
            index_buffer,
            quad_renderer,
            allocator,
            uniform_bg_cache: HashMap::new(),
            texture_bg_cache: HashMap::new(),
            lut_cache: HashMap::new(),
            lut_last_used: HashMap::new(),
            frame_count: 0,
        })
    }

    /// Reset allocator at start of frame
    pub fn begin_frame(&mut self) {
        self.allocator.reset();
        // Clear uniform bind group cache since buffer pages might be reused/reset differently
        // Actually, if allocator resets, page 0 offset 0 is reused. The bind group pointing to it is still valid!
        // So we keep the cache. BUT if the buffer was destroyed (reallocated larger), the bind group is invalid?
        // Allocator in `pipeline.rs` clears `pages`? No, it keeps `pages` and resets `current_page`.
        // So buffers are stable. We can keep the cache!

        // Prune dead texture bind groups
        self.texture_bg_cache
            .retain(|_, (weak, _)| weak.strong_count() > 0);

        self.frame_count += 1;

        // Cleanup LUT cache every 600 frames (approx 10 seconds at 60fps)
        #[allow(clippy::manual_is_multiple_of)]
        if self.frame_count % 600 == 0 {
            let threshold = self.frame_count.saturating_sub(600);
            self.lut_cache
                .retain(|path, _| *self.lut_last_used.get(path).unwrap_or(&0) >= threshold);
            self.lut_last_used.retain(|_, frame| *frame >= threshold);
        }
    }

    /// Ensure ping-pong buffers are the correct size
    pub(crate) fn ensure_ping_pong(&mut self, width: u32, height: u32) {
        if self.ping_pong.is_none() || self.current_size != (width, height) {
            debug!("Creating ping-pong buffers: {}x{}", width, height);
            self.ping_pong = Some(PingPongBuffer::new(
                &self.device,
                width,
                height,
                self.target_format,
            ));
            self.current_size = (width, height);
        }
    }

    /// Reload a custom shader for an effect
    pub fn reload_custom_shader(&mut self, effect_id: u64, shader_source: &str) -> Result<()> {
        // Validate shader by attempting to create a module
        let _shader_module = self
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(&format!("Custom Effect Shader: {}", effect_id)),
                source: wgpu::ShaderSource::Wgsl(shader_source.into()),
            });

        // If we get here, shader compiled successfully
        // In a full implementation, we'd store the custom pipeline
        info!("Custom shader {} compiled successfully", effect_id);

        Ok(())
    }

    /// Update and compile a shader graph using the renderer's layouts
    pub fn update_shader_graph(
        &self,
        manager: &mut crate::ShaderGraphManager,
        graph_id: vorce_core::shader_graph::GraphId,
    ) -> crate::Result<()> {
        manager
            .compile_for_gpu(
                graph_id,
                &self.device,
                &self.bind_group_layout,
                &self.uniform_bind_group_layout,
                self.target_format,
            )
            .map_err(|e| crate::RenderError::ShaderCompilation(e.to_string()))
    }

    /// Get the wgpu device.
    pub fn device(&self) -> &Arc<wgpu::Device> {
        &self.device
    }

    /// Get the wgpu queue.
    pub fn queue(&self) -> &Arc<wgpu::Queue> {
        &self.queue
    }
}
