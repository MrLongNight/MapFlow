//! Phase 6: Dashboard Controls
//!
//! Quick-access parameter controls with customizable layouts.
//! Allows users to assign frequently-used parameters to dashboard dials and sliders.

use imgui::{self, Ui};
use mapmap_media::player::{LoopMode, PlaybackCommand, PlaybackState};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Dashboard control panel
pub struct Dashboard {
    /// Dashboard widgets
    widgets: Vec<DashboardWidget>,
    /// Layout mode
    layout: LayoutMode,
    /// Grid columns (for grid layout)
    grid_columns: usize,
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
    /// Last action triggered by the UI
    pending_action: Option<DashboardAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutMode {
    Grid,
    Freeform,
}

/// Dashboard widget (assignable control)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardWidget {
    pub id: u64,
    pub widget_type: WidgetType,
    pub parameter_name: String,
    pub position: Option<[f32; 2]>,
    pub size: Option<[f32; 2]>,
}

/// Widget type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetType {
    Slider { value: f32, min: f32, max: f32 },
    Knob { value: f32, min: f32, max: f32 },
    Toggle { value: bool },
    Button,
    Label { value: String },
}

impl Default for Dashboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Dashboard {
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
            layout: LayoutMode::Grid,
            grid_columns: 4,
            playback_state: PlaybackState::Idle,
            current_time: Duration::ZERO,
            duration: Duration::ZERO,
            speed: 1.0,
            loop_mode: LoopMode::Off,
            pending_action: None,
        }
    }

    pub fn take_action(&mut self) -> Option<DashboardAction> {
        self.pending_action.take()
    }

    pub fn set_playback_state(&mut self, state: PlaybackState) {
        self.playback_state = state;
    }

    pub fn set_playback_time(&mut self, current_time: Duration, duration: Duration) {
        self.current_time = current_time;
        self.duration = duration;
    }

    pub fn ui(&mut self, ui: &Ui) {
        if let Some(tab_bar) = ui.tab_bar("DashboardTabs") {
            if let Some(tab) = ui.tab_item("Playback") {
                self.render_playback_tab(ui);
                tab.end();
            }
            if let Some(tab) = ui.tab_item("Widgets") {
                self.render_widgets_tab(ui);
                tab.end();
            }
            tab_bar.end();
        }
    }

    fn render_playback_tab(&mut self, ui: &Ui) {
        if ui.button("▶") {
            self.pending_action = Some(DashboardAction::SendCommand(PlaybackCommand::Play));
        }
        ui.same_line();
        if ui.button("⏸") {
            self.pending_action = Some(DashboardAction::SendCommand(PlaybackCommand::Pause));
        }
        ui.same_line();
        if ui.button("⏹") {
            self.pending_action = Some(DashboardAction::SendCommand(PlaybackCommand::Stop));
        }
        ui.same_line();
        ui.text(format!("State: {:?}", self.playback_state));

        let mut seek_to = self.current_time.as_secs_f32();
        let duration_secs = self.duration.as_secs_f32();
        if ui.slider(
            "##Timeline",
            0.0,
            duration_secs.max(0.01), // Ensure max is not less than min
            &mut seek_to,
        ) {
            self.pending_action = Some(DashboardAction::SendCommand(PlaybackCommand::Seek(
                Duration::from_secs_f32(seek_to),
            )));
        }

        ui.set_next_item_width(150.0);
        if ui.slider("Speed", 0.1, 4.0, &mut self.speed) {
            self.pending_action = Some(DashboardAction::SendCommand(PlaybackCommand::SetSpeed(
                self.speed,
            )));
        }
        ui.same_line();
        let mut looping = self.loop_mode == LoopMode::On;
        if ui.checkbox("Loop", &mut looping) {
            self.loop_mode = if looping { LoopMode::On } else { LoopMode::Off };
            self.pending_action = Some(DashboardAction::SendCommand(PlaybackCommand::SetLoopMode(
                self.loop_mode,
            )));
        }
    }

    fn render_widgets_tab(&mut self, ui: &Ui) {
        ui.radio_button("Grid", &mut self.layout, LayoutMode::Grid);
        ui.same_line();
        ui.radio_button("Freeform", &mut self.layout, LayoutMode::Freeform);

        if self.layout == LayoutMode::Grid {
            ui.same_line();
            ui.text("Columns:");
            ui.same_line();
            ui.set_next_item_width(80.0);
            let mut cols = self.grid_columns as i32;
            if ui.input_int("##cols", &mut cols).build() {
                self.grid_columns = cols.max(1) as usize;
            }
        }
        ui.same_line();
        if ui.button("➕ Add Widget") {
            self.pending_action = Some(DashboardAction::AddWidget);
        }

        ui.separator();

        match self.layout {
            LayoutMode::Grid => self.render_grid_layout(ui),
            LayoutMode::Freeform => self.render_freeform_layout(ui),
        }
    }

    fn render_grid_layout(&mut self, ui: &Ui) {
        let mut widget_to_remove = None;
        let mut action = None;

        for (i, widget) in self.widgets.iter_mut().enumerate() {
            ui.group(|| {
                let (widget_action, remove_clicked) = Self::render_widget(ui, widget);
                if widget_action.is_some() {
                    action = widget_action;
                }
                if remove_clicked {
                    widget_to_remove = Some(widget.id);
                }
            });

            if (i + 1) % self.grid_columns != 0 {
                ui.same_line();
            }
        }

        if let Some(a) = action {
            self.pending_action = Some(a);
        }
        if let Some(id) = widget_to_remove {
            self.pending_action = Some(DashboardAction::RemoveWidget(id));
        }
    }

    fn render_freeform_layout(&mut self, ui: &Ui) {
        ui.text("Freeform layout is not yet fully implemented for imgui.");
        self.render_grid_layout(ui);
    }

    fn render_widget(ui: &Ui, widget: &mut DashboardWidget) -> (Option<DashboardAction>, bool) {
        let mut action = None;
        let mut remove_clicked = false;

        ui.text(&widget.parameter_name);
        ui.same_line();
        if ui.small_button(format!("✖##{}", widget.id)) {
            remove_clicked = true;
        }

        match &mut widget.widget_type {
            WidgetType::Slider { value, min, max } => {
                if ui.slider(format!("##slider{}", widget.id), *min, *max, value) {
                    action = Some(DashboardAction::ValueChanged(widget.id, *value));
                }
            }
            WidgetType::Knob { value, min, max } => {
                if ui.slider(format!("##knob{}", widget.id), *min, *max, value) {
                    action = Some(DashboardAction::ValueChanged(widget.id, *value));
                }
            }
            WidgetType::Toggle { value } => {
                if ui.checkbox(format!("##toggle{}", widget.id), value) {
                    action = Some(DashboardAction::BoolChanged(widget.id, *value));
                }
            }
            WidgetType::Button => {
                if ui.button(format!("Trigger##{}", widget.id)) {
                    action = Some(DashboardAction::ButtonPressed(widget.id));
                }
            }
            WidgetType::Label { value } => {
                ui.text(value.as_str());
            }
        }

        (action, remove_clicked)
    }
}

#[derive(Debug, Clone)]
pub enum DashboardAction {
    AddWidget,
    RemoveWidget(u64),
    ValueChanged(u64, f32),
    BoolChanged(u64, bool),
    ButtonPressed(u64),
    SendCommand(PlaybackCommand),
}
