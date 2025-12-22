//! Egui-based transform controls panel (Phase 6)
use crate::i18n::LocaleManager;
use egui::*;
use mapmap_core::ResizeMode;

#[derive(Debug, Clone, Default)]
pub struct TransformValues {
    pub position: (f32, f32),
    pub rotation: f32, // Z-axis rotation in degrees
    pub scale: (f32, f32),
    pub anchor: (f32, f32),
}

#[derive(Debug, Clone)]
pub enum TransformAction {
    UpdateTransform(TransformValues),
    ResetTransform,
    ApplyResizeMode(ResizeMode),
}

#[derive(Debug)]
pub struct TransformPanel {
    pub visible: bool,
    pub current_transform: TransformValues,
    last_action: Option<TransformAction>,
    selected_layer_name: Option<String>,
}

impl Default for TransformPanel {
    fn default() -> Self {
        Self {
            visible: true,
            current_transform: TransformValues::default(),
            last_action: None,
            selected_layer_name: None,
        }
    }
}

impl TransformPanel {
    /// Set the transform values to be displayed and edited.
    pub fn set_transform(&mut self, layer_name: &str, transform: &mapmap_core::Transform) {
        self.selected_layer_name = Some(layer_name.to_string());
        self.current_transform = TransformValues {
            position: (transform.position.x, transform.position.y),
            // The old panel used 3-axis rotation, but for 2D layers, Z is primary.
            // Sticking to the simplified single rotation slider as requested.
            rotation: transform.rotation.z.to_degrees(),
            scale: (transform.scale.x, transform.scale.y),
            anchor: (transform.anchor.x, transform.anchor.y),
        };
    }

    /// Clear the selected layer, showing placeholder text.
    pub fn clear_selection(&mut self) {
        self.selected_layer_name = None;
    }

    /// Take the last action performed in the panel.
    pub fn take_action(&mut self) -> Option<TransformAction> {
        self.last_action.take()
    }

    /// Render the transform panel.
    pub fn render(&mut self, ctx: &egui::Context, i18n: &LocaleManager) {
        if !self.visible {
            return;
        }

        let mut open = self.visible;
        egui::Window::new(i18n.t("panel-transforms"))
            .open(&mut open)
            .default_size([360.0, 520.0])
            .show(ctx, |ui| {
                ui.heading(i18n.t("header-transform-sys"));
                ui.separator();

                if let Some(name) = &self.selected_layer_name {
                    ui.label(format!("{}: {}", i18n.t("label-editing"), name));
                    ui.separator();

                    let mut changed = false;

                    // Position controls
                    ui.label(i18n.t("transform-position"));
                    ui.horizontal(|ui| {
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut self.current_transform.position.0)
                                    .speed(1.0)
                                    .prefix(i18n.t("drag-val-x")),
                            )
                            .changed();
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut self.current_transform.position.1)
                                    .speed(1.0)
                                    .prefix(i18n.t("drag-val-y")),
                            )
                            .changed();
                    });
                    changed |= ui
                        .add(Slider::new(
                            &mut self.current_transform.position.0,
                            -1000.0..=1000.0,
                        ))
                        .changed();
                    changed |= ui
                        .add(Slider::new(
                            &mut self.current_transform.position.1,
                            -1000.0..=1000.0,
                        ))
                        .changed();

                    ui.separator();

                    // Rotation control
                    ui.label(i18n.t("transform-rotation"));
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut self.current_transform.rotation, 0.0..=360.0)
                                .text(i18n.t("slider-z")),
                        )
                        .changed();
                    if ui.button(i18n.t("btn-reset-rotation")).clicked() {
                        self.current_transform.rotation = 0.0;
                        changed = true;
                    }
                    ui.separator();

                    // Scale controls
                    ui.label(i18n.t("transform-scale"));
                    ui.horizontal(|ui| {
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut self.current_transform.scale.0)
                                    .speed(0.01)
                                    .clamp_range(0.01..=5.0)
                                    .prefix(i18n.t("drag-val-w")),
                            )
                            .changed();
                        changed |= ui
                            .add(
                                egui::DragValue::new(&mut self.current_transform.scale.1)
                                    .speed(0.01)
                                    .clamp_range(0.01..=5.0)
                                    .prefix(i18n.t("drag-val-h")),
                            )
                            .changed();
                    });
                    changed |= ui
                        .add(Slider::new(&mut self.current_transform.scale.0, 0.1..=5.0))
                        .changed();
                    changed |= ui
                        .add(Slider::new(&mut self.current_transform.scale.1, 0.1..=5.0))
                        .changed();

                    if ui.button(i18n.t("btn-reset-scale")).clicked() {
                        self.current_transform.scale = (1.0, 1.0);
                        changed = true;
                    }
                    ui.separator();

                    // Anchor point controls
                    ui.label(i18n.t("label-anchor"));
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut self.current_transform.anchor.0, 0.0..=1.0)
                                .text(i18n.t("drag-val-x")),
                        )
                        .changed();
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut self.current_transform.anchor.1, 0.0..=1.0)
                                .text(i18n.t("drag-val-y")),
                        )
                        .changed();

                    if ui.button(i18n.t("btn-center-anchor")).clicked() {
                        self.current_transform.anchor = (0.5, 0.5);
                        changed = true;
                    }
                    ui.separator();

                    // Resize mode presets
                    ui.label(i18n.t("transform-presets"));

                    // By default, any change updates the transform.
                    if changed {
                        self.last_action = Some(TransformAction::UpdateTransform(
                            self.current_transform.clone(),
                        ));
                    }

                    // More specific actions below can overwrite the default action.
                    ui.horizontal(|ui| {
                        if ui.button(i18n.t("transform-fill")).clicked() {
                            self.last_action =
                                Some(TransformAction::ApplyResizeMode(ResizeMode::Fill));
                        }
                        if ui.button(i18n.t("btn-resize-fit")).clicked() {
                            self.last_action =
                                Some(TransformAction::ApplyResizeMode(ResizeMode::Fit));
                        }
                    });
                    ui.horizontal(|ui| {
                        if ui.button(i18n.t("btn-resize-stretch")).clicked() {
                            self.last_action =
                                Some(TransformAction::ApplyResizeMode(ResizeMode::Stretch));
                        }
                        if ui.button(i18n.t("btn-resize-original")).clicked() {
                            self.last_action =
                                Some(TransformAction::ApplyResizeMode(ResizeMode::Original));
                        }
                    });

                    ui.separator();

                    // Reset button is the most specific, so it comes last.
                    if ui.button(i18n.t("btn-reset-defaults")).clicked() {
                        self.last_action = Some(TransformAction::ResetTransform);
                    }
                } else {
                    ui.label(i18n.t("transform-no-layer"));
                    ui.label(i18n.t("transform-select-tip"));
                }
            });
        self.visible = open;
    }
}
