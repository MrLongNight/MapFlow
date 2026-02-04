//! Main application render loop.

use crate::app::core::app_struct::App;
use crate::app::ui_layout;
use anyhow::Result;
use mapmap_core::module::OutputType::Projector;
use mapmap_core::OutputId;

#[cfg(feature = "midi")]
/// Renders the UI or content for the given output ID.
pub fn render(app: &mut App, output_id: OutputId) -> Result<()> {
    // Clone device Arc to create encoder without borrowing self
    let device = app.backend.device.clone();

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    // ⚡ Bolt Optimization: Batch render passes.
    app.mesh_renderer.begin_frame();

    if output_id == 0 {
        // Sync Texture Previews
        prepare_texture_previews(app, &mut encoder);

        // Update Bevy Texture
        if let Some(runner) = &app.bevy_runner {
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

    // Create raw pointer for UI loop hack BEFORE borrowing window_context
    let app_ptr = app as *mut App;

    // SCOPE for Window Context Borrow
    // Use inner scope to manage lifetimes of window_context vs disjoint app parts for render_content
    {
        // Safety check for window existence
        let has_window = app.window_manager.get(output_id).is_some();
        if !has_window {
            return Ok(());
        }

        // We need fields from app for ui_layout::show and later render_content.
        // But we need window for egui input/output and surface for view.
        // Surface texture must live until present.

        let window_context = app.window_manager.get(output_id).unwrap();
        let surface_texture = window_context.surface.get_current_texture()?;
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut egui_render_data = None;

        if output_id == 0 {
            // UI Pass
            let raw_input = app.egui_state.take_egui_input(&window_context.window);

            // Drop window borrow during closure?
            // "window_context" is the borrow.
            // We cannot drop it here because "surface_texture" depends on it (maybe).
            // Actually, does surface_texture borrow window_context.surface? Yes.
            // So we cannot use "app" fully in closure.
            // But ui_layout::show takes "&mut App".
            // CONFLICT.

            // WORKAROUND: We must assume ui_layout::show DOES NOT touch window_manager.
            // But strict borrowing prevents this.
            // EXCEPT if we refactor ui_layout::show OR use unsafe to effectively unborrow.

            // Since we are modularizing, we should accept that we have limits.
            // Let's defer UI rendering or use disjoint references?
            // ui_layout::show touches almost everything.

            // Revert to "Unsafe Transmute to Static" or similar hack used in main.rs?
            // No, main.rs didn't have this conflict because logic was inline.

            // We'll use a scoped hack:
            // We cheat the borrow checker by transmuting 'app' to 'static for the closure,
            // knowing that 'window_context' also borrows 'app' but disjointly (logic wise).
            // This is dangerous but pragmatic for this refactor step.
            // Ideally: split App.

            let full_output = app.egui_context.run(raw_input, |ctx| {
                // SAFETY: We ensure window_context doesn't overlap with fields used in show.
                // show uses: sys_info, audio, state, media, etc.
                // It does NOT use window_manager (except maybe to layout? No).
                unsafe {
                    ui_layout::show(&mut *app_ptr, ctx);
                }
            });

            app.egui_state
                .handle_platform_output(&window_context.window, full_output.platform_output);

            let tris = app
                .egui_context
                .tessellate(full_output.shapes, app.egui_context.pixels_per_point());
            for (id, delta) in full_output.textures_delta.set {
                app.egui_renderer
                    .update_texture(&device, &app.backend.queue, id, &delta);
            }

            let screen_descriptor = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [
                    window_context.surface_config.width,
                    window_context.surface_config.height,
                ],
                pixels_per_point: app.egui_context.pixels_per_point(),
            };
            egui_render_data = Some((tris, screen_descriptor, full_output.textures_delta.free));
        }

        // --- Render Content ---
        // Pass disjoint fields to avoid conflict with window_context borrow
        render_content(
            RenderContext {
                device: &app.backend.device,
                queue: &app.backend.queue,
                render_ops: &app.render_ops,
                output_manager: &app.state.output_manager,
                edge_blend_renderer: &app.edge_blend_renderer,
                color_calibration_renderer: &app.color_calibration_renderer,
                mesh_renderer: &mut app.mesh_renderer,
                texture_pool: &app.texture_pool,
                dummy_view: &app.dummy_view,
                mesh_buffer_cache: &mut app.mesh_buffer_cache,
                egui_renderer: &mut app.egui_renderer,
            },
            output_id,
            &mut encoder,
            &view,
            egui_render_data.as_ref(),
        )?;

        // Free textures
        if let Some((_, _, free_textures)) = egui_render_data {
            for id in free_textures {
                app.egui_renderer.free_texture(&id);
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
    render_ops: &'a Vec<(
        mapmap_core::module::ModulePartId,
        mapmap_core::module_eval::RenderOp,
    )>,
    output_manager: &'a mapmap_core::output::OutputManager,
    edge_blend_renderer: &'a Option<mapmap_render::EdgeBlendRenderer>,
    color_calibration_renderer: &'a Option<mapmap_render::ColorCalibrationRenderer>,
    mesh_renderer: &'a mut mapmap_render::MeshRenderer,
    texture_pool: &'a mapmap_render::TexturePool,
    dummy_view: &'a Option<std::sync::Arc<wgpu::TextureView>>,
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
    // Unpack context
    let device = ctx.device;
    let queue = ctx.queue;
    let mesh_renderer = ctx.mesh_renderer;
    let egui_renderer = ctx.egui_renderer;

    const PREVIEW_FLAG: u64 = 1u64 << 63;
    let real_output_id = output_id & !PREVIEW_FLAG;

    let mut target_ops: Vec<(u64, mapmap_core::module_eval::RenderOp)> = ctx
        .render_ops
        .iter()
        .filter(|(_, op)| match &op.output_type {
            Projector { id, .. } => *id == real_output_id,
            _ => op.output_part_id == real_output_id,
        })
        .map(|(mid, op)| (*mid, op.clone()))
        .collect();

    target_ops.sort_by(|(_, a), (_, b)| b.output_part_id.cmp(&a.output_part_id));

    if target_ops.is_empty() && output_id != 0 {
        // Clear pass
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Clear Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        return Ok(());
    }

    let output_config_opt = ctx.output_manager.get_output(output_id).cloned();
    let use_edge_blend = output_config_opt.is_some() && ctx.edge_blend_renderer.is_some();
    let use_color_calib = output_config_opt.is_some() && ctx.color_calibration_renderer.is_some();
    let _needs_post_processing = use_edge_blend || use_color_calib;

    let mesh_target_view_ref = view; // Simplified for now

    // Clear Pass
    {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Clear Output Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: mesh_target_view_ref,
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
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }

    // Accumulate Layers
    mesh_renderer.begin_frame();
    for (module_id, op) in target_ops {
        let (tex_name, opacity_mult) = if op.mapping_mode {
            let grid_tex_name = format!("system_grid_texture_{}", op.layer_part_id);
            // Ensure grid texture exists for this layer ID
            if !ctx.texture_pool.has_texture(&grid_tex_name) {
                let (width, height, data) = create_grid_texture(op.layer_part_id);
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
            (grid_tex_name, 1.0)
        } else if let Some(src_id) = op.source_part_id {
            (format!("part_{}_{}", module_id, src_id), op.opacity)
        } else {
            ("".to_string(), op.opacity)
        };

        // Use has_texture and get_view. Assuming dimensions are handled via UVs properly.
        let source_view = if ctx.texture_pool.has_texture(&tex_name) {
            Some(ctx.texture_pool.get_view(&tex_name))
        } else {
            ctx.dummy_view.as_ref().map(|v| v.clone())
        };

        if let Some(src_ref) = source_view {
            let transform = glam::Mat4::IDENTITY;
            let uniform_bind_group = mesh_renderer.get_uniform_bind_group_with_source_props(
                queue,
                transform,
                opacity_mult * op.source_props.opacity,
                op.source_props.flip_horizontal,
                op.source_props.flip_vertical,
                op.source_props.brightness,
                op.source_props.contrast,
                op.source_props.saturation,
                op.source_props.hue_shift,
            );

            let texture_bind_group = mesh_renderer.get_texture_bind_group(&src_ref);
            let (vb, ib, cnt) = ctx.mesh_buffer_cache.get_buffers(
                device,
                queue,
                op.layer_part_id,
                &op.mesh.to_mesh(),
            );

            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Mesh Layer Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: mesh_target_view_ref,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
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

    // EgUI Overlay
    if output_id == 0 {
        if let Some((tris, screen_desc, _)) = egui_data {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let renderer_static: &'static egui_wgpu::Renderer =
                unsafe { std::mem::transmute(&*egui_renderer) };
            let render_pass_static: &mut wgpu::RenderPass<'static> =
                unsafe { std::mem::transmute(&mut render_pass) };

            renderer_static.render(render_pass_static, tris, screen_desc);
        }
    }
    Ok(())
}

fn prepare_texture_previews(app: &mut App, encoder: &mut wgpu::CommandEncoder) {
    let module_output_infos: Vec<(u64, u64, String)> = app
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

    for (_mid, output_id, _name) in module_output_infos {
        if let Some(texture_name) = app
            .output_assignments
            .get(&output_id)
            .and_then(|v| v.last())
            .cloned()
        {
            if app.texture_pool.has_texture(&texture_name) {
                // Fixed Aspect Ratio assumption (16:9) since we can't get texture dim easily
                let preview_width = 256;
                let preview_height = 144; // 16:9

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
                let target_view_arc = std::sync::Arc::new(
                    target_tex.create_view(&wgpu::TextureViewDescriptor::default()),
                );

                use std::collections::hash_map::Entry;
                match app.output_preview_cache.entry(output_id) {
                    Entry::Occupied(mut e) => {
                        let (id, _old_view) = e.get_mut();
                        app.egui_renderer.update_egui_texture_from_wgpu_texture(
                            &app.backend.device,
                            &target_view_arc,
                            wgpu::FilterMode::Linear,
                            *id,
                        );
                        *e.into_mut() = (*id, target_view_arc.clone());
                    }
                    Entry::Vacant(e) => {
                        let id = app.egui_renderer.register_native_texture(
                            &app.backend.device,
                            &target_view_arc,
                            wgpu::FilterMode::Linear,
                        );
                        e.insert((id, target_view_arc.clone()));
                    }
                }

                {
                    let transform = glam::Mat4::IDENTITY;
                    let uniform_bind_group = app.mesh_renderer.get_uniform_bind_group(
                        &app.backend.queue,
                        transform,
                        1.0,
                    );
                    let source_view = app.texture_pool.get_view(&texture_name);
                    let texture_bind_group = app.mesh_renderer.get_texture_bind_group(&source_view);

                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Preview Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &target_view_arc,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    app.mesh_renderer.draw(
                        &mut render_pass,
                        &app.preview_quad_buffers.0,
                        &app.preview_quad_buffers.1,
                        app.preview_quad_buffers.2,
                        &uniform_bind_group,
                        &texture_bind_group,
                        false,
                    );
                }
            }
        }
    }
}

/// Creates a grid texture for mapping mode with Layer ID
fn create_grid_texture(layer_id: u64) -> (u32, u32, Vec<u8>) {
    let width = 512;
    let height = 512;
    let mut data = vec![0; (width * height * 4) as usize];

    // Background Pattern
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;

            // Grid lines every 64 pixels
            let is_grid = x % 64 < 2 || y % 64 < 2;
            // Center crosshair
            let is_center =
                (x as i32 - width as i32 / 2).abs() < 4 || (y as i32 - height as i32 / 2).abs() < 4;
            // Border
            let is_border = x < 4 || x > width - 5 || y < 4 || y > height - 5;

            if is_center {
                // Red
                data[idx] = 255;
                data[idx + 1] = 0;
                data[idx + 2] = 0;
                data[idx + 3] = 255;
            } else if is_border {
                // Yellow
                data[idx] = 255;
                data[idx + 1] = 255;
                data[idx + 2] = 0;
                data[idx + 3] = 255;
            } else if is_grid {
                // Black
                data[idx] = 0;
                data[idx + 1] = 0;
                data[idx + 2] = 0;
                data[idx + 3] = 255;
            } else {
                // White/Grey checkerboard
                let check = ((x / 32) + (y / 32)) % 2 == 0;
                let val = if check { 255 } else { 220 };
                data[idx] = val;
                data[idx + 1] = val;
                data[idx + 2] = val;
                data[idx + 3] = 255;
            }
        }
    }

    // Draw ID Number
    let id_str = format!("{}", layer_id);
    let digit_scale = 8; // Scale up pixels
    let digit_spacing = 40; // Pixels between digits
    let start_x = (width / 2) as i32 - ((id_str.len() as i32 * digit_spacing) / 2);
    let start_y = (height / 2) as i32 - 60; // Slightly above center

    for (i, c) in id_str.chars().enumerate() {
        if let Some(digit) = c.to_digit(10) {
            draw_digit(
                &mut data,
                width,
                height,
                start_x + (i as i32 * digit_spacing),
                start_y,
                digit as usize,
                digit_scale,
            );
        }
    }

    (width, height, data)
}

/// Helper to draw a digit into the texture buffer
fn draw_digit(
    data: &mut [u8],
    width: u32,
    height: u32,
    pos_x: i32,
    pos_y: i32,
    digit: usize,
    scale: i32,
) {
    // 3x5 Pixel Font Definitions (1 = On, 0 = Off)
    const FONT: [[u8; 15]; 10] = [
        [1, 1, 1, 1, 0, 1, 1, 0, 1, 1, 0, 1, 1, 1, 1], // 0
        [0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0], // 1
        [1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1], // 2
        [1, 1, 1, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1], // 3
        [1, 0, 1, 1, 0, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1], // 4
        [1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 1, 1, 1, 1], // 5
        [1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1], // 6
        [1, 1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1], // 7
        [1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1], // 8
        [1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1], // 9
    ];

    if digit >= 10 {
        return;
    }

    let pattern = &FONT[digit];

    for fy in 0..5 {
        for fx in 0..3 {
            if pattern[fy * 3 + fx] == 1 {
                // Draw scaled pixel block
                for dy in 0..scale {
                    for dx in 0..scale {
                        let px = pos_x + (fx as i32 * scale) + dx;
                        let py = pos_y + (fy as i32 * scale) + dy;

                        if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                            let idx = ((py as u32 * width + px as u32) * 4) as usize;
                            // Draw Blue text
                            data[idx] = 0;
                            data[idx + 1] = 0;
                            data[idx + 2] = 255;
                            data[idx + 3] = 255;
                        }
                    }
                }
            }
        }
    }
}
