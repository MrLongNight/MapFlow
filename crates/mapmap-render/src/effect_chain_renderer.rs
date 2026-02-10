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
use mapmap_core::{EffectChain, EffectType};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};
use wgpu::util::DeviceExt;

/// Parameters for an effect instance
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct EffectParams {
    /// Time in seconds (for animated effects)
    pub time: f32,
    /// Effect intensity (0.0 - 1.0)
    pub intensity: f32,
    /// Parameter A (effect-specific)
    pub param_a: f32,
    /// Parameter B (effect-specific)
    pub param_b: f32,
    /// Parameter C (vec2 packed as xy)
    pub param_c: [f32; 2],
    /// Resolution (width, height)
    pub resolution: [f32; 2],
}

impl Default for EffectParams {
    fn default() -> Self {
        Self {
            time: 0.0,
            intensity: 1.0,
            param_a: 0.0,
            param_b: 0.0,
            param_c: [0.0, 0.0],
            resolution: [1920.0, 1080.0],
        }
    }
}

/// Ping-pong buffer for multi-pass rendering
#[allow(dead_code)]
struct PingPongBuffer {
    textures: [wgpu::Texture; 2],
    views: [wgpu::TextureView; 2],
    current: usize,
}

#[allow(dead_code)]
impl PingPongBuffer {
    fn new(device: &wgpu::Device, width: u32, height: u32, format: wgpu::TextureFormat) -> Self {
        let create_texture = || {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Effect Chain Ping-Pong Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            })
        };

        let tex_a = create_texture();
        let tex_b = create_texture();

        let view_a = tex_a.create_view(&wgpu::TextureViewDescriptor::default());
        let view_b = tex_b.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            textures: [tex_a, tex_b],
            views: [view_a, view_b],
            current: 0,
        }
    }

    fn current_view(&self) -> &wgpu::TextureView {
        &self.views[self.current]
    }

    fn next_view(&self) -> &wgpu::TextureView {
        &self.views[1 - self.current]
    }

    fn swap(&mut self) {
        self.current = 1 - self.current;
    }
}

/// Effect chain renderer
pub struct EffectChainRenderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    target_format: wgpu::TextureFormat,

    // Render pipeline for each effect type
    pipelines: HashMap<EffectType, wgpu::RenderPipeline>,

    // Bind group layout for effects
    bind_group_layout: wgpu::BindGroupLayout,
    uniform_bind_group_layout: wgpu::BindGroupLayout,

    // Sampler for textures
    sampler: wgpu::Sampler,

    // Ping-pong buffers (lazily created)
    ping_pong: Option<PingPongBuffer>,
    current_size: (u32, u32),

    // Fullscreen quad vertices
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    // Passthrough renderer
    quad_renderer: QuadRenderer,

    // Uniform buffer allocator
    allocator: UniformBufferAllocator,
}

