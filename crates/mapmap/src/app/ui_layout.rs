use crate::app::App;
use mapmap_ui as ui;

/// Main UI orchestration function.
/// Renders the entire application UI layout using egui.
pub fn show(ctx: &egui::Context, app: &mut App) {
    // 0. Main Menu Bar - RESTORED
    let actions = ui::view::menu_bar::show(ctx, &mut app.ui_state);
    app.ui_state.actions.extend(actions);

    // 1. Left Panel: Sidebar (Media Browser + Modules + Dashboard)
    if app.ui_state.show_left_sidebar {
        egui::SidePanel::left("left_sidebar")
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui_obj| {
                ui_obj.set_min_width(200.0);
                
                // Tabs for sidebar
                ui_obj.horizontal(|ui| {
                    ui.selectable_value(&mut app.ui_state.active_sidebar_tab, 0, "📁 Media");
                    ui.selectable_value(&mut app.ui_state.active_sidebar_tab, 1, "📦 Modules");
                    ui.selectable_value(&mut app.ui_state.active_sidebar_tab, 2, "🎛 Dashboard");
                });
                ui_obj.separator();

                egui::ScrollArea::vertical().show(ui_obj, |ui| {
                    match app.ui_state.active_sidebar_tab {
                        0 => {
                            let _ = app.ui_state.media_browser.ui(
                                ui,
                                &app.ui_state.i18n,
                                app.ui_state.icon_manager.as_ref(),
                            );
                        }
                        1 => {
                            if let Some(action) = app.ui_state.module_sidebar.show(
                                ui,
                                std::sync::Arc::make_mut(&mut app.state.module_manager),
                                &app.ui_state.i18n,
                            ) {
                                match action {
                                    ui::ModuleSidebarAction::AddModule => {
                                        let id = std::sync::Arc::make_mut(&mut app.state.module_manager)
                                            .create_module("New Module".to_string());
                                        app.ui_state.module_canvas.set_active_module(Some(id));
                                    }
                                    ui::ModuleSidebarAction::DeleteModule(id) => {
                                        std::sync::Arc::make_mut(&mut app.state.module_manager).remove_module(id);
                                    }
                                    ui::ModuleSidebarAction::SetColor(id, color) => {
                                        if let Some(m) = std::sync::Arc::make_mut(&mut app.state.module_manager).get_module_mut(id) {
                                            m.color = color;
                                        }
                                    }
                                }
                            }
                        }
                        2 => {
                            // Dashboard in Sidebar
                            if let Some(action) = app.ui_state.dashboard.ui_embedded(ui, &app.ui_state.i18n, app.ui_state.icon_manager.as_ref()) {
                                match action {
                                    ui::DashboardAction::SendCommand(cmd) => {
                                        match cmd {
                                            mapmap_media::PlaybackCommand::Play => app.ui_state.actions.push(ui::UIAction::Play),
                                            mapmap_media::PlaybackCommand::Pause => app.ui_state.actions.push(ui::UIAction::Pause),
                                            mapmap_media::PlaybackCommand::Stop => app.ui_state.actions.push(ui::UIAction::Stop),
                                            mapmap_media::PlaybackCommand::SetSpeed(s) => app.ui_state.actions.push(ui::UIAction::SetSpeed(s)),
                                            mapmap_media::PlaybackCommand::SetLoopMode(m) => app.ui_state.actions.push(ui::UIAction::SetLoopMode(m)),
                                            _ => {}
                                        }
                                    }
                                    ui::DashboardAction::AudioDeviceChanged(device) => {
                                        app.ui_state.actions.push(ui::UIAction::SelectAudioDevice(device));
                                    }
                                    ui::DashboardAction::ToggleAudioPanel => {
                                        app.ui_state.actions.push(ui::UIAction::ToggleAudioPanel);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                });
            });
    }

    // 2. Right Panel: Inspector & Functional Panels
    app.ui_state.render_inspector(
        ctx,
        std::sync::Arc::make_mut(&mut app.state.module_manager),
        &app.state.layer_manager,
        &app.state.output_manager,
    );

    // 3. Bottom Panel: Timeline
    if app.ui_state.show_timeline {
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .default_height(200.0)
            .show(ctx, |ui_obj| {
                ui_obj.heading("Timeline");
                let _ = app
                    .ui_state
                    .timeline_panel
                    .ui(ui_obj, app.state.effect_animator_mut());
            });
    }

    // 4. Central Panel: Module Canvas
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

    // 5. Floating Windows / Overlays

    // Settings Window
    app.ui_state.render_settings(ctx);

    // Mesh Visual Editor Window
    if app.ui_state.module_canvas.show_mesh_editor {
        let mut open = app.ui_state.module_canvas.show_mesh_editor;
        egui::Window::new("🖼 Mesh Visual Editor")
            .open(&mut open)
            .default_size([600.0, 500.0])
            .resizable(true)
            .show(ctx, |ui| {
                app.ui_state.module_canvas.mesh_editor.ui(ui);
            });
        app.ui_state.module_canvas.show_mesh_editor = open;
    }

    // Dashboard (Floating window fallback if sidebar is hidden)
    if !app.ui_state.show_left_sidebar && app.ui_state.dashboard.visible {
        if let Some(action) =
            app.ui_state
                .dashboard
                .ui(ctx, &app.ui_state.i18n, app.ui_state.icon_manager.as_ref())
        {
            // ... handling ...
            match action {
                ui::DashboardAction::SendCommand(cmd) => {
                    match cmd {
                        mapmap_media::PlaybackCommand::Play => app.ui_state.actions.push(ui::UIAction::Play),
                        mapmap_media::PlaybackCommand::Pause => app.ui_state.actions.push(ui::UIAction::Pause),
                        mapmap_media::PlaybackCommand::Stop => app.ui_state.actions.push(ui::UIAction::Stop),
                        mapmap_media::PlaybackCommand::SetSpeed(s) => app.ui_state.actions.push(ui::UIAction::SetSpeed(s)),
                        mapmap_media::PlaybackCommand::SetLoopMode(m) => app.ui_state.actions.push(ui::UIAction::SetLoopMode(m)),
                        _ => {}
                    }
                }
                ui::DashboardAction::AudioDeviceChanged(device) => {
                    app.ui_state.actions.push(ui::UIAction::SelectAudioDevice(device));
                }
                ui::DashboardAction::ToggleAudioPanel => {
                    app.ui_state.actions.push(ui::UIAction::ToggleAudioPanel);
                }
            }
        }
    }

    // Media Manager - Show as window
    app.media_manager_ui.visible = app.ui_state.show_media_manager;
    app.media_manager_ui.ui(ctx, &mut app.media_library);
    app.ui_state.show_media_manager = app.media_manager_ui.visible;

    // Audio Panel - Show as window
    if app.ui_state.show_audio {
        egui::Window::new(app.ui_state.i18n.t("panel-audio"))
            .open(&mut app.ui_state.show_audio)
            .show(ctx, |ui_obj| {
                app.ui_state.audio_panel.ui(
                    ui_obj,
                    &app.ui_state.i18n,
                    None, // analysis
                    &app.state.audio_config,
                    &app.ui_state.audio_devices,
                    &mut app.ui_state.selected_audio_device,
                );
            });
    }

    // Effect Chain Panel - Show as window
    if app.ui_state.effect_chain_panel.visible {
        app.ui_state.effect_chain_panel.ui(
            ctx,
            &app.ui_state.i18n,
            app.ui_state.icon_manager.as_ref(),
            Some(&mut app.recent_effect_configs),
        );
    }

    // Cue Panel - Show as window
    if app.ui_state.show_cue_panel {
        app.ui_state.cue_panel.show(
            ctx,
            &app.control_manager,
            &app.ui_state.i18n,
            &mut app.ui_state.actions,
            app.ui_state.icon_manager.as_ref(),
        );
    }

    // Layer Panel (Legacy)
    if app.ui_state.show_layers {
        app.ui_state.layer_panel.show(
            ctx,
            app.state.layer_manager_mut(),
            &mut app.ui_state.selected_layer_id,
            &mut app.ui_state.actions,
            &app.ui_state.i18n,
            app.ui_state.icon_manager.as_ref(),
        );
    }

    // Mapping Panel (Legacy)
    if app.ui_state.show_mappings {
        app.ui_state.mapping_panel.show(
            ctx,
            app.state.mapping_manager_mut(),
            &mut app.ui_state.actions,
            &app.ui_state.i18n,
            app.ui_state.icon_manager.as_ref(),
        );
    }

    // Output Panel (Legacy)
    if app.ui_state.show_outputs {
        app.ui_state.output_panel.show(
            ctx,
            &app.ui_state.i18n,
            app.state.output_manager_mut(),
            &app.ui_state.monitors,
            app.ui_state.icon_manager.as_ref(),
        );
        // Process internal actions from output panel
        let panel_actions = app.ui_state.output_panel.take_actions();
        app.ui_state.actions.extend(panel_actions);
    }

    // Paint Panel (Legacy)
    if app.ui_state.paint_panel.visible {
        app.ui_state.paint_panel.show(
            ctx,
            &app.ui_state.i18n,
            app.state.paint_manager_mut(),
            app.ui_state.icon_manager.as_ref(),
        );
        if let Some(action) = app.ui_state.paint_panel.take_action() {
            match action {
                ui::PaintPanelAction::AddPaint => app.ui_state.actions.push(ui::UIAction::AddPaint),
                ui::PaintPanelAction::RemovePaint(id) => app.ui_state.actions.push(ui::UIAction::RemovePaint(id)),
            }
        }
    }

    // Oscillator Panel (Legacy)
    if app.ui_state.oscillator_panel.visible {
        app.ui_state.oscillator_panel.render(
            ctx,
            &app.ui_state.i18n,
            &mut app.state.oscillator_config,
            app.ui_state.icon_manager.as_ref(),
        );
    }

    // Master controls (Legacy window)
    if app.ui_state.show_master_controls {
        app.ui_state.render_master_controls(ctx, app.state.layer_manager_mut());
    }

    // Transform Panel - Show as window (if not in inspector)
    if app.ui_state.show_transforms && !app.ui_state.show_inspector {
        app.ui_state.transform_panel.render(ctx, &app.ui_state.i18n);
    }

    // Edge Blend Panel - Show as window (if not in inspector)
    if !app.ui_state.show_inspector {
        app.ui_state.edge_blend_panel.show(ctx, &app.ui_state.i18n);
    }

    if app.ui_state.show_shader_graph {
        let mut open = app.ui_state.show_shader_graph;
        egui::Window::new(app.ui_state.i18n.t("panel-node-editor"))
            .open(&mut open)
            .show(ctx, |ui_obj| {
                let _ = app
                    .ui_state
                    .node_editor_panel
                    .ui(ui_obj, &app.ui_state.i18n);
            });
        app.ui_state.show_shader_graph = open;
    }

    app.ui_state.controller_overlay.show(
        ctx,
        app.ui_state.show_controller_overlay,
        false,
        &mut app.ui_state.user_config,
    );

    app.ui_state
        .assignment_panel
        .show(ctx, &app.state.assignment_manager);
    app.ui_state.shortcut_editor.show(ctx, &app.ui_state.i18n);
}
