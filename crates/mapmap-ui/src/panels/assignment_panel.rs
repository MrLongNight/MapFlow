use egui::Context;
use mapmap_core::assignment::AssignmentManager;

/// UI panel for managing control assignments.
#[derive(Debug, Default)]
pub struct AssignmentPanel {
    pub visible: bool,
}

impl AssignmentPanel {
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
            .show(ctx, |ui| {
                ui.heading("Assignments");
                ui.separator();

                // Display a dummy list or debug info for now
                if assignment_manager.assignments().is_empty() {
                    ui.label("No assignments configured.");
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




