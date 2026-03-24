use crate::app::core::app_struct::RuntimeRenderQueueItem;
use anyhow::Result;

use super::effects::build_effect_chain;
use super::logging::{clear_video_issue, should_log_video_issue};
use super::PREVIEW_FLAG;

pub(crate) struct RenderContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub render_queue: &'a std::collections::HashMap<u64, Vec<RuntimeRenderQueueItem>>,
    pub output_manager: &'a vorce_core::output::OutputManager,
    pub edge_blend_renderer: &'a Option<vorce_render::EdgeBlendRenderer>,
    pub color_calibration_renderer: &'a Option<vorce_render::ColorCalibrationRenderer>,
    pub edge_blend_cache:
        &'a mut std::collections::HashMap<u64, (wgpu::Buffer, wgpu::BindGroup, u64)>,
    pub edge_blend_texture_cache: &'a mut std::collections::HashMap<u64, wgpu::BindGroup>,
    pub mesh_renderer: &'a mut vorce_render::MeshRenderer,
    pub effect_chain_renderer: &'a mut vorce_render::EffectChainRenderer,
    pub preview_effect_chain_renderer: &'a mut vorce_render::EffectChainRenderer,
    pub shader_graph_manager: &'a vorce_render::ShaderGraphManager,
    pub texture_pool: &'a vorce_render::TexturePool,
    pub _dummy_view: &'a Option<std::sync::Arc<wgpu::TextureView>>,
    pub mesh_buffer_cache: &'a mut vorce_render::MeshBufferCache,
    pub egui_renderer: &'a mut egui_wgpu::Renderer,
    pub video_diagnostic_log_times: &'a mut std::collections::HashMap<String, std::time::Instant>,
}

use super::texture_gen::{ensure_missing_texture_fallback, generate_grid_texture};

pub(crate) fn render_content(
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
    let video_log_times = ctx.video_diagnostic_log_times;
    let is_preview_output = (output_id & PREVIEW_FLAG) != 0;
    let real_output_id = output_id & !PREVIEW_FLAG;

    // ⚡ BOLT OPTIMIZATION:
    // Read pre-partitioned and sorted target_ops directly from the context.
    let empty_vec = Vec::new();
    let target_ops = ctx.render_queue.get(&real_output_id).unwrap_or(&empty_vec);

    for item in target_ops {
        for diag in &item.diagnostics {
            let issue_key = format!("{}:{}:{}", diag.code, diag.module_id, diag.part_id);
            if should_log_video_issue(video_log_times, issue_key) {
                match diag.severity {
                    crate::app::core::app_struct::DiagnosticSeverity::Warning => {
                        tracing::warn!(
                            "Fehler in Videoausgabe: Modul {} / Part {} - {}",
                            diag.module_id,
                            diag.part_id,
                            diag.message
                        );
                    }
                    crate::app::core::app_struct::DiagnosticSeverity::Error => {
                        tracing::error!(
                            "Fehler in Videoausgabe: Modul {} / Part {} - {}",
                            diag.module_id,
                            diag.part_id,
                            diag.message
                        );
                    }
                }
            }
        }
    }

    let empty_ops_issue_key = format!(
        "video-output-empty-ops:{real_output_id}:{}",
        if is_preview_output {
            "preview"
        } else {
            "output"
        }
    );
    if target_ops.is_empty() {
        if output_id != 0 && should_log_video_issue(video_log_times, empty_ops_issue_key.clone()) {
            tracing::warn!(
                "Fehler in Videoausgabe: {} {} bleibt leer, weil keine RenderOps fuer diesen Output erzeugt wurden.",
                if is_preview_output {
                    "Output-Preview"
                } else {
                    "Output"
                },
                real_output_id
            );
        }
    } else {
        clear_video_issue(video_log_times, empty_ops_issue_key);
    }

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
        .map(|cfg| {
            cfg.edge_blend.left.enabled
                || cfg.edge_blend.right.enabled
                || cfg.edge_blend.top.enabled
                || cfg.edge_blend.bottom.enabled
        })
        .unwrap_or(false)
        && ctx.edge_blend_renderer.is_some();
    // Currently we only support edge blending for post-processing safely.
    // Color calibration is temporarily ignored here to prevent black screen regressions.
    let _use_color_calib = output_config_opt.is_some() && ctx.color_calibration_renderer.is_some();

    let needs_post_processing = use_edge_blend;

    let intermediate_tex_name = format!("output_{}_intermediate", output_id);
    let mesh_target_view_ref = if needs_post_processing {
        if let Some(config) = &output_config_opt {
            ctx.texture_pool.ensure_texture(
                &intermediate_tex_name,
                config.resolution.0,
                config.resolution.1,
                wgpu::TextureFormat::Rgba8Unorm, // Always supported for RENDER_ATTACHMENT and TEXTURE_BINDING
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            );
        }
        Some(ctx.texture_pool.get_view(&intermediate_tex_name))
    } else {
        None
    };

    let target_view = if needs_post_processing {
        mesh_target_view_ref.as_deref().unwrap_or(view)
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
    for item in target_ops {
        let module_id = item.module_id;
        let op = &item.render_op;
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
            clear_video_issue(
                video_log_times,
                format!(
                    "video-output-missing-texture:{real_output_id}:{module_id}:{}",
                    op.source_part_id.unwrap_or_default()
                ),
            );
            Some(ctx.texture_pool.get_view(&tex_name))
        } else if ctx.texture_pool.has_texture("bevy_output") {
            // Fallback for Bevy nodes
            clear_video_issue(
                video_log_times,
                format!(
                    "video-output-missing-texture:{real_output_id}:{module_id}:{}",
                    op.source_part_id.unwrap_or_default()
                ),
            );
            Some(ctx.texture_pool.get_view("bevy_output"))
        } else {
            if let Some(source_part_id) = op.source_part_id {
                let issue_key = format!(
                    "video-output-missing-texture:{real_output_id}:{module_id}:{source_part_id}"
                );
                if should_log_video_issue(video_log_times, issue_key) {
                    tracing::warn!(
                        "Fehler in Videoausgabe: {} {} kann Modul {} / Part {} nicht rendern, weil die erwartete Textur '{}' im TexturePool fehlt.",
                        if is_preview_output { "Preview fuer Output" } else { "Output" },
                        real_output_id,
                        module_id,
                        source_part_id,
                        tex_name
                    );
                }
            }
            // BLACK FALLBACK for missing textures
            ensure_missing_texture_fallback(ctx.texture_pool, queue);
            Some(ctx.texture_pool.get_view("missing_texture_fallback"))
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

            let transform = glam::Mat4::from_scale_rotation_translation(
                glam::vec3(op.source_props.scale_x, op.source_props.scale_y, 1.0),
                glam::Quat::from_rotation_z(op.source_props.rotation.to_radians()),
                glam::vec3(op.source_props.offset_x, op.source_props.offset_y, 0.0),
            );
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
        let intermediate_view = if let Some(v) = mesh_target_view_ref.as_ref() {
            v
        } else {
            view
        };
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
                vorce_core::EdgeBlendConfig::default()
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
