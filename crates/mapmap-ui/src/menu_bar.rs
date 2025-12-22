//! Egui-based Main Menu Bar and Toolbar
//!
//! This module provides the main menu bar and toolbar for the application.

use crate::{AppUI, UIAction};

/// State-holding struct for the main menu bar.
#[derive(Default, Debug)]
pub struct MenuBar {}

/// Renders the main menu bar and returns any action triggered.
pub fn show(ctx: &egui::Context, ui_state: &mut AppUI) -> Vec<UIAction> {
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
                ui.separator();
                if ui.button(ui_state.i18n.t("btn-fullscreen")).clicked() {
                    actions.push(UIAction::ToggleFullscreen);
                    ui.close_menu();
                }
                if ui.button(ui_state.i18n.t("view-reset-layout")).clicked() {
                    actions.push(UIAction::ResetLayout);
                    ui.close_menu();
                }
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
        });

        ui.add_space(4.0);

        // --- Toolbar ---
        ui.horizontal(|ui| {
            ui.style_mut().spacing.button_padding = egui::vec2(8.0, 4.0);

            if ui.button(ui_state.i18n.t("toolbar-save")).clicked() {
                actions.push(UIAction::SaveProject(String::new()));
            }
            if ui.button(ui_state.i18n.t("toolbar-undo")).clicked() {
                actions.push(UIAction::Undo);
            }
            if ui.button(ui_state.i18n.t("toolbar-redo")).clicked() {
                actions.push(UIAction::Redo);
            }
        });

        ui.add_space(4.0);
        ui.separator();
    });

    actions
}
