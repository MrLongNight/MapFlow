use crate::widgets::panel::{cyber_panel_frame, render_panel_header};
use egui::Context;
use vorce_core::assignment::AssignmentManager;

/// UI panel for managing control assignments.
#[derive(Debug, Default)]
pub struct AssignmentPanel {
    pub visible: bool,
}

impl AssignmentPanel {
    /// Creates a new, uninitialized instance with default settings.
    pub fn new() -> Self {
        Self { visible: true }
    }

    /// Show the assignment panel UI.
    pub fn show(&mut self, ctx: &Context, assignment_manager: &AssignmentManager) {
        if !self.visible {
            return;
        }

        egui::Window::new("Assignment Manager")
            .open(&mut self.visible)
            .default_size([400.0, 600.0])
            .frame(cyber_panel_frame(&ctx.style()))
            .show(ctx, |ui| {
                render_panel_header(ui, "Assignments", |_| {});
                ui.add_space(8.0);

                // Display a dummy list or debug info for now
                if assignment_manager.assignments().is_empty() {
                    crate::widgets::custom::render_info_label(ui, "No assignments configured.");
                } else {
                    for assignment in assignment_manager.assignments() {
                        ui.label(format!("{:?}", assignment));
                    }
                }

                ui.separator();
                if ui.button("Add Dummy Assignment").clicked() {
                    // This part is for testing and will be replaced by actual UI actions.
                    // Note: We can't mutate assignment_manager here directly as it's immutable.
                    // Actions would need to be sent back to the main app loop.
                    tracing::info!(
                        "'Add Dummy Assignment' clicked. An action would be dispatched here."
                    );
                }
            });
    }
}
