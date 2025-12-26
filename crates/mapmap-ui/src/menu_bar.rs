//! Egui-based Main Menu Bar and Toolbar
//!
//! This module provides the main menu bar and toolbar for the application.

use crate::audio_meter::AudioMeter;
use crate::icons::AppIcon;
use crate::{AppUI, UIAction};

/// State-holding struct for the main menu bar.
#[derive(Default, Debug)]
pub struct MenuBar {}

/// Renders the main menu bar and returns any action triggered.
pub fn show(ctx: &egui::Context, ui_state: &mut AppUI) -> Vec<UIAction> {
    let mut actions = vec![];

    // Custom frame for modern look
    let frame = egui::Frame::none()
        .fill(ctx.style().visuals.window_fill())
        .inner_margin(egui::Margin::symmetric(16.0, 8.0));

    egui::TopBottomPanel::top("top_panel")
        .frame(frame)
        .show(ctx, |ui| {
            ui.style_mut().visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
            ui.style_mut().visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
            ui.style_mut().visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;

            // Helper for menu items with icons
            let menu_item = |ui: &mut egui::Ui, text: String, icon: Option<AppIcon>| -> bool {
                if let Some(mgr) = &ui_state.icon_manager {
                    if let Some(icon) = icon {
                        if let Some(img) = mgr.image(icon, 14.0) {
                            return ui.add(egui::Button::image_and_text(img, text)).clicked();
                        }
                    }
                }
                ui.button(text).clicked()
            };

            // --- Main Menu Bar ---
            egui::menu::bar(ui, |ui| {
                ui.style_mut().spacing.button_padding = egui::vec2(8.0, 4.0);

                // --- File Menu ---
                ui.menu_button(ui_state.i18n.t("menu-file"), |ui| {
                    if menu_item(
                        ui,
                        ui_state.i18n.t("menu-file-new-project"),
                        Some(AppIcon::Add),
                    ) {
                        actions.push(UIAction::NewProject);
                        ui.close_menu();
                    }
                    if menu_item(
                        ui,
                        ui_state.i18n.t("menu-file-open-project"),
                        Some(AppIcon::LockOpen),
                    ) {
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

                    if menu_item(
                        ui,
                        ui_state.i18n.t("menu-file-save-project"),
                        Some(AppIcon::FloppyDisk),
                    ) {
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

                    if menu_item(
                        ui,
                        ui_state.i18n.t("menu-file-settings"),
                        Some(AppIcon::Cog),
                    ) {
                        actions.push(UIAction::OpenSettings);
                        ui.close_menu();
                    }

                    ui.separator();

                    if menu_item(
                        ui,
                        ui_state.i18n.t("menu-file-exit"),
                        Some(AppIcon::ButtonStop),
                    ) {
                        actions.push(UIAction::Exit);
                        ui.close_menu();
                    }
                });

                // --- Edit Menu ---
                ui.menu_button(ui_state.i18n.t("menu-edit"), |ui| {
                    if menu_item(
                        ui,
                        ui_state.i18n.t("menu-edit-undo"),
                        Some(AppIcon::ArrowLeft),
                    ) {
                        actions.push(UIAction::Undo);
                        ui.close_menu();
                    }
                    if menu_item(
                        ui,
                        ui_state.i18n.t("menu-edit-redo"),
                        Some(AppIcon::ArrowRight),
                    ) {
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
                    if menu_item(
                        ui,
                        ui_state.i18n.t("menu-edit-delete"),
                        Some(AppIcon::Remove),
                    ) {
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
                    if ui.input_mut(|i| {
                        i.consume_shortcut(&egui::KeyboardShortcut::new(
                            egui::Modifiers::CTRL,
                            egui::Key::M,
                        ))
                    }) {
                        actions.push(UIAction::ToggleModuleCanvas);
                    }
                    ui.checkbox(
                        &mut ui_state.show_module_canvas,
                        ui_state.i18n.t("panel-module-canvas"),
                    );
                    ui.checkbox(
                        &mut ui_state.show_controller_overlay,
                        "MIDI Controller Overlay",
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
                        &mut ui_state.show_cue_panel,
                        ui_state.i18n.t("check-show-cues"),
                    );
                    ui.checkbox(
                        &mut ui_state.show_stats,
                        ui_state.i18n.t("check-show-stats"),
                    );
                    ui.checkbox(&mut ui_state.show_timeline, "Timeline");
                    ui.checkbox(&mut ui_state.show_shader_graph, "Shader Graph");
                    ui.checkbox(&mut ui_state.show_toolbar, "Werkzeugleiste");
                    ui.checkbox(&mut ui_state.icon_demo_panel.visible, "Icon Gallery");
                    ui.separator();
                    if menu_item(
                        ui,
                        ui_state.i18n.t("btn-fullscreen"),
                        Some(AppIcon::Monitor),
                    ) {
                        actions.push(UIAction::ToggleFullscreen);
                        ui.close_menu();
                    }
                    if menu_item(
                        ui,
                        ui_state.i18n.t("view-reset-layout"),
                        Some(AppIcon::AppWindow),
                    ) {
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
                    if menu_item(
                        ui,
                        ui_state.i18n.t("menu-help-about"),
                        Some(AppIcon::InfoCircle),
                    ) {
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
            if ui_state.show_toolbar {
                ui.horizontal(|ui| {
                    ui.style_mut().spacing.button_padding = egui::vec2(8.0, 4.0);

                    let icon_size = 32.0;

                    // Helper for icon buttons
                    let mut icon_btn = |icon: AppIcon, tooltip: &str| -> bool {
                        if let Some(mgr) = &ui_state.icon_manager {
                            if let Some(img) = mgr.image(icon, icon_size) {
                                return ui
                                    .add(egui::ImageButton::new(img).frame(false))
                                    .on_hover_text(tooltip)
                                    .clicked();
                            }
                        }
                        ui.button(tooltip).clicked()
                    };

                    if icon_btn(AppIcon::FloppyDisk, &ui_state.i18n.t("toolbar-save")) {
                        actions.push(UIAction::SaveProject(String::new()));
                    }
                    if icon_btn(AppIcon::ArrowLeft, &ui_state.i18n.t("toolbar-undo")) {
                        actions.push(UIAction::Undo);
                    }
                    if icon_btn(AppIcon::ArrowRight, &ui_state.i18n.t("toolbar-redo")) {
                        actions.push(UIAction::Redo);
                    }
                    if icon_btn(AppIcon::Cog, &ui_state.i18n.t("menu-file-settings")) {
                        actions.push(UIAction::OpenSettings);
                    }

                    ui.separator();

                    // === AUDIO LEVEL METER (variable width, dB scale) ===
                    let audio_level = ui_state.current_audio_level;
                    let db = if audio_level > 0.0001 {
                        20.0 * audio_level.log10()
                    } else {
                        -60.0
                    };

                    // Choose width based on style
                    let meter_width = match ui_state.user_config.meter_style {
                        crate::config::AudioMeterStyle::Retro => 300.0,
                        crate::config::AudioMeterStyle::Digital => 360.0,
                    };

                    // Fill available height up to a max
                    let max_height = 120.0;
                    let available_height = ui.available_height().clamp(40.0, max_height);

                    ui.label("🔊");
                    ui.add(
                        AudioMeter::new(ui_state.user_config.meter_style, db, db)
                            .desired_size(egui::vec2(meter_width, available_height)),
                    );

                    // === SPACER - push performance to right ===
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let fps = ui_state.current_fps;
                        let target_fps = ui_state.target_fps;
                        let frame_time = ui_state.current_frame_time_ms;
                        let cpu = ui_state.cpu_usage;
                        let gpu = ui_state.gpu_usage;
                        let ram = ui_state.ram_usage_mb;

                        // Traffic light colors
                        let traffic_light = |value: f32, warn: f32, crit: f32| -> egui::Color32 {
                            if value >= crit {
                                egui::Color32::from_rgb(255, 50, 50)
                            } else if value >= warn {
                                egui::Color32::from_rgb(255, 200, 50)
                            } else {
                                egui::Color32::from_rgb(50, 200, 50)
                            }
                        };

                        let fps_ratio = fps / target_fps.max(1.0);
                        let fps_color = if fps_ratio >= 0.95 {
                            egui::Color32::from_rgb(50, 200, 50)
                        } else if fps_ratio >= 0.8 {
                            egui::Color32::from_rgb(255, 200, 50)
                        } else {
                            egui::Color32::from_rgb(255, 50, 50)
                        };

                        // Overall traffic light
                        let overall_color = if cpu >= 90.0 || gpu >= 90.0 || fps_ratio < 0.8 {
                            egui::Color32::from_rgb(255, 50, 50)
                        } else if cpu >= 70.0 || gpu >= 70.0 || fps_ratio < 0.95 {
                            egui::Color32::from_rgb(255, 200, 50)
                        } else {
                            egui::Color32::from_rgb(50, 200, 50)
                        };

                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
                        ui.painter()
                            .circle_filled(rect.center(), 7.0, overall_color);

                        ui.label(format!("RAM:{:.0}MB", ram));

                        let gpu_color = traffic_light(gpu, 70.0, 90.0);
                        ui.colored_label(gpu_color, format!("GPU:{:.0}%", gpu));

                        let cpu_color = traffic_light(cpu, 70.0, 90.0);
                        ui.colored_label(cpu_color, format!("CPU:{:.0}%", cpu));

                        ui.separator();

                        ui.label(format!("{:.1}ms/f", frame_time))
                            .on_hover_text("Millisekunden pro Frame");

                        ui.colored_label(fps_color, format!("{:.0}/{:.0}FPS", fps, target_fps));
                    });
                });
            }

            ui.add_space(4.0);
            ui.separator();
        });

    actions
}
