use crate::app::core::app_struct::App;
use crate::ui;
use mapmap_core::audio::backend::cpal_backend::CpalBackend;
use mapmap_core::audio::{analyzer_v2::AudioAnalyzerV2Config, backend::AudioBackend};
use mapmap_core::effects::EffectType as RenderEffectType;
use mapmap_ui::effect_chain_panel::{EffectChainAction, EffectType as UIEffectType};
use mapmap_ui::menu_bar;
use mapmap_ui::responsive::ResponsiveLayout;
use tracing::{error, info};

/// Renders the implementation of the UI.
pub fn show(app: &mut App, ctx: &egui::Context) {
    // 1. GLOBAL THEME & SETUP
    // Update Responsive Styles (every 500ms)
    app.ui_state.update_responsive_styles(ctx);

    // Initialize Responsive Layout
    let layout = ResponsiveLayout::new(ctx);

    app.ui_state.user_config.theme.apply(ctx);

    // Update performance and audio values
    app.ui_state.current_fps = app.current_fps;
    app.ui_state.current_frame_time_ms = app.current_frame_time_ms;
    app.ui_state.target_fps = app.ui_state.user_config.target_fps.unwrap_or(60.0);

    // Refresh system info every 500ms
    if app.last_sysinfo_refresh.elapsed().as_millis() > 500 {
        app.sys_info.refresh_cpu_usage();
        app.sys_info.refresh_memory();
        app.last_sysinfo_refresh = std::time::Instant::now();
    }

    let cpu_count = app.sys_info.cpus().len() as f32;
    app.ui_state.cpu_usage = if cpu_count > 0.0 {
        app.sys_info
            .cpus()
            .iter()
            .map(|c| c.cpu_usage())
            .sum::<f32>()
            / cpu_count
    } else {
        0.0
    };

    if let Ok(pid) = sysinfo::get_current_pid() {
        app.ui_state.ram_usage_mb = app
            .sys_info
            .process(pid)
            .map(|p| p.memory() as f32 / 1024.0 / 1024.0)
            .unwrap_or(0.0);
    }

    let fps_ratio = (app.current_fps / app.ui_state.target_fps).clamp(0.0, 1.0);
    app.ui_state.gpu_usage = ((1.0 - fps_ratio) * 100.0 * 1.2).clamp(0.0, 100.0);

    let audio_analysis = app.audio_analyzer.get_latest_analysis();
    app.ui_state.current_audio_level = audio_analysis.rms_volume;

    // MIDI Controller Overlay (Draws on top of everything essentially, but logically here is fine)
    #[cfg(feature = "midi")]
    {
        let midi_connected = app
            .midi_handler
            .as_ref()
            .map(|h| h.is_connected())
            .unwrap_or(false);
        app.ui_state.controller_overlay.show(
            ctx,
            app.ui_state.show_controller_overlay,
            midi_connected,
            &mut app.ui_state.user_config,
        );
    }

    // 2. DOCKED PANELS (Must be rendered BEFORE CentralPanel and Windows)

    // === Top Panel: Menu Bar + Toolbar ===
    let menu_actions = menu_bar::show(ctx, &mut app.ui_state);
    app.ui_state.actions.extend(menu_actions);

    // === Left Panel: Unified Sidebar ===
    if app.ui_state.show_left_sidebar {
        let sidebar_width = layout.sidebar_width();
        let sidebar_max = layout.sidebar_max_width();

        egui::SidePanel::left("unified_left_sidebar")
            .resizable(true)
            .default_width(sidebar_width)
            .min_width(150.0)
            .max_width(sidebar_max)
            .show(ctx, |ui| {
                if layout.is_compact() {
                    render_compact_sidebar(ui, app, &layout);
                } else {
                    render_full_sidebar(ui, app);
                }
            });
    } else {
        // Collapsed sidebar - just show expand button
        egui::SidePanel::left("left_sidebar_collapsed")
            .exact_width(28.0)
            .resizable(false)
            .show(ctx, |ui| {
                if ui.button("▶").on_hover_text("Sidebar ausklappen").clicked() {
                    app.ui_state.show_left_sidebar = true;
                }
            });
    }

    // === RIGHT PANEL: Inspector ===
    let state = &mut app.state;
    app.ui_state.render_inspector(
        ctx,
        std::sync::Arc::make_mut(&mut state.module_manager),
        &state.layer_manager,
        &state.output_manager,
    );

    // === 2. BOTTOM PANEL: Timeline (FULL WIDTH - rendered before side panels!) ===
    // Timeline
    ui::editors::timeline::show(
        ctx,
        ui::editors::timeline::TimelineContext {
            ui_state: &mut app.ui_state,
            state: &mut app.state,
        },
    );

    // === 5. CENTRAL PANEL: Module Canvas ===
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(ctx.style().visuals.panel_fill))
        .show(ctx, |ui| {
            if app.ui_state.show_module_canvas {
                ui::editors::module_canvas::show(
                    ui,
                    ui::editors::module_canvas::ModuleCanvasContext {
                        ui_state: &mut app.ui_state,
                        state: &mut app.state,
                    },
                );
            } else {
                // Placeholder for normal canvas/viewport
                ui.centered_and_justified(|ui| {
                    ui.label("Canvas - Module Canvas deaktiviert (View → Module Canvas)");
                });
            }
        });

    // Node Editor
    ui::editors::node_editor::show(
        ctx,
        ui::editors::node_editor::NodeEditorContext {
            ui_state: &mut app.ui_state,
        },
    );

    // Media Manager
    ui::view::media_manager::show(
        ctx,
        ui::view::media_manager::MediaManagerContext {
            ui: &mut app.media_manager_ui,
            library: &mut app.media_library,
        },
    );

    // === Settings Window (only modal allowed) ===
    #[cfg(feature = "midi")]
    ui::settings::show(
        ctx,
        ui::settings::SettingsContext {
            ui_state: &mut app.ui_state,
            state: &mut app.state,
            backend: &app.backend,
            hue_controller: &mut app.hue_controller,
            midi_handler: &mut app.midi_handler,
            midi_ports: &mut app.midi_ports,
            selected_midi_port: &mut app.selected_midi_port,
            restart_requested: &mut app.restart_requested,
            exit_requested: &mut app.exit_requested,
            tokio_runtime: &app.tokio_runtime,
        },
    );

    #[cfg(not(feature = "midi"))]
    ui::settings::show(
        ctx,
        ui::settings::SettingsContext {
            ui_state: &mut app.ui_state,
            state: &mut app.state,
            backend: &app.backend,
            hue_controller: &mut app.hue_controller,
            restart_requested: &mut app.restart_requested,
            exit_requested: &mut app.exit_requested,
            tokio_runtime: &app.tokio_runtime,
        },
    );

    // === 7. Floating Windows / Modals ===

    // Icon Demo Panel
    ui::dialogs::icon_demo::show(
        ctx,
        ui::dialogs::icon_demo::IconDemoContext {
            ui_state: &mut app.ui_state,
        },
    );

    // Paint Panel
    ui::panels::paint::show(
        ctx,
        ui::panels::paint::PaintContext {
            ui_state: &mut app.ui_state,
            state: &mut app.state,
        },
    );

    // Mapping Panel
    ui::panels::mapping::show(
        ctx,
        ui::panels::mapping::MappingContext {
            ui_state: &mut app.ui_state,
            state: &mut app.state,
        },
    );

    // Output Panel
    ui::panels::output::show(
        ctx,
        ui::panels::output::OutputContext {
            ui_state: &mut app.ui_state,
            state: &mut app.state,
        },
    );

    // Edge Blend Panel
    ui::panels::edge_blend::show(
        ctx,
        ui::panels::edge_blend::EdgeBlendContext {
            ui_state: &mut app.ui_state,
        },
    );

    // Assignment Panel
    ui::panels::assignment::show(
        ctx,
        ui::panels::assignment::AssignmentContext {
            ui_state: &mut app.ui_state,
            state: &mut app.state,
        },
    );

    // ---------------------------------------------------------------------
    // 3. FLOATING WINDOWS (Rendered LAST = On Top)
    // ---------------------------------------------------------------------

    // === Effect Chain Panel ===
    app.ui_state.effect_chain_panel.ui(
        ctx,
        &app.ui_state.i18n,
        app.ui_state.icon_manager.as_ref(),
        Some(&mut app.recent_effect_configs),
    );

    // Render Oscillator Panel
    app.ui_state.oscillator_panel.render(
        ctx, 
        &app.ui_state.i18n, 
        &mut app.state.oscillator_config, 
        app.ui_state.icon_manager.as_ref()
    );

    // Handle Effect Chain Actions
    for action in app.ui_state.effect_chain_panel.take_actions() {
        match action {
            EffectChainAction::AddEffectWithParams(ui_type, params) => {
                let render_type = match ui_type {
                    UIEffectType::Blur => RenderEffectType::Blur,
                    UIEffectType::ColorAdjust => RenderEffectType::ColorAdjust,
                    UIEffectType::ChromaticAberration => RenderEffectType::ChromaticAberration,
                    UIEffectType::EdgeDetect => RenderEffectType::EdgeDetect,
                    UIEffectType::Glow => RenderEffectType::Glow,
                    UIEffectType::Kaleidoscope => RenderEffectType::Kaleidoscope,
                    UIEffectType::Invert => RenderEffectType::Invert,
                    UIEffectType::Pixelate => RenderEffectType::Pixelate,
                    UIEffectType::Vignette => RenderEffectType::Vignette,
                    UIEffectType::FilmGrain => RenderEffectType::FilmGrain,
                    UIEffectType::Wave => RenderEffectType::Wave,
                    UIEffectType::Glitch => RenderEffectType::Glitch,
                    UIEffectType::RgbSplit => RenderEffectType::RgbSplit,
                    UIEffectType::Mirror => RenderEffectType::Mirror,
                    UIEffectType::HueShift => RenderEffectType::HueShift,
                    UIEffectType::Voronoi => RenderEffectType::Voronoi,
                    UIEffectType::Tunnel => RenderEffectType::Tunnel,
                    UIEffectType::Galaxy => RenderEffectType::Galaxy,
                    UIEffectType::Custom => RenderEffectType::Custom,
                };

                let id = app.state.effect_chain_mut().add_effect(render_type);
                if let Some(effect) = app.state.effect_chain_mut().get_effect_mut(id) {
                    for (k, v) in &params {
                        effect.set_param(k, *v);
                    }
                }

                app.recent_effect_configs
                    .add_float_config(&format!("{:?}", ui_type), params);
            }
            EffectChainAction::AddEffect(ui_type) => {
                let render_type = match ui_type {
                    UIEffectType::Blur => RenderEffectType::Blur,
                    UIEffectType::ColorAdjust => RenderEffectType::ColorAdjust,
                    UIEffectType::ChromaticAberration => RenderEffectType::ChromaticAberration,
                    UIEffectType::EdgeDetect => RenderEffectType::EdgeDetect,
                    UIEffectType::Glow => RenderEffectType::Glow,
                    UIEffectType::Kaleidoscope => RenderEffectType::Kaleidoscope,
                    UIEffectType::Invert => RenderEffectType::Invert,
                    UIEffectType::Pixelate => RenderEffectType::Pixelate,
                    UIEffectType::Vignette => RenderEffectType::Vignette,
                    UIEffectType::FilmGrain => RenderEffectType::FilmGrain,
                    UIEffectType::Wave => RenderEffectType::Wave,
                    UIEffectType::Glitch => RenderEffectType::Glitch,
                    UIEffectType::RgbSplit => RenderEffectType::RgbSplit,
                    UIEffectType::Mirror => RenderEffectType::Mirror,
                    UIEffectType::HueShift => RenderEffectType::HueShift,
                    UIEffectType::Voronoi => RenderEffectType::Voronoi,
                    UIEffectType::Tunnel => RenderEffectType::Tunnel,
                    UIEffectType::Galaxy => RenderEffectType::Galaxy,
                    UIEffectType::Custom => RenderEffectType::Custom,
                };
                app.state.effect_chain_mut().add_effect(render_type);
            }
            EffectChainAction::ClearAll => {
                app.state.effect_chain_mut().effects.clear();
            }
            EffectChainAction::RemoveEffect(id) => {
                app.state.effect_chain_mut().remove_effect(id);
            }
            EffectChainAction::MoveUp(id) => {
                app.state.effect_chain_mut().move_up(id);
            }
            EffectChainAction::MoveDown(id) => {
                app.state.effect_chain_mut().move_down(id);
            }
            EffectChainAction::ToggleEnabled(id) => {
                if let Some(effect) = app.state.effect_chain_mut().get_effect_mut(id) {
                    effect.enabled = !effect.enabled;
                }
            }
            EffectChainAction::SetIntensity(id, val) => {
                if let Some(effect) = app.state.effect_chain_mut().get_effect_mut(id) {
                    effect.intensity = val;
                }
            }
            EffectChainAction::SetParameter(id, name, val) => {
                if let Some(effect) = app.state.effect_chain_mut().get_effect_mut(id) {
                    effect.set_param(&name, val);
                }
            }
            _ => {}
        }
    }

    // Handle TransformPanel actions
    if let Some(action) = app.ui_state.transform_panel.take_action() {
        if let Some(selected_id) = app.ui_state.selected_layer_id {
            match action {
                mapmap_ui::TransformAction::UpdateTransform(values) => {
                    if let Some(layer) = app.state.layer_manager_mut().get_layer_mut(selected_id) {
                        layer.transform.position.x = values.position.0;
                        layer.transform.position.y = values.position.1;
                        layer.transform.rotation.z = values.rotation.to_radians();
                        layer.transform.scale.x = values.scale.0;
                        layer.transform.scale.y = values.scale.1;
                        layer.transform.anchor.x = values.anchor.0;
                        layer.transform.anchor.y = values.anchor.1;
                        app.state.dirty = true;
                    }
                }
                mapmap_ui::TransformAction::ResetTransform => {
                    if let Some(layer) = app.state.layer_manager_mut().get_layer_mut(selected_id) {
                        layer.transform = mapmap_core::Transform::default();
                        app.state.dirty = true;
                    }
                }
                mapmap_ui::TransformAction::ApplyResizeMode(mode) => {
                    app.ui_state
                        .actions
                        .push(mapmap_ui::UIAction::ApplyResizeMode(selected_id, mode));
                }
            }
        }
    }
}

