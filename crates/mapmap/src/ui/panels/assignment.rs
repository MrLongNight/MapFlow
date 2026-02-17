//! Modular Assignment Panel orchestration.

use egui::Context;
use mapmap_core::AppState;
use mapmap_ui::AppUI;

/// Context required to render the assignment panel.
pub struct AssignmentContext<'a> {
    /// Reference to the UI state.
    pub ui_state: &'a mut AppUI,
    /// Reference to the app state.
    pub state: &'a mut AppState,
}

/// Renders the assignment panel.
pub fn show(ctx: &Context, context: AssignmentContext) {
    context
        .ui_state
        .assignment_panel
        .show(ctx, &context.state.assignment_manager);
}




