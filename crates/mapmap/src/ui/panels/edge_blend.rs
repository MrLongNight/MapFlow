//! Modular Edge Blend Panel orchestration.

use egui::Context;
use mapmap_ui::AppUI;

/// Context required to render the edge blend panel.
pub struct EdgeBlendContext<'a> {
    /// Reference to the UI state.
    pub ui_state: &'a mut AppUI,
}

/// Renders the edge blend panel.
pub fn show(ctx: &Context, context: EdgeBlendContext) {
    context
        .ui_state
        .edge_blend_panel
        .show(ctx, &context.ui_state.i18n);
}
