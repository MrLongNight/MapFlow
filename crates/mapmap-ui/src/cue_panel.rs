//! Cue System UI Panel
use std::time::Duration;

use egui::{self, ComboBox, RichText, ScrollArea, Slider};
use egui_dnd::dnd;
use mapmap_control::{
    cue::{triggers::*, Cue, CueList},
    ControlManager,
};

use crate::{i18n::LocaleManager, UIAction};

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
    ) {
        if !self.visible {
            return;
        }

        let mut open = self.visible;
        egui::Window::new(i18n.t("panel-cues"))
            .open(&mut open)
            .default_size([300.0, 500.0])
            .show(ctx, |ui| {
                self.render_ui(ui, control_manager.cue_list_mut(), i18n, actions);
            });
        self.visible = open;
    }

    fn render_ui(
        &mut self,
        ui: &mut egui::Ui,
        cue_list: &mut CueList,
        i18n: &LocaleManager,
        actions: &mut Vec<UIAction>,
    ) {
        // --- Top Control Bar ---
        ui.horizontal(|ui| {
            if ui.button(i18n.t("btn-go")).clicked() {
                actions.push(UIAction::NextCue);
            }

            if ui.button(i18n.t("btn-back")).clicked() {
                actions.push(UIAction::PrevCue);
            }

            if ui.button(i18n.t("btn-stop")).clicked() {
                actions.push(UIAction::StopCue);
            }

            ui.separator();

            ui.label(i18n.t("label-jump-to"));
            ui.text_edit_singleline(&mut self.jump_target_id);
            if ui.button(i18n.t("btn-jump")).clicked() {
                if let Ok(id) = self.jump_target_id.parse::<u32>() {
                    actions.push(UIAction::JumpCue(id));
                }
            }
        });

        ui.separator();

        // --- Transport Display ---
        ui.group(|ui| {
            if let Some(crossfade) = cue_list.current_crossfade() {
                let progress = crossfade.progress();
                let duration = crossfade.duration();
                let elapsed = duration.as_secs_f32() * progress;

                ui.add(egui::ProgressBar::new(progress).show_percentage());
                ui.horizontal(|ui| {
                    ui.label(format!("{:.1}s", elapsed));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("{:.1}s", duration.as_secs_f32()));
                    });
                });
            } else {
                ui.add(egui::ProgressBar::new(0.0).show_percentage());
                ui.horizontal(|ui| {
                     ui.label("0.0s");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label("0.0s");
                    });
                });
            }
        });

        ui.separator();

        // --- Cue List ---
        ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
            let current_cue_id = cue_list.current_cue();
            let response = dnd(ui, "cue_list_dnd").show(cue_list.cues().iter(), |ui, cue, handle, _state| {
                ui.horizontal(|ui| {
                    handle.ui(ui, |ui| {
                        let is_current = current_cue_id == Some(cue.id);
                        let is_selected = self.selected_cue_id == Some(cue.id);

                        let trigger_type = if cue.midi_trigger.is_some() {
                            "MIDI"
                        } else if cue.time_trigger.is_some() {
                            "Time"
                        } else {
                            "Manual"
                        };
                        let duration_str = format!("{:.1}s", cue.fade_duration.as_secs_f32());

                        let name = if cue.name.len() > 18 {
                            format!("{}...", &cue.name[..15])
                        } else {
                            cue.name.clone()
                        };

                        let label_text = format!(
                            "{:<4} {:<18} {:<6} {:<8}",
                            cue.id, name, duration_str, trigger_type
                        );

                        let mut rich_text = RichText::new(label_text).monospace();
                        if is_current {
                            rich_text = rich_text.color(ui.visuals().selection.bg_fill);
                        }

                        let response = ui.selectable_label(is_selected, rich_text);

                        if response.clicked() {
                            actions.push(UIAction::GoCue(cue.id));
                        }

                        if response.secondary_clicked() {
                             self.selected_cue_id = Some(cue.id);
                        }
                    });
                });
            });

            if let Some(response) = response.update {
                cue_list.move_cue(response.from, response.to);
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
                    if self.render_cue_editor(ui, &mut updated_cue, i18n, actions) {
                        actions.push(UIAction::UpdateCue(Box::new(updated_cue)));
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
            if ui.button(i18n.t("btn-add-cue")).clicked() {
                actions.push(UIAction::AddCue);
            }

            if let Some(selected_id) = self.selected_cue_id {
                if ui.button(i18n.t("btn-remove-cue")).clicked() {
                    actions.push(UIAction::RemoveCue(selected_id));
                    self.selected_cue_id = None;
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
        actions: &mut Vec<UIAction>,
    ) -> bool {
        let mut changed = false;

        // --- Name ---
        ui.horizontal(|ui| {
            ui.label(i18n.t("label-name"));
            if ui.text_edit_singleline(&mut cue.name).changed() {
                changed = true;
            }
        });

        // --- Description ---
        ui.horizontal(|ui| {
            ui.label(i18n.t("label-description"));
            if ui.text_edit_multiline(&mut cue.description).changed() {
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
                if let Some(midi_trigger) = &mut cue.midi_trigger {
                    ui.horizontal(|ui| {
                        ui.label("Channel:");
                        if ui.add(Slider::new(&mut midi_trigger.channel, 0..=15)).changed() {
                            changed = true;
                        }
                    });
                    // Note field as a drag value for now, can be improved later
                    if let MidiTriggerType::Note { ref mut note } = midi_trigger.trigger_type {
                        ui.horizontal(|ui| {
                            ui.label("Note:");
                            if ui.add(egui::DragValue::new(note)).changed() {
                                changed = true;
                            }
                        });
                    }
                }
            }
            TriggerTypeUI::Time => {
                if let Some(time_trigger) = &mut cue.time_trigger {
                     ui.horizontal(|ui| {
                        ui.label("Time (H:M:S):");
                        if ui.add(egui::DragValue::new(&mut time_trigger.hour).clamp_range(0..=23)).changed() {
                            changed = true;
                        }
                        if ui.add(egui::DragValue::new(&mut time_trigger.minute).clamp_range(0..=59)).changed() {
                            changed = true;
                        }
                        if ui.add(egui::DragValue::new(&mut time_trigger.second).clamp_range(0..=59)).changed() {
                            changed = true;
                        }
                    });
                }
            }
            TriggerTypeUI::Manual => {
                // No settings for manual triggers
            }
        }

        ui.separator();

        if ui.button(i18n.t("btn-capture-state")).clicked() {
            actions.push(UIAction::CaptureStateToCue(cue.id));
        }

        changed
    }
}
