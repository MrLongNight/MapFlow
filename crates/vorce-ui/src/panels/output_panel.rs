use crate::{
    core::responsive::ResponsiveLayout,
    i18n::LocaleManager,
    theme::colors,
    widgets::custom::{cyber_list_item, styled_button, styled_drag_value},
    widgets::icons::IconManager,
    widgets::panel::{cyber_panel_frame, render_panel_header},
    UIAction,
};
use egui::CornerRadius;

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
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        i18n: &LocaleManager,
        output_manager: &mut vorce_core::OutputManager,
        _monitors: &[vorce_core::monitor::MonitorInfo],
        _icon_manager: Option<&IconManager>,
    ) {
        if !self.visible {
            return;
        }

        let layout = ResponsiveLayout::new(ctx);
        let window_size = layout.window_size(420.0, 500.0);

        egui::Window::new(i18n.t("panel-outputs"))
            .default_size(window_size)
            .resizable(true)
            .scroll([false, true])
            .frame(cyber_panel_frame(&ctx.style()))
            .show(ctx, |ui| {
                render_panel_header(ui, &i18n.t("panel-outputs"), |_| {});

                ui.add_space(8.0);

                render_panel_header(ui, &i18n.t("header-multi-output"), |_| {});
                ui.add_space(4.0);

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

                egui::Frame::default()
                    .fill(colors::DARKER_GREY)
                    .stroke(egui::Stroke::new(1.0, colors::STROKE_GREY))
                    .corner_radius(CornerRadius::ZERO)
                    .inner_margin(4.0)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .max_height(150.0)
                            .show(ui, |ui| {
                                let outputs = output_manager.outputs().to_vec();
                                for (i, output) in outputs.iter().enumerate() {
                                    let is_selected = self.selected_output_id == Some(output.id);

                                    cyber_list_item(
                                        ui,
                                        egui::Id::new(output.id),
                                        is_selected,
                                        i % 2 == 1,
                                        |ui| {
                                            if ui
                                                .selectable_label(is_selected, &output.name)
                                                .clicked()
                                            {
                                                self.selected_output_id = Some(output.id);
                                            }
                                        },
                                    );
                                }
                            });
                    });

                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if styled_button(ui, &i18n.t("btn-projector-array"), false).clicked() {
                        self.actions.push(UIAction::CreateProjectorArray2x2(
                            (1920, 1080),
                            0.1, // 10% overlap
                        ));
                    }

                    if styled_button(ui, &i18n.t("btn-add-output"), false).clicked() {
                        self.actions.push(UIAction::AddOutput(
                            "New Output".to_string(),
                            vorce_core::CanvasRegion::new(0.0, 0.0, 1.0, 1.0),
                            (1920, 1080),
                        ));
                    }
                });

                ui.separator();

                if let Some(output_id) = self.selected_output_id {
                    if let Some(output) = output_manager.get_output_mut(output_id) {
                        render_panel_header(ui, &i18n.t("header-selected-output"), |_| {});
                        ui.add_space(4.0);

                        let mut updated_config = output.clone();

                        ui.horizontal(|ui| {
                            ui.label(format!("{}:", i18n.t("label-name")));
                            ui.text_edit_singleline(&mut updated_config.name);
                        });

                        // Resolution (u32, so keeping egui::DragValue for now but styled if possible)
                        ui.label(i18n.t("label-resolution"));
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::DragValue::new(&mut updated_config.resolution.0)
                                    .speed(1.0)
                                    .range(1..=8192)
                                    .prefix("W: "),
                            );
                            ui.add(
                                egui::DragValue::new(&mut updated_config.resolution.1)
                                    .speed(1.0)
                                    .range(1..=8192)
                                    .prefix("H: "),
                            );
                        });

                        ui.label(i18n.t("label-canvas-region"));
                        // Using styled_drag_value for f32 fields
                        ui.horizontal(|ui| {
                            styled_drag_value(
                                ui,
                                &mut updated_config.canvas_region.x,
                                0.01,
                                0.0..=1.0,
                                0.0,
                                "X: ",
                                "",
                            );
                            styled_drag_value(
                                ui,
                                &mut updated_config.canvas_region.y,
                                0.01,
                                0.0..=1.0,
                                0.0,
                                "Y: ",
                                "",
                            );
                        });
                        ui.horizontal(|ui| {
                            styled_drag_value(
                                ui,
                                &mut updated_config.canvas_region.width,
                                0.01,
                                0.0..=1.0,
                                1.0,
                                "W: ",
                                "",
                            );
                            styled_drag_value(
                                ui,
                                &mut updated_config.canvas_region.height,
                                0.01,
                                0.0..=1.0,
                                1.0,
                                "H: ",
                                "",
                            );
                        });

                        crate::widgets::custom::collapsing_header_with_reset(
                            ui,
                            "Edge Blend",
                            false,
                            |ui| {
                                ui.label(format!(
                                    "Left: {}",
                                    updated_config.edge_blend.left.enabled
                                ));
                                ui.label(format!(
                                    "Right: {}",
                                    updated_config.edge_blend.right.enabled
                                ));
                                ui.label(format!("Top: {}", updated_config.edge_blend.top.enabled));
                                ui.label(format!(
                                    "Bottom: {}",
                                    updated_config.edge_blend.bottom.enabled
                                ));
                            },
                        );

                        crate::widgets::custom::collapsing_header_with_reset(
                            ui,
                            "Color Calibration",
                            false,
                            |ui| {
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
                            },
                        );

                        if updated_config != *output {
                            *output = updated_config;
                            self.actions
                                .push(UIAction::ConfigureOutput(output_id, output.clone()));
                        }

                        ui.add_space(8.0);
                        ui.separator();

                        if crate::widgets::custom::delete_button(ui) {
                            self.actions.push(UIAction::RemoveOutput(output_id));
                            self.selected_output_id = None;
                        }
                    }
                }
            });
    }
}
