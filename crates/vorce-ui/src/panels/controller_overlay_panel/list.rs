#[allow(unused_imports)]
use egui::{Color32, Pos2, Rect, Response, Sense, Stroke, TextureHandle, Ui, Vec2};

#[allow(unused_imports)]
use crate::config::{MidiAssignment, MidiAssignmentTarget, UserConfig};

#[allow(unused_imports)]
use std::collections::{HashMap, HashSet};

use super::panel::{ControllerOverlayPanel, ElementFilter};

impl ControllerOverlayPanel {
    /// Show the element list view
    pub(crate) fn show_element_list_view(&mut self, ui: &mut Ui, user_config: &mut UserConfig) {
        // Filter buttons
        ui.horizontal(|ui| {
            ui.label("Filter:");
            if ui
                .selectable_label(self.element_filter == ElementFilter::All, "Alle")
                .clicked()
            {
                self.element_filter = ElementFilter::All;
            }
            if ui
                .selectable_label(self.element_filter == ElementFilter::MapFlow, "MapFlow")
                .clicked()
            {
                self.element_filter = ElementFilter::MapFlow;
            }
            if ui
                .selectable_label(
                    self.element_filter == ElementFilter::StreamerBot,
                    "Streamer.bot",
                )
                .clicked()
            {
                self.element_filter = ElementFilter::StreamerBot;
            }
            if ui
                .selectable_label(self.element_filter == ElementFilter::Mixxx, "Mixxx")
                .clicked()
            {
                self.element_filter = ElementFilter::Mixxx;
            }
            if ui
                .selectable_label(self.element_filter == ElementFilter::Unassigned, "Frei")
                .clicked()
            {
                self.element_filter = ElementFilter::Unassigned;
            }
        });

        ui.separator();

        // Element table
        #[allow(unused_mut)]
        let mut element_to_remove: Option<String> = None;

        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("element_list")
                .num_columns(5)
                .striped(true)
                .show(ui, |ui| {
                    // Header
                    ui.strong("ID");
                    ui.strong("Name");
                    ui.strong("Typ");
                    ui.strong("MIDI");
                    ui.strong("Zuweisung / Aktion");
                    ui.end_row();

                    #[cfg(feature = "midi")]
                    if let Some(elements) = &self.elements {
                        for element in &elements.elements {
                            // Determine assignment status
                            let assignment = user_config.get_midi_assignment(&element.id);

                            // Apply filter
                            let show = match self.element_filter {
                                ElementFilter::All => true,
                                ElementFilter::MapFlow => matches!(assignment, Some(a) if matches!(a.target, MidiAssignmentTarget::MapFlow(_))),
                                ElementFilter::StreamerBot => matches!(assignment, Some(a) if matches!(a.target, MidiAssignmentTarget::StreamerBot(_))),
                                ElementFilter::Mixxx => matches!(assignment, Some(a) if matches!(a.target, MidiAssignmentTarget::Mixxx(_))),
                                ElementFilter::Unassigned => assignment.is_none(),
                            };

                            if !show {
                                continue;
                            }

                            ui.label(&element.id);
                            ui.label(&element.label);
                            ui.label(format!("{:?}", element.element_type));
                            if let Some(midi) = &element.midi {
                                ui.label(format!("{:?}", midi));
                            } else {
                                ui.label("-");
                            }

                            // Show assignment and delete button
                            if let Some(assign) = assignment {
                                ui.horizontal(|ui| {
                                    ui.label(assign.target.to_string());
                                    if ui.small_button("🗑").clone().on_hover_text("Zuweisung löschen").clicked() {
                                        element_to_remove = Some(element.id.clone());
                                    }
                                });
                            } else {
                                ui.label("-");
                            }
                            ui.end_row();
                        }
                    }
                });
        });

        // Handle deletion request outside of borrow loop
        if let Some(id) = element_to_remove {
            user_config.remove_midi_assignment(&id);
        }
    }
}
