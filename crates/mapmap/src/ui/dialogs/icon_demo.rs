//! Modular Icon Demo Panel.

use egui::Context;
use mapmap_ui::AppUI;

/// Context required to render the icon demo panel.
pub struct IconDemoContext<'a> {
    /// Reference to the UI state.
    pub ui_state: &'a mut AppUI,
}

/// Renders the icon demo panel.
pub fn show(ctx: &Context, context: IconDemoContext) {
    if context.ui_state.icon_demo_panel.visible {
        context.ui_state.icon_demo_panel.ui(
            ctx,
            context.ui_state.icon_manager.as_ref(),
            &context.ui_state.i18n,
        );
    }
}




