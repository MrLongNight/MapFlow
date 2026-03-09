use crate::app::core::app_struct::App;
use anyhow::Result;
use std::path::Path;

/// Specialized node logic (e.g. synchronization between graph and state).
pub fn update_node_logic(app: &mut App) {
    // Sync audio config to analyzer
    app.audio_analyzer.update_config(app.state.audio_config.clone());
}

/// Load a project file into the application.
pub fn load_project_file(app: &mut App, path: &Path) -> Result<()> {
    let state = mapmap_io::load_project(path)?;
    app.state = state;
    app.history.clear();

    // Clear selections to avoid referencing deleted IDs
    app.ui_state.selected_layer_id = None;
    app.ui_state.selected_output_id = None;
    app.ui_state.module_canvas.selected_parts.clear();

    // Notify subsystems of new state
    app.audio_analyzer.update_config(app.state.audio_config.clone());

    Ok(())
}
