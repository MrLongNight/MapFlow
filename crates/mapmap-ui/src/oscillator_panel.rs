use egui::{Context, Slider, Ui};
use mapmap_core::{
    ColorMode, CoordinateMode, OscillatorConfig, PhaseInitMode, RingParams, SimulationResolution,
};

use crate::LocaleManager;

#[derive(Debug, Default)]
pub struct OscillatorPanel {
    pub visible: bool,
    config: OscillatorConfig,
    action: Option<OscillatorAction>,
}

#[derive(Debug, Clone)]
pub enum OscillatorAction {
    UpdateConfig(OscillatorConfig),
}

impl OscillatorPanel {
    pub fn new(config: &OscillatorConfig) -> Self {
        Self {
            visible: true,
            config: config.clone(),
            action: None,
        }
    }

    pub fn show(&mut self, ctx: &Context, locale: &LocaleManager) {
        if !self.visible {
            return;
        }

        let mut open = self.visible;
        let original_config = self.config.clone();

        egui::Window::new(locale.t("panel-oscillator"))
            .open(&mut open)
            .vscroll(true)
            .show(ctx, |ui| {
                self.ui(ui, locale);
            });

        self.visible = open;

        if self.config != original_config {
            self.action = Some(OscillatorAction::UpdateConfig(self.config.clone()));
        }
    }

    fn ui(&mut self, ui: &mut Ui, locale: &LocaleManager) {
        // Master enable
        ui.checkbox(&mut self.config.enabled, locale.t("check-enable"));
        ui.separator();

        // Presets
        ui.label(format!("{}:", locale.t("header-quick-presets")));
        ui.horizontal(|ui| {
            if ui.button(locale.t("btn-subtle")).clicked() {
                self.config = OscillatorConfig::preset_subtle();
            }
            if ui.button(locale.t("btn-dramatic")).clicked() {
                self.config = OscillatorConfig::preset_dramatic();
            }
            if ui.button(locale.t("btn-rings")).clicked() {
                self.config = OscillatorConfig::preset_rings();
            }
            if ui.button(locale.t("btn-reset")).clicked() {
                self.config = OscillatorConfig::default();
            }
        });
        ui.separator();

        // Distortion
        ui.strong(locale.t("header-distortion"));
        ui.add(
            Slider::new(&mut self.config.distortion_amount, 0.0..=1.0)
                .text(locale.t("label-amount")),
        );
        ui.add(
            Slider::new(&mut self.config.distortion_scale, 0.0..=0.1)
                .text(locale.t("label-dist-scale")),
        );
        ui.add(
            Slider::new(&mut self.config.distortion_speed, 0.0..=5.0)
                .text(locale.t("label-dist-speed")),
        );
        ui.separator();

        // Visual Overlay
        ui.strong(locale.t("header-visual-overlay"));
        ui.add(
            Slider::new(&mut self.config.overlay_opacity, 0.0..=1.0)
                .text(locale.t("label-overlay-opacity")),
        );

        egui::ComboBox::from_label(locale.t("label-color-mode"))
            .selected_text(format!("{:?}", self.config.color_mode))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.config.color_mode, ColorMode::Off, "Off");
                ui.selectable_value(&mut self.config.color_mode, ColorMode::Rainbow, "Rainbow");
                ui.selectable_value(
                    &mut self.config.color_mode,
                    ColorMode::BlackWhite,
                    "Black & White",
                );
                ui.selectable_value(
                    &mut self.config.color_mode,
                    ColorMode::Complementary,
                    "Complementary",
                );
            });
        ui.separator();

        // Simulation
        ui.strong(locale.t("header-simulation"));
        egui::ComboBox::from_label(locale.t("label-resolution"))
            .selected_text(format!("{:?}", self.config.simulation_resolution))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.config.simulation_resolution,
                    SimulationResolution::Low,
                    "Low (128x128)",
                );
                ui.selectable_value(
                    &mut self.config.simulation_resolution,
                    SimulationResolution::Medium,
                    "Medium (256x256)",
                );
                ui.selectable_value(
                    &mut self.config.simulation_resolution,
                    SimulationResolution::High,
                    "High (512x512)",
                );
            });

        ui.add(
            Slider::new(&mut self.config.kernel_radius, 1.0..=64.0)
                .text(locale.t("label-kernel-radius")),
        );
        ui.add(
            Slider::new(&mut self.config.noise_amount, 0.0..=1.0)
                .text(locale.t("label-noise-amount")),
        );
        ui.add(
            Slider::new(&mut self.config.frequency_min, 0.0..=10.0)
                .text(locale.t("label-freq-min")),
        );
        ui.add(
            Slider::new(&mut self.config.frequency_max, 0.0..=10.0)
                .text(locale.t("label-freq-max")),
        );
        ui.separator();

        // Coordinate mode
        egui::ComboBox::from_label(locale.t("label-coordinate-mode"))
            .selected_text(format!("{:?}", self.config.coordinate_mode))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.config.coordinate_mode,
                    CoordinateMode::Cartesian,
                    "Cartesian",
                );
                ui.selectable_value(
                    &mut self.config.coordinate_mode,
                    CoordinateMode::LogPolar,
                    "Log-Polar",
                );
            });

        // Phase init mode
        egui::ComboBox::from_label(locale.t("label-phase-init"))
            .selected_text(format!("{:?}", self.config.phase_init_mode))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.config.phase_init_mode,
                    PhaseInitMode::Random,
                    "Random",
                );
                ui.selectable_value(
                    &mut self.config.phase_init_mode,
                    PhaseInitMode::Uniform,
                    "Uniform",
                );
                ui.selectable_value(
                    &mut self.config.phase_init_mode,
                    PhaseInitMode::PlaneHorizontal,
                    "Plane H",
                );
                ui.selectable_value(
                    &mut self.config.phase_init_mode,
                    PhaseInitMode::PlaneVertical,
                    "Plane V",
                );
                ui.selectable_value(
                    &mut self.config.phase_init_mode,
                    PhaseInitMode::PlaneDiagonal,
                    "Diagonal",
                );
            });
        ui.separator();

        // Coupling rings
        egui::CollapsingHeader::new(locale.t("header-coupling"))
            .default_open(true)
            .show(ui, |ui| {
                for i in 0..4 {
                    ui.group(|ui| {
                        ui.strong(format!("Ring {}", i + 1));
                        ui.add(
                            Slider::new(&mut self.config.rings[i].distance, 0.0..=1.0)
                                .text(locale.t("label-distance")),
                        );
                        ui.add(
                            Slider::new(&mut self.config.rings[i].width, 0.0..=1.0)
                                .text(locale.t("label-width")),
                        );
                        ui.add(
                            Slider::new(&mut self.config.rings[i].coupling, -5.0..=5.0)
                                .text(locale.t("label-diff-coupling")),
                        );

                        ui.horizontal(|ui| {
                            if ui.button(locale.t("btn-reset-ring")).clicked() {
                                self.config.rings[i] = RingParams::default();
                            }
                            if ui.button(locale.t("btn-clear-ring")).clicked() {
                                self.config.rings[i] = RingParams {
                                    distance: 0.0,
                                    width: 0.0,
                                    coupling: 0.0,
                                };
                            }
                        });
                    });
                }
            });
    }

    pub fn take_action(&mut self) -> Option<OscillatorAction> {
        self.action.take()
    }

    pub fn set_config(&mut self, config: &OscillatorConfig) {
        self.config = config.clone();
    }
}
