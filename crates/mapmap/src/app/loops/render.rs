//! Main application render loop.

use crate::app::core::app_struct::App;
use crate::app::ui_layout;
use anyhow::Result;
use mapmap_core::module::OutputType::Projector;
use mapmap_core::OutputId;

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

    // Create raw pointer for UI loop hack BEFORE borrowing window_context
    let app_ptr = app as *mut App;

    // SCOPE for Window Context Borrow
    {
        // Safety check for window existence
        let has_window = app.window_manager.get(output_id).is_some();
        if !has_window {
            return Ok(());
        }

        let window_context = app.window_manager.get(output_id).unwrap();
        let surface_texture = window_context.surface.get_current_texture()?;
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut egui_render_data = None;

        if output_id == 0 {
            // UI Pass
            let raw_input = app.egui_state.take_egui_input(&window_context.window);

            let full_output = app.egui_context.run(raw_input, |ctx| {
                // SAFETY: We ensure window_context doesn't overlap with fields used in show.  
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

            // CRITICAL: Update GPU buffers before rendering - this prepares vertex/index buffers
            // and ensures all texture references are valid. Without this, egui-wgpu may panic
            // when looking up textures that haven't been properly registered yet.
            app.egui_renderer.update_buffers(
                &device,
                &app.backend.queue,
                &mut encoder,
                &tris,
                &screen_descriptor,
            );

            egui_render_data = Some((tris, screen_descriptor, full_output.textures_delta.free));
        }

        // --- Render Content ---
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

        app.backend.queue.submit(std::iter::once(encoder.finish()));
        window_context.window.pre_present_notify();
        surface_texture.present();

        if output_id != 0 {
            window_context.window.request_redraw();
        }
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
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Clear Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
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

    let output_config_opt = ctx.output_manager.get_output(output_id).cloned();
    let _use_edge_blend = output_config_opt.is_some() && ctx.edge_blend_renderer.is_some();     
    let _use_color_calib = output_config_opt.is_some() && ctx.color_calibration_renderer.is_some();

    let _mesh_target_view_ref = view;
    // Clear Pass
    {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Clear Output Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(if output_id == 0 {
                        wgpu::Color { r: 0.05, g: 0.05, b: 0.05, a: 1.0 }
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
                ctx.texture_pool.upload_data(queue, &grid_tex_name, &data, width, height);
            }
            Some(ctx.texture_pool.get_view(&grid_tex_name))
        } else if ctx.texture_pool.has_texture(&tex_name) {
            Some(ctx.texture_pool.get_view(&tex_name))
        } else {
            ctx.dummy_view.as_ref().map(|v| v.clone())
        };

        if let Some(src_ref) = source_view {
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
                    view,
                    resolve_target: None,
                    
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            mesh_renderer.draw(&mut rpass, vb, ib, cnt, &uniform_bind_group, &texture_bind_group, true);
        }
    }

    // EgUI Overlay
    if output_id == 0 {
        if let Some((tris, screen_desc, free_textures)) = egui_data {
            // Free textures from previous frames
            for id in free_textures {
                egui_renderer.free_texture(id);
            }

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // SAFETY: Hack to bypass lifetime issues in older egui-wgpu versions
            let render_pass_static: &mut wgpu::RenderPass<'static> = unsafe { std::mem::transmute(&mut render_pass) };
            egui_renderer.render(render_pass_static, tris, screen_desc);
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
            if let mapmap_core::module::ModulePartType::Output(mapmap_core::module::OutputType::Projector { id, .. }) = &part.part_type {
                Some((mid, *id, format!("output_{}", id)))
            } else {
                None
            }
        })
        .collect();

    for (_mid, output_id, _name) in module_output_infos {
        if let Some(texture_name) = app.output_assignments.get(&output_id).and_then(|v| v.last()).cloned() {
            if app.texture_pool.has_texture(&texture_name) {
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
                        size: wgpu::Extent3d { width: preview_width, height: preview_height, depth_or_array_layers: 1 },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: app.backend.surface_format(),
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
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
                            let target_view = target_tex.create_view(&wgpu::TextureViewDescriptor::default());
                            let target_view_arc = std::sync::Arc::new(target_view);
                            app.egui_renderer.update_egui_texture_from_wgpu_texture(&app.backend.device, &target_view_arc, wgpu::FilterMode::Linear, *id);
                            *e.get_mut() = (*id, target_view_arc.clone());
                            target_view_arc
                        } else {
                            old_view.clone()
                        }
                    }
                    Entry::Vacant(e) => {
                        let target_view = target_tex.create_view(&wgpu::TextureViewDescriptor::default());
                        let target_view_arc = std::sync::Arc::new(target_view);
                        let id = app.egui_renderer.register_native_texture(&app.backend.device, &target_view_arc, wgpu::FilterMode::Linear);
                        e.insert((id, target_view_arc.clone()));
                        target_view_arc
                    }
                };

                {
                    let transform = glam::Mat4::IDENTITY;
                    let uniform_bind_group = app.mesh_renderer.get_uniform_bind_group(&app.backend.queue, transform, 1.0);
                    let source_view = app.texture_pool.get_view(&texture_name);
                    let texture_bind_group = app.mesh_renderer.get_texture_bind_group(&source_view);

                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Preview Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &current_view_arc,
                            resolve_target: None,
                            
                            ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    app.mesh_renderer.draw(&mut render_pass, &app.preview_quad_buffers.0, &app.preview_quad_buffers.1, app.preview_quad_buffers.2, &uniform_bind_group, &texture_bind_group, false);
                }
            }
        }
    }
}

fn generate_grid_texture(width: u32, height: u32, layer_id: u64) -> Vec<u8> {
    let mut data = vec![0u8; (width * height * 4) as usize];
    let _bg_color = [0, 0, 0, 255]; 
    let _grid_color = [255, 255, 255, 255];
    let _text_color = [0, 255, 255, 255];

    for i in 0..(width * height) {
        let idx = (i * 4) as usize;
        data[idx] = 0; data[idx + 1] = 0; data[idx + 2] = 0; data[idx + 3] = 255;
    }
    let grid_step = 64;
    for y in 0..height {
        for x in 0..width {
            if x % grid_step == 0 || y % grid_step == 0 || x == width - 1 || y == height - 1 { 
                let idx = ((y * width + x) * 4) as usize;
                data[idx] = 255; data[idx + 1] = 255; data[idx + 2] = 255; data[idx + 3] = 255;
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
            draw_digit(&mut data, width, digit as usize, start_x + i as u32 * (digit_w + 2 * digit_scale), start_y, digit_scale, [0, 255, 255, 255]);
        }
    }
    data
}

const BITMAPS: [[u8; 5]; 10] = [
    [7, 5, 5, 5, 7], [2, 6, 2, 2, 7], [7, 1, 7, 4, 7], [7, 1, 7, 1, 7], [5, 5, 7, 1, 1],
    [7, 4, 7, 1, 7], [7, 4, 7, 5, 7], [7, 1, 1, 1, 1], [7, 5, 7, 5, 7], [7, 5, 7, 1, 7],
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
    if digit > 9 { return; }
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
                            data[idx] = color[0]; data[idx+1] = color[1]; data[idx+2] = color[2]; data[idx+3] = color[3];
                        }
                    }
                }
            }
        }
    }
}
