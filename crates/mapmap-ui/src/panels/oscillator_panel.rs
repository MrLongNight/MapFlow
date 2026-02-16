//! Egui-based Oscillator Control Panel

use crate::core::theme::colors;
use crate::i18n::LocaleManager;
use crate::responsive::ResponsiveLayout;
use crate::widgets::custom;use egui::{ComboBox, Ui, Window};
use mapmap_core::oscillator::{ColorMode, OscillatorConfig};

/// UI for the oscillator control panel.
#[derive(Debug, Clone, Default)]
pub struct OscillatorPanel {
    /// Is the panel currently visible?
    pub visible: bool,
}

impl OscillatorPanel {
    /// Creates a new, default oscillator panel.
    pub fn new() -> Self {
        Self::default()
    }

    /// Renders the oscillator panel UI.
    ///
    /// Returns `true` if any value was changed by the user.
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        locale: &LocaleManager,
        config: &mut OscillatorConfig,
    ) -> bool {
        let mut changed = false;
        let mut is_open = self.visible;

        if !is_open {
            return false;
        }

        let layout = ResponsiveLayout::new(ctx);
        let window_size = layout.window_size(400.0, 500.0);

        Window::new(locale.t("oscillator-panel-title"))
            .open(&mut is_open)
            .resizable(true)
            .default_size(window_size)
            .scroll([false, true])
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    changed |= ui
                        .toggle_value(&mut config.enabled, locale.t("oscillator-enable"))
                        .changed();
                });

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Simulation Section
                    ui.add_space(4.0);
                    custom::render_header(ui, &locale.t("oscillator-simulation-params"));
                    egui::Frame::NONE
                        .fill(colors::DARK_GREY)
                        .inner_margin(4.0)
                        .show(ui, |ui| {
                            if self.show_simulation_params(ui, locale, config) {
                                changed = true;
                            }
                        });

                    // Distortion Section
                    ui.add_space(4.0);
                    custom::render_header(ui, &locale.t("oscillator-distortion-params"));
                    egui::Frame::NONE
                        .fill(colors::DARK_GREY)
                        .inner_margin(4.0)
                        .show(ui, |ui| {
                            if self.show_distortion_params(ui, locale, config) {
                                changed = true;
                            }
                        });
                    // Visual Section
                    ui.add_space(4.0);
                    custom::render_header(ui, &locale.t("oscillator-visual-params"));
                    egui::Frame::NONE
                        .fill(colors::DARK_GREY)
                        .inner_margin(4.0)
                        .show(ui, |ui| {
                            if self.show_visual_params(ui, locale, config) {
                                changed = true;
                            }
                        });
                });
            });

        self.visible = is_open;
        changed
    }

    fn show_simulation_params(
        &mut self,
        ui: &mut Ui,
        locale: &LocaleManager,
        config: &mut OscillatorConfig,
    ) -> bool {
        let mut sim_changed = false;

        egui::Grid::new("sim_params_grid")
            .num_columns(2)
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
                ui.label(locale.t("oscillator-frequency-min"));
                sim_changed |= custom::styled_drag_value(
                    ui,
                    &mut config.frequency_min,
                    0.1,
                    0.0..=100.0,
                    0.5,
                    "",
                    " Hz",
                )
                .changed();
                ui.end_row();

                ui.label(locale.t("oscillator-frequency-max"));
                sim_changed |= custom::styled_drag_value(
                    ui,
                    &mut config.frequency_max,
                    0.1,
                    0.0..=100.0,
                    2.0,
                    "",
                    " Hz",
                )
                .changed();
                ui.end_row();

                ui.label(locale.t("oscillator-kernel-radius"));
                sim_changed |= custom::styled_drag_value(
                    ui,
                    &mut config.kernel_radius,
                    0.5,
                    1.0..=64.0,
                    16.0,
                    "",
                    " px",
                )
                .changed();
                ui.end_row();

                ui.label(locale.t("oscillator-noise-amount"));
                sim_changed |= custom::styled_drag_value(
                    ui,
                    &mut config.noise_amount,
                    0.01,
                    0.0..=1.0,
                    0.1,
                    "",
                    "",
                )
                .changed();
                ui.end_row();
            });

        sim_changed
    }

    fn show_distortion_params(
        &mut self,
        ui: &mut Ui,
        locale: &LocaleManager,
        config: &mut OscillatorConfig,
    ) -> bool {
        let mut dist_changed = false;

        egui::Grid::new("dist_params_grid")
            .num_columns(2)
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
                ui.label(locale.t("oscillator-distortion-amount"));
                dist_changed |= custom::styled_drag_value(
                    ui,
                    &mut config.distortion_amount,
                    0.01,
                    0.0..=1.0,
                    0.5,
                    "",
                    "",
                )
                .changed();
                ui.end_row();

                ui.label(locale.t("oscillator-distortion-scale"));
                dist_changed |= custom::styled_drag_value(
                    ui,
                    &mut config.distortion_scale,
                    0.001,
                    0.0..=0.1,
                    0.02,
                    "",
                    "",
                )
                .changed();
                ui.end_row();

                ui.label(locale.t("oscillator-distortion-speed"));
                dist_changed |= custom::styled_drag_value(
                    ui,
                    &mut config.distortion_speed,
                    0.01,
                    0.0..=4.0,
                    1.0,
                    "",
                    "x",
                )
                .changed();
                ui.end_row();
            });

        dist_changed    }

    fn show_visual_params(
        &mut self,
        ui: &mut Ui,
        locale: &LocaleManager,
        config: &mut OscillatorConfig,
    ) -> bool {
        let mut viz_changed = false;

        egui::Grid::new("viz_params_grid")
            .num_columns(2)
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
                ui.label(locale.t("oscillator-overlay-opacity"));
                viz_changed |= custom::styled_drag_value(
                    ui,
                    &mut config.overlay_opacity,
                    0.01,
                    0.0..=1.0,
                    0.0,
                    "",
                    "",
                )
                .changed();
                ui.end_row();
                ui.label(locale.t("oscillator-color-mode"));
                let selected_text = format!("{:?}", config.color_mode);
                viz_changed |= ComboBox::from_id_salt("color_mode")
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        let mut changed = false;
                        changed |= ui
                            .selectable_value(
                                &mut config.color_mode,
                                ColorMode::Off,
                                locale.t("oscillator-color-mode-off"),
                            )
                            .changed();
                        changed |= ui
                            .selectable_value(
                                &mut config.color_mode,
                                ColorMode::Rainbow,
                                locale.t("oscillator-color-mode-rainbow"),
                            )
                            .changed();
                        changed |= ui
                            .selectable_value(
                                &mut config.color_mode,
                                ColorMode::BlackWhite,
                                locale.t("oscillator-color-mode-black-white"),
                            )
                            .changed();
                        changed |= ui
                            .selectable_value(
                                &mut config.color_mode,
                                ColorMode::Complementary,
                                locale.t("oscillator-color-mode-complementary"),
                            )
                            .changed();
                        changed
                    })
                    .inner
                    .unwrap_or(false);
                ui.end_row();
            });

        viz_changed
    }
}
