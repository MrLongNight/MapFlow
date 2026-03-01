use crate::App;

#[allow(missing_docs)]
pub fn show(ctx: &egui::Context, app: &mut App) {
    // Top Panel: Dashboard / Global Controls
    // ui::panels::dashboard::show(ctx, app);

    // Left Panel: Media Browser / Assets
    // ui::view::media_browser::show(ctx, app);

    // Right Panel: Inspector / Transformation
    egui::SidePanel::right("right_panel")
        .resizable(true)
        .default_width(300.0)
        .show(ctx, |_ui| {
            // ui::panels::transform_panel::show(ui, &mut app.ui_state, &mut app.state);
            // ui::panels::edge_blend_panel::show(ui, &mut app.ui_state, &mut app.state);
            // ui::panels::effect_chain_panel::show(ui, &mut app.ui_state, &mut app.state);
        });

    // Bottom Panel: Timeline / Animation
    egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(true)
        .default_height(200.0)
        .show(ctx, |ui| {
            // Timeline view
            ui.heading("Timeline (Work in Progress)");
            ui.label("Animation and sequencing controls will be here.");
        });

    // === 5. CENTRAL PANEL: Module Canvas ===
    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(ctx.style().visuals.panel_fill))
        .show(ctx, |ui| {
            if app.ui_state.show_module_canvas {
                // Ensure we have an active module selected if any exist
                if app.ui_state.module_canvas.active_module_id.is_none() {
                    if let Some(first_mod) = app.state.module_manager.modules().first() {
                        app.ui_state.module_canvas.active_module_id = Some(first_mod.id);
                    }
                }

                // --- Module Selector Toolbar ---
                egui::MenuBar::new().ui(ui, |ui| {
                    // Module dropdown selector (from main)
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
                            .show_ui(ui, |ui| {
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
                                    if ui.selectable_label(is_selected, label).clicked() {
                                        app.ui_state.module_canvas.set_active_module(Some(*id));
                                    }
                                }
                            });
                        ui.separator();
                    }

                    // Add Node button (from HEAD/PR 885)
                    if let Some(module_id) = app.ui_state.module_canvas.active_module_id {
                        ui.menu_button(egui::RichText::new("➕ Hinzufügen").strong(), |ui| {
                            mapmap_ui::editors::module_canvas::draw::render_add_node_menu_content(
                                ui,
                                std::sync::Arc::make_mut(&mut app.state.module_manager),
                                None,
                                Some(module_id),
                            );
                        });
                        ui.separator();
                    }

                    // Presets & Search (from HEAD/PR 885)
                    if ui.button("💾 Speichern").clicked() {
                        app.ui_state.module_canvas.show_presets = true;
                    }
                    if ui.button("🔍 Suchen").clicked() {
                        app.ui_state.module_canvas.show_search =
                            !app.ui_state.module_canvas.show_search;
                    }

                    // New Module button
                    if ui
                        .button(egui::RichText::new("➕ Neues Modul").strong())
                        .clicked()
                    {
                        let new_id = std::sync::Arc::make_mut(&mut app.state.module_manager)
                            .create_module("New Module".to_string());
                        app.ui_state.module_canvas.set_active_module(Some(new_id));
                    }

                    // View Controls (from HEAD/PR 885)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Zentrieren").clicked() {
                            app.ui_state.module_canvas.pan_offset = egui::Vec2::ZERO;
                            app.ui_state.module_canvas.zoom = 1.0;
                        }
                        ui.label(format!("Zoom: {:.1}x", app.ui_state.module_canvas.zoom));
                    });
                });

                crate::ui::editors::module_canvas::show(
                    ui,
                    crate::ui::editors::module_canvas::ModuleCanvasContext {
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
    crate::ui::editors::node_editor::show(
        ctx,
        crate::ui::editors::node_editor::NodeEditorContext {
            ui_state: &mut app.ui_state,
        },
    );

    // Audio Panel
    // ui::panels::audio_panel::show(ctx, &mut app.ui_state, &mut app.state);

    // Controller Overlay
    // ui::panels::controller_overlay_panel::show(ctx, &mut app.ui_state, &mut app.state);

    // Assignment Panel
    // ui::panels::assignment_panel::show(ctx, &mut app.ui_state, &mut app.state);

    // Shortcuts Panel
    // ui::panels::shortcuts_panel::show(ctx, &mut app.ui_state, &mut app.state);
}
