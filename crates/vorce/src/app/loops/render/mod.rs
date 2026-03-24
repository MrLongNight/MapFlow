//! Main application render loop.

use crate::app::core::app_struct::App;
use crate::app::ui_layout;
use anyhow::Result;
#[cfg(feature = "ndi")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(feature = "ndi")]
use std::sync::Arc;
use vorce_core::OutputId;

mod content;
mod effects;
mod logging;
mod previews;
mod texture_gen;

use content::*;
use previews::*;

pub(crate) const PREVIEW_FLAG: u64 = 1u64 << 63;

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
            let runner: &vorce_bevy::BevyRunner = runner;
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
                render_queue: &app.render_queue.items,
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
                video_diagnostic_log_times: &mut app.video_diagnostic_log_times,
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
            let part_id = app.render_queue.items.get(&output_id).and_then(|items| {
                items.iter().find_map(|item| {
                    if let vorce_core::module::OutputType::Projector { id, .. } =
                        &item.render_op.output_type
                    {
                        if *id == output_id {
                            return Some(item.render_op.output_part_id);
                        }
                    }
                    None
                })
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

                                    let video_frame = vorce_io::format::VideoFrame {
                                        data: vorce_io::format::FrameData::Cpu(Arc::new(
                                            frame_data,
                                        )),
                                        format: vorce_io::format::VideoFormat {
                                            width,
                                            height,
                                            pixel_format: vorce_io::format::PixelFormat::BGRA8,
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

    if output_id == 0 {
        app.frame_counter = app.frame_counter.saturating_add(1);
    }

    Ok(())
}
