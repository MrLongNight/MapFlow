//! Audio Analysis Control Panel
//!
//! Provides visual feedback for frequency bands, beat detection,
//! and controls for audio analysis parameters.

use crate::core::i18n::LocaleManager;
use crate::theme::colors;
use crate::widgets::{custom, panel};
use egui::{Rect, Sense, Stroke, Ui};
use mapmap_core::audio::{AudioAnalysis, AudioConfig};

/// Actions that can be triggered from the Audio Panel
#[derive(Debug, Clone)]
pub enum AudioPanelAction {
    ConfigChanged(AudioConfig),
    MeterStyleChanged(crate::config::AudioMeterStyle),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FftVisualizationMode {
    FullFft,
    ThreeBand,
}

#[derive(Debug)]
pub struct AudioPanel {
    pub is_expanded: bool,
}

impl Default for AudioPanel {
    fn default() -> Self {
        Self { is_expanded: true }
    }
}

impl AudioPanel {
    fn grouped_three_band_energies(analysis: &AudioAnalysis) -> [f32; 3] {
        [
            analysis.band_energies[0..3].iter().sum::<f32>() / 3.0,
            analysis.band_energies[3..6].iter().sum::<f32>() / 3.0,
            analysis.band_energies[6..9].iter().sum::<f32>() / 3.0,
        ]
    }

    /// Creates a new, uninitialized instance with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Render the Audio Panel UI
    #[allow(clippy::too_many_arguments)]
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        locale: &LocaleManager,
        analysis: Option<&AudioAnalysis>,
        config: &AudioConfig,
        meter_style: crate::config::AudioMeterStyle,
        show_level_meters: &mut bool,
        fft_mode: &mut FftVisualizationMode,
    ) -> Option<AudioPanelAction> {
        let mut action = None;

        // Use standard Cyber Dark panel frame
        panel::cyber_panel_frame(ui.style()).show(ui, |ui| {
            // Header
            panel::render_panel_header(
                ui,
                &locale.t("panel-audio"),
                |_| {}, // No header actions for now
            );

            ui.add_space(4.0);

            // Visualizer Section
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(show_level_meters, "Show Level Meters");
                    ui.separator();
                    ui.label("FFT View");
                    egui::ComboBox::from_id_salt("audio_fft_mode_combo")
                        .selected_text(match fft_mode {
                            FftVisualizationMode::FullFft => "Full FFT",
                            FftVisualizationMode::ThreeBand => "3-Band",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                fft_mode,
                                FftVisualizationMode::FullFft,
                                "Full FFT",
                            );
                            ui.selectable_value(
                                fft_mode,
                                FftVisualizationMode::ThreeBand,
                                "3-Band",
                            );
                        });
                });

                ui.add_space(6.0);

