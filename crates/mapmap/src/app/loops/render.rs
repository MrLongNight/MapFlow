//! Main application render loop.

use crate::app::core::app_struct::App;
use crate::app::ui_layout;
use anyhow::Result;
use mapmap_core::effects::{Effect, EffectChain, EffectType as ChainEffectType};
use mapmap_core::module::OutputType::Projector;
use mapmap_core::module::{EffectType as ModEffectType, ModulizerType};
use mapmap_core::OutputId;
#[cfg(feature = "ndi")]
use std::sync::atomic::{AtomicBool, Ordering};

const PREVIEW_FLAG: u64 = 1u64 << 63;

fn normalize_output_id(output_id: u64) -> (bool, u64) {
    ((output_id & PREVIEW_FLAG) != 0, output_id & !PREVIEW_FLAG)
}

fn render_op_targets_output(op: &mapmap_core::module_eval::RenderOp, output_id: u64) -> bool {
    let (_, real_output_id) = normalize_output_id(output_id);
    match &op.output_type {
        Projector { id, .. } => *id == real_output_id,
        _ => op.output_part_id == real_output_id,
    }
}

/// Renders the UI or content for the given output ID.
pub fn render(app: &mut App, output_id: OutputId) -> Result<()> {
    // Clone device Arc to create encoder without borrowing self
    let device = app.backend.device.clone();

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    // Batch render passes.
    app.mesh_renderer.begin_frame();
    app.effect_chain_renderer.begin_frame();
    app.preview_effect_chain_renderer.begin_frame();

    if output_id == 0 {
        // Sync Texture Previews
        prepare_texture_previews(app, &mut encoder);
        // Update Bevy Texture
        if let Some(runner) = &app.bevy_runner {
            let runner: &mapmap_bevy::BevyRunner = runner;
            if let Some((data, width, height)) = runner.get_image_data() {
                let tex_name = "bevy_output";
                app.texture_pool.ensure_texture(
                    tex_name,
                    width,
                    height,
                    wgpu::TextureFormat::Bgra8UnormSrgb,
                    wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::RENDER_ATTACHMENT,
                );

                app.texture_pool
                    .upload_data(&app.backend.queue, tex_name, &data, width, height);
            }
        }
    }

    // --- UI PASS (Output 0 only) ---
    let mut egui_render_data = None;

    if output_id == 0 {
        // 1. Get Input and Window Info (Short-lived borrow)
        let (raw_input, screen_descriptor) = {
            let window_context = match app.window_manager.get(0) {
                Some(ctx) => ctx,
                None => return Ok(()),
            };

            let input = app.egui_state.take_egui_input(&window_context.window);
            let desc = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [
                    window_context.surface_config.width,
                    window_context.surface_config.height,
                ],
                pixels_per_point: app.egui_context.pixels_per_point(),
            };
            (input, desc)
        };

        // 2. Run UI Pass (SAFE: app is now available mutably)
        let egui_ctx = app.egui_context.clone();
        let full_output = egui_ctx.run(raw_input, |ctx| {
            ui_layout::show(ctx, app);
        });

        // 3. Handle Output (Requires another short-lived borrow of window)
        {
            let window_context = app.window_manager.get(0).unwrap();
            app.egui_state
                .handle_platform_output(&window_context.window, full_output.platform_output);
        }

        // 4. Update Textures and Buffers
        let tris = app
            .egui_context
            .tessellate(full_output.shapes, app.egui_context.pixels_per_point());

        for (id, delta) in full_output.textures_delta.set {
            app.egui_renderer
                .update_texture(&device, &app.backend.queue, id, &delta);
        }

        app.egui_renderer.update_buffers(
            &device,
            &app.backend.queue,
            &mut encoder,
            &tris,
            &screen_descriptor,
        );

        egui_render_data = Some((tris, screen_descriptor, full_output.textures_delta.free));
    }

    // --- RENDER PASS ---
    {
        let window_context = match app.window_manager.get(output_id) {
            Some(ctx) => ctx,
            None => return Ok(()),
        };

        let surface_texture = window_context.surface.get_current_texture()?;
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Render Content
        render_content(
            RenderContext {
                device: &app.backend.device,
                queue: &app.backend.queue,
                target_format: window_context.surface_config.format,
                render_ops: &app.render_ops,
                output_manager: &app.state.output_manager,
                edge_blend_renderer: &app.edge_blend_renderer,
                color_calibration_renderer: &app.color_calibration_renderer,
                edge_blend_cache: &mut app.edge_blend_cache,
                edge_blend_texture_cache: &mut app.edge_blend_texture_cache,
                mesh_renderer: &mut app.mesh_renderer,
                effect_chain_renderer: &mut app.effect_chain_renderer,
                preview_effect_chain_renderer: &mut app.preview_effect_chain_renderer,
                shader_graph_manager: &app.shader_graph_manager,
                texture_pool: &app.texture_pool,
                _dummy_view: &app.dummy_view,
                mesh_buffer_cache: &mut app.mesh_buffer_cache,
                egui_renderer: &mut app.egui_renderer,
            },
            output_id,
            &mut encoder,
            &view,
            egui_render_data.as_ref(),
        )?;

        // --- NDI Readback (if enabled) ---
        #[cfg(feature = "ndi")]
        {
            // Find if this output has an NDI sender
            let part_id = app.render_ops.iter().find_map(|(_, op)| {
                if let mapmap_core::module::OutputType::Projector { id, .. } = &op.output_type {
                    if *id == output_id {
                        return Some(op.output_part_id);
                    }
                }
                None
            });

            if let Some(pid) = part_id {
                if let Some(sender) = app.ndi_senders.get_mut(&pid) {
                    let mut buffer_ready = false;
                    if let Some((buffer, mapping_requested)) = app.ndi_readbacks.get_mut(&output_id)
                    {
                        if mapping_requested.load(Ordering::SeqCst) {
                            let _ = app.backend.device.poll(wgpu::PollType::Wait {
                                submission_index: None,
                                timeout: Some(std::time::Duration::from_millis(0)),
                            });

                            if !buffer.slice(..).get_mapped_range().is_empty() {
                                {
                                    let view = buffer.slice(..).get_mapped_range();
                                    let frame_data = view.to_vec();
                                    let width = window_context.surface_config.width;
                                    let height = window_context.surface_config.height;

                                    let video_frame = mapmap_io::format::VideoFrame {
                                        data: mapmap_io::format::FrameData::Cpu(Arc::new(
                                            frame_data,
                                        )),
                                        format: mapmap_io::format::VideoFormat {
                                            width,
                                            height,
                                            pixel_format: mapmap_io::format::PixelFormat::BGRA8,
                                            frame_rate: 60.0,
                                        },
                                        timestamp: std::time::Duration::from_secs(0),
                                        metadata: Default::default(),
                                    };

                                    if let Err(e) = sender.send_frame(&video_frame) {
                                        tracing::warn!("Failed to send NDI frame: {}", e);
                                    }
                                }
                                buffer.unmap();
                                mapping_requested.store(false, Ordering::SeqCst);
                                buffer_ready = true;
                            }
                        } else {
                            buffer_ready = true;
                        }
                    } else {
                        let width = window_context.surface_config.width;
                        let height = window_context.surface_config.height;
                        let bytes_per_pixel = 4;
                        let unpadded_bytes_per_row = width * bytes_per_pixel;
                        let padding = (256 - unpadded_bytes_per_row % 256) % 256;
                        let bytes_per_row = unpadded_bytes_per_row + padding;
                        let size = (bytes_per_row * height) as u64;

                        let buffer = app.backend.device.create_buffer(&wgpu::BufferDescriptor {
                            label: Some("NDI Readback Buffer"),
                            size,
                            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                            mapped_at_creation: false,
                        });
                        app.ndi_readbacks.insert(
                            output_id,
                            (buffer, std::sync::Arc::new(AtomicBool::new(false))),
                        );
                        buffer_ready = true;
                    }

                    if buffer_ready {
                        if let Some((buffer, mapping_requested)) =
                            app.ndi_readbacks.get_mut(&output_id)
                        {
                            let width = window_context.surface_config.width;
                            let height = window_context.surface_config.height;
                            let bytes_per_pixel = 4;
                            let unpadded_bytes_per_row = width * bytes_per_pixel;
                            let padding = (256 - unpadded_bytes_per_row % 256) % 256;
                            let bytes_per_row = unpadded_bytes_per_row + padding;

                            encoder.copy_texture_to_buffer(
                                wgpu::TexelCopyTextureInfo {
                                    texture: &surface_texture.texture,
                                    mip_level: 0,
                                    origin: wgpu::Origin3d::ZERO,
                                    aspect: wgpu::TextureAspect::All,
                                },
                                wgpu::TexelCopyBufferInfo {
                                    buffer,
                                    layout: wgpu::TexelCopyBufferLayout {
                                        offset: 0,
                                        bytes_per_row: Some(bytes_per_row),
                                        rows_per_image: Some(height),
                                    },
                                },
                                wgpu::Extent3d {
                                    width,
                                    height,
                                    depth_or_array_layers: 1,
                                },
                            );

                            let slice = buffer.slice(..);
                            let requested_clone = mapping_requested.clone();
                            slice.map_async(wgpu::MapMode::Read, move |res| {
                                if res.is_ok() {
                                    requested_clone.store(true, Ordering::SeqCst);
                                }
                            });
                        }
                    }
                }
            }
        }

        app.backend.queue.submit(std::iter::once(encoder.finish()));
        window_context.window.pre_present_notify();
        surface_texture.present();
    }

    Ok(())
}

