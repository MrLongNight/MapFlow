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
    MeterStyleChanged(crate::config::AudioMeterStyle),
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
        available_devices: &[String],
        selected_device: &mut Option<String>,
        meter_style: crate::config::AudioMeterStyle,
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
                    match meter_style {
                        crate::config::AudioMeterStyle::Retro => {
                            // Convert linear to dB
                            let db = 20.0 * (analysis.rms_volume.max(0.00001).log10()).max(-60.0);

                            let meter = crate::widgets::audio_meter::AudioMeter::new(
                                crate::config::AudioMeterStyle::Retro,
                                db,
                                db, // Mono for now
                            )
                            .height(120.0);

                            // FORCE ALLOCATION to prevent clipping in parent layouts
                            ui.add_space(4.0);
                            let available_width = ui.available_width();
                            ui.allocate_ui(egui::vec2(available_width, 130.0), |ui| {
                                ui.add(meter);
                            });
                            ui.add_space(4.0);
                        }
                        crate::config::AudioMeterStyle::Digital => {
                            let db = 20.0 * (analysis.rms_volume.max(0.00001).log10()).max(-60.0);
                            let meter = crate::widgets::audio_meter::AudioMeter::new(
                                crate::config::AudioMeterStyle::Digital,
                                db,
                                db,
                            )
                            .height(60.0);
                            ui.add(meter);

                            ui.add_space(8.0);
                            self.show_visualizer(ui, analysis, locale);
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

    fn show_visualizer(&self, ui: &mut Ui, analysis: &AudioAnalysis, _locale: &LocaleManager) {
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

        // Draw Bands
        let num_bands = analysis.band_energies.len();
        if num_bands == 0 {
            return;
        }

        let spacing = 2.0;
        let band_width =
            ((rect.width() - (num_bands as f32 + 1.0) * spacing) / num_bands as f32).max(1.0);

        for i in 0..num_bands {
            let energy = analysis.band_energies[i];
            let x = rect.min.x + spacing + i as f32 * (band_width + spacing);
            let h = (energy * (rect.height() - 2.0 * spacing)).max(1.0);

            let band_rect = Rect::from_min_max(
                egui::pos2(x, rect.max.y - spacing - h),
                egui::pos2(x + band_width, rect.max.y - spacing),
            );

            let color = if analysis.beat_detected && i < 2 {
                colors::MINT_ACCENT
            } else {
                colors::CYAN_ACCENT.linear_multiply(0.6 + (energy * 0.4))
            };

            painter.rect_filled(band_rect, egui::CornerRadius::ZERO, color);
        }
    }
}