/// Renders compact sidebar with tabs (for small screens)
fn render_compact_sidebar(ui: &mut egui::Ui, app: &mut App, layout: &ResponsiveLayout) {
    // Tab Bar
    egui::TopBottomPanel::top("sidebar_tabs")
        .show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut app.ui_state.active_sidebar_tab,
                    0,
                    "🎛️ Controls"
                );
                ui.selectable_value(
                    &mut app.ui_state.active_sidebar_tab,
                    1,
                    "👁 Preview"
                );
            });
        });

    ui.separator();

    // Content based on active tab
    egui::CentralPanel::default()
        .show_inside(ui, |ui| {
            match app.ui_state.active_sidebar_tab {
                0 => render_controls_section(ui, app, layout),
                1 => render_preview_section(ui, app, layout),
                _ => {}
            }
        });
}

/// Renders full sidebar (for large screens)
fn render_full_sidebar(ui: &mut egui::Ui, app: &mut App) {
    let layout = ResponsiveLayout::new(ui.ctx());
    
    // Header with collapse button
    ui.horizontal(|ui| {
        ui.heading("Sidebar");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("◀").on_hover_text("Sidebar einklappen").clicked() {
                app.ui_state.show_left_sidebar = false;
            }
        });
    });
    ui.separator();

    // Controls Section
    if app.ui_state.show_control_panel {
        let use_fixed_height = app.ui_state.show_preview_panel;

        if use_fixed_height {
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), app.ui_state.control_panel_height),
                egui::Layout::top_down(egui::Align::LEFT),
                |ui| {
                    render_controls_section(ui, app, &layout);
                },
            );

            // Splitter logic
            let splitter_height = 6.0;
            let (_, splitter_response) = ui.allocate_at_least(
                egui::vec2(ui.available_width(), splitter_height),
                egui::Sense::drag(),
            );
            
            let is_hovered = splitter_response.hovered();
            let is_dragged = splitter_response.dragged();
            let color = if is_dragged {
                ui.visuals().widgets.active.bg_fill
            } else if is_hovered {
                ui.visuals().widgets.hovered.bg_fill
            } else {
                ui.visuals().widgets.noninteractive.bg_fill
            };

            ui.painter().hline(
                splitter_response.rect.left()..=splitter_response.rect.right(),
                splitter_response.rect.center().y,
                (2.0, color),
            );
            
            if splitter_response.dragged() {
                app.ui_state.control_panel_height += splitter_response.drag_delta().y;
                let total_available = ui.available_height();
                app.ui_state.control_panel_height = app.ui_state.control_panel_height.clamp(100.0, total_available - 50.0);
            }
            
            if is_hovered || is_dragged {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
            }
        } else {
            render_controls_section(ui, app, &layout);
        }
    }

    // Preview Section
    if app.ui_state.show_preview_panel {
        ui.separator();
        ui.horizontal(|ui| {
            let arrow = if app.ui_state.show_preview_panel { "▼" } else { "▶" };
            if ui.button(arrow).clicked() {
                app.ui_state.show_preview_panel = !app.ui_state.show_preview_panel;
            }
            ui.heading("👁 Preview");
        });
        
        render_preview_section(ui, app, &layout);
    }
}

