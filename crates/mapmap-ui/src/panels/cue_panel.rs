//! Cue System UI Panel
use std::time::Duration;

use egui::{self, Button, ComboBox, RichText, ScrollArea, Slider, Ui};
use mapmap_control::{
    cue::{triggers::*, Cue, CueList},
    ControlManager,
};

use crate::{
    i18n::LocaleManager,
    icons::{AppIcon, IconManager},
    theme::colors,
    widgets::hold_to_action_icon,
    widgets::panel::{cyber_panel_frame, render_panel_header},
    UIAction,
};

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
        control_manager: &ControlManager,
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
            .frame(cyber_panel_frame(&ctx.style()))
            .show(ctx, |ui| {
                render_panel_header(
                    ui,
                    &i18n.t("panel-cues"),
                    |_| {},
                );

                ui.add_space(8.0);

                self.render_ui(ui, &control_manager.cue_list, i18n, actions, icon_manager);
            });
        self.visible = open;
    }

    fn render_ui(
        &mut self,
        ui: &mut egui::Ui,
        cue_list: &CueList,
        i18n: &LocaleManager,
        actions: &mut Vec<UIAction>,
        icon_manager: Option<&IconManager>,
    ) {
        // --- Top Control Bar ---
        ui.horizontal(|ui| {
            // --- Next Button ---
            let next_enabled = cue_list.next_cue().is_some();
            if self.icon_button(
                ui,
                icon_manager,
                AppIcon::ButtonPlay,
                "btn-go", // "Go" often implies "Next" in cue systems
                i18n,
                next_enabled,
            ) {
                actions.push(UIAction::NextCue);
            }

            // --- Prev Button ---
            let prev_enabled = if let Some(current_id) = cue_list.current_cue() {
                cue_list
                    .cues()
                    .iter()
                    .position(|c| c.id == current_id)
                    .is_some_and(|idx| idx > 0)
            } else {
                false
            };
            if self.icon_button(
                ui,
                icon_manager,
                AppIcon::ArrowLeft,
                "btn-back",
                i18n,
                prev_enabled,
            ) {
                actions.push(UIAction::PrevCue);
            }

            // --- Stop Button ---
            let stop_enabled = cue_list.current_cue().is_some();
            if stop_enabled {
                if hold_to_action_icon(
                    ui,
                    icon_manager,
                    AppIcon::ButtonStop,
                    24.0,
                    colors::ERROR_COLOR,
                ) {
                    actions.push(UIAction::StopCue);
                }
            } else {
                self.icon_button(
                    ui,
                    icon_manager,
                    AppIcon::ButtonStop,
                    "btn-stop",
                    i18n,
                    false,
                );
            }

            ui.separator();

            ui.label(i18n.t("label-jump-to"));
            ui.text_edit_singleline(&mut self.jump_target_id);
            if ui.button(i18n.t("btn-jump")).clicked() {
                if let Ok(id) = self.jump_target_id.parse::<u32>() {
                    actions.push(UIAction::GoCue(id));
                }
            }
        });

        ui.separator();

        // --- Cue List ---
        ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
            let current_cue_id = cue_list.current_cue();
            let next_cue_id = cue_list.next_cue();
            let cues_to_render: Vec<_> = cue_list.cues().to_vec();

            if cues_to_render.is_empty() {
                ui.label(i18n.t("label-no-cues"));
            } else {
                for cue in cues_to_render {
                    let is_current = current_cue_id == Some(cue.id);
                    let is_next = next_cue_id == Some(cue.id);
                    let is_selected = self.selected_cue_id == Some(cue.id);

                    let label_text = format!("{} - {}", cue.id, cue.name);
                    let mut label = RichText::new(label_text);

                    if is_current {
                        label = label.color(ui.visuals().selection.stroke.color).strong();
                    }
                    if is_next {
                        label = label.color(egui::Color32::from_rgb(255, 165, 0));
                        // Orange for next
                    }

                    if ui.selectable_label(is_selected, label).clicked() {
                        self.selected_cue_id = Some(cue.id);
                    }
                }
            }
        });

        ui.separator();

        // --- Cue Editor ---
        if let Some(selected_id) = self.selected_cue_id {
            if let Some(cue_to_edit) = cue_list.get_cue(selected_id).cloned() {
                ui.group(|ui| {
                    ui.heading(i18n.t("header-cue-editor"));

                    let mut updated_cue = cue_to_edit;
                    if self.render_cue_editor(ui, &mut updated_cue, i18n) {
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
            if self.icon_button(ui, icon_manager, AppIcon::Add, "btn-add-cue", i18n, true) {
                actions.push(UIAction::AddCue);
            }

            if self.selected_cue_id.is_some()
                && self.icon_button(
                    ui,
                    icon_manager,
                    AppIcon::Remove,
                    "btn-remove-cue",
                    i18n,
                    true,
                )
            {
                if let Some(id) = self.selected_cue_id {
                    actions.push(UIAction::RemoveCue(id));
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

    /// Helper to render a consistent icon button.
    fn icon_button(
        &self,
        ui: &mut Ui,
        icon_manager: Option<&IconManager>,
        icon: AppIcon,
        tooltip_key: &str,
        i18n: &LocaleManager,
        enabled: bool,
    ) -> bool {
        if let Some(mgr) = icon_manager {
            if let Some(img) = mgr.image(icon, 24.0) {
                let button = egui::Button::image(img);
                return ui
                    .add_enabled(enabled, button)
                    .clone()
                    .on_hover_text(i18n.t(tooltip_key))
                    .clicked();
            }
        }
        // Fallback to text button if icons are not available
        ui.add_enabled(enabled, Button::new(i18n.t(tooltip_key)))
            .clicked()
    }
}
