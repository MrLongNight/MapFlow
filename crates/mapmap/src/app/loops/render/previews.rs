use super::content::{render_content, RenderContext};
use super::logging::{clear_video_issue, should_log_video_issue};
use super::PREVIEW_FLAG;
use crate::app::core::app_struct::App;

#[allow(clippy::manual_is_multiple_of)]
pub(crate) fn prepare_texture_previews(app: &mut App, encoder: &mut wgpu::CommandEncoder) {
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

        if let Err(err) = render_content(
            RenderContext {
                device: &app.backend.device,
                queue: &app.backend.queue,
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
                video_diagnostic_log_times: &mut app.video_diagnostic_log_times,
            },
            output_id | PREVIEW_FLAG,
            encoder,
            current_view_arc.as_ref(),
            None,
        ) {
            let issue_key = format!("output-preview-render-failed:{output_id}");
            if should_log_video_issue(&mut app.video_diagnostic_log_times, issue_key) {
                tracing::error!(
                    "Fehler in Videoausgabe: Output-Preview {} konnte nicht gerendert werden, weil {}.",
                    output_id,
                    err
                );
            }
        }
    }

    // --- NEW: Sync individual Node Previews for the Canvas ---
    if let Some(active_id) = app.ui_state.module_canvas.active_module_id {
        let part_descriptors = app
            .state
            .module_manager
            .get_module(active_id)
            .map(|module| {
                module
                    .parts
                    .iter()
                    .map(|part| {
                        let media_path = match &part.part_type {
                            mapmap_core::module::ModulePartType::Source(
                                mapmap_core::module::SourceType::MediaFile { path, .. }
                                | mapmap_core::module::SourceType::VideoUni { path, .. }
                                | mapmap_core::module::SourceType::ImageUni { path, .. },
                            ) if !path.trim().is_empty() => Some(path.clone()),
                            mapmap_core::module::ModulePartType::Source(
                                mapmap_core::module::SourceType::VideoMulti { shared_id, .. }
                                | mapmap_core::module::SourceType::ImageMulti { shared_id, .. },
                            ) => app
                                .state
                                .module_manager
                                .shared_media
                                .get(shared_id)
                                .map(|item| item.path.clone())
                                .filter(|path| !path.trim().is_empty()),
                            _ => None,
                        };

                        (part.id, media_path)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        for (part_id, media_path) in part_descriptors {
            let texture_name = format!("part_{}_{}", active_id, part_id);
            if app.texture_pool.has_texture(&texture_name) {
                clear_video_issue(
                    &mut app.video_diagnostic_log_times,
                    format!("node-preview-missing-player:{active_id}:{part_id}"),
                );
                clear_video_issue(
                    &mut app.video_diagnostic_log_times,
                    format!("node-preview-missing-texture:{active_id}:{part_id}"),
                );

                let view = app.texture_pool.get_view(&texture_name);

                use std::collections::hash_map::Entry;
                let tex_id = match app
                    .ui_state
                    .module_canvas
                    .node_previews
                    .entry((active_id, part_id))
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
                app.ui_state
                    .module_canvas
                    .node_previews
                    .insert((active_id, part_id), tex_id);
            } else {
                app.ui_state
                    .module_canvas
                    .node_previews
                    .remove(&(active_id, part_id));

                if let Some(path) = media_path {
                    if app.media_players.contains_key(&(active_id, part_id)) {
                        let issue_key =
                            format!("node-preview-missing-texture:{active_id}:{part_id}");
                        if should_log_video_issue(&mut app.video_diagnostic_log_times, issue_key) {
                            tracing::warn!(
                                "Fehler in Videoausgabe: Node-Vorschau fuer Modul {} / Part {} bleibt leer, weil fuer '{}' noch keine Textur '{}' vorliegt.",
                                active_id,
                                part_id,
                                path,
                                texture_name
                            );
                        }
                    } else {
                        let issue_key =
                            format!("node-preview-missing-player:{active_id}:{part_id}");
                        if should_log_video_issue(&mut app.video_diagnostic_log_times, issue_key) {
                            tracing::warn!(
                                "Fehler in Videoausgabe: Node-Vorschau fuer Modul {} / Part {} bleibt leer, weil kein MediaPlayer fuer '{}' aktiv ist.",
                                active_id,
                                part_id,
                                path
                            );
                        }
                    }
                }
            }
        }
    }
}
