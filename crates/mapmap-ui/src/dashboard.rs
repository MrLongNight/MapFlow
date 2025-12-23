//! Phase 6: Dashboard Controls
//!
//! Quick-access parameter controls for playback and audio analysis.

use crate::i18n::LocaleManager;
use egui::{Color32, Ui};
use mapmap_core::AudioAnalysis;
use mapmap_media::{LoopMode, PlaybackCommand, PlaybackState};
use std::time::Duration;

/// Dashboard control panel
pub struct Dashboard {
    /// Is the panel currently visible?
    pub visible: bool,
    /// Playback state
    playback_state: PlaybackState,
    /// Current playback time
    current_time: Duration,
    /// Total duration of the media
    duration: Duration,
    /// Playback speed
    speed: f32,
    /// Loop mode
    loop_mode: LoopMode,
    /// Latest audio analysis
    audio_analysis: Option<AudioAnalysis>,
    /// Available audio devices
    audio_devices: Vec<String>,
    /// Selected audio device
    selected_audio_device: Option<String>,
}

impl Default for Dashboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Dashboard {
    pub fn new() -> Self {
        Self {
            visible: true,
            playback_state: PlaybackState::Idle,
            current_time: Duration::ZERO,
            duration: Duration::ZERO,
            speed: 1.0,
            loop_mode: LoopMode::Loop,
            audio_analysis: None,
            audio_devices: Vec::new(),
            selected_audio_device: None,
        }
    }

    /// Update the playback state
    pub fn set_playback_state(&mut self, state: PlaybackState) {
        self.playback_state = state;
    }

    /// Update the playback time
    pub fn set_playback_time(&mut self, current_time: Duration, duration: Duration) {
        self.current_time = current_time;
        self.duration = duration;
    }

    /// Update the audio analysis data
    pub fn set_audio_analysis(&mut self, analysis: AudioAnalysis) {
        self.audio_analysis = Some(analysis);
    }

    /// Update the list of available audio devices
    pub fn set_audio_devices(&mut self, devices: Vec<String>) {
        self.audio_devices = devices;
        if self.selected_audio_device.is_none() {
            self.selected_audio_device = self.audio_devices.first().cloned();
        }
    }

    /// Render the dashboard UI
    pub fn ui(&mut self, ctx: &egui::Context, locale: &LocaleManager) -> Option<DashboardAction> {
        let mut action = None;

        if self.visible {
            let mut is_open = self.visible;
            egui::Window::new("Dashboard")
                .open(&mut is_open)
                .show(ctx, |ui| {
                    action = self.render_contents(ui, locale);
                });
            self.visible = is_open;
        }

        action
    }

