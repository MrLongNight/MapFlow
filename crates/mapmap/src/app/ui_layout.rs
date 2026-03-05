use crate::app::App;
use mapmap_ui as ui;

/// Main UI orchestration function.
/// Renders the entire application UI layout using egui.
pub fn show(ctx: &egui::Context, app: &mut App) {
    // 1. Top Panel: Dashboard / Global Controls
    let _ = app
        .ui_state
        .dashboard
        .ui(ctx, &app.ui_state.i18n, app.ui_state.icon_manager.as_ref());

    // 2. Left Panel: Media Browser
    if app.ui_state.show_media_browser {
        egui::SidePanel::left("media_browser_panel")
            .resizable(true)
            .default_width(280.0)
            .show(ctx, |ui_obj| {
                let _ = app.ui_state.media_browser.ui(
                    ui_obj,
                    &app.ui_state.i18n,
                    app.ui_state.icon_manager.as_ref(),
                );
            });
    }

    // 3. Right Panel: Inspector & Functional Panels
    egui::SidePanel::right("right_panel")
        .resizable(true)
        .default_width(300.0)
        .show(ctx, |_ui_obj| {
            // Transform and Edge Blend panels manage their own windows
            app.ui_state.transform_panel.render(ctx, &app.ui_state.i18n);
            app.ui_state.edge_blend_panel.show(ctx, &app.ui_state.i18n);

            app.ui_state.effect_chain_panel.ui(
                ctx,
                &app.ui_state.i18n,
                app.ui_state.icon_manager.as_ref(),
                Some(&mut app.recent_effect_configs),
            );
        });

    // 4. Bottom Panel: Timeline
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

    // Cue Panel
    app.ui_state.cue_panel.show(
        ctx,
        &app.control_manager,
        &app.ui_state.i18n,
        &mut app.ui_state.actions,
        app.ui_state.icon_manager.as_ref(),
    );

    // 5. Central Panel: Module Canvas
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

    // 6. Floating Windows / Overlays

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
