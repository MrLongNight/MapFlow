//! Window and Output orchestration.

use crate::app::core::app_struct::App;
use anyhow::Result;
use mapmap_core::module::OutputType;
use std::collections::HashSet;

/// Synchronizes output windows with the current module graph configuration.
pub fn sync_output_windows(
    app: &mut App,
    elwt: &winit::event_loop::ActiveEventLoop,
    ui_needs_sync: bool,
    graph_dirty: bool,
) -> Result<()> {
    let mut active_window_ids: HashSet<u64> = HashSet::new();

    // 1. Identify all active outputs in the graph
    for module in app.state.module_manager.modules() {
        for part in &module.parts {
            if let mapmap_core::module::ModulePartType::Output(output_type) = &part.part_type {
                match output_type {
                    OutputType::Projector {
                        id,
                        name,
                        target_screen,
                        ..
                    } => {
                        active_window_ids.insert(*id);

                        // Create window if it doesn't exist
                        if !app.window_manager.window_ids().any(|&wid| wid == *id) {
                            app.window_manager.create_projector_window(
                                elwt,
                                &app.backend,
                                *id,
                                name,
                                false, // Default or fetch from config
                                false, // Default or fetch from config
                                *target_screen,
                                app.ui_state.user_config.vsync_mode,
                            )?;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
