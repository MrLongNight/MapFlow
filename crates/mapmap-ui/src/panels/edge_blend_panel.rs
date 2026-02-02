//! Egui-based Edge Blend and Color Calibration Panel
use crate::i18n::LocaleManager;
use egui::*;
use mapmap_core::{output::OutputConfig, ColorCalibration, EdgeBlendConfig};

#[derive(Debug, Clone, PartialEq)]
pub struct EdgeBlendValues {
    pub left_enabled: bool,
    pub left_width: f32,
    pub left_offset: f32,
    pub right_enabled: bool,
    pub right_width: f32,
    pub right_offset: f32,
    pub top_enabled: bool,
    pub top_width: f32,
    pub top_offset: f32,
    pub bottom_enabled: bool,
    pub bottom_width: f32,
    pub bottom_offset: f32,
    pub gamma: f32,
}

impl From<&EdgeBlendConfig> for EdgeBlendValues {
    fn from(config: &EdgeBlendConfig) -> Self {
        Self {
            left_enabled: config.left.enabled,
            left_width: config.left.width,
            left_offset: config.left.offset,
            right_enabled: config.right.enabled,
            right_width: config.right.width,
            right_offset: config.right.offset,
            top_enabled: config.top.enabled,
            top_width: config.top.width,
            top_offset: config.top.offset,
            bottom_enabled: config.bottom.enabled,
            bottom_width: config.bottom.width,
            bottom_offset: config.bottom.offset,
            gamma: config.gamma,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorCalibrationValues {
    pub brightness: f32,
    pub contrast: f32,
    pub gamma_r: f32,
    pub gamma_g: f32,
    pub gamma_b: f32,
    pub saturation: f32,
    pub color_temp: f32,
}

impl From<&ColorCalibration> for ColorCalibrationValues {
    fn from(cal: &ColorCalibration) -> Self {
        Self {
            brightness: cal.brightness,
            contrast: cal.contrast,
            gamma_r: cal.gamma.x,
            gamma_g: cal.gamma.y,
            gamma_b: cal.gamma_b,
            saturation: cal.saturation,
            color_temp: cal.color_temp,
        }
    }
}

#[derive(Debug, Clone)]
pub enum EdgeBlendAction {
    UpdateEdgeBlend(u64, EdgeBlendValues),
    UpdateColorCalibration(u64, ColorCalibrationValues),
    ResetEdgeBlend(u64),
    ResetColorCalibration(u64),
}

#[derive(Debug, Default)]
pub struct EdgeBlendPanel {
    pub visible: bool,
    pub selected_output_id: Option<u64>,
    pub selected_output_name: Option<String>,

    // Local state for the UI controls
    edge_blend_values: Option<EdgeBlendValues>,
    color_calibration_values: Option<ColorCalibrationValues>,

    last_action: Option<EdgeBlendAction>,
}

impl EdgeBlendPanel {
    pub fn set_selected_output(&mut self, output: &OutputConfig) {
        self.selected_output_id = Some(output.id);
        self.selected_output_name = Some(output.name.clone());
        self.edge_blend_values = Some((&output.edge_blend).into());
        self.color_calibration_values = Some((&output.color_calibration).into());
    }

    pub fn clear_selection(&mut self) {
        self.selected_output_id = None;
        self.selected_output_name = None;
        self.edge_blend_values = None;
        self.color_calibration_values = None;
    }

    pub fn take_action(&mut self) -> Option<EdgeBlendAction> {
        self.last_action.take()
    }

    pub fn show(&mut self, ctx: &egui::Context, i18n: &LocaleManager) {
        if !self.visible {
            return;
        }

        let mut open = self.visible;
        egui::Window::new(i18n.t("panel-edge-blend-color"))
            .open(&mut open)
            .default_size([380.0, 600.0])
            .show(ctx, |ui| {
                if let (Some(output_id), Some(output_name)) =
                    (self.selected_output_id, &self.selected_output_name)
                {
                    ui.heading(i18n.t_args("label-output", &[("name", output_name)]));
                    ui.separator();

                    self.show_edge_blend_controls(ui, i18n, output_id);
                    ui.separator();
                    self.show_color_calibration_controls(ui, i18n, output_id);
                } else {
                    ui.label(i18n.t("edge-blend-no-output"));
                }
            });
        self.visible = open;
    }

    fn show_edge_blend_controls(
        &mut self,
        ui: &mut egui::Ui,
        i18n: &LocaleManager,
        output_id: u64,
    ) {
        let mut changed = false;
        if let Some(values) = &mut self.edge_blend_values {
            ui.collapsing(i18n.t("header-edge-blend"), |ui| {
                // Left
                changed |= ui
                    .checkbox(&mut values.left_enabled, i18n.t("check-left"))
                    .changed();
                if values.left_enabled {
                    ui.indent("left_indent", |ui| {
                        changed |= ui
                            .add(
                                Slider::new(&mut values.left_width, 0.0..=0.5)
                                    .text(i18n.t("label-width")),
                            )
                            .changed();
                        changed |= ui
                            .add(
                                Slider::new(&mut values.left_offset, -0.1..=0.1)
                                    .text(i18n.t("label-offset")),
                            )
                            .changed();
                    });
                }
                // Right
                changed |= ui
                    .checkbox(&mut values.right_enabled, i18n.t("check-right"))
                    .changed();
                if values.right_enabled {
                    ui.indent("right_indent", |ui| {
                        changed |= ui
                            .add(
                                Slider::new(&mut values.right_width, 0.0..=0.5)
                                    .text(i18n.t("label-width")),
                            )
                            .changed();
                        changed |= ui
                            .add(
                                Slider::new(&mut values.right_offset, -0.1..=0.1)
                                    .text(i18n.t("label-offset")),
                            )
                            .changed();
                    });
                }
                // Top
                changed |= ui
                    .checkbox(&mut values.top_enabled, i18n.t("check-top"))
                    .changed();
                if values.top_enabled {
                    ui.indent("top_indent", |ui| {
                        changed |= ui
                            .add(
                                Slider::new(&mut values.top_width, 0.0..=0.5)
                                    .text(i18n.t("label-width")),
                            )
                            .changed();
                        changed |= ui
                            .add(
                                Slider::new(&mut values.top_offset, -0.1..=0.1)
                                    .text(i18n.t("label-offset")),
                            )
                            .changed();
                    });
                }
                // Bottom
                changed |= ui
                    .checkbox(&mut values.bottom_enabled, i18n.t("check-bottom"))
                    .changed();
                if values.bottom_enabled {
                    ui.indent("bottom_indent", |ui| {
                        changed |= ui
                            .add(
                                Slider::new(&mut values.bottom_width, 0.0..=0.5)
                                    .text(i18n.t("label-width")),
                            )
                            .changed();
                        changed |= ui
                            .add(
                                Slider::new(&mut values.bottom_offset, -0.1..=0.1)
                                    .text(i18n.t("label-offset")),
                            )
                            .changed();
                    });
                }

                ui.separator();
                changed |= ui
                    .add(Slider::new(&mut values.gamma, 1.0..=3.0).text(i18n.t("label-gamma")))
                    .changed();

                if ui.button(i18n.t("btn-reset-defaults")).clicked() {
                    self.last_action = Some(EdgeBlendAction::ResetEdgeBlend(output_id));
                }
            });

            if changed {
                self.last_action =
                    Some(EdgeBlendAction::UpdateEdgeBlend(output_id, values.clone()));
            }
        }
    }

    fn show_color_calibration_controls(
        &mut self,
        ui: &mut egui::Ui,
        i18n: &LocaleManager,
        output_id: u64,
    ) {
        let mut changed = false;
        if let Some(values) = &mut self.color_calibration_values {
            ui.collapsing(i18n.t("header-color-calibration"), |ui| {
                changed |= ui
                    .add(
                        Slider::new(&mut values.brightness, -1.0..=1.0)
                            .text(i18n.t("label-brightness")),
                    )
                    .changed();
                changed |= ui
                    .add(
                        Slider::new(&mut values.contrast, 0.0..=2.0).text(i18n.t("label-contrast")),
                    )
                    .changed();
                changed |= ui
                    .add(
                        Slider::new(&mut values.saturation, 0.0..=2.0)
                            .text(i18n.t("label-saturation")),
                    )
                    .changed();

                ui.separator();
                ui.label(i18n.t("label-gamma-channels"));
                changed |= ui
                    .add(
                        Slider::new(&mut values.gamma_r, 0.5..=3.0).text(i18n.t("label-gamma-red")),
                    )
                    .changed();
                changed |= ui
                    .add(
                        Slider::new(&mut values.gamma_g, 0.5..=3.0)
                            .text(i18n.t("label-gamma-green")),
                    )
                    .changed();
                changed |= ui
                    .add(
                        Slider::new(&mut values.gamma_b, 0.5..=3.0)
                            .text(i18n.t("label-gamma-blue")),
                    )
                    .changed();

                ui.separator();
                changed |= ui
                    .add(
                        Slider::new(&mut values.color_temp, 2000.0..=10000.0)
                            .text(i18n.t("label-color-temp")),
                    )
                    .changed();

                if ui.button(i18n.t("btn-reset-defaults")).clicked() {
                    self.last_action = Some(EdgeBlendAction::ResetColorCalibration(output_id));
                }
            });

            if changed {
                self.last_action = Some(EdgeBlendAction::UpdateColorCalibration(
                    output_id,
                    values.clone(),
                ));
            }
        }
    }
}
