//! Cue System UI Panel
use std::time::Duration;

use egui::{self, ComboBox, RichText, ScrollArea, Slider};
use mapmap_control::{
    cue::{triggers::*, Cue, CueList},
    ControlManager,
};

use crate::{
    i18n::LocaleManager,
    icons::{AppIcon, IconManager},
    UIAction,
};

// This will be moved to lib.rs's UIAction enum later
// to integrate properly with the main application action loop.
#[derive(Debug, Clone)]
pub enum CueAction {
    Add,
    Remove(u32),
    Go(u32),
    Next,
    Prev,
    Jump(u32),
    UpdateCue(Box<Cue>),
}

#[derive(Default)]
pub struct CuePanel {
    pub visible: bool, // Allow visibility control
    selected_cue_id: Option<u32>,
    jump_target_id: String,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum TriggerTypeUI {
    Manual,
    Osc,
    Midi,
    Time,
}

impl CuePanel {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        control_manager: &mut ControlManager,
        i18n: &LocaleManager,
        actions: &mut Vec<UIAction>,
        icon_manager: Option<&IconManager>,
    ) {
        if !self.visible {
            return;
        }

        let mut open = self.visible;
        egui::Window::new(i18n.t("panel-cues"))
            .open(&mut open)
            .default_size([300.0, 500.0])
            .show(ctx, |ui| {
                self.render_ui(
                    ui,
                    control_manager.cue_list_mut(),
                    i18n,
                    actions,
                    icon_manager,
                );
            });
        self.visible = open;
    }

