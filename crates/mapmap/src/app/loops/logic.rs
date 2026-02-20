use crate::app::actions::{handle_mcp_actions, handle_ui_actions};
use crate::app::core::app_struct::App;
use crate::orchestration::media::{sync_media_players, update_media_players};
use crate::orchestration::outputs::sync_output_windows;
use anyhow::Result;
use mapmap_core::module::{ModulePartType, OutputType};
use mapmap_io::save_project;
use std::collections::HashSet;
use tracing::info;

/// Global update loop (physics/logic), independent of render rate per window.
pub fn update(app: &mut App, elwt: &winit::event_loop::ActiveEventLoop, dt: f32) -> Result<()> {
    // Process internal MCP actions first
    handle_mcp_actions(app);

    let ui_needs_sync = handle_ui_actions(app).unwrap_or(false);

    // --- Media Player Update ---
    sync_media_players(app);
    update_media_players(app, dt);

    // --- Effect Animator Update ---
    let param_updates = app.state.effect_animator_mut().update(dt as f64);
    if !param_updates.is_empty() {
        tracing::trace!("Effect updates: {}", param_updates.len());
    }

    // --- Bevy Runner Update ---
    if let Some(runner) = &mut app.bevy_runner {
        // First sync graph state
        for module in app.state.module_manager.list_modules() {
            runner.apply_graph_state(module);
        }

        let analysis = app.audio_analyzer.get_latest_analysis();
        let trigger_data = mapmap_core::audio_reactive::AudioTriggerData {
            band_energies: analysis.band_energies,
            rms_volume: analysis.rms_volume,
            peak_volume: analysis.peak_volume,
            beat_detected: analysis.beat_detected,
            beat_strength: analysis.beat_strength,
            bpm: analysis.tempo_bpm,
        };
        runner.update(&trigger_data);
    }

    // --- Module Graph Evaluation ---
    // Evaluate ALL modules and merge render_ops for multi-output support
    app.render_ops.clear();
    for module in app.state.module_manager.list_modules() {
        let module_id = module.id;
        if let Some(module_ref) = app.state.module_manager.get_module(module_id) {
            let eval_result = app
                .module_evaluator
                .evaluate(module_ref, &app.state.module_manager.shared_media);
            // Push (ModuleId, RenderOp) tuple
            app.render_ops.extend(
                eval_result
                    .render_ops
                    .iter()
                    .cloned()
                    .map(|op| (module_id, op)),
            );
        }
    }

    // Sync output windows based on MODULE GRAPH STRUCTURE (stable),
    // NOT render_ops (which can be empty/fluctuate).
    let current_output_ids: HashSet<u64> = app
        .state
        .module_manager
        .list_modules()
        .iter()
        .flat_map(|m| m.parts.iter())
        .filter_map(|part| {
            if let ModulePartType::Output(OutputType::Projector { id, .. }) = &part.part_type {
                Some(*id)
            } else {
                None
            }
        })
        .collect();

    // Get current window IDs (excluding main window 0)
    let prev_output_ids: HashSet<u64> = app
        .window_manager
        .iter()
        .filter(|wc| wc.output_id != 0)
        .map(|wc| wc.output_id)
        .collect();

    // Only sync if module graph's projector set changed
    if ui_needs_sync || current_output_ids != prev_output_ids {
        info!(
            "Output set changed: {:?} -> {:?}",
            prev_output_ids, current_output_ids
        );
        // Create temp list of ops for sync
        let ops_only: Vec<mapmap_core::module_eval::RenderOp> =
            app.render_ops.iter().map(|(_, op)| op.clone()).collect();
        if let Err(e) = sync_output_windows(app, elwt, &ops_only, None) {
            tracing::error!("Failed to sync output windows: {}", e);
        }
    }

    // --- Oscillator Update ---
    if let Some(renderer) = &mut app.oscillator_renderer {
        if app.state.oscillator_config.enabled {
            renderer.update(dt, &app.state.oscillator_config);
        }
    }

    // --- FPS Calculation ---
    let frame_time_ms = dt * 1000.0;
    app.fps_samples.push_back(frame_time_ms);
    if app.fps_samples.len() > 60 {
        app.fps_samples.pop_front();
    }
    if !app.fps_samples.is_empty() {
        let avg_frame_time: f32 =
            app.fps_samples.iter().sum::<f32>() / app.fps_samples.len() as f32;
        app.current_frame_time_ms = avg_frame_time;
        app.current_fps = if avg_frame_time > 0.0 {
            1000.0 / avg_frame_time
        } else {
            0.0
        };
    }

    // Check auto-save (every 30s)
    if app.last_autosave.elapsed().as_secs() >= 30 {
        if app.state.dirty {
            if let Some(path) =
                dirs::data_local_dir().map(|p| p.join("MapFlow").join("autosave.mflow"))
            {
                // Ensure dir exists
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                match save_project(&app.state, &path) {
                    Ok(_) => {
                        info!("Autosaved project to {:?}", path);
                        app.state.dirty = false;
                    }
                    Err(e) => {
                        tracing::error!("Autosave failed: {}", e);
                    }
                }
            }
        }
        app.last_autosave = std::time::Instant::now();
    }

    // System Info Update (every 1s)
    if app.last_sysinfo_refresh.elapsed().as_secs() >= 1 {
        app.sys_info.refresh_cpu_usage();
        app.sys_info.refresh_memory();
        app.last_sysinfo_refresh = std::time::Instant::now();
    }

    // Periodic Performance Status (every 10s)
    // Use a simpler approach without static mut to avoid warnings/safety issues
    // We can use the app.last_sysinfo_refresh as a rough proxy or just log every N frames.
    // Let's use a frame counter based approach since we don't want to modify App struct.
    // 600 frames @ 60fps = 10 seconds.
    static PERF_LOG_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    #[allow(clippy::manual_is_multiple_of)]
    if PERF_LOG_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % 600 == 0 {
        let ram_mb = if let Ok(pid) = sysinfo::get_current_pid() {
            app.sys_info
                .process(pid)
                .map(|p| p.memory() as f32 / 1024.0 / 1024.0)
                .unwrap_or(0.0)
        } else {
            0.0
        };
        info!(
            "[PERF] FPS: {:.1}, Frame: {:.2}ms, RAM: {:.1}MB, Modules: {}",
            app.current_fps,
            app.current_frame_time_ms,
            ram_mb,
            app.state.module_manager.list_modules().len()
        );
    }

    // Periodic VRAM Garbage Collection (every 10s)
    if app.last_texture_gc.elapsed().as_secs() >= 10 {
        let removed = app
            .texture_pool
            .collect_garbage(std::time::Duration::from_secs(30));
        if removed > 0 {
            info!("VRAM GC: Removed {} unused textures from pool", removed);
        }
        app.last_texture_gc = std::time::Instant::now();
    }

    Ok(())
}
