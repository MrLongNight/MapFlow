//! Window and Output orchestration.

use crate::app::core::app_struct::App;
use anyhow::Result;
use mapmap_core::module::OutputType;
use std::collections::HashSet;
use tracing::info;

/// Synchronizes output windows with the current module evaluation result.
pub fn sync_output_windows(
    app: &mut App,
    elwt: &winit::event_loop::ActiveEventLoop,
    _render_ops: &[mapmap_core::module_eval::RenderOp],
    _active_module_id: Option<mapmap_core::module::ModuleId>,
) -> Result<()> {
    const PREVIEW_FLAG: u64 = 1u64 << 63;

    // Track active IDs for cleanup
    let mut active_window_ids = HashSet::new();
    let mut active_sender_ids = HashSet::new();
    let global_fullscreen = app.ui_state.user_config.global_fullscreen;

    // 1. Iterate over ALL modules to collect required outputs
    for module in app.state.module_manager.list_modules() {
        if let Some(module_ref) = app.state.module_manager.get_module(module.id) {
            for part in &module_ref.parts {
                if let mapmap_core::module::ModulePartType::Output(output_type) = &part.part_type {
                    // Use part.id for consistency with render pipeline
                    let _output_id = part.id;

                    match output_type {
                        OutputType::Projector {
                            id: projector_id,
                            name,
                            hide_cursor,
                            target_screen,
                            show_in_preview_panel: _,
                            extra_preview_window,
                            ..
                        } => {
                            // 1. Primary Window - Use Logical ID (projector_id) not Part ID
                            let window_id = *projector_id;
                            active_window_ids.insert(window_id);

                            if let Some(window_context) = app.window_manager.get(window_id) {
                                // Update existing
                                let is_fullscreen = window_context.window.fullscreen().is_some();
                                if is_fullscreen != global_fullscreen {
                                    info!(
                                        "Toggling fullscreen for window {}: {}",
                                        window_id, global_fullscreen
                                    );
                                    window_context.window.set_fullscreen(if global_fullscreen {
                                        Some(winit::window::Fullscreen::Borderless(None))
                                    } else {
                                        None
                                    });
                                }
                                window_context.window.set_cursor_visible(!*hide_cursor);
                            } else {
                                // Create new
                                app.window_manager.create_projector_window(
                                    elwt,
                                    &app.backend,
                                    window_id,
                                    name,
                                    global_fullscreen,
                                    *hide_cursor,
                                    *target_screen,
                                    app.ui_state.user_config.vsync_mode,
                                )?;
                                info!("Created projector window for output {}", window_id);
                            }

                            // 2. Extra Preview Window
                            if *extra_preview_window {
                                let preview_id = window_id | PREVIEW_FLAG;
                                active_window_ids.insert(preview_id);

                                if app.window_manager.get(preview_id).is_none() {
                                    app.window_manager.create_projector_window(
                                        elwt,
                                        &app.backend,
                                        preview_id,
                                        &format!("Preview: {}", name),
                                        false, // Always windowed
                                        false, // Show cursor
                                        0,     // Default screen (0)
                                        app.ui_state.user_config.vsync_mode,
                                    )?;
                                    info!("Created preview window for output {}", window_id);
                                }
                            }
                        }
                        OutputType::NdiOutput { name: _name } => {
                            // For NDI, use part.id as the unique identifier
                            let output_id = part.id;
                            active_sender_ids.insert(output_id);

                            #[cfg(feature = "ndi")]
                            {
                                if let std::collections::hash_map::Entry::Vacant(e) =
                                    app.ndi_senders.entry(output_id)
                                {
                                    let width = 1920;
                                    let height = 1080;
                                    match mapmap_io::ndi::NdiSender::new(
                                        _name.clone(),
                                        mapmap_io::format::VideoFormat {
                                            width,
                                            height,
                                            pixel_format: mapmap_io::format::PixelFormat::BGRA8,
                                            frame_rate: 60.0,
                                        },
                                    ) {
                                        Ok(sender) => {
                                            e.insert(sender);
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to create NDI sender: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                        // Handle other output types or ignore them
                        #[cfg(target_os = "windows")]
                        OutputType::Spout { .. } => {
                            // Spout not yet implemented in synchronization loop
                        }
                        OutputType::Hue { .. } => {
                            // Hue handled by HueController separately
                        }
                    }
                }
            }
        }
    }

    // Cleanup logic for closed windows/senders would go here (omitted for brevity in this step)

    Ok(())
}




