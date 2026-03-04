//! Egui-based Main Menu Bar and Toolbar
//!
//! This module provides the main menu bar and toolbar for the application.

use crate::audio_meter::AudioMeter;
use crate::{AppUI, UIAction};

/// State-holding menu bar rendering
pub fn show(ctx: &egui::Context, ui_state: &mut AppUI) -> Vec<UIAction> {
    let mut actions = Vec::new();

    // 1. Top Menu Bar
    egui::TopBottomPanel::top("main_menu_bar")
        .frame(egui::Frame::menu(ctx.style().as_ref()))
        .show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // File Menu
                ui.menu_button(ui_state.i18n.t("menu-file"), |ui| {
                    if ui.button(ui_state.i18n.t("menu-new")).clicked() {
                        actions.push(UIAction::NewProject);
                        ui.close();
                    }
                    if ui.button(ui_state.i18n.t("menu-open")).clicked() {
                        actions.push(UIAction::LoadProject("".to_string()));
                        ui.close();
                    }

                    ui.separator();

                    if ui.button(ui_state.i18n.t("menu-save")).clicked() {
                        actions.push(UIAction::SaveProject("".to_string()));
                        ui.close();
                    }
                    if ui.button(ui_state.i18n.t("menu-save-as")).clicked() {
                        actions.push(UIAction::SaveProjectAs);
                        ui.close();
                    }

                    ui.separator();

                    if ui.button(ui_state.i18n.t("menu-export")).clicked() {
                        actions.push(UIAction::Export);
                        ui.close();
                    }

                    ui.separator();

                    if ui.button(ui_state.i18n.t("menu-exit")).clicked() {
                        actions.push(UIAction::Exit);
                        ui.close();
                    }
                });

                // Edit Menu
                ui.menu_button(ui_state.i18n.t("menu-edit"), |ui| {
                    if ui.button(ui_state.i18n.t("menu-undo")).clicked() {
                        actions.push(UIAction::Undo);
                        ui.close();
                    }
                    if ui.button(ui_state.i18n.t("menu-redo")).clicked() {
                        actions.push(UIAction::Redo);
                        ui.close();
                    }
                    ui.separator();
                    if ui.button(ui_state.i18n.t("menu-settings")).clicked() {
                        actions.push(UIAction::OpenSettings);
                        ui.close();
                    }
                });

                // View Menu - CLEANED UP & REORGANIZED
                ui.menu_button(ui_state.i18n.t("menu-view"), |ui| {
                    ui.checkbox(&mut ui_state.show_left_sidebar, "🖥 Sidebar");
                    ui.checkbox(&mut ui_state.show_module_canvas, "📦 Module Canvas");
                    ui.checkbox(&mut ui_state.show_timeline, format!("🎞 {}", ui_state.i18n.t("panel-timeline")));
                    ui.checkbox(&mut ui_state.show_inspector, format!("🔍 {}", ui_state.i18n.t("panel-inspector")));
                    
                    ui.separator();

                    if ui.checkbox(&mut ui_state.show_media_manager, "🎞 Media Manager").clicked() {
                        actions.push(UIAction::ToggleMediaManager);
                    }
                    
                    ui.separator();
                    
                    ui.menu_button("🛠 Erweitert", |ui| {
                        ui.checkbox(&mut ui_state.show_shader_graph, "⚛ Shader Editor");
                        ui.checkbox(&mut ui_state.show_controller_overlay, "🎹 Controller Overlay");
                        ui.checkbox(&mut ui_state.show_cue_panel, "🎭 Cue List");
                        ui.checkbox(&mut ui_state.is_midi_learn_mode, "🛰 MIDI Learn Mode");
                    });

                    ui.menu_button("📺 Windows (Legacy)", |ui| {
                        ui.checkbox(&mut ui_state.show_layers, "Layers");
                        ui.checkbox(&mut ui_state.show_mappings, "Mappings");
                        ui.checkbox(&mut ui_state.show_outputs, "Outputs");
                        ui.checkbox(&mut ui_state.paint_panel.visible, "Paints");
                        ui.checkbox(&mut ui_state.oscillator_panel.visible, "Oscillator");
                        ui.checkbox(&mut ui_state.icon_demo_panel.visible, "Icon Demo");
                    });

                    ui.separator();
                    
                    ui.checkbox(&mut ui_state.show_toolbar, ui_state.i18n.t("view-show-toolbar"));
                    ui.checkbox(&mut ui_state.show_stats, ui_state.i18n.t("view-show-stats"));
                    
                    ui.separator();
                    
                    if ui.button(ui_state.i18n.t("view-reset-layout")).clicked() {
                        actions.push(UIAction::ResetLayout);
                        ui.close();
                    }
                });

                // Help Menu
                ui.menu_button(ui_state.i18n.t("menu-help"), |ui| {
                    if ui.button(ui_state.i18n.t("menu-docs")).clicked() {
                        actions.push(UIAction::OpenDocs);
                        ui.close();
                    }
                    if ui.button(ui_state.i18n.t("menu-license")).clicked() {
                        actions.push(UIAction::OpenLicense);
                        ui.close();
                    }
                    ui.separator();
                    if ui.button(ui_state.i18n.t("menu-about")).clicked() {
                        actions.push(UIAction::OpenAbout);
                        ui.close();
                    }
                });
            });
        });

    // 2. Toolbar
    if ui_state.show_toolbar {
        egui::TopBottomPanel::top("toolbar")
            .frame(egui::Frame::NONE.fill(crate::theme::colors::DARK_GREY))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(8.0);

                    // Transport
                    if ui.button("▶").on_hover_text(ui_state.i18n.t("btn-play")).clicked() {
                        actions.push(UIAction::Play);
                    }
                    if ui.button("⏸").on_hover_text(ui_state.i18n.t("btn-pause")).clicked() {
                        actions.push(UIAction::Pause);
                    }
                    if ui.button("⏹").on_hover_text(ui_state.i18n.t("btn-stop")).clicked() {
                        actions.push(UIAction::Stop);
                    }

                    ui.separator();

                    // Global Tools
                    if ui.selectable_label(ui_state.dashboard.visible, "🎛 Dashboard").clicked() {
                        ui_state.dashboard.visible = !ui_state.dashboard.visible;
                    }
                    if ui.selectable_label(ui_state.show_audio, "🔊 Audio").clicked() {
                        actions.push(UIAction::ToggleAudioPanel);
                    }

                    ui.separator();

                    // Stats Overlay
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        
                        // Audio Level
                        let meter = AudioMeter::new(ui_state.user_config.meter_style, ui_state.current_audio_level, ui_state.current_audio_level);
                        ui.add(meter);
                        
                        ui.separator();
                        
                        // Performance
                        ui.label(
                            egui::RichText::new(format!("{:.0} FPS", ui_state.current_fps))
                                .color(crate::theme::colors::MINT_ACCENT)
                                .small(),
                        );
                        ui.label(
                            egui::RichText::new(format!("{:.1} MB", ui_state.ram_usage_mb))
                                .color(crate::theme::colors::CYAN_ACCENT)
                                .small(),
                        );
                        ui.label(
                            egui::RichText::new(format!("{:.0}% CPU", ui_state.cpu_usage))
                                .color(crate::theme::colors::WARN_COLOR)
                                .small(),
                        );
                    });
                });
            });
    }

    actions
}
