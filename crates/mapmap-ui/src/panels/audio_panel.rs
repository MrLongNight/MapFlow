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
    DeviceChanged(String),
    ConfigChanged(AudioConfig),
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
    pub fn new() -> Self {
        Self::default()
    }

    /// Render the Audio Panel UI
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        locale: &LocaleManager,
        analysis: Option<&AudioAnalysis>,
        config: &AudioConfig,
        available_devices: &[String],
        selected_device: &mut Option<String>,
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
                if let Some(analysis) = analysis {
                    self.show_visualizer(ui, analysis);
                } else {
                    // Placeholder visualizer when no signal
                    let height = 60.0;
                    let (rect, _) = ui.allocate_at_least(
                        egui::vec2(ui.available_width(), height),
                        Sense::hover(),
                    );
                    ui.painter().rect_filled(rect, 2.0, colors::DARKER_GREY);
                    ui.painter().rect_stroke(
                        rect,
                        2.0,
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
                    // Device Selection
                    ui.label(locale.t("dashboard-device"));
                    let no_device_text = locale.t("no-device");
                    let current_text = selected_device.as_deref().unwrap_or(&no_device_text);

                    egui::ComboBox::from_id_salt("audio_device_combo")
                        .selected_text(current_text)
                        .show_ui(ui, |ui| {
                            for device in available_devices {
                                let is_selected = selected_device.as_ref() == Some(device);
                                if ui.selectable_label(is_selected, device).clicked() {
                                    *selected_device = Some(device.clone());
                                    action = Some(AudioPanelAction::DeviceChanged(device.clone()));
                                }
                            }
                        });
                    ui.end_row();

                    // Gain
                    ui.label(locale.t("audio-gain"));
                    let mut gain = config.gain;
                    if custom::styled_slider(ui, &mut gain, 0.0..=10.0, 1.0).changed() {
                        let mut new_cfg = config.clone();
                        new_cfg.gain = gain;
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
                });
        });

        action
    }

    fn show_visualizer(&self, ui: &mut Ui, analysis: &AudioAnalysis) {
        let height = 60.0;
        let (rect, _response) =
            ui.allocate_at_least(egui::vec2(ui.available_width(), height), Sense::hover());
        let painter = ui.painter();

        // Background
        painter.rect_filled(rect, 2.0, colors::DARKER_GREY);
        painter.rect_stroke(
            rect,
            2.0,
            Stroke::new(1.0, colors::STROKE_GREY),
            egui::StrokeKind::Middle,
        );

        // Draw Bands
        let num_bands = analysis.band_energies.len();
        if num_bands == 0 {
            return;
        }

        let spacing = 2.0;
        // Ensure band_width is positive
        let band_width =
            ((rect.width() - (num_bands as f32 + 1.0) * spacing) / num_bands as f32).max(1.0);

        for i in 0..num_bands {
            let energy = analysis.band_energies[i];
            let x = rect.min.x + spacing + i as f32 * (band_width + spacing);
            let h = (energy * (rect.height() - 2.0 * spacing)).max(1.0); // Minimum height for visibility

            let band_rect = Rect::from_min_max(
                egui::pos2(x, rect.max.y - spacing - h),
                egui::pos2(x + band_width, rect.max.y - spacing),
            );

            // Use Theme Colors for Bands
            let color = if analysis.beat_detected && i < 2 {
                colors::MINT_ACCENT // Beat hit!
            } else {
                // Gradient from Cyan to Blue-ish based on intensity
                colors::CYAN_ACCENT.linear_multiply(0.6 + (energy * 0.4))
            };

            painter.rect_filled(band_rect, 1.0, color);
        }
    }
}
