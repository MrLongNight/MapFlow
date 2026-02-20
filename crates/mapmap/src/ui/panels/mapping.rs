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
        context.state.mapping_manager_mut(),
        &mut context.ui_state.actions,
        &context.ui_state.i18n,
        context.ui_state.icon_manager.as_ref(),
    );
}
