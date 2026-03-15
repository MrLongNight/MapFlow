use crate::app::App;
use mapmap_media::LoopMode;
use mapmap_ui as ui;

const STARTUP_OVERLAY_DURATION_SECS: f32 = 4.0;

fn resolve_startup_animation_source(source_path: &str) -> Option<std::path::PathBuf> {
    let trimmed = source_path.trim();
    if trimmed.is_empty() {
        return None;
    }

    let direct_path = std::path::PathBuf::from(trimmed);
    if direct_path.exists() {
        return Some(direct_path);
    }

    if let Ok(relative_to_resources) = direct_path.strip_prefix("resources") {
        if let Some(resolved) =
            mapmap_core::runtime_paths::existing_resource_path(relative_to_resources)
        {
            return Some(resolved);
        }
    }

    mapmap_core::runtime_paths::existing_resource_path(trimmed)
}

fn load_startup_animation(app: &mut App, source_path: &str) {
    let resolved_path = resolve_startup_animation_source(source_path);
    let startup = &mut app.startup_animation;
    let needs_reload =
        startup.requested_path != source_path || startup.resolved_path != resolved_path;
    if !needs_reload {
        return;
    }

    startup.reset();
    startup.requested_path = source_path.to_string();
    startup.resolved_path = resolved_path.clone();

    let Some(path) = resolved_path else {
        startup.error = Some("Startup-Quelle fehlt".to_string());
        return;
    };

    match mapmap_media::open_path_with_hw_accel(&path, mapmap_media::HwAccelType::None) {
        Ok(mut player) => {
            let _ = player.set_loop_mode(LoopMode::PlayOnce);
            let _ = player.play();
            let _ = player.update(std::time::Duration::ZERO);
            startup.player = Some(player);
            startup.last_update = Some(std::time::Instant::now());
        }
        Err(err) => {
            startup.error = Some(format!("Startup-Video konnte nicht geladen werden: {err}"));
        }
    }
}

fn update_startup_animation_texture(
    ctx: &egui::Context,
    app: &mut App,
) -> Option<(egui::TextureId, egui::Vec2)> {
    let startup = &mut app.startup_animation;
    let now = std::time::Instant::now();
    let dt = startup
        .last_update
        .map(|last| now.saturating_duration_since(last))
        .unwrap_or_default();
    startup.last_update = Some(now);

    let player = startup.player.as_mut()?;
    let _ = player.update(dt);
    let frame = player.last_frame()?;
    let mapmap_io::format::FrameData::Cpu(data) = &frame.data else {
        startup.error = Some("Startup-Video lieferte keinen CPU-Frame".to_string());
        return None;
    };

    let width = frame.format.width as usize;
    let height = frame.format.height as usize;
    let image = egui::ColorImage::from_rgba_unmultiplied([width, height], data.as_slice());

    if let Some(texture) = startup.texture.as_mut() {
        texture.set(
            image,
            egui::TextureOptions {
                magnification: egui::TextureFilter::Linear,
                minification: egui::TextureFilter::Linear,
                wrap_mode: egui::TextureWrapMode::ClampToEdge,
                mipmap_mode: None,
            },
        );
        Some((texture.id(), egui::vec2(width as f32, height as f32)))
    } else {
        let texture = ctx.load_texture(
            "startup_animation_video",
            image,
            egui::TextureOptions {
                magnification: egui::TextureFilter::Linear,
                minification: egui::TextureFilter::Linear,
                wrap_mode: egui::TextureWrapMode::ClampToEdge,
                mipmap_mode: None,
            },
        );
        let id = texture.id();
        startup.texture = Some(texture);
        Some((id, egui::vec2(width as f32, height as f32)))
    }
}

fn fit_size_within(source_size: egui::Vec2, max_size: egui::Vec2) -> egui::Vec2 {
    if source_size.x <= 0.0 || source_size.y <= 0.0 || max_size.x <= 0.0 || max_size.y <= 0.0 {
        return max_size;
    }

    let scale = (max_size.x / source_size.x).min(max_size.y / source_size.y);
    source_size * scale
}

