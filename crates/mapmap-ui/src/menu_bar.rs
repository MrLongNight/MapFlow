//! Egui-based Main Menu Bar and Toolbar
//!
//! This module provides the main menu bar and toolbar for the application.

use crate::{AppUI, UIAction};

/// State-holding struct for the main menu bar.
#[derive(Default, Debug)]
pub struct MenuBar {}

/// Renders the main menu bar and returns any action triggered.
pub fn show(ctx: &egui::Context, ui_state: &mut AppUI, fps: f32, frame_time: f32) -> Vec<UIAction> {
    let mut actions = vec![];

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        // --- Main Menu Bar ---
        egui::menu::bar(ui, |ui| {
            ui.style_mut().spacing.button_padding = egui::vec2(8.0, 4.0);

            // --- File Menu ---
            ui.menu_button(ui_state.i18n.t("menu-file"), |ui| {
                if ui
                    .button(ui_state.i18n.t("menu-file-new-project"))
                    .clicked()
                {
                    actions.push(UIAction::NewProject);
                    ui.close_menu();
                }
                if ui
                    .button(ui_state.i18n.t("menu-file-open-project"))
                    .clicked()
                {
                    actions.push(UIAction::LoadProject(String::new()));
                    ui.close_menu();
                }

                // Recent files submenu
                let recent_files = ui_state.recent_files.clone();
                if !recent_files.is_empty() {
                    ui.menu_button(ui_state.i18n.t("menu-file-open-recent"), |ui| {
                        for path in &recent_files {
                            if ui.button(path).clicked() {
                                actions.push(UIAction::LoadRecentProject(path.clone()));
                                ui.close_menu();
                            }
                        }
                    });
                }

                ui.separator();

                if ui
                    .button(ui_state.i18n.t("menu-file-save-project"))
                    .clicked()
                {
                    actions.push(UIAction::SaveProject(String::new()));
                    ui.close_menu();
                }
                if ui.button(ui_state.i18n.t("menu-file-save-as")).clicked() {
                    actions.push(UIAction::SaveProjectAs);
                    ui.close_menu();
                }
                if ui.button(ui_state.i18n.t("menu-file-export")).clicked() {
                    actions.push(UIAction::Export);
                    ui.close_menu();
                }

                ui.separator();

                if ui.button(ui_state.i18n.t("menu-file-settings")).clicked() {
                    actions.push(UIAction::OpenSettings);
                    ui.close_menu();
                }

                ui.separator();

                if ui.button(ui_state.i18n.t("menu-file-exit")).clicked() {
                    actions.push(UIAction::Exit);
                    ui.close_menu();
                }
            });

            // --- Edit Menu ---
            ui.menu_button(ui_state.i18n.t("menu-edit"), |ui| {
                if ui.button(ui_state.i18n.t("menu-edit-undo")).clicked() {
                    actions.push(UIAction::Undo);
                    ui.close_menu();
                }
                if ui.button(ui_state.i18n.t("menu-edit-redo")).clicked() {
                    actions.push(UIAction::Redo);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(ui_state.i18n.t("menu-edit-cut")).clicked() {
                    actions.push(UIAction::Cut);
                    ui.close_menu();
                }
                if ui.button(ui_state.i18n.t("menu-edit-copy")).clicked() {
                    actions.push(UIAction::Copy);
                    ui.close_menu();
                }
                if ui.button(ui_state.i18n.t("menu-edit-paste")).clicked() {
                    actions.push(UIAction::Paste);
                    ui.close_menu();
                }
                if ui.button(ui_state.i18n.t("menu-edit-delete")).clicked() {
                    actions.push(UIAction::Delete);
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(ui_state.i18n.t("menu-edit-select-all")).clicked() {
                    actions.push(UIAction::SelectAll);
                    ui.close_menu();
                }
            });

            // --- View Menu ---
            ui.menu_button(ui_state.i18n.t("menu-view"), |ui| {
                ui.checkbox(
                    &mut ui_state.show_toolbar,
                    ui_state.i18n.t("view-toolbar"),
                );
                ui.separator();
                ui.label(ui_state.i18n.t("view-egui-panels"));
                ui.checkbox(
                    &mut ui_state.dashboard.visible,
                    ui_state.i18n.t("panel-dashboard"),
                );
                ui.checkbox(
                    &mut ui_state.effect_chain_panel.visible,
                    ui_state.i18n.t("panel-effect-chain"),
                );
                ui.separator();
                ui.label(ui_state.i18n.t("view-legacy-panels"));
                ui.checkbox(
                    &mut ui_state.show_osc_panel,
                    ui_state.i18n.t("check-show-osc"),
                );
                ui.checkbox(
                    &mut ui_state.show_controls,
                    ui_state.i18n.t("check-show-controls"),
                );
                ui.checkbox(
                    &mut ui_state.show_layers,
                    ui_state.i18n.t("check-show-layers"),
                );
                ui.checkbox(
                    &mut ui_state.paint_panel.visible,
                    ui_state.i18n.t("check-show-paints"),
                );
                ui.checkbox(
                    &mut ui_state.show_mappings,
                    ui_state.i18n.t("check-show-mappings"),
                );
                ui.checkbox(
                    &mut ui_state.show_transforms,
                    ui_state.i18n.t("check-show-transforms"),
                );
                ui.checkbox(
                    &mut ui_state.show_master_controls,
                    ui_state.i18n.t("check-show-master"),
                );
                ui.checkbox(
                    &mut ui_state.oscillator_panel.visible,
                    ui_state.i18n.t("check-show-oscillator"),
                );
                ui.checkbox(
                    &mut ui_state.show_audio,
                    ui_state.i18n.t("check-show-audio"),
                );
                ui.checkbox(
                    &mut ui_state.show_cue_panel,
                    ui_state.i18n.t("check-show-cues"),
                );
                ui.checkbox(
                    &mut ui_state.show_stats,
                    ui_state.i18n.t("check-show-stats"),
                );
                ui.checkbox(&mut ui_state.show_timeline, "Timeline");
                ui.checkbox(&mut ui_state.show_shader_graph, "Shader Graph");
                ui.separator();
                if ui.button(ui_state.i18n.t("btn-fullscreen")).clicked() {
                    actions.push(UIAction::ToggleFullscreen);
                    ui.close_menu();
                }
                if ui.button(ui_state.i18n.t("view-reset-layout")).clicked() {
                    actions.push(UIAction::ResetLayout);
                    ui.close_menu();
                }

                ui.separator();
                ui.label("UI Settings");

                // Theme
                let mut theme = ui_state
                    .user_config
                    .theme
                    .unwrap_or(crate::theme::Theme::Dark);
                if crate::theme::theme_picker(ui, &mut theme) {
                    ui_state.user_config.theme = Some(theme);
                    let _ = ui_state.user_config.save();
                }

                // Scaling
                let mut scale = ui_state.user_config.ui_scale.unwrap_or(1.0);
                ui.horizontal(|ui| {
                    ui.label("UI Scale:");
                    if ui.add(egui::Slider::new(&mut scale, 0.5..=2.5)).changed() {
                        ui_state.user_config.ui_scale = Some(scale);
                        let _ = ui_state.user_config.save();
                    }
                });
            });

            // --- Help Menu ---
            ui.menu_button(ui_state.i18n.t("menu-help"), |ui| {
                if ui.button(ui_state.i18n.t("menu-help-docs")).clicked() {
                    actions.push(UIAction::OpenDocs);
                    ui.close_menu();
                }
                if ui.button(ui_state.i18n.t("menu-help-about")).clicked() {
                    actions.push(UIAction::OpenAbout);
                    ui.close_menu();
                }
                if ui.button(ui_state.i18n.t("menu-help-license")).clicked() {
                    actions.push(UIAction::OpenLicense);
                    ui.close_menu();
                }
                ui.separator();
                ui.menu_button("Language", |ui| {
                    if ui.button("English").clicked() {
                        actions.push(UIAction::SetLanguage("en".to_string()));
                        ui.close_menu();
                    }
                    if ui.button("Deutsch").clicked() {
                        actions.push(UIAction::SetLanguage("de".to_string()));
                        ui.close_menu();
                    }
                });
            });

            // Theme is applied in render_docked_layout
            let _ = ui_state.user_config.theme;

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                ui.label(format!("{:.1} FPS", fps));
                ui.separator();
                ui.label(format!("{:.2} ms", frame_time));
            });
        });

        ui.add_space(4.0);

        // --- Icon Toolbar ---
        if ui_state.show_toolbar {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.style_mut().spacing.button_padding = egui::vec2(8.0, 8.0);

                if ui.button("💾").on_hover_text("Save").clicked() {
                    actions.push(UIAction::SaveProject(String::new()));
                }
                if ui.button("↩️").on_hover_text("Undo").clicked() {
                    actions.push(UIAction::Undo);
                }
                if ui.button("↪️").on_hover_text("Redo").clicked() {
                    actions.push(UIAction::Redo);
                }

                ui.separator();

                if ui.button("➕").on_hover_text("Add Layer").clicked() {
                    actions.push(UIAction::AddLayer);
                }

                let remove_button = egui::Button::new("➖");
                let response = if ui_state.selected_layer_id.is_none() {
                    ui.add_enabled(false, remove_button)
                } else {
                    ui.add(remove_button)
                };

                if response.on_hover_text("Remove Layer").clicked() {
                    if let Some(id) = ui_state.selected_layer_id {
                        actions.push(UIAction::RemoveLayer(id));
                    }
                }

                ui.separator();

                if ui.button("▶️").on_hover_text("Play").clicked() {
                    actions.push(UIAction::Play);
                }
                if ui.button("⏸️").on_hover_text("Pause").clicked() {
                    actions.push(UIAction::Pause);
                }
                if ui.button("⏹️").on_hover_text("Stop").clicked() {
                    actions.push(UIAction::Stop);
                }
            });
            ui.add_space(4.0);
        }
    });

    actions
}
