//! Assignment Panel - UI for managing control assignments
//!
//! Provides a UI for creating, editing, and deleting control assignments.
//! Follows the strict ID-based pattern for source/target selection.

use egui::{ComboBox, Context, Grid, Ui, Window};
use mapmap_core::{Assignment, AssignmentManager};

/// State for the Assignment Panel.
pub struct AssignmentPanel {
    /// Is the panel currently visible?
    pub visible: bool,
    /// ID of the assignment being edited, if any.
    editing_id: Option<u64>,
    /// Temporary storage for the name of the assignment being edited.
    edited_name: String,
}

impl AssignmentPanel {
    /// Creates a new `AssignmentPanel`.
    pub fn new() -> Self {
        Self {
            visible: false,
            editing_id: None,
            edited_name: String::new(),
        }
    }

    /// Renders the Assignment Panel.
    pub fn show(
        &mut self,
        ctx: &Context,
        assignment_manager: &mut AssignmentManager,
        // TODO: Pass in sources and targets managers to resolve names from IDs
    ) {
        let mut is_open = self.visible;
        if !is_open {
            return;
        }

        Window::new("Assignment Panel")
            .open(&mut is_open)
            .resizable(true)
            .default_width(400.0)
            .show(ctx, |ui| {
                self.ui(ui, assignment_manager);
            });

        self.visible = is_open;
    }

    /// Renders the panel's main UI.
    fn ui(&mut self, ui: &mut Ui, assignment_manager: &mut AssignmentManager) {
        ui.heading("Control Assignments");
        ui.separator();

        let assignments_clone = assignment_manager.assignments().to_vec();
        let mut assignment_to_remove = None;

        // List of existing assignments
        for assignment in &assignments_clone {
            ui.horizontal(|ui| {
                ui.label(format!("#{}:", assignment.id));
                ui.label(&assignment.name);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🗑").on_hover_text("Remove").clicked() {
                        assignment_to_remove = Some(assignment.id);
                    }
                    if ui.button("✏").on_hover_text("Edit").clicked() {
                        self.editing_id = Some(assignment.id);
                        self.edited_name = assignment.name.clone();
                    }
                });
            });
        }

        if let Some(id) = assignment_to_remove {
            assignment_manager.remove_assignment(id);
        }

        ui.separator();

        if ui.button("➕ Add New Assignment").clicked() {
            let new_id = assignment_manager.generate_id();
            let new_assignment = Assignment {
                id: new_id,
                name: format!("New Assignment {}", new_id),
                source_id: 0,
                target_module_id: 0,
                target_part_id: 0,
                target_param_name: "".to_string(),
                enabled: true,
            };
            assignment_manager.add_assignment(new_assignment);
            self.editing_id = Some(new_id);
            self.edited_name = format!("New Assignment {}", new_id);
        }

        // Editor view
        if let Some(editing_id) = self.editing_id {
            if let Some(assignment) = assignment_manager
                .assignments_mut()
                .iter_mut()
                .find(|a| a.id == editing_id)
            {
                ui.separator();
                ui.heading(format!("Editing Assignment #{}", editing_id));

                Grid::new("assignment_editor")
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.edited_name);
                        ui.end_row();

                        ui.label("Source:");
                        ComboBox::from_label("Select Source")
                            .selected_text(format!("Source ID: {}", assignment.source_id))
                            .show_ui(ui, |ui| {
                                // TODO: Populate with real sources
                                ui.selectable_value(&mut assignment.source_id, 0, "MIDI CC 1");
                                ui.selectable_value(
                                    &mut assignment.source_id,
                                    1,
                                    "OSC /layer/opacity",
                                );
                            });
                        ui.end_row();

                        ui.label("Target:");
                        ComboBox::from_label("Select Target")
                            .selected_text(format!("Target ID: {}", assignment.target_module_id))
                            .show_ui(ui, |ui| {
                                // TODO: Populate with real targets
                                ui.selectable_value(
                                    &mut assignment.target_module_id,
                                    0,
                                    "Layer 1 Opacity",
                                );
                                ui.selectable_value(
                                    &mut assignment.target_module_id,
                                    1,
                                    "Effect Radius",
                                );
                            });
                        ui.end_row();
                    });

                ui.horizontal(|ui| {
                    if ui.button("Done").clicked() {
                        assignment.name = self.edited_name.clone();
                        self.editing_id = None;
                    }
                });
            } else {
                // The assignment was removed while being edited
                self.editing_id = None;
            }
        }
    }
}

impl Default for AssignmentPanel {
    fn default() -> Self {
        Self::new()
    }
}
