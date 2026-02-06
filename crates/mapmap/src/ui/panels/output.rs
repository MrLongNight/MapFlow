//! Modular Output Panel orchestration.

use egui::Context;
use mapmap_core::AppState;
use mapmap_ui::AppUI;

/// Context required to render the output panel.
pub struct OutputContext<'a> {
    /// Reference to the UI state.
    pub ui_state: &'a mut AppUI,
    /// Reference to the app state.
    pub state: &'a mut AppState,
}

/// Renders the output panel.
pub fn show(ctx: &Context, context: OutputContext) {
    context.ui_state.output_panel.render(
        ctx,
        &context.ui_state.i18n,
        context.state.output_manager_mut(),
        &[], // Monitors placeholder
    );
}