/// Renders Controls Section
fn render_controls_section(ui: &mut egui::Ui, app: &mut App, layout: &ResponsiveLayout) {
    egui::ScrollArea::vertical()
        .id_salt("controls_scroll")
        .show(ui, |ui| {
            // Master Controls
            app.ui_state.render_master_controls_embedded(ui, app.state.layer_manager_mut());
            ui.separator();

            // Media Browser
            egui::CollapsingHeader::new("📁 Media")
                .default_open(!layout.is_compact())
                .show(ui, |ui| {
                    if let Some(action) = app.ui_state.media_browser.ui(
                        ui,
                        &app.ui_state.i18n,
                        app.ui_state.icon_manager.as_ref(),
                    ) {
                        use mapmap_ui::media_browser::MediaBrowserAction;
                        match action {
                            MediaBrowserAction::FileSelected(path) 
                            | MediaBrowserAction::FileDoubleClicked(path) => {
                                if let (Some(module_id), Some(part_id)) = (
                                    app.ui_state.module_canvas.active_module_id,
                                    app.ui_state.module_canvas.editing_part_id
                                ) {
                                    app.ui_state.actions.push(mapmap_ui::UIAction::SetMediaFile(
                                        module_id,
                                        part_id,
                                        path.to_string_lossy().to_string()
                                    ));
                                }
                            }
                            _ => {}
                        }
                    }
                });
                
            // Audio Section
            egui::CollapsingHeader::new("🔊 Audio")
                .default_open(true)
                .show(ui, |ui| {
                    let analysis_v2 = app.audio_analyzer.get_latest_analysis();
                    let legacy_analysis = if app.audio_backend.is_some() {
                        Some(mapmap_core::audio::AudioAnalysis {
                            timestamp: analysis_v2.timestamp,
                            fft_magnitudes: analysis_v2.fft_magnitudes.clone(),
                            band_energies: [
                                analysis_v2.band_energies[0],
                                analysis_v2.band_energies[1],
                                analysis_v2.band_energies[2],
                                analysis_v2.band_energies[3],
                                analysis_v2.band_energies[4],
                                analysis_v2.band_energies[5],
                                analysis_v2.band_energies[6],
                            ],
                            rms_volume: analysis_v2.rms_volume,
                            peak_volume: analysis_v2.peak_volume,
                            beat_detected: analysis_v2.beat_detected,
                            beat_strength: analysis_v2.beat_strength,
                            onset_detected: false,
                            tempo_bpm: None,
                            waveform: analysis_v2.waveform.clone(),
                        })
                    } else {
                        None
                    };

                    if let Some(action) = app.ui_state.audio_panel.ui(
                        ui,
                        &app.ui_state.i18n,
                        legacy_analysis.as_ref(),
                        &app.state.audio_config,
                        &app.audio_devices,
                        &mut app.ui_state.selected_audio_device,
                    ) {
                        match action {
                            mapmap_ui::audio_panel::AudioPanelAction::DeviceChanged(device) => {
                                info!("Audio device changed to: {}", device);
                                app.ui_state.user_config.set_audio_device(Some(device.clone()));
                                app.audio_analyzer.reset();
                                if let Some(backend) = &mut app.audio_backend {
                                    backend.stop();
                                }
                                app.audio_backend = None;
                                match CpalBackend::new(Some(device.clone())) {
                                    Ok(mut backend) => {
                                        if let Err(e) = backend.start() {
                                            error!("Failed to start audio backend: {}", e);
                                        } else {
                                            info!("Audio backend started successfully");
                                        }
                                        app.audio_backend = Some(backend);
                                    }
                                    Err(e) => {
                                        error!("Failed to create audio backend for device '{}': {}", device, e);
                                    }
                                }
                            }
                            mapmap_ui::audio_panel::AudioPanelAction::ConfigChanged(cfg) => {
                                app.audio_analyzer.update_config(AudioAnalyzerV2Config {
                                    sample_rate: cfg.sample_rate,
                                    fft_size: cfg.fft_size,
                                    overlap: cfg.overlap,
                                    smoothing: cfg.smoothing,
                                });
                                app.state.audio_config = cfg;
                            }
                        }
                    }
                });
        });
}

