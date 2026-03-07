use crate::app::App;
use mapmap_ui as ui;

/// Main UI orchestration function.
/// Renders the entire application UI layout using egui.
pub fn show(ctx: &egui::Context, app: &mut App) {
    // 1. Global Menu Bar (Top-most)
    let menu_actions = ui::view::menu_bar::show(ctx, &mut app.ui_state);
    for action in menu_actions {
        app.ui_state.actions.push(action);
    }

    // 2. Toolbar (Separate Panel below Menu)
    if app.ui_state.show_toolbar {
        egui::TopBottomPanel::top("toolbar_panel")
            .resizable(true)
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
                ui::view::menu_bar::toolbar::show(ui_obj, &mut app.ui_state);
            });
    }

    // 3. Left Panel: Sidebar (Collapsible & Resizable)
    if app.ui_state.show_left_sidebar {
        egui::SidePanel::left("left_sidebar_panel")
            .resizable(true)
            .default_width(300.0)
            .min_width(200.0)
            .show(ctx, |ui_obj| {
                egui::ScrollArea::vertical().show(ui_obj, |ui_obj| {
                    // --- Dashboard Section ---
                    egui::CollapsingHeader::new(app.ui_state.i18n.t("dashboard"))
                        .default_open(true)
                        .show(ui_obj, |ui| {
                            if let Some(ui::view::dashboard::DashboardAction::SendCommand(cmd)) =
                                app.ui_state.dashboard.render_contents(
                                    ui,
                                    &app.ui_state.i18n,
                                    app.ui_state.icon_manager.as_ref(),
                                )
                            {
                                if let Some(_module_id) =
                                    app.ui_state.module_canvas.active_module_id
                                {
                                    if let Some(part_id) =
                                        app.ui_state.module_canvas.get_selected_part_id()
                                    {
                                        app.ui_state
                                            .actions
                                            .push(ui::UIAction::MediaCommand(part_id, cmd));
                                    }
                                }
                            }
                        });
                    ui_obj.separator();

                    // --- Audio Analysis Section ---
                    egui::CollapsingHeader::new(app.ui_state.i18n.t("audio"))
                        .default_open(false)
                        .show(ui_obj, |ui| {
                            let analysis = app.audio_analyzer.get_latest_analysis();
                            if let Some(audio_action) = app.ui_state.audio_panel.ui(
                                ui,
                                &app.ui_state.i18n,
                                Some(&analysis),
                                &app.state.audio_config,
                                &app.ui_state.audio_devices,
                                &mut app.ui_state.selected_audio_device,
                            ) {
                                match audio_action {
                                    ui::panels::audio_panel::AudioPanelAction::DeviceChanged(
                                        device,
                                    ) => {
                                        app.ui_state
                                            .actions
                                            .push(ui::UIAction::SelectAudioDevice(device));
                                    }
                                    ui::panels::audio_panel::AudioPanelAction::ConfigChanged(
                                        cfg,
                                    ) => {
                                        app.state.audio_config = cfg;
                                    }
                                }
                            }
                        });
                    ui_obj.separator();

                    // --- Media Browser Section ---
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
            });
    }

    // 4. Right Panel: Inspector (Docked & Resizable)
    if app.ui_state.show_inspector {
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .default_width(400.0)
            .min_width(300.0)
            .max_width(600.0)
            .show(ctx, |ui_obj| {
                // Render the unified Inspector
                app.ui_state.render_inspector(
                    ui_obj,
                    std::sync::Arc::make_mut(&mut app.state.module_manager),
                    &app.state.layer_manager,
                    &app.state.output_manager,
                );

                // Legacy panels (can be toggled separately or integrated)
                if app.ui_state.show_transforms {
                    app.ui_state.transform_panel.render(ctx, &app.ui_state.i18n);
                }

                // Effect chain integrated into inspector side
                app.ui_state.effect_chain_panel.ui(
                    ctx,
                    &app.ui_state.i18n,
                    app.ui_state.icon_manager.as_ref(),
                    Some(&mut app.recent_effect_configs),
                );
            });
    }

    // 5. Bottom Panel: Timeline (Resizable)
    if app.ui_state.show_timeline {
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .default_height(200.0)
            .min_height(100.0)
            .show(ctx, |ui_obj| {
                ui_obj.heading(app.ui_state.i18n.t("timeline"));
                let mut modules: Vec<ui::TimelineModule> = app
                    .state
                    .module_manager
                    .modules()
                    .iter()
                    .map(|m| ui::TimelineModule {
                        id: m.id,
                        name: m.name.clone(),
                    })
                    .collect();
                modules.sort_by_key(|m| m.id);

                if let Some(action) = app.ui_state.timeline_panel.ui(
                    ui_obj,
                    app.state.effect_animator_mut(),
                    &modules,
                ) {
                    app.ui_state
                        .actions
                        .push(ui::UIAction::TimelineAction(action));
                }
            });
    }

    // 6. Floating Windows / Overlays

    // Performance Stats Overlay
    if app.ui_state.show_stats {
        app.ui_state.render_stats_overlay(
            ctx,
            app.ui_state.current_fps,
            app.ui_state.current_frame_time_ms,
        );
    }

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
                );
            } else {
                ui_obj.centered_and_justified(|ui_obj| {
                    ui_obj.label("Canvas - Module Canvas deaktiviert (View → Module Canvas)");
                });
            }
        });

    // 8. Other Overlays (Shader Graph, Audio, MIDI)

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

    if app.ui_state.show_master_controls {
        let mut layer_manager = std::sync::Arc::make_mut(&mut app.state.layer_manager).clone();
        app.ui_state.render_master_controls(ctx, &mut layer_manager);
        if layer_manager != *app.state.layer_manager {
            *std::sync::Arc::make_mut(&mut app.state.layer_manager) = layer_manager;
            app.state.dirty = true;
        }
    }

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
