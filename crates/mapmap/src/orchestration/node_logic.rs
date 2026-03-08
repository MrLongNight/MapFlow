use crate::app::core::app_struct::App;
use anyhow::Result;
use mapmap_io::load_project;
use std::path::Path;
use tracing::info;

/// Loads a project file and updates the application state.
pub fn load_project_file(app: &mut App, path: &Path) -> Result<()> {
    info!("Loading project from {:?}", path);
    let loaded_state = load_project(path)?;
    app.state = loaded_state;
    app.state.dirty = false;

    // Sync analyzer config
    app.audio_analyzer
        .update_config(app.state.audio_config.clone());

    // Clear caches
    app.render_ops.clear();
    app.last_graph_revision = 0; // Force sync

    info!("Project loaded successfully.");
    Ok(())
}
