//! Project I/O and State orchestration.

use crate::app::core::app_struct::App;
use anyhow::{Context, Result};
use mapmap_io::{load_project, save_project};
use std::path::{Path, PathBuf};
use tracing::{error, info};

/// Helper to load a project file and update state
pub fn load_project_file(app: &mut App, path: &PathBuf) {
    match load_project(path) {
        Ok(new_state) => {
            app.state = new_state;
            // Sync language to UI
            app.ui_state.i18n.set_locale(&app.state.settings.language);

            info!("Project loaded from {:?}", path);

            // Add to recent files
            if let Some(path_str) = path.to_str() {
                let p = path_str.to_string();
                // Remove if exists to move to top
                if let Some(pos) = app.ui_state.recent_files.iter().position(|x| x == &p) {
                    app.ui_state.recent_files.remove(pos);
                }
                app.ui_state.recent_files.insert(0, p.clone());
                // Limit to 10
                if app.ui_state.recent_files.len() > 10 {
                    app.ui_state.recent_files.pop();
                }
                // Persist to user config
                app.ui_state.user_config.add_recent_file(&p);
            }
        }
        Err(e) => error!("Failed to load project: {}", e),
    }
}

/// Helper to save project
pub fn save_app_project(app: &App, path: &Path) -> Result<()> {
    save_project(&app.state, path).context("Failed to save project")
}




