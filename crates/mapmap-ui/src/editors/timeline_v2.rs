//! Phase 6: Enhanced Timeline Editor with Keyframe Animation
//!
//! Multi-track timeline with keyframe animation, using mapmap_core::animation types.

use crate::theme::colors;
use crate::widgets::hold_to_action_button;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Ui, Vec2};
use mapmap_core::animation::AnimValue;
use mapmap_core::effect_animation::EffectParameterAnimator;
use mapmap_core::module::ModuleId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Lightweight module descriptor for timeline arrangement UI.
#[derive(Debug, Clone)]
pub struct TimelineModule<'a> {
    /// Module ID
    pub id: ModuleId,
    /// Module display name
    // Optimization: Borrow name string to prevent allocation overhead in UI hot loop.
    pub name: &'a str,
}

/// Show orchestration mode for module arrangement.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum ShowMode {
    /// Fully automatic module switching by timeline time.
    #[default]
    FullyAutomated,
    /// Timeline advances automatically, module switch is confirmed manually.
    SemiAutomated,
    /// Module switching is manual only (timeline acts as arrangement board).
    Manual,
    /// Hybrid logic combining time and triggers.
    Hybrid,
    /// Playback stops at markers, waiting for explicit trigger to continue.
    Trackline,
}

impl ShowMode {
    fn label(self) -> &'static str {
        match self {
            Self::FullyAutomated => "Fully Auto",
            Self::SemiAutomated => "Semi Auto",
            Self::Manual => "Manual",
            Self::Hybrid => "Hybrid",
            Self::Trackline => "Trackline",
        }
    }
}

/// A scheduled module block on the show timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ModuleArrangementItem {
    /// Unique ID for stable runtime selection.
    pub id: u64,
    /// Target module.
    pub module_id: ModuleId,
    /// Block start time in seconds.
    pub start_time: f32,
    /// Block duration in seconds.
    pub duration: f32,
    /// Whether this block is active in runtime.
    pub enabled: bool,
    /// Trigger that must be active to start this block (Hybrid Mode).
    pub start_trigger: Option<String>,
}

impl Default for ModuleArrangementItem {
    fn default() -> Self {
        Self {
            id: 0,
            module_id: 0,
            start_time: 0.0,
            duration: 8.0,
            enabled: true,
            start_trigger: None,
        }
    }
}

impl ModuleArrangementItem {
    fn end_time(&self) -> f32 {
        self.start_time + self.duration.max(0.1)
    }
}

/// Timeline editor view state (data is in AnimationClip)
#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct TimelineV2 {
    /// Playhead position (in seconds) - purely for visualization if not synced
    pub playhead: f32,
    /// Zoom level (pixels per second)
    pub zoom: f32,
    /// Pan offset
    pub pan_offset: f32,
    /// Snap settings
    pub snap_enabled: bool,
    pub snap_interval: f32,
    /// Selected keyframes (track_name, key_time_us)
    pub selected_keyframes: Vec<(String, u64)>,
    /// Show curve editor
    pub show_curve_editor: bool,
    /// Expanded automation tracks/groups
    pub expanded_tracks: HashSet<String>,
    /// Enable module arrangement show-control.
    pub show_control_enabled: bool,
    /// Selected show mode.
    pub show_mode: ShowMode,
    /// Scheduled module blocks.
    pub module_arrangement: Vec<ModuleArrangementItem>,
    /// UI add-block module selection.
    pub selected_module_id: Option<ModuleId>,
    /// ID counter for arrangement blocks.
    pub next_arrangement_id: u64,
    /// Manual mode current block.
    pub manual_current_block_id: Option<u64>,
    /// Semi-auto current block.
    pub semi_auto_current_block_id: Option<u64>,
    /// Semi-auto pending block (needs GO).
    pub semi_auto_pending_block_id: Option<u64>,
    /// Full-auto last block.
    pub full_auto_current_block_id: Option<u64>,
    /// Hybrid mode current block.
    pub hybrid_current_block_id: Option<u64>,
    /// Hybrid mode active triggers.
    pub hybrid_active_triggers: HashSet<String>,
    /// Selected marker ID.
    pub selected_marker_id: Option<u64>,
}

impl Default for TimelineV2 {
    fn default() -> Self {
        Self {
            playhead: 0.0,
            zoom: 100.0,
            pan_offset: 0.0,
            snap_enabled: true,
            snap_interval: 0.1, // 100ms default snap
            selected_keyframes: Vec::new(),
            show_curve_editor: false,
            expanded_tracks: HashSet::new(),
            show_control_enabled: true,
            show_mode: ShowMode::FullyAutomated,
            module_arrangement: Vec::new(),
            selected_module_id: None,
            next_arrangement_id: 1,
            manual_current_block_id: None,
            semi_auto_current_block_id: None,
            semi_auto_pending_block_id: None,
            full_auto_current_block_id: None,
            hybrid_current_block_id: None,
            hybrid_active_triggers: HashSet::new(),
            selected_marker_id: None,
        }
    }
}

