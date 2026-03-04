//! Phase 6: Dashboard Controls
//!
//! Quick-access parameter controls for playback and audio analysis.

use crate::i18n::LocaleManager;
use crate::theme::colors;
use crate::widgets::hold_to_action_icon;
use egui::Ui;
use mapmap_core::AudioAnalysis;
use mapmap_media::{LoopMode, PlaybackCommand, PlaybackState};
use std::time::Duration;

/// Dashboard control panel
pub struct Dashboard {
    /// Is the panel currently visible?
    pub visible: bool,
    /// Playback state
    pub playback_state: PlaybackState,
    /// Current playback time
    pub current_time: Duration,
    /// Total duration of the media
    pub duration: Duration,
    /// Playback speed
    pub speed: f32,
    /// Loop mode
    pub loop_mode: LoopMode,
    /// Latest audio analysis
    pub audio_analysis: Option<AudioAnalysis>,
    /// Available audio devices
    pub audio_devices: Vec<String>,
    /// Selected audio device
    pub selected_audio_device: Option<String>,
}

impl Default for Dashboard {
    fn default() -> Self {
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
}

impl Dashboard {
    /// Creates a new, uninitialized instance with default settings.
    pub fn new() -> Self {
        Self::default()
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

    /// Render the dashboard UI as a standalone window
    pub fn ui(
        &mut self,
        ctx: &egui::Context,
        locale: &LocaleManager,
        icon_manager: Option<&crate::icons::IconManager>,
    ) -> Option<DashboardAction> {
        let mut action = None;

        if self.visible {
            let mut is_open = self.visible;
            egui::Window::new("Dashboard")
                .open(&mut is_open)
                .show(ctx, |ui| {
                    action = self.render_contents(ui, locale, icon_manager);
                });
            self.visible = is_open;
        }

        action
    }

    /// Render the dashboard UI embedded in a panel
    pub fn ui_embedded(
        &mut self,
        ui: &mut Ui,
        locale: &LocaleManager,
        icon_manager: Option<&crate::icons::IconManager>,
    ) -> Option<DashboardAction> {
        self.render_contents(ui, locale, icon_manager)
    }

    /// Renders the contents of the dashboard panel.
    pub fn render_contents(
        &mut self,
        ui: &mut Ui,
        locale: &LocaleManager,
        icon_manager: Option<&crate::icons::IconManager>,
    ) -> Option<DashboardAction> {
        let mut action = None;

        ui.vertical(|ui| {
            ui.heading(locale.t("panel-dashboard"));
            ui.add_space(4.0);

            ui.group(|ui| {
                // Playback controls
                ui.horizontal(|ui| {
                    let icon_size = 20.0;

                    // Helper for icon buttons
                    let mut icon_btn = |icon: Option<crate::icons::AppIcon>, text: &str| -> bool {
                        if let (Some(mgr), Some(ic)) = (icon_manager, icon) {
                            if let Some(img) = mgr.image(ic, icon_size) {
                                return ui
                                    .add(egui::Button::image(img))
                                    .clone()
                                    .on_hover_text(text)
                                    .clicked();
                            }
                        }
                        ui.button(text).clicked()
                    };

                    // Play
                    if icon_btn(
                        Some(crate::icons::AppIcon::ButtonPlay),
                        &locale.t("btn-play"),
                    ) {
                        action = Some(DashboardAction::SendCommand(PlaybackCommand::Play));
                    }
                    // Pause
                    if icon_btn(
                        Some(crate::icons::AppIcon::ButtonPause),
                        &locale.t("btn-pause"),
                    ) {
                        action = Some(DashboardAction::SendCommand(PlaybackCommand::Pause));
                    }
                    // Stop
                    if hold_to_action_icon(
                        ui,
                        icon_manager,
                        crate::icons::AppIcon::ButtonStop,
                        icon_size,
                        colors::WARN_COLOR,
                    ) {
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

            // Audio controls
            ui.group(|ui| {
                ui.label(locale.t("dashboard-audio-section"));
                
                // Audio level visualization
                if let Some(analysis) = &self.audio_analysis {
                    let meter_rect = ui.available_rect_before_wrap();
                    let height = 10.0;
                    let width = meter_rect.width();
                    let (rect, _) = ui.allocate_at_least(egui::vec2(width, height), egui::Sense::hover());
                    
                    let rms = analysis.rms_volume.clamp(0.0, 1.0);
                    let peak = analysis.peak_volume.clamp(0.0, 1.0);
                    
                    ui.painter().rect_filled(rect, 2.0, colors::DARK_GREY);
                    
                    let rms_width = width * rms;
                    let rms_rect = egui::Rect::from_min_size(rect.min, egui::vec2(rms_width, height));
                    ui.painter().rect_filled(rms_rect, 2.0, colors::MINT_ACCENT);
                    
                    let peak_x = rect.min.x + width * peak;
                    ui.painter().line_segment(
                        [egui::pos2(peak_x, rect.min.y), egui::pos2(peak_x, rect.max.y)],
                        egui::Stroke::new(2.0, colors::WARN_COLOR)
                    );
                }

                if ui.button(locale.t("dashboard-open-audio-panel")).clicked() {
                    action = Some(DashboardAction::ToggleAudioPanel);
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
    ToggleAudioPanel,
}