struct RenderContext<'a> {
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    target_format: wgpu::TextureFormat,
    render_ops: &'a Vec<(
        mapmap_core::module::ModulePartId,
        mapmap_core::module_eval::RenderOp,
    )>,
    output_manager: &'a mapmap_core::output::OutputManager,
    edge_blend_renderer: &'a Option<mapmap_render::EdgeBlendRenderer>,
    color_calibration_renderer: &'a Option<mapmap_render::ColorCalibrationRenderer>,
    edge_blend_cache: &'a mut std::collections::HashMap<u64, (wgpu::Buffer, wgpu::BindGroup, u64)>,
    edge_blend_texture_cache: &'a mut std::collections::HashMap<u64, wgpu::BindGroup>,
    mesh_renderer: &'a mut mapmap_render::MeshRenderer,
    effect_chain_renderer: &'a mut mapmap_render::EffectChainRenderer,
    preview_effect_chain_renderer: &'a mut mapmap_render::EffectChainRenderer,
    shader_graph_manager: &'a mapmap_render::ShaderGraphManager,
    texture_pool: &'a mapmap_render::TexturePool,
    _dummy_view: &'a Option<std::sync::Arc<wgpu::TextureView>>,
    mesh_buffer_cache: &'a mut mapmap_render::MeshBufferCache,
    egui_renderer: &'a mut egui_wgpu::Renderer,
}