impl TimelineV2 {
    /// Snap time to grid
    fn snap_time(&self, time: f32) -> f32 {
        if self.snap_enabled && self.snap_interval > 0.0 {
            (time / self.snap_interval).round() * self.snap_interval
        } else {
            time
        }
    }

    fn sorted_enabled_blocks(&self) -> Vec<&ModuleArrangementItem> {
        let mut blocks: Vec<&ModuleArrangementItem> = self
            .module_arrangement
            .iter()
            .filter(|item| item.enabled)
            .collect();
        blocks.sort_by(|a, b| a.start_time.total_cmp(&b.start_time).then(a.id.cmp(&b.id)));
        blocks
    }

    fn sorted_enabled_block_ids(&self) -> Vec<u64> {
        let mut pairs: Vec<(u64, f32)> = self
            .module_arrangement
            .iter()
            .filter(|item| item.enabled)
            .map(|item| (item.id, item.start_time))
            .collect();
        pairs.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
        pairs.into_iter().map(|(id, _)| id).collect()
    }

    fn find_block(&self, block_id: u64) -> Option<&ModuleArrangementItem> {
        self.module_arrangement
            .iter()
            .find(|item| item.id == block_id)
    }

    fn first_enabled_block_id(&self) -> Option<u64> {
        self.sorted_enabled_blocks().first().map(|item| item.id)
    }

    fn active_block_for_time(&self, time: f32) -> Option<&ModuleArrangementItem> {
        let blocks = self.sorted_enabled_blocks();
        if blocks.is_empty() {
            return None;
        }

        for block in &blocks {
            if time >= block.start_time && time < block.end_time() {
                return Some(block);
            }
        }

        if let Some(last_before) = blocks.iter().rev().find(|block| time >= block.start_time) {
            return Some(last_before);
        }

        blocks.first().copied()
    }

