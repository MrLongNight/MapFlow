//! Phase 6: Egui-based Audio Visualization Panel
//!
//! Displays audio analysis data such as frequency band levels, peak indicators,
//! beat detection, and RMS volume.

use crate::i18n::LocaleManager;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use mapmap_core::audio::{AudioAnalysis, AudioConfig};
use std::time::Instant;

const PEAK_DECAY_RATE: f32 = 0.5; // units per second
const PEAK_HOLD_TIME_SECS: f32 = 1.5;

/// Visualization mode for the audio panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Spectrum,
    Waveform,
    Bars,
}

/// Actions that can be triggered by the audio panel
#[derive(Debug, Clone)]
pub enum AudioPanelAction {
    DeviceChanged(String),
    ConfigChanged(AudioConfig),
}

/// Audio visualization panel widget
pub struct AudioPanel {
    /// Peak levels for each of the 7 frequency bands
    peak_levels: [f32; 7],
    /// Timestamps of the last peak for each band
    peak_timers: [Instant; 7],
    /// Timestamp of the last beat detection
    last_beat_time: Instant,
    /// Current view mode
    view_mode: ViewMode,
    /// Local configuration state for sliders (to avoid jumpiness)
    local_config: Option<AudioConfig>,
}

impl Default for AudioPanel {
    fn default() -> Self {
        Self {
            peak_levels: [0.0; 7],
            peak_timers: [Instant::now(); 7],
            last_beat_time: Instant::now(),
            view_mode: ViewMode::Spectrum,
            local_config: None,
        }
    }
}

