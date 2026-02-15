//! Egui-based Oscillator Control Panel

use crate::i18n::LocaleManager;
use crate::responsive::ResponsiveLayout;

use egui::{ComboBox, Ui, Window};
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
                    ui.add_space(4.0);
                    if ui
                        .collapsing(locale.t("oscillator-simulation-params"), |ui| {
                            self.show_simulation_params(ui, locale, config)
                        })
                        .body_returned
                        .unwrap_or(false)
                    {
                        changed = true;
                    }
                    ui.add_space(4.0);

                    if ui
                        .collapsing(locale.t("oscillator-distortion-params"), |ui| {
                            self.show_distortion_params(ui, locale, config)
                        })
                        .body_returned
                        .unwrap_or(false)
                    {
                        changed = true;
                    }
                    ui.add_space(4.0);

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

    fn show_simulation_params(
        &mut self,
        ui: &mut Ui,
        locale: &LocaleManager,
        config: &mut OscillatorConfig,
    ) -> bool {
        let mut sim_changed = false;



        sim_changed
    }

    fn show_distortion_params(
        &mut self,
        ui: &mut Ui,
        locale: &LocaleManager,
        config: &mut OscillatorConfig,
    ) -> bool {
        let mut dist_changed = false;



        dist_changed
    }

    fn show_visual_params(
        &mut self,
        ui: &mut Ui,
        locale: &LocaleManager,
        config: &mut OscillatorConfig,
    ) -> bool {
        let mut viz_changed = false;



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