    fn module_name_map<'a>(modules: &[TimelineModule<'a>]) -> HashMap<ModuleId, &'a str> {
        modules.iter().map(|m| (m.id, m.name)).collect()
    }

    fn module_name(module_names: &HashMap<ModuleId, &str>, module_id: ModuleId) -> String {
        module_names
            .get(&module_id)
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("Module {}", module_id))
    }

    fn reset_runtime_selection(&mut self) {
        self.manual_current_block_id = None;
        self.semi_auto_current_block_id = None;
        self.semi_auto_pending_block_id = None;
        self.full_auto_current_block_id = None;
        self.hybrid_current_block_id = None;
    }

    fn cleanup_missing_modules(&mut self, available_module_ids: &[ModuleId]) {
        let valid: HashSet<ModuleId> = available_module_ids.iter().copied().collect();
        self.module_arrangement
            .retain(|item| valid.contains(&item.module_id));

        let has_block = |id: Option<u64>, blocks: &[ModuleArrangementItem]| {
            id.is_some_and(|block_id| blocks.iter().any(|item| item.id == block_id))
        };

        if !has_block(self.manual_current_block_id, &self.module_arrangement) {
            self.manual_current_block_id = None;
        }
        if !has_block(self.semi_auto_current_block_id, &self.module_arrangement) {
            self.semi_auto_current_block_id = None;
        }
        if !has_block(self.semi_auto_pending_block_id, &self.module_arrangement) {
            self.semi_auto_pending_block_id = None;
        }
        if !has_block(self.full_auto_current_block_id, &self.module_arrangement) {
            self.full_auto_current_block_id = None;
        }
        if !has_block(self.hybrid_current_block_id, &self.module_arrangement) {
            self.hybrid_current_block_id = None;
        }
    }

    fn add_module_block(&mut self, module_id: ModuleId) {
        let default_start = self
            .module_arrangement
            .iter()
            .map(ModuleArrangementItem::end_time)
            .fold(0.0, f32::max);
        let id = self.next_arrangement_id;
        self.next_arrangement_id = self.next_arrangement_id.saturating_add(1);

        self.module_arrangement.push(ModuleArrangementItem {
            id,
            module_id,
            start_time: default_start,
            duration: 8.0,
            enabled: true,
            start_trigger: None,
        });
    }

    fn set_manual_current(&mut self, block_id: Option<u64>) {
        self.manual_current_block_id = block_id;
    }

    fn module_for_block_id(&self, block_id: Option<u64>) -> Option<ModuleId> {
        block_id
            .and_then(|id| self.find_block(id))
            .map(|block| block.module_id)
    }

    /// Returns the module that should be active for show playback.
    /// `None` means "do not filter modules".
    pub fn runtime_show_module(
        &mut self,
        current_time: f32,
        is_playing: bool,
        available_module_ids: &[ModuleId],
    ) -> Option<ModuleId> {
        self.cleanup_missing_modules(available_module_ids);

        if !self.show_control_enabled {
            return None;
        }
        if self.sorted_enabled_blocks().is_empty() {
            return None;
        }

        match self.show_mode {
            ShowMode::FullyAutomated | ShowMode::Trackline => {
                let active_id = self.active_block_for_time(current_time).map(|b| b.id);
                self.full_auto_current_block_id = active_id;
                self.manual_current_block_id = active_id;
                self.module_for_block_id(active_id)
            }
            ShowMode::SemiAutomated => {
                if self.semi_auto_current_block_id.is_none() {
                    self.semi_auto_current_block_id = self.first_enabled_block_id();
                }

                if is_playing {
                    if let Some(time_block_id) =
                        self.active_block_for_time(current_time).map(|b| b.id)
                    {
                        if self.semi_auto_current_block_id != Some(time_block_id) {
                            self.semi_auto_pending_block_id = Some(time_block_id);
                        }
                    }
                }

                if self.semi_auto_current_block_id.is_none() {
                    self.semi_auto_current_block_id = self.first_enabled_block_id();
                }

                self.module_for_block_id(self.semi_auto_current_block_id)
            }
            ShowMode::Manual => {
                if self.manual_current_block_id.is_none() {
                    self.manual_current_block_id = self.first_enabled_block_id();
                }
                self.module_for_block_id(self.manual_current_block_id)
            }
            ShowMode::Hybrid => {
                if self.hybrid_current_block_id.is_none() {
                    self.hybrid_current_block_id = self.first_enabled_block_id();
                }

                if is_playing {
                    let blocks = self.sorted_enabled_blocks();

                    // Find all blocks that overlap with the current time
                    let mut active_blocks: Vec<&ModuleArrangementItem> = blocks
                        .iter()
                        .copied()
                        .filter(|b| current_time >= b.start_time && current_time < b.end_time())
                        .collect();

                    // Sort by whether they require triggers (those without triggers are defaults)
                    active_blocks
                        .sort_by(|a, b| a.start_trigger.is_some().cmp(&b.start_trigger.is_some()));

                    let mut next_block_id = self.hybrid_current_block_id;

                    // Evaluate blocks matching current time
                    for block in active_blocks {
                        if let Some(trigger) = &block.start_trigger {
                            if self.hybrid_active_triggers.contains(trigger) {
                                next_block_id = Some(block.id);
                                break; // Trigger matched, take this block
                            }
                        } else {
                            // Block has no trigger, it's the default for this time
                            let current_is_active =
                                if let Some(current_id) = self.hybrid_current_block_id {
                                    blocks.iter().find(|b| b.id == current_id).is_some_and(|b| {
                                        current_time >= b.start_time && current_time < b.end_time()
                                    })
                                } else {
                                    false
                                };

                            if !current_is_active {
                                next_block_id = Some(block.id);
                            }
                        }
                    }

                    if next_block_id != self.hybrid_current_block_id {
                        self.hybrid_current_block_id = next_block_id;
                    }
                }

                self.module_for_block_id(self.hybrid_current_block_id)
            }
        }
    }

    /// In manual mode, advance to next arranged module.
    pub fn step_manual_next(&mut self) -> Option<ModuleId> {
        let block_ids = self.sorted_enabled_block_ids();
        if block_ids.is_empty() {
            self.manual_current_block_id = None;
            return None;
        }

        let next_index = if let Some(current_id) = self.manual_current_block_id {
            let idx = block_ids
                .iter()
                .position(|id| *id == current_id)
                .unwrap_or(0);
            (idx + 1) % block_ids.len()
        } else {
            0
        };

        self.manual_current_block_id = Some(block_ids[next_index]);
        self.module_for_block_id(self.manual_current_block_id)
    }

    /// In manual mode, go to previous arranged module.
    pub fn step_manual_prev(&mut self) -> Option<ModuleId> {
        let block_ids = self.sorted_enabled_block_ids();
        if block_ids.is_empty() {
            self.manual_current_block_id = None;
            return None;
        }

        let prev_index = if let Some(current_id) = self.manual_current_block_id {
            let idx = block_ids
                .iter()
                .position(|id| *id == current_id)
                .unwrap_or(0);
            if idx == 0 {
                block_ids.len() - 1
            } else {
                idx - 1
            }
        } else {
            0
        };

        self.manual_current_block_id = Some(block_ids[prev_index]);
        self.module_for_block_id(self.manual_current_block_id)
    }

    /// In semi-auto mode, confirm or advance to next module.
    pub fn step_semi_auto_next(&mut self) -> Option<ModuleId> {
        if let Some(pending) = self.semi_auto_pending_block_id.take() {
            self.semi_auto_current_block_id = Some(pending);
            return self.module_for_block_id(self.semi_auto_current_block_id);
        }

        let block_ids = self.sorted_enabled_block_ids();
        if block_ids.is_empty() {
            self.semi_auto_current_block_id = None;
            return None;
        }

        let next_index = if let Some(current_id) = self.semi_auto_current_block_id {
            let idx = block_ids
                .iter()
                .position(|id| *id == current_id)
                .unwrap_or(0);
            (idx + 1).min(block_ids.len().saturating_sub(1))
        } else {
            0
        };

        self.semi_auto_current_block_id = Some(block_ids[next_index]);
        self.module_for_block_id(self.semi_auto_current_block_id)
    }

    /// Render the timeline UI interacting with the EffectParameterAnimator
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        animator: &mut EffectParameterAnimator,
        modules: &[TimelineModule<'_>],
    ) -> Option<TimelineAction> {
        let mut action = None;
        let module_names = Self::module_name_map(modules);
        let available_module_ids: Vec<ModuleId> = modules.iter().map(|m| m.id).collect();

        // Ensure pause_at_markers reflects the current ShowMode
        animator.set_pause_at_markers(self.show_mode == ShowMode::Trackline);

        // Sync local playhead with animator
        self.playhead = animator.get_current_time() as f32;

        let duration = animator.duration() as f32;

        // Toolbar
        ui.horizontal(|ui| {
            if animator.is_playing() {
                if ui.button("Pause").clicked() {
                    action = Some(TimelineAction::Pause);
                }
            } else if ui.button("Play").clicked() {
                action = Some(TimelineAction::Play);
            }

            if hold_to_action_button(ui, "Stop", colors::ERROR_COLOR) {
                action = Some(TimelineAction::Stop);
            }

            ui.separator();

            ui.label(format!("Time: {:.2}s", self.playhead));

            ui.separator();

            // Snap settings
            ui.checkbox(&mut self.snap_enabled, "Snap");
            if self.snap_enabled {
                ui.add(
                    egui::DragValue::new(&mut self.snap_interval)
                        .prefix("Snap: ")
                        .suffix("s")
                        .speed(0.01)
                        .range(0.01..=10.0),
                );
            }

            ui.separator();

            // Zoom controls
            ui.label(format!("Zoom: {:.0}px/s", self.zoom));
            if ui.button("+").clicked() {
                self.zoom *= 1.2;
            }
            if ui.button("-").clicked() {
                self.zoom /= 1.2;
            }

            ui.separator();

            if ui.button("Add Marker").clicked() {
                action = Some(TimelineAction::AddMarker(self.playhead));
            }

            // Playback Mode selection
            let mut current_mode = animator.clip().playback_mode;
            let mut mode_changed = false;
            egui::ComboBox::from_id_salt("playback_mode_combo")
                .selected_text(format!("{:?}", current_mode))
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_value(
                            &mut current_mode,
                            mapmap_core::animation::PlaybackMode::Loop,
                            "Loop",
                        )
                        .clicked()
                    {
                        mode_changed = true;
                    }
                    if ui
                        .selectable_value(
                            &mut current_mode,
                            mapmap_core::animation::PlaybackMode::PingPong,
                            "PingPong",
                        )
                        .clicked()
                    {
                        mode_changed = true;
                    }
                    if ui
                        .selectable_value(
                            &mut current_mode,
                            mapmap_core::animation::PlaybackMode::OneShot,
                            "OneShot",
                        )
                        .clicked()
                    {
                        mode_changed = true;
                    }
                });
            if mode_changed {
                animator.set_playback_mode(current_mode);
            }

            // Reverse playback
            let mut reverse = animator.clip().reverse;
            if ui.checkbox(&mut reverse, "Reverse").changed() {
                animator.set_reverse(reverse);
            }

            ui.separator();

            // BPM Sync
            let mut bpm_sync = animator.clip().bpm_sync;
            let mut bpm = animator.clip().bpm;
            let mut beats = animator.clip().beats;
            let mut bpm_changed = false;
            if ui.checkbox(&mut bpm_sync, "BPM Sync").changed() {
                bpm_changed = true;
            }
            if bpm_sync {
                if ui
                    .add(
                        egui::DragValue::new(&mut bpm)
                            .prefix("BPM: ")
                            .speed(1.0)
                            .range(1.0..=999.0),
                    )
                    .changed()
                {
                    bpm_changed = true;
                }
                if ui
                    .add(
                        egui::DragValue::new(&mut beats)
                            .prefix("Beats: ")
                            .speed(1.0)
                            .range(1.0..=128.0),
                    )
                    .changed()
                {
                    bpm_changed = true;
                }
            }
            if bpm_changed {
                animator.set_bpm_sync(bpm_sync, bpm, beats);
            }

            ui.separator();

            // In/Out Points
            let mut in_pt = animator.clip().in_point.unwrap_or(0.0);
            let mut out_pt = animator.clip().out_point.unwrap_or(animator.duration());
            let mut pts_changed = false;

            ui.label("In:");
            if ui
                .add(
                    egui::DragValue::new(&mut in_pt)
                        .speed(0.1)
                        .range(0.0..=out_pt - 0.1),
                )
                .changed()
            {
                pts_changed = true;
            }
            ui.label("Out:");
            if ui
                .add(
                    egui::DragValue::new(&mut out_pt)
                        .speed(0.1)
                        .range(in_pt + 0.1..=animator.duration()),
                )
                .changed()
            {
                pts_changed = true;
            }

            if ui.button("Clear I/O").clicked() {
                animator.set_in_out_points(None, None);
            } else if pts_changed {
                animator.set_in_out_points(Some(in_pt), Some(out_pt));
            }

            ui.separator();

            ui.checkbox(&mut self.show_control_enabled, "Module Show");
            if self.show_control_enabled {
                egui::ComboBox::from_id_salt("show_mode_combo")
                    .selected_text(self.show_mode.label())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.show_mode,
                            ShowMode::FullyAutomated,
                            ShowMode::FullyAutomated.label(),
                        );
                        ui.selectable_value(
                            &mut self.show_mode,
                            ShowMode::SemiAutomated,
                            ShowMode::SemiAutomated.label(),
                        );
                        ui.selectable_value(
                            &mut self.show_mode,
                            ShowMode::Manual,
                            ShowMode::Manual.label(),
                        );
                        ui.selectable_value(
                            &mut self.show_mode,
                            ShowMode::Hybrid,
                            ShowMode::Hybrid.label(),
                        );
                        ui.selectable_value(
                            &mut self.show_mode,
                            ShowMode::Trackline,
                            ShowMode::Trackline.label(),
                        );
                    });

                match self.show_mode {
                    ShowMode::SemiAutomated => {
                        if ui.button("GO Next").clicked() {
                            if let Some(module_id) = self.step_semi_auto_next() {
                                action = Some(TimelineAction::SelectModule(module_id));
                            }
                        }
                    }
                    ShowMode::Manual => {
                        if ui.button("Prev").clicked() {
                            if let Some(module_id) = self.step_manual_prev() {
                                action = Some(TimelineAction::SelectModule(module_id));
                            }
                        }
                        if ui.button("Next").clicked() {
                            if let Some(module_id) = self.step_manual_next() {
                                action = Some(TimelineAction::SelectModule(module_id));
                            }
                        }
                    }
                    ShowMode::FullyAutomated | ShowMode::Hybrid | ShowMode::Trackline => {}
                }
            }
        });

        ui.separator();

        if self.selected_module_id.is_none() {
            self.selected_module_id = modules.first().map(|m| m.id);
        }

        ui.group(|ui| {
            ui.label("Module Arrangement");
            ui.horizontal(|ui| {
                if modules.is_empty() {
                    ui.label(egui::RichText::new("No modules available").weak().italics());
                } else {
                    let selected = self.selected_module_id.unwrap_or(modules[0].id);
                    let selected_label = Self::module_name(&module_names, selected);
                    egui::ComboBox::from_id_salt("timeline_module_select")
                        .selected_text(selected_label)
                        .show_ui(ui, |ui| {
                            for module in modules {
                                ui.selectable_value(
                                    &mut self.selected_module_id,
                                    Some(module.id),
                                    module.name,
                                );
                            }
                        });

                    if ui.button("Add Block").clicked() {
                        if let Some(module_id) = self.selected_module_id {
                            self.add_module_block(module_id);
                        }
                    }
                }

                if ui.button("Sort").clicked() {
                    self.module_arrangement.sort_by(|a, b| {
                        a.start_time.total_cmp(&b.start_time).then(a.id.cmp(&b.id))
                    });
                }
                if crate::widgets::custom::hold_to_action_button(
                    ui,
                    "Clear",
                    crate::theme::colors::WARN_COLOR,
                ) {
                    self.module_arrangement.clear();
                    self.reset_runtime_selection();
                }
            });

            let mut remove_block_id: Option<u64> = None;
            let mut jump_to_block: Option<(f32, u64)> = None;

            for block in &mut self.module_arrangement {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut block.enabled, "");

                    let selected_label = Self::module_name(&module_names, block.module_id);
                    egui::ComboBox::from_id_salt(format!("timeline_block_module_{}", block.id))
                        .selected_text(selected_label)
                        .show_ui(ui, |ui| {
                            for module in modules {
                                if ui
                                    .selectable_label(block.module_id == module.id, module.name)
                                    .clicked()
                                {
                                    block.module_id = module.id;
                                }
                            }
                        });

                    ui.add(
                        egui::DragValue::new(&mut block.start_time)
                            .prefix("Start ")
                            .suffix("s")
                            .speed(0.05)
                            .range(0.0..=36000.0),
                    );
                    ui.add(
                        egui::DragValue::new(&mut block.duration)
                            .prefix("Len ")
                            .suffix("s")
                            .speed(0.05)
                            .range(0.1..=36000.0),
                    );

                    if self.show_mode == ShowMode::Hybrid {
                        let mut trigger_str = block.start_trigger.clone().unwrap_or_default();
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut trigger_str)
                                .hint_text("Trigger (e.g. MIDI/OSC)")
                                .desired_width(120.0),
                        );
                        if response.changed() {
                            if trigger_str.trim().is_empty() {
                                block.start_trigger = None;
                            } else {
                                block.start_trigger = Some(trigger_str);
                            }
                        }
                    }

                    if ui.button("Jump").clicked() {
                        jump_to_block = Some((block.start_time, block.id));
                    }
                    if ui.button("X").clicked() {
                        remove_block_id = Some(block.id);
                    }
                });
            }

            if let Some((start_time, block_id)) = jump_to_block {
                action = Some(TimelineAction::Seek(start_time));
                if self.show_mode == ShowMode::Manual {
                    self.set_manual_current(Some(block_id));
                }
            }

            if let Some(id) = remove_block_id {
                self.module_arrangement.retain(|block| block.id != id);
                self.cleanup_missing_modules(&available_module_ids);
            }
        });

        ui.separator();

        // Timeline area
        egui::ScrollArea::both().show(ui, |ui| {
            let clip = animator.clip(); // Get immutable ref first to calculate size

            // Group tracks by "lane" (e.g. `Blur_0` from `Blur_0.radius`)
            let mut track_groups: std::collections::BTreeMap<
                String,
                Vec<&mapmap_core::animation::AnimationTrack>,
            > = std::collections::BTreeMap::new();
            for track in &clip.tracks {
                let parts: Vec<&str> = track.name.split('.').collect();
                let group_name = if parts.len() > 1 {
                    parts[0].to_string()
                } else {
                    "General".to_string()
                };
                track_groups.entry(group_name).or_default().push(track);
            }

            let mut visible_lanes_count = 0;
            for (group_name, tracks) in &track_groups {
                visible_lanes_count += 1; // Group header
                if self.expanded_tracks.contains(group_name) {
                    visible_lanes_count += tracks.len(); // Expanded tracks
                }
            }

            let module_track_height = if self.module_arrangement.is_empty() {
                0.0
            } else {
                64.0
            };

            let available_height = 50.0 + (visible_lanes_count as f32 * 60.0) + module_track_height;
            let available_width = (duration * self.zoom).max(ui.available_width());

            let (response, painter) = ui.allocate_painter(
                Vec2::new(available_width, available_height),
                Sense::click_and_drag(),
            );

            let rect = response.rect;

            // Draw time ruler
            let ruler_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 30.0));
            painter.rect_filled(ruler_rect, 0.0, Color32::from_rgb(40, 40, 40));

            // Draw time ticks
            let tick_interval = if self.zoom > 100.0 { 0.1 } else { 1.0 };
            let mut time = 0.0;
            while time <= duration {
                let x = rect.min.x + time * self.zoom;
                let h = if (time % 1.0).abs() < 0.001 {
                    15.0
                } else {
                    8.0
                };

                if x >= rect.min.x && x <= rect.max.x {
                    painter.line_segment(
                        [
                            Pos2::new(x, ruler_rect.max.y - h),
                            Pos2::new(x, ruler_rect.max.y),
                        ],
                        Stroke::new(1.0, Color32::from_rgb(150, 150, 150)),
                    );

                    if (time % 1.0).abs() < 0.001 {
                        painter.text(
                            Pos2::new(x + 2.0, ruler_rect.min.y + 2.0),
                            egui::Align2::LEFT_TOP,
                            format!("{:.0}s", time),
                            egui::FontId::proportional(12.0),
                            Color32::WHITE,
                        );
                    }
                }
                time += tick_interval;
            }

            // Draw markers
            let mut remove_marker_id: Option<u64> = None;
            for marker in &clip.markers {
                let x = rect.min.x + (marker.time as f32) * self.zoom;
                if x >= rect.min.x && x <= rect.max.x {
                    // Marker line
                    painter.line_segment(
                        [Pos2::new(x, ruler_rect.min.y), Pos2::new(x, rect.max.y)],
                        Stroke::new(1.0, Color32::from_rgb(100, 200, 100)),
                    );

                    // Marker flag
                    let flag_rect =
                        Rect::from_min_size(Pos2::new(x, ruler_rect.min.y), Vec2::new(14.0, 14.0));
                    let is_selected = self.selected_marker_id == Some(marker.id);
                    let flag_color = if is_selected {
                        Color32::from_rgb(150, 255, 150)
                    } else {
                        Color32::from_rgb(50, 150, 50)
                    };

                    painter.rect_filled(flag_rect, 2.0, flag_color);
                    painter.text(
                        Pos2::new(x + 2.0, ruler_rect.min.y + 1.0),
                        egui::Align2::LEFT_TOP,
                        "M",
                        egui::FontId::proportional(10.0),
                        Color32::WHITE,
                    );

                    let interact_rect = Rect::from_min_size(
                        Pos2::new(x - 5.0, ruler_rect.min.y),
                        Vec2::new(20.0, 16.0),
                    );
                    let marker_response =
                        ui.interact(interact_rect, ui.id().with(marker.id), Sense::click());

                    if marker_response.clicked() {
                        self.selected_marker_id = Some(marker.id);
                        action = Some(TimelineAction::Seek(marker.time as f32));
                    }
                    if marker_response.secondary_clicked() {
                        remove_marker_id = Some(marker.id);
                    }

                    // Tooltip
                    marker_response
                        .on_hover_text(format!("Marker: {}\nRight-click to remove", marker.name));
                }
            }
            if let Some(id) = remove_marker_id {
                action = Some(TimelineAction::RemoveMarker(id));
            }

            // Draw playhead
            let playhead_x = rect.min.x + self.playhead * self.zoom;
            painter.line_segment(
                [
                    Pos2::new(playhead_x, ruler_rect.min.y),
                    Pos2::new(playhead_x, rect.max.y),
                ],
                Stroke::new(2.0, Color32::from_rgb(255, 50, 50)),
            );

            // Handle ruler scrubbing
            if response.hovered() || response.dragged() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if pos.y <= ruler_rect.max.y && response.is_pointer_button_down_on() {
                        let time = (pos.x - rect.min.x) / self.zoom;
                        let snapped = if ui.input(|i| i.modifiers.shift) {
                            time // Bypass snap
                        } else {
                            self.snap_time(time)
                        };

                        action = Some(TimelineAction::Seek(snapped));
                    }
                }
            }

            // Access immutable clip for drawing tracks
            let track_start_y = ruler_rect.max.y;
            let mut current_lane_index = 0;

            for (group_name, tracks) in &track_groups {
                let is_expanded = self.expanded_tracks.contains(group_name);
                let header_y = track_start_y + (current_lane_index as f32 * 60.0);
                let header_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x, header_y),
                    Vec2::new(rect.width(), 60.0),
                );

                // Draw header lane
                let header_bg_color = Color32::from_rgb(45, 45, 45);
                painter.rect_filled(header_rect, 0.0, header_bg_color);

                let fold_icon = if is_expanded { "▼" } else { "▶" };
                let header_label = format!("{} {}", fold_icon, group_name);

                // Make header interactive for folding/unfolding
                let header_response =
                    ui.interact(header_rect, ui.id().with(group_name), Sense::click());
                if header_response.clicked() {
                    if is_expanded {
                        self.expanded_tracks.remove(group_name);
                    } else {
                        self.expanded_tracks.insert(group_name.clone());
                    }
                }

                let text_color = if header_response.hovered() {
                    Color32::WHITE
                } else {
                    Color32::from_rgb(220, 220, 220)
                };

                painter.text(
                    Pos2::new(header_rect.min.x + 10.0, header_rect.min.y + 22.0),
                    egui::Align2::LEFT_TOP,
                    header_label,
                    egui::FontId::proportional(14.0),
                    text_color,
                );

                current_lane_index += 1;

                if is_expanded {
                    for track in tracks {
                        let track_y = track_start_y + (current_lane_index as f32 * 60.0);
                        let track_rect = Rect::from_min_size(
                            Pos2::new(rect.min.x, track_y),
                            Vec2::new(rect.width(), 60.0),
                        );

                        // Alternating background for automation tracks
                        let bg_color = if current_lane_index % 2 == 0 {
                            Color32::from_rgb(30, 30, 30)
                        } else {
                            Color32::from_rgb(35, 35, 35)
                        };
                        painter.rect_filled(track_rect, 0.0, bg_color);

                        // Draw track tree line
                        painter.line_segment(
                            [
                                Pos2::new(track_rect.min.x + 15.0, track_rect.min.y),
                                Pos2::new(track_rect.min.x + 15.0, track_rect.max.y),
                            ],
                            Stroke::new(1.0, Color32::from_rgb(80, 80, 80)),
                        );

                        // Track name (parameter)
                        let param_name = track.name.split('.').next_back().unwrap_or(&track.name);
                        painter.text(
                            Pos2::new(track_rect.min.x + 25.0, track_rect.min.y + 10.0),
                            egui::Align2::LEFT_TOP,
                            param_name,
                            egui::FontId::proportional(13.0),
                            Color32::from_rgb(180, 180, 180),
                        );

                        // Draw keyframes and curves
                        let keyframes = track.keyframes_ordered();

                        if keyframes.len() >= 2 {
                            let mut points = Vec::new();
                            for kf in &keyframes {
                                let t = kf.time as f32;
                                let val = match &kf.value {
                                    AnimValue::Float(v) => *v,
                                    AnimValue::Vec3(v) => v[0],
                                    AnimValue::Vec4(v) => v[0],
                                    AnimValue::Color(v) => v[0],
                                    _ => 0.0,
                                };

                                let normalized = val.clamp(0.0, 1.0);
                                let x = rect.min.x + t * self.zoom;
                                let y = track_rect.max.y - 10.0 - (normalized * 40.0);
                                points.push(Pos2::new(x, y));
                            }

                            if !points.is_empty() {
                                painter.add(egui::Shape::line(
                                    points,
                                    Stroke::new(2.0, Color32::from_rgb(100, 200, 255)),
                                ));
                            }
                        }

                        for kf in &keyframes {
                            let kf_time = kf.time as f32;
                            let val = match &kf.value {
                                AnimValue::Float(v) => *v,
                                _ => 0.0,
                            };
                            let normalized = val.clamp(0.0, 1.0);

                            let x = rect.min.x + kf_time * self.zoom;
                            let y = track_rect.max.y - 10.0 - (normalized * 40.0);

                            let diamond_size = 6.0;
                            let diamond = vec![
                                Pos2::new(x, y - diamond_size),
                                Pos2::new(x + diamond_size, y),
                                Pos2::new(x, y + diamond_size),
                                Pos2::new(x - diamond_size, y),
                            ];

                            painter.add(egui::Shape::convex_polygon(
                                diamond,
                                Color32::YELLOW,
                                Stroke::new(1.0, Color32::WHITE),
                            ));
                        }

                        current_lane_index += 1;
                    }
                }
            }

            if module_track_height > 0.0 {
                let module_track_y = track_start_y + (visible_lanes_count as f32 * 60.0);
                let module_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x, module_track_y),
                    Vec2::new(rect.width(), module_track_height),
                );
                painter.rect_filled(module_rect, 0.0, Color32::from_rgb(22, 22, 22));
                painter.text(
                    Pos2::new(module_rect.min.x + 5.0, module_rect.min.y + 6.0),
                    egui::Align2::LEFT_TOP,
                    "Module Show",
                    egui::FontId::proportional(13.0),
                    Color32::from_rgb(200, 220, 255),
                );

                let active_module = self.runtime_show_module(
                    self.playhead,
                    animator.is_playing(),
                    &available_module_ids,
                );

                // TRIGGER ACTION IF CHANGED
                if let Some(mod_id) = active_module {
                    // Check if we need to emit a select action (only if not already the active one in the app)
                    // We use a simple heuristic: if it's the first frame or the ID changed.
                    // For now, we just emit it, the handler in actions.rs should be idempotent.
                    if action.is_none()
                        && animator.is_playing()
                        && (self.show_mode == ShowMode::FullyAutomated
                            || self.show_mode == ShowMode::Hybrid
                            || self.show_mode == ShowMode::Trackline)
                    {
                        action = Some(TimelineAction::SelectModule(mod_id));
                    }
                }

                let active_block_id = match self.show_mode {
                    ShowMode::FullyAutomated | ShowMode::Trackline => {
                        self.full_auto_current_block_id
                    }
                    ShowMode::SemiAutomated => self.semi_auto_current_block_id,
                    ShowMode::Manual => self.manual_current_block_id,
                    ShowMode::Hybrid => self.hybrid_current_block_id,
                };

                for block in self.sorted_enabled_blocks() {
                    let block_x = rect.min.x + block.start_time * self.zoom;
                    let block_w = (block.duration * self.zoom).max(8.0);
                    let block_rect = Rect::from_min_size(
                        Pos2::new(block_x, module_rect.min.y + 24.0),
                        Vec2::new(block_w, 28.0),
                    );

                    let color = if self.semi_auto_pending_block_id == Some(block.id) {
                        Color32::from_rgb(255, 170, 0)
                    } else if active_block_id == Some(block.id) {
                        Color32::from_rgb(40, 180, 80)
                    } else if active_module == Some(block.module_id) {
                        Color32::from_rgb(55, 130, 200)
                    } else {
                        Color32::from_rgb(70, 70, 90)
                    };

                    painter.rect_filled(block_rect, 3.0, color);
                    painter.rect_stroke(
                        block_rect,
                        3.0,
                        Stroke::new(1.0, Color32::from_rgb(230, 230, 230)),
                        egui::StrokeKind::Middle,
                    );

                    let label = Self::module_name(&module_names, block.module_id);
                    painter.text(
                        Pos2::new(block_rect.min.x + 4.0, block_rect.min.y + 6.0),
                        egui::Align2::LEFT_TOP,
                        label,
                        egui::FontId::proportional(12.0),
                        Color32::WHITE,
                    );
                }
            }
        });

        action
    }
}

/// Actions triggered by timeline
#[derive(Debug, Clone, Copy)]
pub enum TimelineAction {
    Play,
    Pause,
    Stop,
    Seek(f32),
    SelectModule(ModuleId),
    AddMarker(f32),
    RemoveMarker(u64),
    ToggleMarkerPause(f32),
    JumpNextMarker,
    JumpPrevMarker,
}
