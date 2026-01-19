use egui::{ComboBox, Slider, Ui, Window};
use mapmap_core::{
    ColorMode, CoordinateMode, OscillatorConfig, PhaseInitMode, RingParams, SimulationResolution,
};

use crate::LocaleManager;

/// Panel for controlling oscillator distortion effects
#[derive(Default)]
pub struct OscillatorPanel {
    pub visible: bool,
}

impl OscillatorPanel {
    /// Render the oscillator panel
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        i18n: &LocaleManager,
        config: &mut OscillatorConfig,
    ) {
        if !self.visible {
            return;
        }

        let mut visible = self.visible;
        Window::new(i18n.t("panel-oscillator"))
            .open(&mut visible)
            .default_width(300.0)
            .show(ctx, |ui| {
                Self::ui(ui, i18n, config);
            });
        self.visible = visible;
    }

    /// Inner UI rendering logic
    fn ui(ui: &mut Ui, i18n: &LocaleManager, config: &mut OscillatorConfig) {
        // Master enable
        ui.checkbox(&mut config.enabled, i18n.t("check-enable"));
        ui.separator();

        // Preset buttons
        ui.label(format!("{}:", i18n.t("header-quick-presets")));
        ui.horizontal(|ui| {
            if ui.button(i18n.t("btn-subtle")).clicked() {
                *config = OscillatorConfig::preset_subtle();
            }
            if ui.button(i18n.t("btn-dramatic")).clicked() {
                *config = OscillatorConfig::preset_dramatic();
            }
            if ui.button(i18n.t("btn-rings")).clicked() {
                *config = OscillatorConfig::preset_rings();
            }
            if ui.button(i18n.t("btn-reset")).clicked() {
                *config = OscillatorConfig::default();
            }
        });

        ui.separator();

        // Distortion parameters
        ui.label(i18n.t("header-distortion"));
        ui.add(Slider::new(&mut config.distortion_amount, 0.0..=1.0).text(i18n.t("label-amount")))
            .on_hover_text("Intensity of the distortion effect");

        ui.add(
            Slider::new(&mut config.distortion_scale, 0.0..=0.1)
                .text(i18n.t("label-dist-scale")),
        )
        .on_hover_text("Spatial scale of distortion");

        ui.add(
            Slider::new(&mut config.distortion_speed, 0.0..=5.0)
                .text(i18n.t("label-dist-speed")),
        )
        .on_hover_text("Animation speed");

        ui.separator();

        // Visual overlay
        ui.label(i18n.t("header-visual-overlay"));
        ui.add(
            Slider::new(&mut config.overlay_opacity, 0.0..=1.0)
                .text(i18n.t("label-overlay-opacity")),
        );

        // Color mode combo
        ComboBox::from_label(i18n.t("label-color-mode"))
            .selected_text(match config.color_mode {
                ColorMode::Off => "Off",
                ColorMode::Rainbow => "Rainbow",
                ColorMode::BlackWhite => "Black & White",
                ColorMode::Complementary => "Complementary",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut config.color_mode, ColorMode::Off, "Off");
                ui.selectable_value(&mut config.color_mode, ColorMode::Rainbow, "Rainbow");
                ui.selectable_value(
                    &mut config.color_mode,
                    ColorMode::BlackWhite,
                    "Black & White",
                );
                ui.selectable_value(
                    &mut config.color_mode,
                    ColorMode::Complementary,
                    "Complementary",
                );
            });

        ui.separator();

        // Simulation parameters
        ui.label(i18n.t("header-simulation"));

        // Resolution combo
        ComboBox::from_label(i18n.t("label-resolution"))
            .selected_text(match config.simulation_resolution {
                SimulationResolution::Low => "Low (128x128)",
                SimulationResolution::Medium => "Medium (256x256)",
                SimulationResolution::High => "High (512x512)",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut config.simulation_resolution,
                    SimulationResolution::Low,
                    "Low (128x128)",
                );
                ui.selectable_value(
                    &mut config.simulation_resolution,
                    SimulationResolution::Medium,
                    "Medium (256x256)",
                );
                ui.selectable_value(
                    &mut config.simulation_resolution,
                    SimulationResolution::High,
                    "High (512x512)",
                );
            })
            .response
            .on_hover_text("Higher resolution = more detail but slower");

        ui.add(
            Slider::new(&mut config.kernel_radius, 1.0..=64.0)
                .text(i18n.t("label-kernel-radius")),
        )
        .on_hover_text("Coupling interaction distance");

        ui.add(
            Slider::new(&mut config.noise_amount, 0.0..=1.0).text(i18n.t("label-noise-amount")),
        )
        .on_hover_text("Random variation in oscillation");

        ui.add(
            Slider::new(&mut config.frequency_min, 0.0..=10.0).text(i18n.t("label-freq-min")),
        );
        ui.add(
            Slider::new(&mut config.frequency_max, 0.0..=10.0).text(i18n.t("label-freq-max")),
        );

        ui.separator();

        // Coordinate mode
        ComboBox::from_label(i18n.t("label-coordinate-mode"))
            .selected_text(match config.coordinate_mode {
                CoordinateMode::Cartesian => "Cartesian",
                CoordinateMode::LogPolar => "Log-Polar",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut config.coordinate_mode,
                    CoordinateMode::Cartesian,
                    "Cartesian",
                );
                ui.selectable_value(
                    &mut config.coordinate_mode,
                    CoordinateMode::LogPolar,
                    "Log-Polar",
                );
            })
            .response
            .on_hover_text("Log-Polar creates radial/spiral patterns");

        // Phase initialization mode
        ComboBox::from_label(i18n.t("label-phase-init"))
            .selected_text(match config.phase_init_mode {
                PhaseInitMode::Random => "Random",
                PhaseInitMode::Uniform => "Uniform",
                PhaseInitMode::PlaneHorizontal => "Plane H",
                PhaseInitMode::PlaneVertical => "Plane V",
                PhaseInitMode::PlaneDiagonal => "Diagonal",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut config.phase_init_mode,
                    PhaseInitMode::Random,
                    "Random",
                );
                ui.selectable_value(
                    &mut config.phase_init_mode,
                    PhaseInitMode::Uniform,
                    "Uniform",
                );
                ui.selectable_value(
                    &mut config.phase_init_mode,
                    PhaseInitMode::PlaneHorizontal,
                    "Plane H",
                );
                ui.selectable_value(
                    &mut config.phase_init_mode,
                    PhaseInitMode::PlaneVertical,
                    "Plane V",
                );
                ui.selectable_value(
                    &mut config.phase_init_mode,
                    PhaseInitMode::PlaneDiagonal,
                    "Diagonal",
                );
            })
            .response
            .on_hover_text("Initial phase pattern for oscillators");

        ui.separator();

        // Coupling rings
        ui.collapsing(i18n.t("header-coupling"), |ui| {
            for i in 0..4 {
                let is_active = config.rings[i].distance > 0.0
                    || config.rings[i].width > 0.0
                    || config.rings[i].coupling.abs() > 0.01;

                ui.push_id(i, |ui| {
                    egui::CollapsingHeader::new(format!("Ring {}", i + 1))
                        .default_open(is_active)
                        .show(ui, |ui| {
                            ui.add(
                                Slider::new(&mut config.rings[i].distance, 0.0..=1.0)
                                    .text(i18n.t("label-dist-scale")),
                            )
                            .on_hover_text("Distance from center (0-1)");

                            ui.add(
                                Slider::new(&mut config.rings[i].width, 0.0..=1.0)
                                    .text(i18n.t("label-width")),
                            )
                            .on_hover_text("Ring width (0-1)");

                            ui.add(
                                Slider::new(&mut config.rings[i].coupling, -5.0..=5.0)
                                    .text(i18n.t("label-diff-coupling")),
                            )
                            .on_hover_text("Negative = anti-sync, Positive = sync");

                            ui.horizontal(|ui| {
                                if ui.button(i18n.t("btn-reset-ring")).clicked() {
                                    config.rings[i] = RingParams::default();
                                }
                                if ui.button(i18n.t("btn-clear-ring")).clicked() {
                                    config.rings[i] = RingParams {
                                        distance: 0.0,
                                        width: 0.0,
                                        coupling: 0.0,
                                    };
                                }
                            });
                        });
                });
            }
        });
    }
}
