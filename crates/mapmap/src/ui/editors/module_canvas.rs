//! Modular Module Canvas orchestration.

use egui::Ui;
use mapmap_core::AppState;
use mapmap_ui::AppUI;

/// Context required to render the module canvas.
pub struct ModuleCanvasContext<'a> {
    /// Reference to the UI state.
    pub ui_state: &'a mut AppUI,
    /// Reference to the app state.
    pub state: &'a mut AppState,
}

/// Renders the module canvas inside the provided UI.
pub fn show(ui: &mut Ui, context: ModuleCanvasContext) {
    // Update available outputs for the ModuleCanvas dropdown
    context.ui_state.module_canvas.available_outputs = context
        .state
        .output_manager
        .outputs()
        .iter()
        .map(|o| (o.id, o.name.clone()))
        .collect();

    context.ui_state.module_canvas.show(
        ui,
        &mut context.state.module_manager,
        &context.ui_state.i18n,
        &mut context.ui_state.actions,
    );
}
