use crate::{i18n::LocaleManager, responsive::ResponsiveLayout, widgets, UIAction};

/// Represents the UI panel for configuring render outputs.
pub struct OutputPanel {
    /// The ID of the currently selected output for configuration.
    pub selected_output_id: Option<u64>,
    /// Flag to control the visibility of the panel.
    pub visible: bool,
    /// A list of actions to be processed by the main application.
    actions: Vec<UIAction>,
}

impl Default for OutputPanel {
    fn default() -> Self {
        Self {
            selected_output_id: None,
            visible: true,
            actions: Vec::new(),
        }
    }
}

impl OutputPanel {
    /// Takes all pending actions, clearing the internal list.
    pub fn take_actions(&mut self) -> Vec<UIAction> {
        std::mem::take(&mut self.actions)
    }

    /// Renders the output configuration panel using `egui`.
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        i18n: &LocaleManager,
        output_manager: &mut mapmap_core::OutputManager,
        _monitors: &[mapmap_core::monitor::MonitorInfo],
    ) {
        if !self.visible {
            return;
        }

        let layout = ResponsiveLayout::new(ctx);
        let window_size = layout.window_size(420.0, 500.0);

        egui::Window::new(i18n.t("panel-outputs"))
            .default_size(window_size)
            .scroll([false, true])
            .frame(widgets::panel::cyber_panel_frame(&ctx.style()))
            .show(ctx, |ui| {
                widgets::panel::render_panel_header(
                    ui,
                    &i18n.t("header-multi-output"),
                    |_ui| {},
                );

                let canvas_size = output_manager.canvas_size();
                ui.label(format!(
                    "{}: {}x{}",
                    i18n.t("label-canvas"),
                    canvas_size.0,
                    canvas_size.1
                ));
                ui.separator();

                ui.label(format!(
                    "{}: {}",
                    i18n.t("panel-outputs"),
                    output_manager.outputs().len()
                ));

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        let outputs = output_manager.outputs().to_vec();
                        for output in outputs {
                            let is_selected = self.selected_output_id == Some(output.id);
                            if ui.selectable_label(is_selected, &output.name).clicked() {
                                self.selected_output_id = Some(output.id);
                            }
                        }
                    });

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button(i18n.t("btn-projector-array")).clicked() {
                        self.actions.push(UIAction::CreateProjectorArray2x2(
                            (1920, 1080),
                            0.1, // 10% overlap
                        ));
                    }

                    if ui.button(i18n.t("btn-add-output")).clicked() {
                        self.actions.push(UIAction::AddOutput(
                            "New Output".to_string(),
                            mapmap_core::CanvasRegion::new(0.0, 0.0, 1.0, 1.0),
                            (1920, 1080),
                        ));
                    }
                });

                ui.separator();

                if let Some(output_id) = self.selected_output_id {
                    if let Some(output) = output_manager.get_output_mut(output_id) {
                        ui.heading(i18n.t("header-selected-output"));
                        ui.separator();

                        let mut updated_config = output.clone();

                        ui.horizontal(|ui| {
                            ui.label(format!("{}:", i18n.t("label-name")));
                            ui.text_edit_singleline(&mut updated_config.name);
                        });

                        // Note: enabled, fullscreen, and monitor_id fields are not available in OutputConfig
                        // These would need to be added to the OutputConfig struct in mapmap-core

                        ui.label(i18n.t("label-resolution"));
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::DragValue::new(&mut updated_config.resolution.0)
                                    .speed(1.0)
                                    .range(1..=8192),
                            );
                            ui.add(
                                egui::DragValue::new(&mut updated_config.resolution.1)
                                    .speed(1.0)
                                    .range(1..=8192),
                            );
                        });

                        ui.label(i18n.t("label-canvas-region"));
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::DragValue::new(&mut updated_config.canvas_region.x)
                                    .speed(0.01),
                            );
                            ui.add(
                                egui::DragValue::new(&mut updated_config.canvas_region.y)
                                    .speed(0.01),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::DragValue::new(&mut updated_config.canvas_region.width)
                                    .speed(0.01),
                            );
                            ui.add(
                                egui::DragValue::new(&mut updated_config.canvas_region.height)
                                    .speed(0.01),
                            );
                        });

                        ui.collapsing("Edge Blend", |ui| {
                            ui.label(format!("Left: {}", updated_config.edge_blend.left.enabled));
                            ui.label(format!(
                                "Right: {}",
                                updated_config.edge_blend.right.enabled
                            ));
                            ui.label(format!("Top: {}", updated_config.edge_blend.top.enabled));
                            ui.label(format!(
                                "Bottom: {}",
                                updated_config.edge_blend.bottom.enabled
                            ));
                        });

                        ui.collapsing("Color Calibration", |ui| {
                            ui.label(format!(
                                "Brightness: {}",
                                updated_config.color_calibration.brightness
                            ));
                            ui.label(format!(
                                "Contrast: {}",
                                updated_config.color_calibration.contrast
                            ));
                            ui.label(format!(
                                "Saturation: {}",
                                updated_config.color_calibration.saturation
                            ));
                        });

                        if updated_config != *output {
                            *output = updated_config;
                            self.actions
                                .push(UIAction::ConfigureOutput(output_id, output.clone()));
                        }

                        ui.separator();

                        if ui.button(i18n.t("btn-remove-output")).clicked() {
                            self.actions.push(UIAction::RemoveOutput(output_id));
                            self.selected_output_id = None;
                        }
                    }
                }
            });
    }
}