fn render_startup_animation_overlay(ctx: &egui::Context, app: &mut App) {
    if !app.ui_state.user_config.startup_animation_enabled
        || app.ui_state.user_config.reduce_motion_enabled
    {
        app.startup_animation.reset();
        return;
    }

    let elapsed = app.start_time.elapsed().as_secs_f32();
    if elapsed >= STARTUP_OVERLAY_DURATION_SECS {
        app.startup_animation.reset();
        return;
    }

    let t = elapsed / STARTUP_OVERLAY_DURATION_SECS;
    let fade_in = (t / 0.2).clamp(0.0, 1.0);
    let fade_out = ((1.0 - t) / 0.25).clamp(0.0, 1.0);
    let alpha = fade_in.min(fade_out);

    let source_path = app
        .ui_state
        .user_config
        .startup_animation_path
        .trim()
        .to_string();
    load_startup_animation(app, &source_path);

    let source_status = app
        .startup_animation
        .error
        .clone()
        .unwrap_or_else(|| "Startup-Video aktiv".to_string());
    let resolved_path = app
        .startup_animation
        .resolved_path
        .as_ref()
        .map(|path| path.display().to_string())
        .filter(|path| !path.is_empty())
        .unwrap_or_else(|| source_path.clone());
    let video_frame = update_startup_animation_texture(ctx, app);

    let backdrop = egui::Color32::from_black_alpha((190.0 * alpha) as u8);
    let frame_fill = egui::Color32::from_rgba_premultiplied(14, 18, 26, (230.0 * alpha) as u8);

    ctx.request_repaint();

    egui::Area::new("startup_animation_overlay".into())
        .order(egui::Order::Foreground)
        .fixed_pos(ctx.content_rect().min)
        .show(ctx, |ui| {
            let rect = ctx.content_rect();
            ui.painter().rect_filled(rect, 0.0, backdrop);

            if let Some((texture_id, source_size)) = &video_frame {
                let video_rect = egui::Rect::from_center_size(
                    rect.center(),
                    fit_size_within(*source_size, rect.size()),
                );
                ui.painter().image(
                    *texture_id,
                    video_rect,
                    egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE.gamma_multiply(alpha),
                );
            }

            ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    egui::Frame::default()
                        .fill(frame_fill)
                        .corner_radius(egui::CornerRadius::same(12))
                        .inner_margin(egui::Margin::symmetric(24, 18))
                        .stroke(egui::Stroke::new(
                            1.0,
                            egui::Color32::from_rgba_premultiplied(
                                111,
                                188,
                                255,
                                (180.0 * alpha) as u8,
                            ),
                        ))
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.heading("MapFlow");
                                ui.label("Startup Animation");
                                ui.add_space(4.0);
                                ui.label(source_status);
                                if !resolved_path.is_empty() {
                                    ui.label(egui::RichText::new(resolved_path).small().weak());
                                }
                                if app.ui_state.user_config.silent_startup_enabled {
                                    ui.label(egui::RichText::new("Silent Startup aktiv").small());
                                }
                                ui.add_space(8.0);
                                ui.add(
                                    egui::ProgressBar::new(t)
                                        .desired_width(280.0)
                                        .show_percentage(),
                                );
                            });
                        });
                });
            });
        });
}

