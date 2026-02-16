//! Audio Analysis Control Panel
//!
//! Provides visual feedback for frequency bands, beat detection,
//! and controls for audio analysis parameters.

use crate::theme::colors;
use crate::widgets::{custom, panel::StyledPanel};
use egui::{Color32, Rect, Stroke, Ui};
use mapmap_core::audio::{AudioConfig, FrequencyBand};
use mapmap_core::LocaleManager;

pub struct AudioPanel {
    pub is_expanded: bool,
}

impl Default for AudioPanel {
    fn default() -> Self {
        Self { is_expanded: true }
    }
}

impl AudioPanel {
    pub fn show(
        &mut self,
        ui: &mut Ui,
        locale: &LocaleManager,
        config: &mut AudioConfig,
        analysis: &mapmap_core::audio::AudioAnalysis,
    ) -> bool {
        let mut changed = false;

        StyledPanel::new(locale.t("audio-panel-title")).show(ui, |ui| {
            // Visualizer Section
            self.show_visualizer(ui, analysis);
            ui.add_space(8.0);

            // Controls Section
            egui::Grid::new("audio_controls_grid")
                .num_columns(2)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    // Gain
                    ui.label(locale.t("audio-gain"));
                    changed |= custom::styled_slider(ui, &mut config.gain, 0.0..=10.0, 1.0).changed();
                    ui.end_row();

                    // Smoothing
                    ui.label(locale.t("audio-smoothing"));
                    changed |= custom::styled_slider(ui, &mut config.smoothing, 0.0..=1.0, 0.8).changed();
                    ui.end_row();

                    // Noise Gate
                    ui.label(locale.t("audio-noise-gate"));
                    changed |= custom::styled_slider_log(ui, &mut config.noise_gate, 0.0001..=0.1, 0.001).changed();
                    ui.end_row();
                });
        });

        changed
    }

    fn show_visualizer(&self, ui: &mut Ui, analysis: &mapmap_core::audio::AudioAnalysis) {
        let height = 60.0;
        let (rect, _response) = ui.allocate_at_least(egui::vec2(ui.available_width(), height), egui::Sense::hover());
        let painter = ui.painter();

        // Background
        painter.rect_filled(rect, 2.0, colors::DARKER_GREY);
        painter.rect_stroke(rect, 2.0, Stroke::new(1.0, colors::STROKE_GREY), egui::StrokeKind::Middle);

        // Draw Bands
        let num_bands = analysis.band_energies.len();
        let spacing = 2.0;
        let band_width = (rect.width() - (num_bands as f32 + 1.0) * spacing) / num_bands as f32;

        for i in 0..num_bands {
            let energy = analysis.band_energies[i];
            let x = rect.min.x + spacing + i as f32 * (band_width + spacing);
            let h = energy * (rect.height() - 2.0 * spacing);
            let band_rect = Rect::from_min_max(
                egui::pos2(x, rect.max.y - spacing - h),
                egui::pos2(x + band_width, rect.max.y - spacing),
            );

            let color = if analysis.beat_detected && i < 2 {
                colors::CYAN_ACCENT // Highlight bass on beat
            } else {
                colors::CYAN_ACCENT.linear_multiply(0.6)
            };

            painter.rect_filled(band_rect, 1.0, color);
        }
    }
}