                if let Some(analysis) = analysis {
                    match meter_style {
                        crate::config::AudioMeterStyle::Retro => {
                            if *show_level_meters {
                                // Convert linear to dB
                                let db =
                                    20.0 * (analysis.rms_volume.max(0.00001).log10()).max(-60.0);

                                let meter = crate::widgets::audio_meter::AudioMeter::new(
                                    crate::config::AudioMeterStyle::Retro,
                                    db,
                                    db, // Mono for now
                                )
                                .height(60.0);

                                ui.add(meter);
                                ui.add_space(6.0);
                            }

                            self.show_visualizer(ui, analysis, locale, *fft_mode);
                        }
                        crate::config::AudioMeterStyle::Digital => {
                            if *show_level_meters {
                                let db =
                                    20.0 * (analysis.rms_volume.max(0.00001).log10()).max(-60.0);
                                let meter = crate::widgets::audio_meter::AudioMeter::new(
                                    crate::config::AudioMeterStyle::Digital,
                                    db,
                                    db,
                                )
                                .height(30.0);
                                ui.add(meter);
                                ui.add_space(8.0);
                            }

                            self.show_visualizer(ui, analysis, locale, *fft_mode);
                        }
                    }
                } else {
                    // Placeholder visualizer when no signal
                    let height = 60.0;
                    let (rect, _) = ui.allocate_at_least(
                        egui::vec2(ui.available_width(), height),
                        Sense::hover(),
                    );
                    ui.painter()
                        .rect_filled(rect, egui::CornerRadius::ZERO, colors::DARKER_GREY);
                    ui.painter().rect_stroke(
                        rect,
                        egui::CornerRadius::ZERO,
                        Stroke::new(1.0, colors::STROKE_GREY),
                        egui::StrokeKind::Middle,
                    );

                    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                        ui.centered_and_justified(|ui| {
                            ui.label(locale.t("no-signal"));
                        });
                    });
                }
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Controls Section
            egui::Grid::new("audio_controls_grid")
                .num_columns(2)
                .spacing([8.0, 8.0])
                .show(ui, |ui| {
                    // Gain
                    ui.label(locale.t("audio-gain"));
                    let mut gain = config.gain;
                    if custom::styled_slider(ui, &mut gain, 0.0..=10.0, 1.0).changed() {
                        let mut new_cfg = config.clone();
                        new_cfg.gain = gain;
                        action = Some(AudioPanelAction::ConfigChanged(new_cfg));
                    }
                    ui.end_row();

                    // Low Band Gain
                    ui.label(locale.t("audio-gain-low"));
                    let mut low_band_gain = config.low_band_gain;
                    if custom::styled_slider(ui, &mut low_band_gain, 0.0..=10.0, 1.0).changed() {
                        let mut new_cfg = config.clone();
                        new_cfg.low_band_gain = low_band_gain;
                        action = Some(AudioPanelAction::ConfigChanged(new_cfg));
                    }
                    ui.end_row();

                    // Mid Band Gain
                    ui.label(locale.t("audio-gain-mid"));
                    let mut mid_band_gain = config.mid_band_gain;
                    if custom::styled_slider(ui, &mut mid_band_gain, 0.0..=10.0, 1.0).changed() {
                        let mut new_cfg = config.clone();
                        new_cfg.mid_band_gain = mid_band_gain;
                        action = Some(AudioPanelAction::ConfigChanged(new_cfg));
                    }
                    ui.end_row();

                    // High Band Gain
                    ui.label(locale.t("audio-gain-high"));
                    let mut high_band_gain = config.high_band_gain;
                    if custom::styled_slider(ui, &mut high_band_gain, 0.0..=10.0, 1.0).changed() {
                        let mut new_cfg = config.clone();
                        new_cfg.high_band_gain = high_band_gain;
                        action = Some(AudioPanelAction::ConfigChanged(new_cfg));
                    }
                    ui.end_row();

                    // Smoothing
                    ui.label(locale.t("audio-smoothing"));
                    let mut smoothing = config.smoothing;
                    if custom::styled_slider(ui, &mut smoothing, 0.0..=1.0, 0.8).changed() {
                        let mut new_cfg = config.clone();
                        new_cfg.smoothing = smoothing;
                        action = Some(AudioPanelAction::ConfigChanged(new_cfg));
                    }
                    ui.end_row();

                    // Meter Style
                    ui.label("Meter Style");
                    egui::ComboBox::from_id_salt("audio_meter_style_combo")
                        .selected_text(meter_style.to_string())
                        .show_ui(ui, |ui| {
                            for style in [
                                crate::config::AudioMeterStyle::Retro,
                                crate::config::AudioMeterStyle::Digital,
                            ] {
                                if ui
                                    .selectable_label(meter_style == style, style.to_string())
                                    .clicked()
                                {
                                    action = Some(AudioPanelAction::MeterStyleChanged(style));
                                }
                            }
                        });
                    ui.end_row();
                });
        });

        action
    }

    fn show_visualizer(
        &self,
        ui: &mut Ui,
        analysis: &AudioAnalysis,
        _locale: &LocaleManager,
        mode: FftVisualizationMode,
    ) {
        let height = 60.0;
        let (rect, _response) =
            ui.allocate_at_least(egui::vec2(ui.available_width(), height), Sense::hover());
        let painter = ui.painter();

        // Background
        painter.rect_filled(rect, egui::CornerRadius::ZERO, colors::DARKER_GREY);
        painter.rect_stroke(
            rect,
            egui::CornerRadius::ZERO,
            Stroke::new(1.0, colors::STROKE_GREY),
            egui::StrokeKind::Middle,
        );

        let (num_bands, mut energy_for_index): (usize, Box<dyn FnMut(usize) -> f32>) = match mode {
            FftVisualizationMode::FullFft => {
                let energy = analysis.band_energies;
                (energy.len(), Box::new(move |i| energy[i]))
            }
            FftVisualizationMode::ThreeBand => {
                let grouped = Self::grouped_three_band_energies(analysis);
                (3, Box::new(move |i| grouped[i]))
            }
        };

        if num_bands == 0 {
            return;
        }

        let spacing = 4.0;
        let band_width =
            ((rect.width() - (num_bands as f32 + 1.0) * spacing) / num_bands as f32).max(1.0);

        let db_ticks = [0.2_f32, 0.4_f32, 0.6_f32, 0.8_f32, 1.0_f32];
        for tick in db_ticks {
            let y = rect.max.y - tick * (rect.height() - spacing * 2.0) - spacing;
            painter.line_segment(
                [egui::pos2(rect.min.x, y), egui::pos2(rect.max.x, y)],
                Stroke::new(1.0, colors::STROKE_GREY.linear_multiply(0.4)),
            );
            let db = -60.0 + tick * 60.0;
            painter.text(
                egui::pos2(rect.min.x + 2.0, y - 1.0),
                egui::Align2::LEFT_BOTTOM,
                format!("{db:.0}"),
                egui::TextStyle::Small.resolve(ui.style()),
                colors::STROKE_GREY,
            );
        }

        for i in 0..num_bands {
            let energy = energy_for_index(i).clamp(0.0, 1.0);
            let x = rect.min.x + spacing + i as f32 * (band_width + spacing);
            let h = (energy * (rect.height() - 2.0 * spacing)).max(1.0);

            let band_rect = Rect::from_min_max(
                egui::pos2(x, rect.max.y - spacing - h),
                egui::pos2(x + band_width, rect.max.y - spacing),
            );

            let color = match mode {
                FftVisualizationMode::ThreeBand => match i {
                    0 => egui::Color32::from_rgb(70, 180, 255),
                    1 => egui::Color32::from_rgb(90, 220, 150),
                    _ => egui::Color32::from_rgb(255, 180, 80),
                },
                FftVisualizationMode::FullFft => {
                    if analysis.beat_detected && i < 2 {
                        colors::MINT_ACCENT
                    } else {
                        colors::CYAN_ACCENT.linear_multiply(0.6 + (energy * 0.4))
                    }
                }
            };

            painter.rect_filled(band_rect, egui::CornerRadius::ZERO, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_band_grouping_averages_ranges() {
        let analysis = AudioAnalysis {
            band_energies: [0.0, 0.3, 0.6, 0.2, 0.5, 0.8, 0.1, 0.4, 0.7],
            ..AudioAnalysis::default()
        };

        let grouped = AudioPanel::grouped_three_band_energies(&analysis);

        assert!((grouped[0] - 0.3).abs() < f32::EPSILON);
        assert!((grouped[1] - 0.5).abs() < f32::EPSILON);
        assert!((grouped[2] - 0.4).abs() < f32::EPSILON);
    }
}