/// Main UI orchestration function.
/// Renders the entire application UI layout using egui.
pub fn show(ctx: &egui::Context, app: &mut App) {
    app.ui_state.update_responsive_styles(ctx);

    let viewport_rect = ctx.content_rect();
    let viewport_width = viewport_rect.width();
    let viewport_height = viewport_rect.height();
    let compact_height = viewport_height < 760.0;

    let active_layout = app.ui_state.user_config.active_layout().cloned();
    let layout_sizes = active_layout
        .as_ref()
        .map(|layout| layout.panel_sizes)
        .unwrap_or_default();
    let layout_locked = active_layout
        .as_ref()
        .map(|layout| layout.lock_layout)
        .unwrap_or(false);

    let sidebar_default = if layout_sizes.left_sidebar_width > 0.0 {
        layout_sizes.left_sidebar_width
    } else {
        (viewport_width * 0.2).clamp(240.0, 420.0)
    }
    .clamp(220.0, (viewport_width * 0.45).max(340.0));

    let inspector_default = if layout_sizes.inspector_width > 0.0 {
        layout_sizes.inspector_width
    } else {
        (viewport_width * 0.24).clamp(260.0, 520.0)
    }
    .clamp(260.0, (viewport_width * 0.5).max(420.0));

    let timeline_default_height = if layout_sizes.timeline_height > 0.0 {
        layout_sizes.timeline_height
    } else if compact_height {
        (viewport_height * 0.22).clamp(90.0, 150.0)
    } else {
        (viewport_height * 0.26).clamp(140.0, 300.0)
    }
    .clamp(100.0, 500.0);

    // 1. Global Menu Bar (Top-most)
    let menu_actions = ui::view::menu_bar::show(ctx, &mut app.ui_state);
    for action in menu_actions {
        app.ui_state.actions.push(action);
    }

    // 2. Toolbar (Separate Panel below Menu)
    if app.ui_state.show_toolbar {
        egui::TopBottomPanel::top("toolbar_panel")
            .resizable(!layout_locked)
            .min_height(if compact_height { 36.0 } else { 44.0 })
            .frame(
                egui::Frame::default()
                    .fill(ctx.style().visuals.window_fill())
                    .inner_margin(egui::Margin::symmetric(16, 4))
                    .stroke(egui::Stroke::new(
                        1.0,
                        ctx.style().visuals.widgets.noninteractive.bg_stroke.color,
                    )),
            )
            .show(ctx, |ui_obj| {
                ui_obj.horizontal_wrapped(|ui_obj| {
                    if ui_obj
                        .small_button(if app.ui_state.show_left_sidebar {
                            "◀ Sidebar"
                        } else {
                            "▶ Sidebar"
                        })
                        .clicked()
                    {
                        app.ui_state.show_left_sidebar = !app.ui_state.show_left_sidebar;
                        app.ui_state.user_config.show_left_sidebar = app.ui_state.show_left_sidebar;
                        let _ = app.ui_state.user_config.save();
                    }
                    if ui_obj
                        .small_button(if app.ui_state.show_inspector {
                            "Inspector ▶"
                        } else {
                            "Inspector ◀"
                        })
                        .clicked()
                    {
                        app.ui_state.show_inspector = !app.ui_state.show_inspector;
                        app.ui_state.user_config.show_inspector = app.ui_state.show_inspector;
                        let _ = app.ui_state.user_config.save();
                    }
                    if ui_obj
                        .small_button(if app.ui_state.show_timeline {
                            "▼ Timeline"
                        } else {
                            "▲ Timeline"
                        })
                        .clicked()
                    {
                        app.ui_state.show_timeline = !app.ui_state.show_timeline;
                        app.ui_state.user_config.show_timeline = app.ui_state.show_timeline;
                        let _ = app.ui_state.user_config.save();
                    }
                    ui_obj.separator();
                    ui::view::menu_bar::toolbar::show(ui_obj, &mut app.ui_state);
                });
            });
    }

    // 3. Left Panel: Sidebar (Collapsible & Resizable)
    if app.ui_state.show_left_sidebar {
        egui::SidePanel::left("left_sidebar_panel")
            .resizable(!layout_locked)
            .default_width(sidebar_default)
            .min_width(if compact_height { 180.0 } else { 220.0 })
            .max_width((viewport_width * 0.45).max(340.0))
            .show(ctx, |ui_obj| {
                egui::TopBottomPanel::bottom("left_sidebar_preview")
                    .resizable(true)
                    .default_height(if compact_height { 120.0 } else { 180.0 })
                    .min_height(110.0)
                    .show_inside(ui_obj, |ui_obj| {
                        ui_obj.horizontal(|ui| {
                            ui.heading(app.ui_state.i18n.t("preview"));
                        });
                        use mapmap_core::module::{ModulePartType, OutputType};
                        let preview_outputs = app
                            .state
                            .module_manager
                            .modules()
                            .iter()
                            .flat_map(|m| m.parts.iter())
                            .filter_map(|part| {
                                if let ModulePartType::Output(OutputType::Projector {
                                    id,
                                    name,
                                    show_in_preview_panel,
                                    ..
                                }) = &part.part_type
                                {
                                    Some(ui::OutputPreviewInfo {
                                        id: *id,
                                        name: name.clone(),
                                        show_in_panel: *show_in_preview_panel,
                                        texture_name: None,
                                        texture_id: app
                                            .output_preview_cache
                                            .get(id)
                                            .map(|(texture_id, _)| *texture_id),
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();

                        app.ui_state.module_canvas.output_previews = preview_outputs
                            .iter()
                            .filter_map(|output| {
                                output.texture_id.map(|texture_id| (output.id, texture_id))
                            })
                            .collect();
                        if app.ui_state.show_preview_panel {
                            app.ui_state.preview_panel.update_outputs(preview_outputs);
                            app.ui_state.preview_panel.show(ui_obj);
                        }
                    });

                egui::TopBottomPanel::bottom("left_sidebar_media")
                    .resizable(true)
                    .default_height(if compact_height { 160.0 } else { 240.0 })
                    .min_height(120.0)
                    .show_inside(ui_obj, |ui_obj| {
                        egui::CollapsingHeader::new(app.ui_state.i18n.t("media"))
                            .default_open(true)
                            .show(ui_obj, |ui| {
                                if app.ui_state.show_media_browser {
                                    let _ = app.ui_state.media_browser.ui(
                                        ui,
                                        &app.ui_state.i18n,
                                        app.ui_state.icon_manager.as_ref(),
                                    );
                                } else {
                                    ui.label(app.ui_state.i18n.t("media-sidebar-placeholder"));
                                }
                            });
                    });

                egui::ScrollArea::vertical().show(ui_obj, |ui_obj| {
                    egui::CollapsingHeader::new(app.ui_state.i18n.t("dashboard"))
                        .default_open(true)
                        .show(ui_obj, |ui| {
                            if let Some(dash_action) = app.ui_state.dashboard.render_contents(
                                ui,
                                &app.ui_state.i18n,
                                app.ui_state.icon_manager.as_ref(),
                            ) {
                                match dash_action {
                                    ui::view::dashboard::DashboardAction::SendCommand(cmd) => {
                                        if let Some(_module_id) = app.ui_state.module_canvas.active_module_id {
                                            if let Some(part_id) = app.ui_state.module_canvas.get_selected_part_id() {
                                                app.ui_state.actions.push(ui::UIAction::MediaCommand(part_id, cmd));
                                            }
                                        }
                                    }
                                    ui::view::dashboard::DashboardAction::ToggleAudioPanel => {
                                        app.ui_state.show_audio = !app.ui_state.show_audio;
                                    }
                                    _ => {}
                                }
                            }
                        });
                    ui_obj.separator();

                    if app.ui_state.show_master_controls {
                        egui::CollapsingHeader::new(app.ui_state.i18n.t("panel-master"))
                            .default_open(true)
                            .show(ui_obj, |ui| {
                                let mut layer_manager = std::sync::Arc::make_mut(&mut app.state.layer_manager).clone();
                                app.ui_state.render_master_controls_embedded(ui, &mut layer_manager);
                                if layer_manager != *app.state.layer_manager {
                                    *std::sync::Arc::make_mut(&mut app.state.layer_manager) = layer_manager;
                                    app.state.dirty = true;
                                }
                            });
                        ui_obj.separator();
                    }

                    egui::CollapsingHeader::new(app.ui_state.i18n.t("audio"))
                        .default_open(app.ui_state.show_audio)
                        .show(ui_obj, |ui| {
                            let analysis = app.audio_analyzer.get_latest_analysis();
                            if let Some(audio_action) = app.ui_state.audio_panel.ui(
                                ui,
                                &app.ui_state.i18n,
                                Some(&analysis),
                                &app.state.audio_config,
                                app.ui_state.user_config.meter_style,
                                &mut app.ui_state.show_audio_panel_meters,
                                &mut app.ui_state.audio_fft_mode,
                            ) {
                                match audio_action {
                                    ui::panels::audio_panel::AudioPanelAction::ConfigChanged(cfg) => {
                                        app.ui_state.actions.push(ui::UIAction::UpdateAudioConfig(cfg));
                                    }
                                    ui::panels::audio_panel::AudioPanelAction::MeterStyleChanged(style) => {
                                        app.ui_state.actions.push(ui::UIAction::SetMeterStyle(style));
                                    }
                                }
                            }
                        });
                });
            });
    }

    // 4. Right Panel: Inspector (Docked & Resizable)
    if app.ui_state.show_inspector {
        egui::SidePanel::right("right_panel")
            .resizable(!layout_locked)
            .default_width(inspector_default)
            .min_width(if compact_height { 220.0 } else { 260.0 })
            .max_width((viewport_width * 0.5).max(420.0))
            .show(ctx, |ui_obj| {
                ui_obj.horizontal(|ui| {
                    ui.heading(app.ui_state.i18n.t("inspector"));
                    if ui
                        .small_button("✕")
                        .on_hover_text("Inspector ausblenden")
                        .clicked()
                    {
                        app.ui_state.show_inspector = false;
                        app.ui_state.user_config.show_inspector = false;
                        let _ = app.ui_state.user_config.save();
                    }
                });

                ui_obj.separator();

                egui::ScrollArea::vertical().show(ui_obj, |ui_obj| {
                    // Render the unified Inspector
                    app.ui_state.render_inspector(
                        ui_obj,
                        std::sync::Arc::make_mut(&mut app.state.module_manager),
                        &app.state.layer_manager,
                        &app.state.output_manager,
                        &app.state.mapping_manager,
                    );

                    // Legacy panels (can be toggled separately or integrated)
                    if app.ui_state.show_transforms {
                        app.ui_state.transform_panel.render(ctx, &app.ui_state.i18n);
                    }

                    // Effect chain integrated into inspector side
                    egui::TopBottomPanel::bottom("inspector_effect_chain_split")
                        .resizable(true)
                        .default_height(if compact_height { 180.0 } else { 240.0 })
                        .min_height(120.0)
                        .show_inside(ui_obj, |_ui| {
                            app.ui_state.effect_chain_panel.ui(
                                ctx,
                                &app.ui_state.i18n,
                                app.ui_state.icon_manager.as_ref(),
                                Some(&mut app.recent_effect_configs),
                            );
                        });
                });
            });
    }

    // 5. Bottom Panel: Timeline (Resizable)
    if app.ui_state.show_timeline {
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(!layout_locked)
            .default_height(timeline_default_height)
            .min_height(if compact_height { 80.0 } else { 100.0 })
            .show(ctx, |ui_obj| {
                ui_obj.horizontal(|ui| {
                    ui.heading(app.ui_state.i18n.t("timeline"));
                    if ui
                        .small_button("✕")
                        .on_hover_text("Timeline ausblenden")
                        .clicked()
                    {
                        app.ui_state.show_timeline = false;
                        app.ui_state.user_config.show_timeline = false;
                        let _ = app.ui_state.user_config.save();
                    }
                });

                let state = &mut app.state;
                let animator = std::sync::Arc::make_mut(&mut state.effect_animator);
                let mut modules: Vec<ui::TimelineModule> = state
                    .module_manager
                    .modules()
                    .iter()
                    .map(|m| ui::TimelineModule {
                        id: m.id,
                        // Optimization: Borrow name string to prevent allocation overhead in UI hot loop.
                        name: &m.name,
                    })
                    .collect();
                modules.sort_by_key(|m| m.id);

                if let Some(action) = app.ui_state.timeline_panel.ui(ui_obj, animator, &modules) {
                    app.ui_state
                        .actions
                        .push(ui::UIAction::TimelineAction(action));
                }
            });
    }

    // 6. Floating Windows / Overlays

    // Cue Panel
    if app.ui_state.show_cue_panel {
        app.ui_state.cue_panel.show(
            ctx,
            &app.control_manager,
            &app.ui_state.i18n,
            &mut app.ui_state.actions,
            app.ui_state.icon_manager.as_ref(),
        );
    }

    // 7. Central Panel: Module Canvas
    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(ctx.style().visuals.panel_fill))
        .show(ctx, |ui_obj| {
            if app.ui_state.show_module_canvas {
                app.ui_state.module_canvas.ensure_icons_loaded(ctx);

                if app.ui_state.module_canvas.active_module_id.is_none() {
                    if let Some(first_mod) = app.state.module_manager.modules().first() {
                        app.ui_state.module_canvas.active_module_id = Some(first_mod.id);
                    }
                }

                // --- Module Selector Toolbar ---
                egui::MenuBar::new().ui(ui_obj, |ui_obj| {
                    let modules: Vec<(u64, String, [f32; 4])> = app
                        .state
                        .module_manager
                        .modules()
                        .iter()
                        .map(|m| (m.id, m.name.clone(), m.color))
                        .collect();

                    if !modules.is_empty() {
                        let active_name = app
                            .ui_state
                            .module_canvas
                            .active_module_id
                            .and_then(|id| modules.iter().find(|(mid, _, _)| *mid == id))
                            .map(|(_, name, _)| name.clone())
                            .unwrap_or_else(|| "Module wählen...".to_string());

                        egui::ComboBox::from_id_salt("module_selector")
                            .selected_text(format!("📦 {}", active_name))
                            .show_ui(ui_obj, |ui_obj| {
                                for (id, name, color) in &modules {
                                    let color32 = egui::Color32::from_rgba_premultiplied(
                                        (color[0] * 255.0) as u8,
                                        (color[1] * 255.0) as u8,
                                        (color[2] * 255.0) as u8,
                                        255,
                                    );
                                    let is_selected =
                                        app.ui_state.module_canvas.active_module_id == Some(*id);
                                    let label =
                                        egui::RichText::new(format!("● {}", name)).color(color32);
                                    if ui_obj.selectable_label(is_selected, label).clicked() {
                                        app.ui_state.module_canvas.set_active_module(Some(*id));
                                    }
                                }
                            });
                        ui_obj.separator();
                    }

                    if let Some(module_id) = app.ui_state.module_canvas.active_module_id {
                        ui_obj.menu_button(
                            egui::RichText::new("➕ Hinzufügen").strong(),
                            |ui_obj| {
                                ui::editors::module_canvas::draw::render_add_node_menu_content(
                                    ui_obj,
                                    std::sync::Arc::make_mut(&mut app.state.module_manager),
                                    None,
                                    Some(module_id),
                                );
                            },
                        );
                        ui_obj.separator();
                    }

                    if ui_obj.button("💾 Speichern").clicked() {
                        app.ui_state.module_canvas.show_presets = true;
                    }
                    if ui_obj.button("🔍 Suchen").clicked() {
                        app.ui_state.module_canvas.show_search =
                            !app.ui_state.module_canvas.show_search;
                    }

                    if ui_obj
                        .button(egui::RichText::new("➕ Neues Modul").strong())
                        .clicked()
                    {
                        let new_id = std::sync::Arc::make_mut(&mut app.state.module_manager)
                            .create_module("New Module".to_string());
                        app.ui_state.module_canvas.set_active_module(Some(new_id));
                    }

                    ui_obj.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui_obj| {
                            if ui_obj.button("Zentrieren").clicked() {
                                app.ui_state.module_canvas.pan_offset = egui::Vec2::ZERO;
                                app.ui_state.module_canvas.zoom = 1.0;
                            }
                            ui_obj.label(format!("Zoom: {:.1}x", app.ui_state.module_canvas.zoom));
                        },
                    );
                });

                app.ui_state.module_canvas.show(
                    ui_obj,
                    std::sync::Arc::make_mut(&mut app.state.module_manager),
                    &app.ui_state.i18n,
                    &mut app.ui_state.actions,
                    ui::ModuleCanvasRenderOptions::from(&app.ui_state.user_config),
                );
            } else {
                ui_obj.centered_and_justified(|ui_obj| {
                    ui_obj.label("Canvas - Module Canvas deaktiviert (View → Module Canvas)");
                });
            }
        });

    // 8. Overlays (Shader Graph, Audio, MIDI, Startup)
    render_startup_animation_overlay(ctx, app);

    crate::ui::panels::output::show(
        ctx,
        crate::ui::panels::output::OutputContext {
            ui_state: &mut app.ui_state,
            state: &mut app.state,
        },
    );

    crate::ui::panels::edge_blend::show(
        ctx,
        crate::ui::panels::edge_blend::EdgeBlendContext {
            ui_state: &mut app.ui_state,
        },
    );

    crate::ui::panels::mapping::show(
        ctx,
        crate::ui::panels::mapping::MappingContext {
            ui_state: &mut app.ui_state,
            state: &mut app.state,
        },
    );

    crate::ui::panels::paint::show(
        ctx,
        crate::ui::panels::paint::PaintContext {
            ui_state: &mut app.ui_state,
            state: &mut app.state,
        },
    );

    app.ui_state.render_controls(ctx);

    mapmap_ui::panels::osc_panel::show_osc_panel(ctx, &mut app.ui_state, &mut app.control_manager);

    app.ui_state.oscillator_panel.render(
        ctx,
        &app.ui_state.i18n,
        &mut app.state.oscillator_config,
        app.ui_state.icon_manager.as_ref(),
    );

    let mut actions = vec![];
    let mut selected_layer = app.ui_state.selected_layer_id;
    app.ui_state.layer_panel.show(
        ctx,
        std::sync::Arc::make_mut(&mut app.state.layer_manager),
        &mut selected_layer,
        &mut actions,
        &app.ui_state.i18n,
        app.ui_state.icon_manager.as_ref(),
    );
    app.ui_state.selected_layer_id = selected_layer;
    app.ui_state.actions.extend(actions);

    if app.ui_state.show_shader_graph {
        app.ui_state.render_node_editor(ctx);
    }

    app.ui_state.controller_overlay.show(
        ctx,
        app.ui_state.show_controller_overlay,
        false,
        &mut app.ui_state.user_config,
    );

    if app.ui_state.show_about {
        crate::ui::dialogs::about::show(ctx, &mut app.ui_state.show_about);
    }

    if app.ui_state.show_settings {
        let context = crate::ui::dialogs::settings::SettingsContext {
            ui_state: &mut app.ui_state,
            state: &mut app.state,
            backend: &app.backend,
            hue_controller: &mut app.hue_controller,
            #[cfg(feature = "midi")]
            midi_handler: &mut app.midi_handler,
            #[cfg(feature = "midi")]
            midi_ports: &mut app.midi_ports,
            #[cfg(feature = "midi")]
            selected_midi_port: &mut app.selected_midi_port,
            restart_requested: &mut app.restart_requested,
            exit_requested: &mut app.exit_requested,
            tokio_runtime: &app.tokio_runtime,
        };
        crate::ui::dialogs::settings::show(ctx, context);
    }

    app.ui_state
        .assignment_panel
        .show(ctx, &app.state.assignment_manager);
    app.ui_state.shortcut_editor.show(ctx, &app.ui_state.i18n);
}
