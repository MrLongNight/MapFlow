use crate::audio_meter::AudioMeter;
use crate::core::config::ToolbarMetricMode;
use crate::icons::AppIcon;
use crate::{AppUI, UIAction};

pub fn show(ui: &mut egui::Ui, ui_state: &mut AppUI) {
    let mut actions = Vec::new();

    egui::ScrollArea::horizontal()
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.style_mut().spacing.button_padding = egui::vec2(8.0, 4.0);
                ui.style_mut().spacing.item_spacing.y = 0.0;

                let icon_size = 32.0;
                let is_hovering_toolbar = ui.rect_contains_pointer(ui.max_rect());

                let mut icon_btn = |icon: AppIcon, tooltip: &str| -> bool {
                    if let Some(mgr) = &ui_state.icon_manager {
                        if let Some(img) = mgr.image(icon, icon_size) {
                            return ui
                                .add(egui::Button::image(img).frame(false))
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
                ui.toggle_value(&mut ui_state.show_left_sidebar, "Sidebar");
                ui.toggle_value(&mut ui_state.show_inspector, "Inspector");
                ui.toggle_value(&mut ui_state.show_timeline, "Timeline");

                ui.separator();

                let metrics = &ui_state.user_config.toolbar_metrics;
                let show_metric = |cfg: &crate::core::config::ToolbarMetricConfig| {
                    cfg.visible
                        && (cfg.mode == ToolbarMetricMode::Always
                            || (cfg.mode == ToolbarMetricMode::Hover && is_hovering_toolbar))
                };

                if show_metric(&metrics.bpm) {
                    let bpm = ui_state.current_bpm;
                    let bpm_text = if let Some(tempo) = bpm.filter(|tempo| *tempo > 0.0) {
                        format!("{tempo:.0} BPM")
                    } else {
                        "--- BPM".to_string()
                    };

                    ui.add(egui::Label::new(
                        egui::RichText::new(bpm_text)
                            .size(16.0)
                            .color(egui::Color32::from_rgb(255, 200, 0))
                            .strong(),
                    ))
                    .clone()
                    .on_hover_text("Erkanntes Tempo (Beats per Minute)");
                    ui.separator();
                }

                #[cfg(feature = "midi")]
                {
                    let fader_clicked = if let Some(mgr) = &ui_state.icon_manager {
                        if let Some(img) = mgr.image(AppIcon::Fader, 32.0) {
                            let btn = if ui_state.show_controller_overlay {
                                egui::Button::image(img).frame(true)
                            } else {
                                egui::Button::image(img).frame(false)
                            };
                            ui.add(btn)
                                .on_hover_text("MIDI Controller Overlay ein/aus")
                                .clicked()
                        } else {
                            ui.button("MIDI")
                                .on_hover_text("MIDI Controller Overlay ein/aus")
                                .clicked()
                        }
                    } else {
                        ui.button("MIDI")
                            .on_hover_text("MIDI Controller Overlay ein/aus")
                            .clicked()
                    };

                    if fader_clicked {
                        ui_state.show_controller_overlay = !ui_state.show_controller_overlay;
                    }

                    ui.separator();

                    let learn_btn = if ui_state.is_midi_learn_mode {
                        egui::Button::new("Learn").fill(egui::Color32::YELLOW)
                    } else {
                        egui::Button::new("Learn")
                    };
                    if ui
                        .add(learn_btn)
                        .on_hover_text("Global MIDI Learn Mode aktivieren")
                        .clicked()
                    {
                        actions.push(UIAction::ToggleMidiLearn);
                    }
                }

                if show_metric(&metrics.audio_meter) {
                    ui.separator();
                    let audio_level = ui_state.current_audio_level;
                    let db = if audio_level > 0.0001 {
                        20.0 * audio_level.log10()
                    } else {
                        -60.0
                    };

                    ui.label("🔊");
                    let meter_height = ui.available_height().clamp(16.0, 28.0);
                    ui.add(
                        AudioMeter::new(ui_state.user_config.meter_style, db, db)
                            .height(meter_height),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let fps = ui_state.current_fps;
                    let target_fps = ui_state.target_fps;
                    let frame_time = ui_state.current_frame_time_ms;
                    let cpu = ui_state.cpu_usage;
                    let gpu = ui_state.gpu_usage;
                    let ram = ui_state.ram_usage_mb;

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

                    if show_metric(&metrics.status) {
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
                    }

                    if show_metric(&metrics.ram) {
                        ui.label(format!("RAM:{ram:.0}MB"));
                    }
                    if show_metric(&metrics.gpu) {
                        let gpu_color = traffic_light(gpu, 70.0, 90.0);
                        ui.colored_label(gpu_color, format!("Load:{gpu:.0}%"));
                    }
                    if show_metric(&metrics.cpu) {
                        let cpu_color = traffic_light(cpu, 70.0, 90.0);
                        ui.colored_label(cpu_color, format!("CPU:{cpu:.0}%"));
                    }
                    if show_metric(&metrics.frame_time) {
                        ui.label(format!("{frame_time:.1}ms/f"))
                            .on_hover_text("Millisekunden pro Frame");
                    }
                    if show_metric(&metrics.fps) {
                        ui.colored_label(fps_color, format!("{fps:.0}/{target_fps:.0}FPS"));
                    }
                });
            });
        });

    ui_state.actions.append(&mut actions);
}
