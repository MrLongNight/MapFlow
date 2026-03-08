import os

with open("crates/mapmap/src/orchestration/outputs.rs", "r") as f:
    content = f.read()

content = content.replace("ui_needs_sync: bool,", "_ui_needs_sync: bool,")
content = content.replace("graph_dirty: bool,", "_graph_dirty: bool,")

bad_match = """            if let mapmap_core::module::ModulePartType::Output(output_type) = &part.part_type {
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
            }"""

good_match = """            if let mapmap_core::module::ModulePartType::Output(OutputType::Projector {
                id,
                name,
                target_screen,
                ..
            }) = &part.part_type {
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
            }"""

content = content.replace(bad_match, good_match)

with open("crates/mapmap/src/orchestration/outputs.rs", "w") as f:
    f.write(content)
