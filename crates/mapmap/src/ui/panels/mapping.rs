//! Modular Mapping Panel orchestration.

use egui::Context;
use mapmap_core::AppState;
use mapmap_ui::AppUI;

/// Context required to render the mapping panel.
pub struct MappingContext<'a> {
    /// Reference to the UI state.
    pub ui_state: &'a mut AppUI,
    /// Reference to the app state.
    pub state: &'a mut AppState,
}

/// Renders the mapping panel.
pub fn show(ctx: &Context, context: MappingContext) {
    context.ui_state.mapping_panel.show(
        ctx,
        &mut context.state.mapping_manager,
        &mut context.ui_state.actions,
        &context.ui_state.i18n,
    );
}
