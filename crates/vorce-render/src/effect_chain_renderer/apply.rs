use super::{types::EffectParams, EffectChainRenderer};
use crate::pipeline::Allocation;
use vorce_core::{warn_once, EffectChain, EffectType};
use std::collections::HashMap;
use std::sync::{Arc, Weak};
use tracing::{debug, warn};
use wgpu::util::DeviceExt;

impl EffectChainRenderer {
    /// Get or create a bind group for an input texture
    pub(crate) fn get_texture_bind_group_static(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        cache: &mut HashMap<usize, (Weak<wgpu::TextureView>, Arc<wgpu::BindGroup>)>,
        input_view: &Arc<wgpu::TextureView>,
    ) -> Arc<wgpu::BindGroup> {
        let key = Arc::as_ptr(input_view) as usize;

        if let Some((weak, bg)) = cache.get(&key) {
            if weak.upgrade().is_some() {
                return bg.clone();
            }
        }

        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Effect Chain Input Bind Group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(input_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });
        let bg = Arc::new(bg);
        cache.insert(key, (Arc::downgrade(input_view), bg.clone()));
        bg
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

    /// Get or create a uniform bind group
    pub(crate) fn get_uniform_bind_group_static(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        cache: &mut HashMap<(usize, u64, u64), Arc<wgpu::BindGroup>>,
        allocation: &Allocation,
        size: u64,
    ) -> Arc<wgpu::BindGroup> {
        let key = (allocation.page_index, allocation.offset, size);
        if let Some(bg) = cache.get(&key) {
            return bg.clone();
        }

        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Effect Chain Uniform Bind Group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: allocation.buffer,
                    offset: allocation.offset,
                    size: std::num::NonZeroU64::new(size),
                }),
            }],
        });
        let bg = Arc::new(bg);
        cache.insert(key, bg.clone());
        bg
    }

    /// Apply the effect chain to an input texture
    ///
    /// Returns the final output texture view after all effects are applied.
    #[allow(clippy::too_many_arguments)]
    pub fn apply_chain(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        input_view: &Arc<wgpu::TextureView>,
        output_view: &Arc<wgpu::TextureView>,
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
                    depth_slice: None,
                    view: output_view,
                    resolve_target: None,

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
                match self.pipelines.get(&effect.effect_type.normalized()) {
                    Some(p) => Some(p),
                    None => {
                        warn_once!("No pipeline for effect type: {:?}", effect.effect_type);
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

            let mut lut_bind_group_resource = None;

            match &effect.effect_type {
                EffectType::LoadLUT { path } => {
                    if !path.is_empty() {
                        if !self.lut_cache.contains_key(path) {
                            // Load LUT
                            match vorce_core::lut::Lut3D::from_file(path) {
                                Ok(lut) => {
                                    let (data, width, height) = lut.to_2d_texture_data();
                                    let lut_size = lut.size as f32;

                                    let texture = self.device.create_texture_with_data(
                                        &self.queue,
                                        &wgpu::TextureDescriptor {
                                            label: Some(&format!("LUT Texture: {}", path)),
                                            size: wgpu::Extent3d {
                                                width,
                                                height,
                                                depth_or_array_layers: 1,
                                            },
                                            mip_level_count: 1,
                                            sample_count: 1,
                                            dimension: wgpu::TextureDimension::D2,
                                            format: wgpu::TextureFormat::Rgba8Unorm,
                                            usage: wgpu::TextureUsages::TEXTURE_BINDING
                                                | wgpu::TextureUsages::COPY_DST,
                                            view_formats: &[],
                                        },
                                        wgpu::util::TextureDataOrder::LayerMajor,
                                        &data,
                                    );

                                    let view = texture
                                        .create_view(&wgpu::TextureViewDescriptor::default());

                                    let bind_group =
                                        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                                            label: Some(&format!("LUT Bind Group: {}", path)),
                                            layout: &self.lut_bind_group_layout,
                                            entries: &[
                                                wgpu::BindGroupEntry {
                                                    binding: 0,
                                                    resource: wgpu::BindingResource::TextureView(
                                                        &view,
                                                    ),
                                                },
                                                wgpu::BindGroupEntry {
                                                    binding: 1,
                                                    resource: wgpu::BindingResource::Sampler(
                                                        &self.sampler,
                                                    ),
                                                },
                                            ],
                                        });

                                    self.lut_cache.insert(
                                        path.clone(),
                                        Some((lut_size, view, Arc::new(bind_group))),
                                    );
                                }
                                Err(e) => {
                                    warn_once!("Failed to load LUT from {}: {}", path, e);
                                    self.lut_cache.insert(path.clone(), None);
                                }
                            }
                        }

                        // Mark as used
                        self.lut_last_used.insert(path.clone(), self.frame_count);

                        if let Some(Some((size, _, bg))) = self.lut_cache.get(path) {
                            params.param_a = *size;
                            lut_bind_group_resource = Some(bg.clone());
                        }
                    }
                }
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
                input_view.clone()
            } else {
                let ping_pong = self.ping_pong.as_ref().unwrap();
                ping_pong.views[current_idx].clone()
            };

            // Create bind groups
            let input_bind_group = Self::get_texture_bind_group_static(
                &self.device,
                &self.bind_group_layout,
                &self.sampler,
                &mut self.texture_bg_cache,
                &current_input,
            );

            // Allocate uniform buffer from pool
            let allocation = self
                .allocator
                .allocate(&self.queue, bytemuck::cast_slice(&[params]));
            let size = std::mem::size_of::<EffectParams>() as u64;

            let uniform_bind_group = Self::get_uniform_bind_group_static(
                &self.device,
                &self.uniform_bind_group_layout,
                &mut self.uniform_bg_cache,
                &allocation,
                size,
            );

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
                            &current_input,
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
                            depth_slice: None,
                            view: render_target,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    render_pass.set_pipeline(pipeline);
                    render_pass.set_bind_group(0, &*input_bind_group, &[]);
                    render_pass.set_bind_group(1, &*uniform_bind_group, &[]);
                    if let Some(lut_bg) = lut_bind_group_resource {
                        render_pass.set_bind_group(2, &*lut_bg, &[]);
                    }
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
}