    /// Renders the contents of the dashboard panel.
    fn render_contents(&mut self, ui: &mut Ui, locale: &LocaleManager) -> Option<DashboardAction> {
        let mut action = None;

        ui.group(|ui| {
            // Playback controls
            ui.horizontal(|ui| {
                // TODO: Icon
                if ui.button(locale.t("btn-play")).clicked() {
                    action = Some(DashboardAction::SendCommand(PlaybackCommand::Play));
                }
                // TODO: Icon
                if ui.button(locale.t("btn-pause")).clicked() {
                    action = Some(DashboardAction::SendCommand(PlaybackCommand::Pause));
                }
                // TODO: Icon
                if ui.button(locale.t("btn-stop")).clicked() {
                    action = Some(DashboardAction::SendCommand(PlaybackCommand::Stop));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{:?}", self.playback_state));
                });
            });

            // Timeline scrubber
            let total_secs = self.duration.as_secs_f32();
            let mut current_secs = self.current_time.as_secs_f32();
            if ui
                .add(egui::Slider::new(&mut current_secs, 0.0..=total_secs).show_value(false))
                .changed()
            {
                action = Some(DashboardAction::SendCommand(PlaybackCommand::Seek(
                    Duration::from_secs_f32(current_secs),
                )));
            }
            ui.label(format!(
                "{}/ {}",
                Self::format_duration(self.current_time),
                Self::format_duration(self.duration)
            ));

            // Speed and loop controls
            ui.horizontal(|ui| {
                ui.label(locale.t("dashboard-speed"));
                if ui
                    .add(
                        egui::Slider::new(&mut self.speed, 0.1..=4.0)
                            .logarithmic(true)
                            .show_value(true),
                    )
                    .changed()
                {
                    let new_speed = self.speed;
                    action = Some(DashboardAction::SendCommand(PlaybackCommand::SetSpeed(
                        new_speed,
                    )));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut looping = self.loop_mode == LoopMode::Loop;
                    if ui
                        .checkbox(&mut looping, locale.t("dashboard-loop"))
                        .changed()
                    {
                        let new_mode = if looping {
                            LoopMode::Loop
                        } else {
                            LoopMode::PlayOnce
                        };
                        self.loop_mode = new_mode;
                        action = Some(DashboardAction::SendCommand(PlaybackCommand::SetLoopMode(
                            new_mode,
                        )));
                    }
                });
            });
        });

        ui.add_space(8.0);

        // Audio visualization
        if let Some(audio_action) = self.render_audio_visuals(ui, locale) {
            action = Some(audio_action);
        }

        action
    }

    /// Render audio visualization
    fn render_audio_visuals(
        &mut self,
        ui: &mut Ui,
        locale: &LocaleManager,
    ) -> Option<DashboardAction> {
        let mut action = None;
        ui.group(|ui| {
            ui.collapsing(locale.t("dashboard-audio-analysis"), |ui| {
                // Device selector
                let no_device_text = locale.t("dashboard-no-device");
                let selected_text = self
                    .selected_audio_device
                    .as_deref()
                    .unwrap_or(&no_device_text);
                let mut selected_device = self.selected_audio_device.clone();

                egui::ComboBox::from_label(locale.t("dashboard-device"))
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        for device in &self.audio_devices {
                            if ui
                                .selectable_value(
                                    &mut selected_device,
                                    Some(device.clone()),
                                    device,
                                )
                                .changed()
                            {
                                if let Some(new_device) = selected_device.clone() {
                                    action = Some(DashboardAction::AudioDeviceChanged(new_device));
                                }
                            }
                        }
                    });
                self.selected_audio_device = selected_device;

                if let Some(analysis) = &self.audio_analysis {
                    // RMS and Peak Volume Meters
                    ui.label(locale.t("dashboard-volume"));
                    ui.add(egui::ProgressBar::new(analysis.rms_volume).text(format!(
                        "{}: {:.2}",
                        locale.t("dashboard-rms"),
                        analysis.rms_volume
                    )));
                    ui.add(egui::ProgressBar::new(analysis.peak_volume).text(format!(
                        "{}: {:.2}",
                        locale.t("dashboard-peak"),
                        analysis.peak_volume
                    )));

                    ui.separator();

                    // FFT Visualization
                    ui.label(locale.t("dashboard-spectrum"));
                    let painter = ui.painter();
                    let rect = ui.available_rect_before_wrap();
                    let plot_rect =
                        egui::Rect::from_min_size(rect.min, egui::vec2(rect.width(), 80.0));
                    painter.rect_filled(plot_rect, 3.0, Color32::from_rgb(20, 20, 20));

                    let fft_magnitudes = &analysis.fft_magnitudes;
                    let num_bars = (fft_magnitudes.len() / 2).min(256); // Display up to 256 bands for clarity
                    if num_bars > 0 {
                        let bar_width = plot_rect.width() / num_bars as f32;
                        for (i, &magnitude) in fft_magnitudes.iter().take(num_bars).enumerate() {
                            let bar_height = (magnitude.powf(0.5) * plot_rect.height())
                                .min(plot_rect.height())
                                .max(1.0);
                            let x = plot_rect.min.x + i as f32 * bar_width;
                            let y = plot_rect.max.y;
                            let color = Color32::from_rgb(
                                (magnitude * 200.0) as u8,
                                255 - (magnitude * 200.0) as u8,
                                50,
                            );
                            painter.rect_filled(
                                egui::Rect::from_min_size(
                                    egui::pos2(x, y - bar_height),
                                    egui::vec2(bar_width - 1.0, bar_height),
                                ),
                                1.0,
                                color,
                            );
                        }
                    }
                    ui.advance_cursor_after_rect(plot_rect);
                } else {
                    ui.label(locale.t("dashboard-no-audio-data"));
                }
            });
        });
        action
    }

    /// Formats a duration into a MM:SS string.
    fn format_duration(duration: Duration) -> String {
        let total_seconds = duration.as_secs();
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }
}

/// Actions that can be triggered by the dashboard
#[derive(Debug, Clone)]
pub enum DashboardAction {
    SendCommand(PlaybackCommand),
    AudioDeviceChanged(String),
}