fn render_content(
    ctx: RenderContext<'_>,
    output_id: u64,
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    egui_data: Option<&(
        Vec<egui::ClippedPrimitive>,
        egui_wgpu::ScreenDescriptor,
        Vec<egui::TextureId>,
    )>,
) -> Result<()> {
    let device = ctx.device;
    let queue = ctx.queue;
    let mesh_renderer = ctx.mesh_renderer;
    let egui_renderer = ctx.egui_renderer;

    let (is_preview_output, real_output_id) = normalize_output_id(output_id);

    let mut target_ops: Vec<(u64, mapmap_core::module_eval::RenderOp)> = ctx
        .render_ops
        .iter()
        .filter(|(_, op)| render_op_targets_output(op, output_id))
        .map(|(mid, op)| (*mid, op.clone()))
        .collect();

    target_ops.sort_by(|(_, a), (_, b)| b.output_part_id.cmp(&a.output_part_id));

    if target_ops.is_empty() && output_id != 0 {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Clear Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                depth_slice: None,
                view,
                resolve_target: None,

                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        return Ok(());
    }

    let output_config_opt = ctx.output_manager.get_output(real_output_id).cloned();
    let use_edge_blend = output_config_opt
        .as_ref()
        .map(has_active_edge_blend)
        .unwrap_or(false)
        && ctx.edge_blend_renderer.is_some();
    // Color calibration detection stays separate, but we do not route frames
    // through a calibration-only post-process path until that flow is stable.
    let _color_calibration_requested = output_config_opt
        .as_ref()
        .map(has_active_color_calibration)
        .unwrap_or(false)
        && ctx.color_calibration_renderer.is_some();

    let needs_post_processing = use_edge_blend;

    let intermediate_tex_name = format!("output_{}_intermediate", output_id);
    let mesh_target_view_ref = if needs_post_processing {
        if let Some(config) = &output_config_opt {
            ctx.texture_pool.ensure_texture(
                &intermediate_tex_name,
                config.resolution.0,
                config.resolution.1,
                ctx.target_format,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            );
        }
        Some(ctx.texture_pool.get_view(&intermediate_tex_name))
    } else {
        None
    };

    let target_view = if needs_post_processing {
        mesh_target_view_ref.as_deref().unwrap()
    } else {
        view
    };
    // Clear Pass
    {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Clear Output Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                depth_slice: None,
                view: target_view,
                resolve_target: None,

                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(if output_id == 0 {
                        wgpu::Color {
                            r: 0.05,
                            g: 0.05,
                            b: 0.05,
                            a: 1.0,
                        }
                    } else {
                        wgpu::Color::BLACK
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }

    // Accumulate Layers
    for (module_id, op) in target_ops {
        let tex_name = if let Some(src_id) = op.source_part_id {
            format!("part_{}_{}", module_id, src_id)
        } else {
            "".to_string()
        };

        let source_view = if op.mapping_mode {
            let grid_tex_name = format!("grid_layer_{}", op.layer_part_id);
            if !ctx.texture_pool.has_texture(&grid_tex_name) {
                let width = 512;
                let height = 512;
                let data = generate_grid_texture(width, height, op.layer_part_id);
                ctx.texture_pool.ensure_texture(
                    &grid_tex_name,
                    width,
                    height,
                    wgpu::TextureFormat::Rgba8UnormSrgb,
                    wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                );
                ctx.texture_pool
                    .upload_data(queue, &grid_tex_name, &data, width, height);
            }
            Some(ctx.texture_pool.get_view(&grid_tex_name))
        } else if ctx.texture_pool.has_texture(&tex_name) {
            Some(ctx.texture_pool.get_view(&tex_name))
        } else if ctx.texture_pool.has_texture("bevy_output") {
            // Fallback for Bevy nodes
            Some(ctx.texture_pool.get_view("bevy_output"))
        } else {
            // BLACK FALLBACK for missing textures
            let fallback_name = "missing_texture_fallback";
            if !ctx.texture_pool.has_texture(fallback_name) {
                let width = 64;
                let height = 64;
                let data = [0, 0, 0, 255].repeat((width * height) as usize);
                ctx.texture_pool.ensure_texture(
                    fallback_name,
                    width,
                    height,
                    wgpu::TextureFormat::Rgba8UnormSrgb,
                    wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                );
                ctx.texture_pool
                    .upload_data(queue, fallback_name, &data, width, height);
            }
            Some(ctx.texture_pool.get_view(fallback_name))
        };

        if let Some(src_ref) = source_view {
            let mut final_source_view = src_ref.clone();

            if !op.effects.is_empty() {
                let effect_chain = build_effect_chain(&op.effects);
                if !effect_chain.effects.is_empty() {
                    let output_texture_name = format!(
                        "effect_tmp_output_{}_layer_{}",
                        real_output_id, op.layer_part_id
                    );
                    let effect_width = 1024;
                    let effect_height = 1024;
                    ctx.texture_pool.ensure_texture(
                        &output_texture_name,
                        effect_width,
                        effect_height,
                        wgpu::TextureFormat::Bgra8UnormSrgb,
                        wgpu::TextureUsages::TEXTURE_BINDING
                            | wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::COPY_DST,
                    );

                    let output_effect_view = ctx.texture_pool.get_view(&output_texture_name);
                    if is_preview_output {
                        ctx.preview_effect_chain_renderer.apply_chain(
                            encoder,
                            &src_ref,
                            &output_effect_view,
                            &effect_chain,
                            ctx.shader_graph_manager,
                            0.0,
                            effect_width,
                            effect_height,
                        );
                    } else {
                        ctx.effect_chain_renderer.apply_chain(
                            encoder,
                            &src_ref,
                            &output_effect_view,
                            &effect_chain,
                            ctx.shader_graph_manager,
                            0.0,
                            effect_width,
                            effect_height,
                        );
                    }

                    final_source_view = output_effect_view;
                }
            }

            let transform = glam::Mat4::IDENTITY;
            let uniform_bind_group = mesh_renderer.get_uniform_bind_group_with_source_props(
                queue,
                transform,
                op.opacity * op.source_props.opacity,
                op.source_props.flip_horizontal,
                op.source_props.flip_vertical,
                op.source_props.brightness,
                op.source_props.contrast,
                op.source_props.saturation,
                op.source_props.hue_shift,
            );

            let texture_bind_group = mesh_renderer.get_texture_bind_group(&final_source_view);
            let (vb, ib, cnt) = ctx.mesh_buffer_cache.get_buffers(
                device,
                queue,
                op.layer_part_id,
                &op.mesh.to_mesh(),
            );

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Mesh Layer Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    depth_slice: None,
                    view: target_view,
                    resolve_target: None,

                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            mesh_renderer.draw(
                &mut rpass,
                vb,
                ib,
                cnt,
                &uniform_bind_group,
                &texture_bind_group,
                true,
            );
        }
    }

    // --- POST PROCESSING PASSES ---
    if needs_post_processing {
        let intermediate_view = mesh_target_view_ref.as_ref().unwrap();
        // Re-create the texture bind group each frame since the intermediate texture may be re-allocated by the pool,
        // but we could optimize this later by checking if the texture's ID changed.
        // For now, creating a texture bind group is relatively cheap compared to buffers.
        if let Some(edge_blend_renderer) = ctx.edge_blend_renderer.as_ref() {
            let texture_bind_group = ctx
                .edge_blend_texture_cache
                .entry(output_id)
                .or_insert_with(|| {
                    edge_blend_renderer.create_texture_bind_group(intermediate_view)
                });
            // Update texture bind group if view changed (TexturePool creates new textures on resize)
            // As a simple fix to avoid holding stale views across resizes, we just recreate it.
            *texture_bind_group = edge_blend_renderer.create_texture_bind_group(intermediate_view);

            let config_to_use = if use_edge_blend {
                output_config_opt.map(|c| c.edge_blend).unwrap_or_default()
            } else {
                mapmap_core::EdgeBlendConfig::default()
            };

            // Simple hash for config changes
            use std::hash::Hasher;
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            hasher.write(&[config_to_use.left.enabled as u8]);
            hasher.write(&[config_to_use.right.enabled as u8]);
            hasher.write(&[config_to_use.top.enabled as u8]);
            hasher.write(&[config_to_use.bottom.enabled as u8]);
            hasher.write(&config_to_use.left.width.to_le_bytes());
            hasher.write(&config_to_use.right.width.to_le_bytes());
            hasher.write(&config_to_use.top.width.to_le_bytes());
            hasher.write(&config_to_use.bottom.width.to_le_bytes());
            hasher.write(&config_to_use.gamma.to_le_bytes());
            let config_hash = hasher.finish();

            let (uniform_buffer, uniform_bind_group, last_hash) =
                ctx.edge_blend_cache.entry(output_id).or_insert_with(|| {
                    let buffer = edge_blend_renderer.create_uniform_buffer(&config_to_use);
                    let bind_group = edge_blend_renderer.create_uniform_bind_group(&buffer);
                    (buffer, bind_group, config_hash)
                });

            if *last_hash != config_hash {
                edge_blend_renderer.update_uniform_buffer(queue, uniform_buffer, &config_to_use);
                *last_hash = config_hash;
            }

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(if use_edge_blend {
                    "Edge Blending Pass"
                } else {
                    "Passthrough Pass"
                }),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    depth_slice: None,
                    view, // Draw to the final surface view
                    resolve_target: None,

                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), // Clear previous if any
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            edge_blend_renderer.render(&mut rpass, texture_bind_group, uniform_bind_group);
        }
    }

    // EgUI Overlay
    if output_id == 0 {
        if let Some((tris, screen_desc, free_textures)) = egui_data {
            // Free textures from previous frames
            for id in free_textures {
                egui_renderer.free_texture(id);
            }

            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    depth_slice: None,
                    view,
                    resolve_target: None,

                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Render egui UI
            egui_renderer.render(&mut render_pass.forget_lifetime(), tris, screen_desc);
        }
    }
    Ok(())
}

fn has_active_edge_blend(config: &mapmap_core::OutputConfig) -> bool {
    config.edge_blend.left.enabled
        || config.edge_blend.right.enabled
        || config.edge_blend.top.enabled
        || config.edge_blend.bottom.enabled
}

fn has_active_color_calibration(config: &mapmap_core::OutputConfig) -> bool {
    const EPSILON: f32 = 0.0001;

    let calibration = &config.color_calibration;
    calibration.brightness.abs() > EPSILON
        || (calibration.contrast - 1.0).abs() > EPSILON
        || (calibration.gamma.x - 1.0).abs() > EPSILON
        || (calibration.gamma.y - 1.0).abs() > EPSILON
        || (calibration.gamma_b - 1.0).abs() > EPSILON
        || (calibration.color_temp - 6500.0).abs() > EPSILON
        || (calibration.saturation - 1.0).abs() > EPSILON
}

#[allow(clippy::manual_is_multiple_of)]
fn prepare_texture_previews(app: &mut App, encoder: &mut wgpu::CommandEncoder) {
    // 1. THROTTLING: Only update previews every 5 frames to save GPU time
    app.frame_counter = app.frame_counter.wrapping_add(1);
    #[allow(clippy::manual_is_multiple_of)]
    if app.frame_counter % 5 != 0 {
        return;
    }

    // 2. CACHING: Only rebuild the list of output parts if graph changed
    if app.cached_output_infos.is_empty()
        || app.last_graph_revision != app.state.module_manager.graph_revision
    {
        app.cached_output_infos = app
            .state
            .module_manager
            .list_modules()
            .iter()
            .flat_map(|m| m.parts.iter().map(move |p| (m.id, p)))
            .filter_map(|(mid, part)| {
                if let mapmap_core::module::ModulePartType::Output(
                    mapmap_core::module::OutputType::Projector { id, .. },
                ) = &part.part_type
                {
                    Some((mid, *id, format!("output_{}", id)))
                } else {
                    None
                }
            })
            .collect();
    }

    for (_mid, output_id, _name) in &app.cached_output_infos {
        let output_id = *output_id;
        if !app
            .render_ops
            .iter()
            .any(|(_, op)| render_op_targets_output(op, output_id))
        {
            continue;
        }

        let preview_width = 256;
        let preview_height = 144;

        let needs_recreate = if let Some(tex) = app.output_temp_textures.get(&output_id) {
            tex.width() != preview_width || tex.height() != preview_height
        } else {
            true
        };

        if needs_recreate {
            let texture = app.backend.device.create_texture(&wgpu::TextureDescriptor {
                label: Some(&format!("Preview Tex {}", output_id)),
                size: wgpu::Extent3d {
                    width: preview_width,
                    height: preview_height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: app.backend.surface_format(),
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            app.output_temp_textures.insert(output_id, texture);
        }

        let target_tex = app.output_temp_textures.get(&output_id).unwrap();

        use std::collections::hash_map::Entry;
        let current_view_arc = match app.output_preview_cache.entry(output_id) {
            Entry::Occupied(mut e) => {
                let (id, old_view) = e.get_mut();
                if needs_recreate {
                    let target_view =
                        target_tex.create_view(&wgpu::TextureViewDescriptor::default());
                    let target_view_arc = std::sync::Arc::new(target_view);
                    app.egui_renderer.update_egui_texture_from_wgpu_texture(
                        &app.backend.device,
                        &target_view_arc,
                        wgpu::FilterMode::Linear,
                        *id,
                    );
                    *e.get_mut() = (*id, target_view_arc.clone());
                    target_view_arc
                } else {
                    old_view.clone()
                }
            }
            Entry::Vacant(e) => {
                let target_view = target_tex.create_view(&wgpu::TextureViewDescriptor::default());
                let target_view_arc = std::sync::Arc::new(target_view);
                let id = app.egui_renderer.register_native_texture(
                    &app.backend.device,
                    &target_view_arc,
                    wgpu::FilterMode::Linear,
                );
                e.insert((id, target_view_arc.clone()));
                target_view_arc
            }
        };

        let preview_output_id = output_id | PREVIEW_FLAG;
        if let Err(e) = render_content(
            RenderContext {
                device: &app.backend.device,
                queue: &app.backend.queue,
                target_format: app.backend.surface_format(),
                render_ops: &app.render_ops,
                output_manager: &app.state.output_manager,
                edge_blend_renderer: &app.edge_blend_renderer,
                color_calibration_renderer: &app.color_calibration_renderer,
                edge_blend_cache: &mut app.edge_blend_cache,
                edge_blend_texture_cache: &mut app.edge_blend_texture_cache,
                mesh_renderer: &mut app.mesh_renderer,
                effect_chain_renderer: &mut app.effect_chain_renderer,
                preview_effect_chain_renderer: &mut app.preview_effect_chain_renderer,
                shader_graph_manager: &app.shader_graph_manager,
                texture_pool: &app.texture_pool,
                _dummy_view: &app.dummy_view,
                mesh_buffer_cache: &mut app.mesh_buffer_cache,
                egui_renderer: &mut app.egui_renderer,
            },
            preview_output_id,
            encoder,
            current_view_arc.as_ref(),
            None,
        ) {
            tracing::warn!(
                "Failed to render preview texture for output {}: {}",
                output_id,
                e
            );
        }
    }

    // --- NEW: Sync individual Node Previews for the Canvas ---
    if let Some(active_id) = app.ui_state.module_canvas.active_module_id {
        if let Some(module) = app.state.module_manager.get_module(active_id) {
            for part in &module.parts {
                let texture_name = format!("part_{}_{}", active_id, part.id);
                if app.texture_pool.has_texture(&texture_name) {
                    let view = app.texture_pool.get_view(&texture_name);

                    // Register or update egui texture
                    use std::collections::hash_map::Entry;
                    let tex_id = match app
                        .ui_state
                        .module_canvas
                        .node_previews
                        .entry((active_id, part.id))
                    {
                        Entry::Occupied(e) => {
                            let id = *e.get();
                            app.egui_renderer.update_egui_texture_from_wgpu_texture(
                                &app.backend.device,
                                &view,
                                wgpu::FilterMode::Linear,
                                id,
                            );
                            id
                        }
                        Entry::Vacant(e) => {
                            let id = app.egui_renderer.register_native_texture(
                                &app.backend.device,
                                &view,
                                wgpu::FilterMode::Linear,
                            );
                            e.insert(id);
                            id
                        }
                    };
                    // Ensure it stays in the map (redundant but safe)
                    app.ui_state
                        .module_canvas
                        .node_previews
                        .insert((active_id, part.id), tex_id);
                }
            }
        }
    }
}

fn build_effect_chain(modulizers: &[ModulizerType]) -> EffectChain {
    let mut chain = EffectChain::new();
    let mut next_id = 1u64;

    for modulizer in modulizers {
        let ModulizerType::Effect {
            effect_type,
            params,
        } = modulizer
        else {
            continue;
        };

        let Some(chain_effect_type) = map_effect_type(*effect_type) else {
            continue;
        };

        let mut effect = Effect::new(next_id, chain_effect_type);
        effect.parameters.extend(params.clone());
        chain.effects.push(effect);
        next_id += 1;
    }

    chain
}

fn map_effect_type(effect_type: ModEffectType) -> Option<ChainEffectType> {
    Some(match effect_type {
        ModEffectType::ShaderGraph(id) => ChainEffectType::ShaderGraph(id),
        ModEffectType::Blur => ChainEffectType::Blur,
        ModEffectType::Invert => ChainEffectType::Invert,
        ModEffectType::HueShift => ChainEffectType::HueShift,
        ModEffectType::Wave => ChainEffectType::Wave,
        ModEffectType::Mirror => ChainEffectType::Mirror,
        ModEffectType::Kaleidoscope => ChainEffectType::Kaleidoscope,
        ModEffectType::Pixelate => ChainEffectType::Pixelate,
        ModEffectType::EdgeDetect => ChainEffectType::EdgeDetect,
        ModEffectType::Glitch => ChainEffectType::Glitch,
        ModEffectType::RgbSplit => ChainEffectType::RgbSplit,
        ModEffectType::ChromaticAberration => ChainEffectType::ChromaticAberration,
        ModEffectType::FilmGrain => ChainEffectType::FilmGrain,
        ModEffectType::Vignette => ChainEffectType::Vignette,
        ModEffectType::LoadLUT
        | ModEffectType::Brightness
        | ModEffectType::Contrast
        | ModEffectType::Saturation
        | ModEffectType::Colorize
        | ModEffectType::Sharpen
        | ModEffectType::Threshold
        | ModEffectType::Spiral
        | ModEffectType::Pinch
        | ModEffectType::Halftone
        | ModEffectType::Posterize
        | ModEffectType::VHS => return None,
    })
}

fn generate_grid_texture(width: u32, height: u32, layer_id: u64) -> Vec<u8> {
    let mut data = vec![0u8; (width * height * 4) as usize];
    let _bg_color = [0, 0, 0, 255];
    let _grid_color = [255, 255, 255, 255];
    let _text_color = [0, 255, 255, 255];

    for i in 0..(width * height) {
        let idx = (i * 4) as usize;
        data[idx] = 0;
        data[idx + 1] = 0;
        data[idx + 2] = 0;
        data[idx + 3] = 255;
    }
    let grid_step = 64;
    for y in 0..height {
        for x in 0..width {
            if x % grid_step == 0 || y % grid_step == 0 || x == width - 1 || y == height - 1 {
                let idx = ((y * width + x) * 4) as usize;
                data[idx] = 255;
                data[idx + 1] = 255;
                data[idx + 2] = 255;
                data[idx + 3] = 255;
            }
        }
    }

    let id_str = format!("{}", layer_id);
    let digit_scale = 8;
    let digit_w = 3 * digit_scale;
    let total_w = id_str.len() as u32 * (digit_w + 2 * digit_scale);
    let start_x = (width.saturating_sub(total_w)) / 2;
    let start_y = (height.saturating_sub(5 * digit_scale)) / 2;
    for (i, char) in id_str.chars().enumerate() {
        if let Some(digit) = char.to_digit(10) {
            draw_digit(
                &mut data,
                width,
                digit as usize,
                start_x + i as u32 * (digit_w + 2 * digit_scale),
                start_y,
                digit_scale,
                [0, 255, 255, 255],
            );
        }
    }
    data
}

const BITMAPS: [[u8; 5]; 10] = [
    [7, 5, 5, 5, 7],
    [2, 6, 2, 2, 7],
    [7, 1, 7, 4, 7],
    [7, 1, 7, 1, 7],
    [5, 5, 7, 1, 1],
    [7, 4, 7, 1, 7],
    [7, 4, 7, 5, 7],
    [7, 1, 1, 1, 1],
    [7, 5, 7, 5, 7],
    [7, 5, 7, 1, 7],
];

fn draw_digit(
    data: &mut [u8],
    width: u32,
    digit: usize,
    offset_x: u32,
    offset_y: u32,
    scale: u32,
    color: [u8; 4],
) {
    if digit > 9 {
        return;
    }
    let bitmap = BITMAPS[digit];
    for (row, row_bits) in bitmap.iter().enumerate() {
        for col in 0..3 {
            if (row_bits >> (2 - col)) & 1 == 1 {
                for dy in 0..scale {
                    for dx in 0..scale {
                        let x = offset_x + col as u32 * scale + dx;
                        let y = offset_y + row as u32 * scale + dy;
                        if x < width && y < (data.len() as u32 / width / 4) {
                            let idx = ((y * width + x) * 4) as usize;
                            data[idx] = color[0];
                            data[idx + 1] = color[1];
                            data[idx + 2] = color[2];
                            data[idx + 3] = color[3];
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        has_active_color_calibration, has_active_edge_blend, render_op_targets_output, PREVIEW_FLAG,
    };
    use mapmap_core::module::{MeshType, OutputType};
    use mapmap_core::{CanvasRegion, OutputConfig, RenderOp, SourceProperties};

    fn output_config() -> OutputConfig {
        OutputConfig::new(
            1,
            "Projector".to_string(),
            CanvasRegion::new(0.0, 0.0, 1.0, 1.0),
            (1920, 1080),
        )
    }

    #[test]
    fn default_output_has_no_active_post_processing() {
        let config = output_config();

        assert!(!has_active_edge_blend(&config));
        assert!(!has_active_color_calibration(&config));
    }

    #[test]
    fn modified_output_enables_post_processing() {
        let mut config = output_config();
        config.edge_blend.left.enabled = true;
        config.color_calibration.saturation = 1.2;

        assert!(has_active_edge_blend(&config));
        assert!(has_active_color_calibration(&config));
    }

    fn projector_render_op(projector_id: u64) -> RenderOp {
        RenderOp {
            output_part_id: 99,
            output_type: OutputType::Projector {
                id: projector_id,
                name: format!("Output {}", projector_id),
                hide_cursor: false,
                target_screen: 0,
                show_in_preview_panel: true,
                extra_preview_window: false,
                output_width: 0,
                output_height: 0,
                output_fps: 60.0,
                ndi_enabled: false,
                ndi_stream_name: String::new(),
            },
            layer_part_id: 7,
            mesh: MeshType::default(),
            opacity: 1.0,
            blend_mode: None,
            mapping_mode: false,
            source_part_id: Some(5),
            source_props: SourceProperties::default_identity(),
            effects: Vec::new(),
            masks: Vec::new(),
        }
    }

    #[test]
    fn preview_flag_targets_same_projector_render_ops() {
        let op = projector_render_op(7);

        assert!(render_op_targets_output(&op, 7));
        assert!(render_op_targets_output(&op, PREVIEW_FLAG | 7));
        assert!(!render_op_targets_output(&op, 8));
    }
}
