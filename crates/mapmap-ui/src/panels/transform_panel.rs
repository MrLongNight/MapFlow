//! Egui-based transform controls panel (Phase 6)
use crate::i18n::LocaleManager;
use crate::theme::colors;
use crate::widgets::custom::{styled_button, styled_drag_value, styled_slider};
use crate::widgets::panel::{cyber_panel_frame, render_panel_header};
use egui::*;
use mapmap_core::ResizeMode;

#[derive(Debug, Clone, Default)]
pub struct TransformValues {
    /// 3D position coordinates [x, y, z].
    pub position: (f32, f32),
    /// Rotation angles in degrees.
    pub rotation: f32, // Z-axis rotation in degrees
    /// Scale factors for the object's dimensions.
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

// Helper for labeled rows (moved outside to avoid impl Trait in closure issues)
fn labeled_row(ui: &mut Ui, label: &str, content: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), content);
    });
    ui.add_space(4.0);
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
            .frame(cyber_panel_frame(&ctx.style()))
            .show(ctx, |ui| {
                render_panel_header(ui, &i18n.t("header-transform-sys"), |_| {});
                ui.add_space(8.0);

                if let Some(name) = &self.selected_layer_name {
                    ui.label(
                        egui::RichText::new(format!("{}: {}", i18n.t("label-editing"), name))
                            .color(colors::CYAN_ACCENT),
                    );
                    ui.add_space(8.0);

                    let mut changed = false;

                    // Position
                    ui.label(egui::RichText::new(i18n.t("transform-position")).strong());
                    ui.horizontal(|ui| {
                        changed |= styled_drag_value(
                            ui,
                            &mut self.current_transform.position.0,
                            1.0,
                            -10000.0..=10000.0,
                            0.0,
                            "X: ",
                            "px",
                        )
                        .changed();
                        changed |= styled_drag_value(
                            ui,
                            &mut self.current_transform.position.1,
                            1.0,
                            -10000.0..=10000.0,
                            0.0,
                            "Y: ",
                            "px",
                        )
                        .changed();
                    });
                    changed |= styled_slider(
                        ui,
                        &mut self.current_transform.position.0,
                        -1000.0..=1000.0,
                        0.0,
                    )
                    .changed();
                    changed |= styled_slider(
                        ui,
                        &mut self.current_transform.position.1,
                        -1000.0..=1000.0,
                        0.0,
                    )
                    .changed();

                    ui.add_space(8.0);

                    // Rotation
                    labeled_row(ui, &i18n.t("transform-rotation"), |ui| {
                        if styled_button(ui, &i18n.t("btn-reset-rotation"), false).clicked() {
                            self.current_transform.rotation = 0.0;
                            changed = true;
                        }
                    });
                    changed |=
                        styled_slider(ui, &mut self.current_transform.rotation, 0.0..=360.0, 0.0)
                            .changed();

                    ui.add_space(8.0);

                    // Scale
                    labeled_row(ui, &i18n.t("transform-scale"), |ui| {
                        if styled_button(ui, &i18n.t("btn-reset-scale"), false).clicked() {
                            self.current_transform.scale = (1.0, 1.0);
                            changed = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        changed |= styled_drag_value(
                            ui,
                            &mut self.current_transform.scale.0,
                            0.01,
                            0.01..=10.0,
                            1.0,
                            "W: ",
                            "x",
                        )
                        .changed();
                        changed |= styled_drag_value(
                            ui,
                            &mut self.current_transform.scale.1,
                            0.01,
                            0.01..=10.0,
                            1.0,
                            "H: ",
                            "x",
                        )
                        .changed();
                    });
                    changed |=
                        styled_slider(ui, &mut self.current_transform.scale.0, 0.1..=5.0, 1.0)
                            .changed();
                    changed |=
                        styled_slider(ui, &mut self.current_transform.scale.1, 0.1..=5.0, 1.0)
                            .changed();

                    ui.add_space(8.0);

                    // Anchor
                    labeled_row(ui, &i18n.t("label-anchor"), |ui| {
                        if styled_button(ui, &i18n.t("btn-center-anchor"), false).clicked() {
                            self.current_transform.anchor = (0.5, 0.5);
                            changed = true;
                        }
                    });
                    changed |=
                        styled_slider(ui, &mut self.current_transform.anchor.0, 0.0..=1.0, 0.5)
                            .changed();
                    changed |=
                        styled_slider(ui, &mut self.current_transform.anchor.1, 0.0..=1.0, 0.5)
                            .changed();

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Resize Presets
                    ui.label(egui::RichText::new(i18n.t("transform-presets")).strong());

                    // By default, any change updates the transform.
                    if changed {
                        self.last_action = Some(TransformAction::UpdateTransform(
                            self.current_transform.clone(),
                        ));
                    }

                    // Preset buttons
                    ui.horizontal(|ui| {
                        if styled_button(ui, &i18n.t("transform-fill"), false).clicked() {
                            self.last_action =
                                Some(TransformAction::ApplyResizeMode(ResizeMode::Fill));
                        }
                        if styled_button(ui, &i18n.t("btn-resize-fit"), false).clicked() {
                            self.last_action =
                                Some(TransformAction::ApplyResizeMode(ResizeMode::Fit));
                        }
                    });
                    ui.horizontal(|ui| {
                        if styled_button(ui, &i18n.t("btn-resize-stretch"), false).clicked() {
                            self.last_action =
                                Some(TransformAction::ApplyResizeMode(ResizeMode::Stretch));
                        }
                        if styled_button(ui, &i18n.t("btn-resize-original"), false).clicked() {
                            self.last_action =
                                Some(TransformAction::ApplyResizeMode(ResizeMode::Original));
                        }
                    });

                    ui.add_space(16.0);

                    // Reset All
                    if styled_button(ui, &i18n.t("btn-reset-defaults"), false).clicked() {
                        self.last_action = Some(TransformAction::ResetTransform);
                    }
                } else {
                    ui.vertical_centered(|ui| {
                        ui.add_space(40.0);
                        ui.label(
                            egui::RichText::new("∅")
                                .size(48.0)
                                .color(colors::CYAN_ACCENT.linear_multiply(0.3)),
                        );
                        ui.add_space(16.0);
                        ui.label(i18n.t("transform-no-layer"));
                        ui.label(
                            egui::RichText::new(i18n.t("transform-select-tip"))
                                .weak()
                                .italics(),
                        );
                    });
                }
            });
        self.visible = open;
    }
}