/// Renders Preview Section
fn render_preview_section(ui: &mut egui::Ui, app: &mut App, layout: &ResponsiveLayout) {
    egui::ScrollArea::vertical()
        .id_salt("preview_scroll")
        .show(ui, |ui| {
            let output_infos: Vec<mapmap_ui::OutputPreviewInfo> = app
                .state
                .module_manager
                .modules()
                .iter()
                .flat_map(|module| {
                    module.parts.iter().filter_map(|part| {
                        if let mapmap_core::module::ModulePartType::Output(output_type) = &part.part_type {
                            match output_type {
                                mapmap_core::module::OutputType::Projector { 
                                    ref id, 
                                    ref name, 
                                    ref show_in_preview_panel, 
                                    .. 
                                } => {
                                    Some(mapmap_ui::OutputPreviewInfo {
                                        id: *id,
                                        name: name.clone(),
                                        show_in_panel: *show_in_preview_panel,
                                        texture_name: app.output_assignments.get(id).and_then(|v| v.last().cloned()),
                                        texture_id: app.output_preview_cache.get(id).map(|(id, _)| *id),
                                    })
                                }
                                _ => None,
                            }
                        } else {
                            None
                        }
                    })
                })
                .collect();

            // Thumbnail size based on layout
            let thumbnail_size = if layout.is_compact() {
                120.0
            } else {
                180.0
            };

            // Deduplicate output previews
            let mut seen_ids = std::collections::HashSet::new();
            for info in output_infos {
                if seen_ids.insert(info.id) {
                    if info.show_in_panel {
                        ui.group(|ui| {
                            ui.label(&info.name);
                            if let Some(tex_id) = info.texture_id {
                                ui.image((
                                    tex_id,
                                    egui::vec2(thumbnail_size, thumbnail_size * 9.0 / 16.0),
                                ));
                            } else {
                                ui.label("No Preview");
                            }
                        });
                    }
                }
            }
        });
}