impl EffectChainRenderer {
    /// Create a new effect chain renderer
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        target_format: wgpu::TextureFormat,
    ) -> Result<Self> {
        info!("Creating EffectChainRenderer");

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
                target_format,
                effect_type,
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
            sampler,
            ping_pong: None,
            current_size: (0, 0),
            vertex_buffer,
            index_buffer,
            quad_renderer,
            allocator,
        })
    }

    /// Reset allocator at start of frame
    pub fn begin_frame(&mut self) {
        self.allocator.reset();
    }

    /// Create a render pipeline for a specific effect type
    fn create_effect_pipeline(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
        target_format: wgpu::TextureFormat,
        effect_type: EffectType,
    ) -> Result<wgpu::RenderPipeline> {
        let shader_source = Self::get_effect_shader_source(effect_type);

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("Effect Shader: {:?}", effect_type)),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("Effect Pipeline Layout: {:?}", effect_type)),
            bind_group_layouts: &[bind_group_layout, uniform_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("Effect Pipeline: {:?}", effect_type)),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 16,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Ok(pipeline)
    }

    /// Get the WGSL shader source for an effect type
    fn get_effect_shader_source(effect_type: EffectType) -> &'static str {
        match effect_type {
            EffectType::ColorAdjust => include_str!("../../../shaders/effect_color_adjust.wgsl"),
            EffectType::Blur => include_str!("../../../shaders/effect_blur.wgsl"),
            EffectType::ChromaticAberration => {
                include_str!("../../../shaders/effect_chromatic_aberration.wgsl")
            }
            EffectType::EdgeDetect => include_str!("../../../shaders/effect_edge_detect.wgsl"),
            EffectType::Invert => include_str!("../../../shaders/effect_invert.wgsl"),
            EffectType::Pixelate => include_str!("../../../shaders/effect_pixelate.wgsl"),
            EffectType::Vignette => include_str!("../../../shaders/effect_vignette.wgsl"),
            EffectType::FilmGrain => include_str!("../../../shaders/effect_film_grain.wgsl"),
            EffectType::Wave => include_str!("../../../shaders/effect_wave.wgsl"),
            EffectType::Glitch => include_str!("../../../shaders/effect_glitch.wgsl"),
            EffectType::RgbSplit => include_str!("../../../shaders/effect_rgb_split.wgsl"),
            EffectType::Mirror => include_str!("../../../shaders/effect_mirror.wgsl"),
            EffectType::HueShift => include_str!("../../../shaders/effect_hue_shift.wgsl"),
            EffectType::Kaleidoscope => include_str!("../../../shaders/effect_kaleidoscope.wgsl"),
            EffectType::Voronoi => include_str!("../../../shaders/effect_voronoi.wgsl"),
            EffectType::Tunnel => include_str!("../../../shaders/effect_tunnel.wgsl"),
            EffectType::Galaxy => include_str!("../../../shaders/effect_galaxy.wgsl"),
            _ => include_str!("../../../shaders/effect_passthrough.wgsl"),
        }
    }

    /// Ensure ping-pong buffers are the correct size
    fn ensure_ping_pong(&mut self, width: u32, height: u32) {
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

    /// Create a bind group for an input texture
    pub fn create_bind_group(&self, input_view: &wgpu::TextureView) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Effect Chain Input Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }

    /// Create a uniform buffer for effect parameters
    pub fn create_uniform_buffer(&self, params: &EffectParams) -> wgpu::Buffer {
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Effect Chain Uniform Buffer"),
                contents: bytemuck::cast_slice(&[*params]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            })
    }

    /// Create a uniform bind group
    pub fn create_uniform_bind_group(&self, buffer: &wgpu::Buffer) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Effect Chain Uniform Bind Group"),
            layout: &self.uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        })
    }

    /// Apply the effect chain to an input texture
    ///
    /// Returns the final output texture view after all effects are applied.
    #[allow(clippy::too_many_arguments)]
    pub fn apply_chain(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        input_view: &wgpu::TextureView,
        output_view: &wgpu::TextureView,
        chain: &EffectChain,
        shader_graph_manager: &crate::ShaderGraphManager,
        time: f32,
        width: u32,
        height: u32,
    ) {
        let enabled_effects: Vec<_> = chain.enabled_effects().collect();

        if enabled_effects.is_empty() {
            // No effects, use quad renderer to copy input to output
            debug!("No effects enabled, passing through with QuadRenderer");
            let bind_group = self
                .quad_renderer
                .create_bind_group(&self.device, input_view);
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Passthrough Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output_view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            self.quad_renderer.draw(&mut rpass, &bind_group);
            return;
        }

        // Ensure ping-pong buffers exist
        self.ensure_ping_pong(width, height);

        // We need to handle this differently to avoid borrow checker issues
        // by not holding mutable borrow of ping_pong across the loop
        let mut current_idx = 0usize;
        let mut use_input = true;

        for (i, effect) in enabled_effects.iter().enumerate() {
            let is_last = i == enabled_effects.len() - 1;

            // Determine if this is a custom shader graph
            let is_custom_graph = matches!(effect.effect_type, EffectType::ShaderGraph(_));

            // Get the pipeline for this effect (if standard)
            let pipeline = if !is_custom_graph {
                match self.pipelines.get(&effect.effect_type) {
                    Some(p) => Some(p),
                    None => {
                        warn!("No pipeline for effect type: {:?}", effect.effect_type);
                        continue;
                    }
                }
            } else {
                None
            };

            // Create effect parameters
            let mut params = EffectParams {
                time,
                intensity: effect.intensity,
                resolution: [width as f32, height as f32],
                ..Default::default()
            };

            match effect.effect_type {
                EffectType::ColorAdjust => {
                    params.param_a = effect.get_param("brightness", 0.0);
                    params.param_b = effect.get_param("contrast", 1.0);
                    params.param_c[0] = effect.get_param("saturation", 1.0);
                }
                EffectType::Blur => {
                    params.param_a = effect.get_param("radius", 5.0);
                    params.param_b = effect.get_param("samples", 9.0);
                }
                EffectType::Vignette => {
                    params.param_a = effect.get_param("radius", 0.5);
                    params.param_b = effect.get_param("softness", 0.5);
                }
                EffectType::FilmGrain => {
                    params.param_a = effect.get_param("amount", 0.1);
                    params.param_b = effect.get_param("speed", 1.0);
                }
                EffectType::Wave => {
                    params.param_a = effect.get_param("frequency", 10.0);
                    params.param_b = effect.get_param("amplitude", 1.0);
                }
                EffectType::Glitch => {
                    params.param_a = effect.get_param("block_size", 16.0);
                    params.param_b = effect.get_param("color_shift", 5.0);
                }
                EffectType::RgbSplit => {
                    params.param_a = effect.get_param("offset_x", 5.0);
                    params.param_b = effect.get_param("offset_y", 0.0);
                }
                EffectType::Mirror => {
                    params.param_a = effect.get_param("mode", 0.0);
                    params.param_b = effect.get_param("center", 0.5);
                }
                EffectType::Kaleidoscope => {
                    params.param_a = effect.get_param("segments", 8.0);
                    params.param_b = effect.get_param("rotation", 0.5);
                }
                EffectType::HueShift => {
                    params.param_a = effect.get_param("hue_shift", 0.0);
                }
                EffectType::Pixelate => {
                    params.param_a = effect.get_param("pixel_size", 8.0);
                }
                EffectType::Voronoi => {
                    params.param_a = effect.get_param("scale", 10.0);
                    params.param_b = effect.get_param("offset", 1.0);
                    params.param_c[0] = effect.get_param("cell_size", 1.0);
                    params.param_c[1] = effect.get_param("distortion", 0.5);
                }
                EffectType::Tunnel => {
                    params.param_a = effect.get_param("scale", 0.5);
                    params.param_b = effect.get_param("rotation", 0.5);
                    params.param_c[0] = effect.get_param("speed", 0.5);
                    params.param_c[1] = effect.get_param("distortion", 0.5);
                }
                EffectType::Galaxy => {
                    params.param_a = effect.get_param("zoom", 0.5);
                    params.param_b = effect.get_param("speed", 0.2);
                    params.param_c[0] = effect.get_param("radius", 1.0);
                    params.param_c[1] = effect.get_param("brightness", 1.0);
                }
                // Custom graphs handle params differently (via Uniform nodes usually),
                // but we can map standard params to defaults if needed.
                // For now, custom graphs will rely on their compiled bindings.
                _ => {}
            }

            // Get input view
            let current_input = if use_input {
                input_view
            } else {
                let ping_pong = self.ping_pong.as_ref().unwrap();
                &ping_pong.views[current_idx]
            };

            // Create bind groups
            let input_bind_group = self.create_bind_group(current_input);

            // Allocate uniform buffer from pool
            let (buffer, offset) = self
                .allocator
                .allocate(&self.queue, bytemuck::cast_slice(&[params]));
            let size = std::mem::size_of::<EffectParams>() as u64;

            let uniform_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Effect Chain Uniform Bind Group"),
                layout: &self.uniform_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer,
                        offset,
                        size: std::num::NonZeroU64::new(size),
                    }),
                }],
            });

            // Determine output target
            let render_target = if is_last {
                output_view
            } else {
                let ping_pong = self.ping_pong.as_ref().unwrap();
                &ping_pong.views[1 - current_idx]
            };

            if let EffectType::ShaderGraph(graph_id) = effect.effect_type {
                // --- CUSTOM SHADER GRAPH PATH ---
                use crate::ShaderGraphRendering; // Trait must be in scope

                if let Some(compiled) = shader_graph_manager.get_compiled(graph_id) {
                    if compiled.is_ready() {
                        self.apply_shader_graph(
                            encoder,
                            compiled,
                            current_input,
                            render_target,
                            &input_bind_group,
                            &uniform_bind_group,
                        );
                    } else {
                        // Fallback if not ready: Passthrough
                        // (Just draw a quad with input texture using QuadRenderer would be best, but we are inside complex loop)
                        // For now, since we cleared to BLACK at start of pass (in standard path below), skipping might result in black.
                        // But we didn't start a render pass yet here.
                    }
                } else {
                    warn!("Shader Graph {} not found or not compiled", graph_id);
                }
            } else {
                // --- STANDARD FIXED PIPELINE PATH ---
                if let Some(pipeline) = pipeline {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some(&format!("Effect Pass: {:?}", effect.effect_type)),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: render_target,
                            resolve_target: None,
                            depth_slice: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    render_pass.set_pipeline(pipeline);
                    render_pass.set_bind_group(0, &input_bind_group, &[]);
                    render_pass.set_bind_group(1, &uniform_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                    render_pass
                        .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(0..6, 0, 0..1);
                }
            }

            // Swap ping-pong for next iteration
            if !is_last {
                current_idx = 1 - current_idx;
                use_input = false;
            }
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
        graph_id: mapmap_core::shader_graph::GraphId,
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
