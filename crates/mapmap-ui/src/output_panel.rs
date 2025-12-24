use crate::i18n::LocaleManager;
use crate::icons::{AppIcon, IconManager};
use crate::UIAction;
use egui::*;
use mapmap_core::OutputManager;

#[derive(Debug, Default)]
pub struct OutputPanel {
    pub visible: bool,
}

impl OutputPanel {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        output_manager: &mut OutputManager,
        selected_output_id: &mut Option<u64>,
        actions: &mut Vec<UIAction>,
        i18n: &LocaleManager,
        icon_manager: Option<&IconManager>,
    ) {
        if !self.visible {
            return;
        }

        let mut open = self.visible;
        egui::Window::new(i18n.t("panel-outputs"))
            .open(&mut open)
            .default_size([420.0, 500.0])
            .show(ctx, |ui| {
                ui.heading(i18n.t("header-outputs"));
                ui.separator();

                // Canvas size display
                let canvas_size = output_manager.canvas_size();
                ui.label(format!(
                    "{}: {}x{}",
                    i18n.t("label-canvas"),
                    canvas_size.0,
                    canvas_size.1
                ));
                ui.separator();

                // Output list
                ui.label(format!(
                    "{}: {}",
                    i18n.t("panel-outputs"),
                    output_manager.outputs().len()
                ));

                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        for output in output_manager.outputs() {
                            let is_selected = *selected_output_id == Some(output.id);

                            ui.horizontal(|ui| {
                                if let Some(mgr) = icon_manager {
                                    let icon = if output.fullscreen {
                                        AppIcon::Screen
                                    } else {
                                        AppIcon::AppWindow
                                    };
                                    mgr.show(ui, icon, 16.0);
                                }

                                if ui.selectable_label(is_selected, &output.name).clicked() {
                                    *selected_output_id = Some(output.id);
                                }

                                ui.label(format!(
                                    "{}x{}",
                                    output.resolution.0, output.resolution.1
                                ));
                            });
                        }
                    });

                ui.separator();

                // Quick setup buttons
                ui.horizontal(|ui| {
                    if ui.button(i18n.t("btn-projector-array")).clicked() {
                        actions.push(UIAction::CreateProjectorArray2x2(
                            (1920, 1080),
                            0.1, // 10% overlap
                        ));
                    }

                    if ui.button(i18n.t("btn-add-output")).clicked() {
                        actions.push(UIAction::AddOutput(
                            "New Output".to_string(),
                            mapmap_core::CanvasRegion::new(0.0, 0.0, 1.0, 1.0),
                            (1920, 1080),
                        ));
                    }
                });

                ui.separator();

                // Edit selected output
                if let Some(output_id) = *selected_output_id {
                    let output_opt = output_manager.outputs().iter().find(|o| o.id == output_id);
                    if let Some(output) = output_opt {
                        ui.group(|ui| {
                            ui.heading(i18n.t("header-selected-output"));

                            ui.horizontal(|ui| {
                                ui.label(format!("{}:", i18n.t("label-name")));
                                ui.label(&output.name); // Editable later?
                            });

                            ui.label(format!(
                                "{}: {}x{}",
                                i18n.t("label-resolution"),
                                output.resolution.0,
                                output.resolution.1
                            ));

                            ui.separator();
                            ui.label(i18n.t("label-canvas-region"));
                            ui.indent("region", |ui| {
                                ui.label(format!(
                                    "{}: {:.2}, {}: {:.2}",
                                    i18n.t("label-x"),
                                    output.canvas_region.x,
                                    i18n.t("label-y"),
                                    output.canvas_region.y
                                ));
                                ui.label(format!(
                                    "{}: {:.2}, {}: {:.2}",
                                    i18n.t("label-width"),
                                    output.canvas_region.width,
                                    i18n.t("label-height"),
                                    output.canvas_region.height
                                ));
                            });

                            ui.separator();

                            // Edge blending status
                            ui.label(format!("{}:", i18n.t("panel-edge-blend")));
                            ui.indent("blend", |ui| {
                                let blend = &output.edge_blend;
                                if blend.left.enabled {
                                    ui.label(format!("• {}", i18n.t("check-left")));
                                }
                                if blend.right.enabled {
                                    ui.label(format!("• {}", i18n.t("check-right")));
                                }
                                if blend.top.enabled {
                                    ui.label(format!("• {}", i18n.t("check-top")));
                                }
                                if blend.bottom.enabled {
                                    ui.label(format!("• {}", i18n.t("check-bottom")));
                                }
                                if !blend.left.enabled
                                    && !blend.right.enabled
                                    && !blend.top.enabled
                                    && !blend.bottom.enabled
                                {
                                    ui.weak(i18n.t("label-none"));
                                }
                            });

                            ui.separator();

                            // Color calibration status
                            ui.label(format!("{}:", i18n.t("panel-color-cal")));
                            ui.indent("cal", |ui| {
                                let cal = &output.color_calibration;
                                let mut any = false;
                                if cal.brightness != 0.0 {
                                    ui.label(format!(
                                        "• {}: {:.2}",
                                        i18n.t("label-brightness"),
                                        cal.brightness
                                    ));
                                    any = true;
                                }
                                if cal.contrast != 1.0 {
                                    ui.label(format!(
                                        "• {}: {:.2}",
                                        i18n.t("label-contrast"),
                                        cal.contrast
                                    ));
                                    any = true;
                                }
                                if cal.saturation != 1.0 {
                                    ui.label(format!(
                                        "• {}: {:.2}",
                                        i18n.t("label-saturation"),
                                        cal.saturation
                                    ));
                                    any = true;
                                }
                                if !any {
                                    ui.weak(format!("({})", i18n.t("label-none")));
                                }
                            });

                            ui.separator();

                            ui.colored_label(
                                Color32::from_rgb(128, 200, 255),
                                format!("{}:", i18n.t("output-tip")),
                            );
                            ui.label(i18n.t("tip-panels-auto-open"));

                            ui.separator();

                            if ui.button(i18n.t("btn-remove-output")).clicked() {
                                actions.push(UIAction::RemoveOutput(output_id));
                                *selected_output_id = None;
                            }
                        });
                    }
                }

                ui.separator();
                ui.colored_label(Color32::GREEN, i18n.t("msg-multi-window-active"));
                ui.weak(i18n.t("msg-output-windows-tip"));
            });

        self.visible = open;
    }
}