    fn render_ui(
        &mut self,
        ui: &mut egui::Ui,
        cue_list: &mut CueList,
        i18n: &LocaleManager,
        _actions: &mut Vec<UIAction>,
        icon_manager: Option<&IconManager>,
    ) {
        // --- Top Control Bar ---
        ui.horizontal(|ui| {
            if let Some(mgr) = icon_manager {
                if let Some(img) = mgr.image(AppIcon::ButtonPlay, 24.0) {
                    if ui
                        .add(egui::ImageButton::new(img))
                        .on_hover_text(i18n.t("btn-go"))
                        .clicked()
                    {
                        // TODO: Fire CueAction::Next
                    }
                }
            }

            if let Some(mgr) = icon_manager {
                if let Some(img) = mgr.image(AppIcon::ArrowLeft, 24.0) {
                    if ui
                        .add(egui::ImageButton::new(img))
                        .on_hover_text(i18n.t("btn-back"))
                        .clicked()
                    {
                        // TODO: Fire CueAction::Prev
                    }
                }
            }

            ui.separator();

            ui.label(i18n.t("label-jump-to"));
            ui.text_edit_singleline(&mut self.jump_target_id);
            if ui.button(i18n.t("btn-jump")).clicked() {
                if let Ok(id) = self.jump_target_id.parse::<u32>() {
                    // TODO: Fire CueAction::Jump(id)
                    println!("Jump to {}", id);
                }
            }
        });

        ui.separator();

        // --- Cue List ---
        ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
            let current_cue_id = cue_list.current_cue();
            let cues_to_render: Vec<_> = cue_list.cues().to_vec();

            if cues_to_render.is_empty() {
                ui.label(i18n.t("label-no-cues"));
            } else {
                for cue in cues_to_render {
                    let is_current = current_cue_id == Some(cue.id);
                    let is_selected = self.selected_cue_id == Some(cue.id);

                    let label_text = format!("{} - {}", cue.id, cue.name);
                    let label = if is_current {
                        RichText::new(label_text).color(ui.visuals().selection.bg_fill)
                    } else {
                        RichText::new(label_text)
                    };

                    if ui.selectable_label(is_selected, label).clicked() {
                        self.selected_cue_id = Some(cue.id);
                    }
                }
            }
        });

        ui.separator();

        // --- Cue Editor ---
        if let Some(selected_id) = self.selected_cue_id {
            // We clone the cue to edit it without holding a mutable borrow on the cue_list,
            // which would prevent us from using cue_list for other things inside the editor.
            if let Some(cue_to_edit) = cue_list
                .cues()
                .iter()
                .find(|c| c.id == selected_id)
                .cloned()
            {
                ui.group(|ui| {
                    ui.heading(i18n.t("header-cue-editor"));

                    let mut updated_cue = cue_to_edit;
                    if self.render_cue_editor(ui, &mut updated_cue, i18n) {
                        // If changed, find the original cue in the list and update it.
                        if let Some(original_cue) = cue_list.get_cue_mut(selected_id) {
                            *original_cue = updated_cue;
                        }
                    }
                });
            } else {
                // The selected cue might have been removed.
                self.selected_cue_id = None;
            }
        }

        ui.separator();

        // --- Management Buttons ---
        ui.horizontal(|ui| {
            if let Some(mgr) = icon_manager {
                if let Some(img) = mgr.image(AppIcon::Add, 16.0) {
                    if ui
                        .add(egui::ImageButton::new(img))
                        .on_hover_text(i18n.t("btn-add-cue"))
                        .clicked()
                    {
                        let new_id = cue_list.cues().iter().map(|c| c.id).max().unwrap_or(0) + 1;
                        let new_cue = Cue::new(new_id, format!("New Cue {}", new_id));
                        cue_list.add_cue(new_cue);
                        self.selected_cue_id = Some(new_id);
                    }
                }
            }

            if let Some(selected_id) = self.selected_cue_id {
                if let Some(mgr) = icon_manager {
                    if let Some(img) = mgr.image(AppIcon::Remove, 16.0) {
                        if ui
                            .add(egui::ImageButton::new(img))
                            .on_hover_text(i18n.t("btn-remove-cue"))
                            .clicked()
                        {
                            cue_list.remove_cue(selected_id);
                            self.selected_cue_id = None;
                        }
                    }
                }
            }
        });
    }

    /// Renders the editor for a given cue's properties.
    /// Returns `true` if the cue was changed.
    fn render_cue_editor(
        &mut self,
        ui: &mut egui::Ui,
        cue: &mut Cue,
        i18n: &LocaleManager,
    ) -> bool {
        let mut changed = false;

        // --- Name ---
        ui.horizontal(|ui| {
            ui.label(i18n.t("label-name"));
            if ui.text_edit_singleline(&mut cue.name).changed() {
                changed = true;
            }
        });

        // --- Fade Duration ---
        ui.horizontal(|ui| {
            ui.label(i18n.t("label-fade-duration"));
            let mut fade_secs = cue.fade_duration.as_secs_f32();
            if ui
                .add(Slider::new(&mut fade_secs, 0.0..=30.0).suffix("s"))
                .changed()
            {
                cue.fade_duration = Duration::from_secs_f32(fade_secs);
                changed = true;
            }
        });

        // --- Trigger Type ---
        // TODO: The Cue struct in mapmap-core doesn't have an OSC trigger field yet.
        let mut current_trigger_type = if cue.midi_trigger.is_some() {
            TriggerTypeUI::Midi
        } else if cue.time_trigger.is_some() {
            TriggerTypeUI::Time
        } else {
            TriggerTypeUI::Manual
        };

        let old_trigger_type = current_trigger_type;

        ComboBox::from_label(i18n.t("label-trigger-type"))
            .selected_text(format!("{:?}", current_trigger_type))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut current_trigger_type, TriggerTypeUI::Manual, "Manual");
                ui.selectable_value(&mut current_trigger_type, TriggerTypeUI::Osc, "OSC");
                ui.selectable_value(&mut current_trigger_type, TriggerTypeUI::Midi, "MIDI");
                ui.selectable_value(&mut current_trigger_type, TriggerTypeUI::Time, "Time");
            });

        if current_trigger_type != old_trigger_type {
            changed = true;
            cue.midi_trigger = None;
            cue.time_trigger = None;
            match current_trigger_type {
                TriggerTypeUI::Midi => {
                    cue.midi_trigger = Some(MidiTrigger::note(0, 60)); // Default trigger
                }
                TriggerTypeUI::Time => {
                    cue.time_trigger = TimeTrigger::new(0, 0, 0); // Default trigger
                }
                _ => {}
            }
        }

        // --- Trigger-specific settings ---
        match current_trigger_type {
            TriggerTypeUI::Osc => {
                ui.label("OSC trigger settings (not implemented).");
            }
            TriggerTypeUI::Midi => {
                if let Some(_midi_trigger) = &mut cue.midi_trigger {
                    ui.label("MIDI trigger settings (not implemented).");
                }
            }
            TriggerTypeUI::Time => {
                if let Some(_time_trigger) = &mut cue.time_trigger {
                    ui.label("Time trigger settings (not implemented).");
                }
            }
            TriggerTypeUI::Manual => {
                // No settings for manual triggers
            }
        }

        changed
    }
}
