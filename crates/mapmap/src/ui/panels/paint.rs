//! Modular Paint Panel orchestration.

use egui::Context;
use mapmap_core::AppState;
use mapmap_ui::AppUI;

/// Context required to render the paint panel.
pub struct PaintContext<'a> {
    /// Reference to the UI state.
    pub ui_state: &'a mut AppUI,
    /// Reference to the app state.
    pub state: &'a mut AppState,
}

/// Renders the paint panel.
pub fn show(ctx: &Context, context: PaintContext) {
    context.ui_state.paint_panel.show(
        ctx,
        &context.ui_state.i18n,
        context.state.paint_manager_mut(),
        context.ui_state.icon_manager.as_ref(),
    );
}

/// Handles actions from the paint panel.
pub fn handle_actions(ui_state: &mut AppUI, state: &mut AppState) {
    if let Some(action) = ui_state.paint_panel.take_action() {
        match action {
            mapmap_ui::paint_panel::PaintPanelAction::AddPaint => {
                state
                    .paint_manager_mut()
                    .add_paint(mapmap_core::paint::Paint::color(
                        0,
                        "New Color",
                        [1.0, 1.0, 1.0, 1.0],
                    ));
                state.dirty = true;
            }
            mapmap_ui::paint_panel::PaintPanelAction::RemovePaint(id) => {
                state.paint_manager_mut().remove_paint(id);
                state.dirty = true;
            }
        }
    }
}




