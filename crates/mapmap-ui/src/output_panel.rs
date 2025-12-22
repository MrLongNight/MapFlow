//! Phase 6: Egui-based Output Configuration Panel
//!
//! Manages display outputs, resolution, fullscreen, positioning, and links to
//! calibration and edge blending tools.

use crate::i18n::LocaleManager;
use egui::{DragValue, Slider, Ui};
use mapmap_core::monitor::MonitorInfo;
use mapmap_core::output::{OutputConfig, OutputManager};

/// Actions that can be triggered from the output panel.
#[derive(Debug, Clone, PartialEq)]
pub enum OutputPanelAction {
    /// Request to add a new default output.
    AddOutput,
    /// Request to remove the output with the given ID.
    RemoveOutput(u64),
    /// Request to update a specific output's configuration.
    UpdateOutput(OutputConfig),
    /// Request to show the test pattern on a specific output.
    ShowTestPattern(u64),
    /// Request to show the color calibration panel for a specific output.
    ShowColorCalibration(u64),
    /// Request to show the edge blend panel for a specific output.
    ShowEdgeBlend(u64),
}

/// The state and UI for the output configuration panel.
#[derive(Default)]
pub struct OutputPanel {
    /// Is the panel currently visible?
    pub visible: bool,
    selected_output_id: Option<u64>,
}

impl OutputPanel {
    /// Renders the output panel and returns any actions triggered by the user.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        locale: &LocaleManager,
        output_manager: &mut OutputManager,
        monitors: &[MonitorInfo],
    ) -> Vec<OutputPanelAction> {
        let mut actions = Vec::new();

        if !self.visible {
            return actions;
        }

        let mut window_open = self.visible;
        egui::Window::new(locale.t("output-panel-title"))
            .open(&mut window_open)
            .resizable(true)
            .default_width(550.0)
            .default_height(400.0)
            .show(ctx, |ui| {
                self.ui(ui, locale, output_manager, monitors, &mut actions);
            });
        self.visible = window_open;

        actions
    }

    fn ui(
        &mut self,
        ui: &mut Ui,
        locale: &LocaleManager,
        output_manager: &mut OutputManager,
        monitors: &[MonitorInfo],
        actions: &mut Vec<OutputPanelAction>,
    ) {
        // Ensure selection is valid
        if let Some(id) = self.selected_output_id {
            if output_manager.get_output(id).is_none() {
                self.selected_output_id = None;
            }
        }
        if self.selected_output_id.is_none() {
            self.selected_output_id = output_manager.outputs().first().map(|o| o.id);
        }

        egui::SidePanel::left("output_list_panel")
            .resizable(true)
            .default_width(150.0)
            .show_inside(ui, |ui| {
                ui.heading(locale.t("output-panel-list-header"));
                ui.separator();

                for output in output_manager.outputs() {
                    let is_selected = self.selected_output_id == Some(output.id);
                    if ui.selectable_label(is_selected, &output.name).clicked() {
                        self.selected_output_id = Some(output.id);
                    }
                }

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button(locale.t("output-panel-add")).clicked() {
                        actions.push(OutputPanelAction::AddOutput);
                    }
                    if ui.button(locale.t("output-panel-remove")).clicked() {
                        if let Some(id) = self.selected_output_id {
                            actions.push(OutputPanelAction::RemoveOutput(id));
                        }
                    }
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            if let Some(id) = self.selected_output_id {
                if let Some(output) = output_manager.get_output_mut(id) {
                    self.render_output_details(ui, locale, output, monitors, actions);
                } else {
                    ui.label(locale.t("output-panel-no-output-selected"));
                }
            } else {
                ui.label(locale.t("output-panel-no-outputs"));
            }
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn render_output_details(
        &self,
        ui: &mut Ui,
        locale: &LocaleManager,
        output: &mut OutputConfig,
        monitors: &[MonitorInfo],
        actions: &mut Vec<OutputPanelAction>,
    ) {
        let mut changed = false;
        let mut output_clone = output.clone();

        ui.heading(locale.t("output-panel-details-header"));
        ui.separator();

        // --- Basic Settings ---
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(locale.t("output-panel-name"));
                changed |= ui.text_edit_singleline(&mut output_clone.name).changed();
            });

            ui.horizontal(|ui| {
                ui.label(locale.t("output-panel-monitor"));
                let no_device_text = locale.t("output-panel-no-device");
                let selected_monitor_text = output_clone
                    .monitor_name
                    .as_deref()
                    .unwrap_or(&no_device_text);

                egui::ComboBox::from_id_source("monitor_select")
                    .selected_text(selected_monitor_text)
                    .show_ui(ui, |ui| {
                        // Option for no monitor
                        if ui
                            .selectable_value(
                                &mut output_clone.monitor_name,
                                None,
                                locale.t("output-panel-no-device"),
                            )
                            .changed()
                        {
                            changed = true;
                        }

                        // Options for each detected monitor
                        for monitor in monitors {
                            if ui
                                .selectable_value(
                                    &mut output_clone.monitor_name,
                                    Some(monitor.name.clone()),
                                    monitor.display_string(),
                                )
                                .changed()
                            {
                                changed = true;
                            }
                        }
                    });
            });

            changed |= ui
                .checkbox(
                    &mut output_clone.fullscreen,
                    locale.t("output-panel-fullscreen"),
                )
                .changed();
        });

        // --- Resolution and Positioning ---
        ui.group(|ui| {
            ui.label(locale.t("output-panel-resolution"));
            ui.horizontal(|ui| {
                ui.label("W:");
                changed |= ui
                    .add(DragValue::new(&mut output_clone.resolution.0).speed(1.0))
                    .changed();
                ui.label("H:");
                changed |= ui
                    .add(DragValue::new(&mut output_clone.resolution.1).speed(1.0))
                    .changed();
            });

            ui.separator();
            ui.label(locale.t("output-panel-canvas-region"));

            ui.horizontal(|ui| {
                ui.label("X:");
                changed |= ui
                    .add(Slider::new(&mut output_clone.canvas_region.x, 0.0..=1.0))
                    .changed();
                ui.label("Y:");
                changed |= ui
                    .add(Slider::new(&mut output_clone.canvas_region.y, 0.0..=1.0))
                    .changed();
            });
            ui.horizontal(|ui| {
                ui.label("W:");
                changed |= ui
                    .add(Slider::new(
                        &mut output_clone.canvas_region.width,
                        0.0..=1.0,
                    ))
                    .changed();
                ui.label("H:");
                changed |= ui
                    .add(Slider::new(
                        &mut output_clone.canvas_region.height,
                        0.0..=1.0,
                    ))
                    .changed();
            });
        });

        // --- Actions and Tools ---
        ui.group(|ui| {
            ui.label(locale.t("output-panel-tools"));
            ui.horizontal(|ui| {
                if ui.button(locale.t("output-panel-test-pattern")).clicked() {
                    actions.push(OutputPanelAction::ShowTestPattern(output.id));
                }
                if ui
                    .button(locale.t("output-panel-color-calibration"))
                    .clicked()
                {
                    actions.push(OutputPanelAction::ShowColorCalibration(output.id));
                }
                if ui.button(locale.t("output-panel-edge-blend")).clicked() {
                    actions.push(OutputPanelAction::ShowEdgeBlend(output.id));
                }
            });
        });

        if changed {
            actions.push(OutputPanelAction::UpdateOutput(output_clone));
        }
    }
}
