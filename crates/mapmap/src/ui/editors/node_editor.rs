//! Modular Node Editor orchestration.

use egui::Context;
use mapmap_ui::AppUI;

/// Context required to render the node editor.
pub struct NodeEditorContext<'a> {
    /// Reference to the UI state.
    pub ui_state: &'a mut AppUI,
}

/// Renders the node editor panel.
pub fn show(ctx: &Context, context: NodeEditorContext) {
    context.ui_state.render_node_editor(ctx);
}