impl AudioPanel {
    /// Renders the audio panel UI and returns actions.
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        locale: &LocaleManager,
        analysis: Option<&AudioAnalysis>,
        current_config: &AudioConfig,
        audio_devices: &[String],
        selected_audio_device: &mut Option<String>,
    ) -> Option<AudioPanelAction> {
        let mut action = None;

        // Initialize local config if needed
        if self.local_config.is_none() {
            self.local_config = Some(current_config.clone());
        }
        // Sync local config if external config changed significantly (e.g. preset load)
        // For simplicity, we trust local config while editing, assuming single user.

        let mut config = self.local_config.clone().unwrap_or(current_config.clone());
        let mut config_changed = false;

        crate::widgets::render_panel_header(ui, &locale.t("audio-panel-title"), |_ui| {});

        // --- Audio Device Selector ---
        let no_device_text = locale.t("audio-panel-no-device");
        let selected_text = selected_audio_device.as_deref().unwrap_or(&no_device_text);

        egui::ComboBox::from_label(locale.t("audio-panel-device"))
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                for device in audio_devices {
                    if ui
                        .selectable_value(selected_audio_device, Some(device.clone()), device)
                        .changed()
                    {
                        if let Some(new_device) = selected_audio_device.clone() {
                            action = Some(AudioPanelAction::DeviceChanged(new_device));
                        }
                    }
                }
            });

        ui.separator();

        // --- Settings (Gain, Gate, Smoothing) ---
        ui.collapsing(locale.t("audio-panel-settings"), |ui| {
            if ui
                .add(
                    egui::Slider::new(&mut config.gain, 0.1..=10.0)
                        .text("Gain")
                        .logarithmic(true),
                )
                .changed()
            {
                config_changed = true;
            }

            if ui
                .add(
                    egui::Slider::new(&mut config.noise_gate, 0.0..=0.2)
                        .text("Noise Gate")
                        .clamping(egui::SliderClamping::Always),
                )
                .changed()
            {
                config_changed = true;
            }

            if ui
                .add(egui::Slider::new(&mut config.smoothing, 0.0..=0.99).text("Smoothing"))
                .changed()
            {
                config_changed = true;
            }
        });

        if config_changed {
            self.local_config = Some(config.clone());
            action = Some(AudioPanelAction::ConfigChanged(config));
        }

        ui.separator();

        // --- View Mode Switcher ---
        ui.horizontal(|ui| {
            ui.label("View:");
            ui.selectable_value(&mut self.view_mode, ViewMode::Spectrum, "Spectrum");
            ui.selectable_value(&mut self.view_mode, ViewMode::Bars, "Bands");
            ui.selectable_value(&mut self.view_mode, ViewMode::Waveform, "Waveform");
        });

        ui.separator();

        // --- Visualizations ---
        if let Some(analysis) = analysis {
            // Update beat timer
            if analysis.beat_detected {
                self.last_beat_time = Instant::now();
            }

            // RMS Volume
            self.render_rms_volume(ui, locale, analysis.rms_volume);

            // Beat Indicator
            self.render_beat_indicator(ui, locale);

            // Main Visualization
            match self.view_mode {
                ViewMode::Spectrum => self.render_spectrum(ui, analysis),
                ViewMode::Bars => self.render_frequency_bands(ui, locale, &analysis.band_energies),
                ViewMode::Waveform => self.render_waveform(ui, &analysis.waveform),
            }
        } else {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                if selected_audio_device.is_none() {
                    ui.label(locale.t("audio-panel-select-device"));
                    ui.label(egui::RichText::new("⚠️").size(24.0));
                } else {
                    ui.label(locale.t("audio-panel-waiting-signal"));
                    ui.spinner();
                }
                ui.add_space(10.0);
            });
        }

        action
    }

    /// Renders the RMS volume progress bar
    fn render_rms_volume(&self, ui: &mut Ui, locale: &LocaleManager, rms_volume: f32) {
        let rms_text = format!("{}: {:.2}", locale.t("audio-panel-rms"), rms_volume);
        ui.add(egui::ProgressBar::new(rms_volume).text(rms_text));
    }

    /// Renders the beat indicator
    fn render_beat_indicator(&self, ui: &mut Ui, locale: &LocaleManager) {
        ui.horizontal(|ui| {
            ui.label(locale.t("audio-panel-beat"));
            let beat_pulse =
                (1.0 - (self.last_beat_time.elapsed().as_secs_f32() * 5.0).min(1.0)).powi(2);

            let (rect, _) = ui.allocate_exact_size(Vec2::splat(20.0), Sense::hover());
            ui.painter().circle_filled(
                rect.center(),
                10.0,
                Color32::from_rgba_premultiplied(
                    (beat_pulse * 255.0) as u8,
                    (beat_pulse * 150.0) as u8,
                    (beat_pulse * 50.0) as u8,
                    255,
                ),
            );
        });
    }

    /// Renders the FFT Spectrum
    fn render_spectrum(&self, ui: &mut Ui, analysis: &AudioAnalysis) {
        let (rect, _response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 150.0), Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(rect, 3.0, Color32::from_rgb(20, 20, 20));

        let fft_magnitudes = &analysis.fft_magnitudes;
        let num_bars = (fft_magnitudes.len() / 2).min(128); // Limit bars for performance
        if num_bars > 0 {
            let bar_width = rect.width() / num_bars as f32;
            for (i, &magnitude) in fft_magnitudes.iter().take(num_bars).enumerate() {
                let bar_height = (magnitude.powf(0.5) * rect.height())
                    .min(rect.height())
                    .max(1.0);
                let x = rect.min.x + i as f32 * bar_width;
                let y = rect.max.y;
                let color = Color32::from_rgb(
                    (magnitude * 200.0) as u8,
                    255 - (magnitude * 200.0) as u8,
                    50,
                );
                painter.rect_filled(
                    Rect::from_min_size(
                        Pos2::new(x, y - bar_height),
                        Vec2::new(bar_width.ceil(), bar_height),
                    ),
                    1.0,
                    color,
                );
            }
        }
    }

    /// Renders the 7 frequency band meters
    fn render_frequency_bands(&mut self, ui: &mut Ui, locale: &LocaleManager, bands: &[f32; 7]) {
        ui.label(locale.t("audio-panel-bands"));

        let (rect, _response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 150.0), Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(rect, 3.0, Color32::from_rgb(20, 20, 20));

        let num_bands = bands.len();
        let bar_spacing = 5.0;
        let total_spacing = (num_bands + 1) as f32 * bar_spacing;
        let bar_width = (rect.width() - total_spacing) / num_bands as f32;
        let dt = ui.input(|i| i.stable_dt);

        for (i, &energy) in bands.iter().enumerate() {
            // Update peak level
            if energy >= self.peak_levels[i] {
                self.peak_levels[i] = energy;
                self.peak_timers[i] = Instant::now();
            } else {
                // Decay peak level after hold time
                if self.peak_timers[i].elapsed().as_secs_f32() > PEAK_HOLD_TIME_SECS {
                    self.peak_levels[i] -= PEAK_DECAY_RATE * dt;
                    if self.peak_levels[i] < energy {
                        self.peak_levels[i] = energy;
                    }
                }
            }
            self.peak_levels[i] = self.peak_levels[i].max(0.0);

            let bar_height = (energy * rect.height()).min(rect.height()).max(1.0);
            let x = rect.min.x + (i + 1) as f32 * bar_spacing + i as f32 * bar_width;
            let bar_rect = Rect::from_min_size(
                Pos2::new(x, rect.max.y - bar_height),
                Vec2::new(bar_width, bar_height),
            );

            // Bar color based on energy
            let color = Color32::from_rgb((energy * 200.0) as u8, 255 - (energy * 200.0) as u8, 50);
            painter.rect_filled(bar_rect, 1.0, color);

            // Peak indicator
            let peak_y = rect.max.y
                - (self.peak_levels[i] * rect.height())
                    .min(rect.height())
                    .max(1.0);

            painter.line_segment(
                [Pos2::new(x, peak_y), Pos2::new(x + bar_width, peak_y)],
                Stroke::new(2.0, Color32::from_rgb(255, 100, 100)),
            );
        }
    }

    /// Renders the audio waveform
    fn render_waveform(&self, ui: &mut Ui, waveform: &[f32]) {
        let (rect, _response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 150.0), Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(rect, 3.0, Color32::from_rgb(20, 20, 20));

        if waveform.is_empty() {
            return;
        }

        let center_y = rect.center().y;
        let points: Vec<Pos2> = waveform
            .iter()
            .enumerate()
            .map(|(i, &sample)| {
                let x = rect.min.x + (i as f32 / waveform.len() as f32) * rect.width();
                // Clamp sample to -1.0..1.0 range and scale to fit height
                let y = center_y - (sample.clamp(-1.0, 1.0) * rect.height() * 0.5);
                Pos2::new(x, y)
            })
            .collect();

        // Draw the waveform line
        painter.add(egui::Shape::line(
            points,
            Stroke::new(1.5, Color32::from_rgb(100, 200, 255)),
        ));
    }
}
