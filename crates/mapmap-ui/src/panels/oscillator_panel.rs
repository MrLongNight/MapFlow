//! Egui-based Oscillator Control Panel

use crate::i18n::LocaleManager;
use egui::{ComboBox, DragValue, Ui, Window};
use mapmap_core::oscillator::{ColorMode, OscillatorConfig};

/// UI for the oscillator control panel.
#[derive(Debug, Clone)]
pub struct OscillatorPanel {

    /// Is the panel currently visible?
    pub visible: bool,
}

impl Default for OscillatorPanel {
    fn default() -> Self {
        Self {
            visible: false,
        }
    }
}

impl OscillatorPanel {
    /// Creates a new, default oscillator panel.
    pub fn new() -> Self {
        Self::default()
    }

    /// Renders the oscillator panel UI.
    ///
    /// Returns `true` if any value was changed by the user.
    pub fn render(&mut self, ctx: &egui::Context, locale: &LocaleManager, config: &mut OscillatorConfig) -> bool {
        let mut changed = false;
        let mut is_open = self.visible;

        if !is_open {
            return false;
        }

        Window::new(locale.t("oscillator-panel-title"))
            .open(&mut is_open)
            .resizable(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    changed |= ui
                        .toggle_value(&mut config.enabled, locale.t("oscillator-enable"))
                        .changed();
                });

                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    if ui
                        .collapsing(locale.t("oscillator-simulation-params"), |ui| {
                            self.show_simulation_params(ui, locale, config)
                        })
                        .body_returned
                        .unwrap_or(false)
                    {
                        changed = true;
                    }

                    if ui
                        .collapsing(locale.t("oscillator-distortion-params"), |ui| {
                            self.show_distortion_params(ui, locale, config)
                        })
                        .body_returned
                        .unwrap_or(false)
                    {
                        changed = true;
                    }

                    if ui
                        .collapsing(locale.t("oscillator-visual-params"), |ui| {
                            self.show_visual_params(ui, locale, config)
                        })
                        .body_returned
                        .unwrap_or(false)
                    {
                        changed = true;
                    }
                });
            });

        self.visible = is_open;
        changed
    }

    fn show_simulation_params(&mut self, ui: &mut Ui, locale: &LocaleManager, config: &mut OscillatorConfig) -> bool {
        let mut sim_changed = false;

        ui.horizontal(|ui| {
            ui.label(locale.t("oscillator-frequency-min"));
            sim_changed |= ui
                .add(DragValue::new(&mut config.frequency_min).speed(0.1))
                .changed();
        });

        ui.horizontal(|ui| {
            ui.label(locale.t("oscillator-frequency-max"));
            sim_changed |= ui
                .add(DragValue::new(&mut config.frequency_max).speed(0.1))
                .changed();
        });

        ui.horizontal(|ui| {
            ui.label(locale.t("oscillator-kernel-radius"));
            sim_changed |= ui
                .add(
                    DragValue::new(&mut config.kernel_radius)
                        .range(1.0..=64.0)
                        .speed(0.5),
                )
                .changed();
        });

        ui.horizontal(|ui| {
            ui.label(locale.t("oscillator-noise-amount"));
            sim_changed |= ui
                .add(
                    DragValue::new(&mut config.noise_amount)
                        .range(0.0..=1.0)
                        .speed(0.01),
                )
                .changed();
        });

        sim_changed
    }

    fn show_distortion_params(&mut self, ui: &mut Ui, locale: &LocaleManager, config: &mut OscillatorConfig) -> bool {
        let mut dist_changed = false;

        ui.horizontal(|ui| {
            ui.label(locale.t("oscillator-distortion-amount"));
            dist_changed |= ui
                .add(
                    DragValue::new(&mut config.distortion_amount)
                        .range(0.0..=1.0)
                        .speed(0.01),
                )
                .changed();
        });

        ui.horizontal(|ui| {
            ui.label(locale.t("oscillator-distortion-scale"));
            dist_changed |= ui
                .add(
                    DragValue::new(&mut config.distortion_scale)
                        .range(0.0..=0.1)
                        .speed(0.001),
                )
                .changed();
        });

        ui.horizontal(|ui| {
            ui.label(locale.t("oscillator-distortion-speed"));
            dist_changed |= ui
                .add(
                    DragValue::new(&mut config.distortion_speed)
                        .range(0.0..=4.0)
                        .speed(0.01),
                )
                .changed();
        });

        dist_changed
    }

    fn show_visual_params(&mut self, ui: &mut Ui, locale: &LocaleManager, config: &mut OscillatorConfig) -> bool {
        let mut viz_changed = false;

        ui.horizontal(|ui| {
            ui.label(locale.t("oscillator-overlay-opacity"));
            viz_changed |= ui
                .add(
                    DragValue::new(&mut config.overlay_opacity)
                        .range(0.0..=1.0)
                        .speed(0.01),
                )
                .changed();
        });

        ui.horizontal(|ui| {
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
        });

        viz_changed
    }
}
