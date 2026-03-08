use crate::app::actions::{handle_mcp_actions, handle_ui_actions};
use crate::app::core::app_struct::App;
use crate::orchestration::media::{sync_media_players, update_media_players};
use crate::orchestration::outputs::sync_output_windows;
use crate::orchestration::evaluation::perform_evaluation;
use anyhow::Result;
use mapmap_core::audio::backend::AudioBackend;
use mapmap_io::save_project;
use std::collections::HashSet;

/// Global update loop (physics/logic), independent of render rate per window.
pub fn update(app: &mut App, elwt: &winit::event_loop::ActiveEventLoop, dt: f32) -> Result<()> {
    // 1. Process internal MCP actions first
    handle_mcp_actions(app);

    // 2. Handle UI actions and check if they requested a structural sync
    let ui_needs_sync = handle_ui_actions(app).unwrap_or(false);

    // 3. Get all module IDs
    let all_module_ids: Vec<u64> = app.state.module_manager.modules().iter().map(|m| m.id).collect();
    
    // --- Performance Optimization: Early return if idle ---
    if all_module_ids.is_empty() {
        app.ui_state.current_fps = app.current_fps;
        app.ui_state.current_frame_time_ms = app.current_frame_time_ms;
        app.last_graph_revision = app.state.module_manager.graph_revision;
        return Ok(());
    }

    // 4. Update evaluator with reaktive events
    let active_keys: HashSet<String> = app.egui_context.input(|i| {
        i.keys_down.iter().map(|k| format!("{:?}", k)).collect()
    });
    app.module_evaluator.update_keys(&active_keys);

    for (channel, note) in &app.control_manager.raw_midi_events {
        app.module_evaluator.record_midi(*channel, *note);
    }
    for addr in &app.control_manager.raw_osc_events {
        app.module_evaluator.record_osc(addr);
    }

    // 5. Audio Analysis Update
    let timestamp = app.start_time.elapsed().as_secs_f64();
    if let Some(backend) = &mut app.audio_backend {
        let samples = backend.get_samples();
        if !samples.is_empty() {
            app.audio_analyzer.process_samples(&samples, timestamp);
        }
    }
    
    // Get analysis results for different targets (UI and Evaluator)
    let analysis_v1 = app.audio_analyzer.get_latest_analysis();
    let analysis_v2 = app.audio_analyzer.v2.get_latest_analysis();
    
    // Update evaluator with V2 analysis (9 bands)
    app.module_evaluator.update_audio(&analysis_v2);

    // 6. Media & Animation Updates
    sync_media_players(app);
    update_media_players(app, dt);
    let _param_updates = app.state.effect_animator_mut().update(dt as f64);

    // 7. Graph Evaluation & Bevy Sync (MODULARIZED)
    let graph_dirty = app.state.module_manager.graph_revision != app.last_graph_revision;
    perform_evaluation(app, &all_module_ids, &analysis_v1, graph_dirty);

    // 8. UI State Sync
    app.ui_state.current_audio_level = analysis_v1.rms_volume;
    app.ui_state.current_bpm = analysis_v1.tempo_bpm;
    app.ui_state.dashboard.set_audio_analysis(analysis_v1.clone());
    app.ui_state.dashboard.set_audio_devices(app.audio_devices.clone());
    app.ui_state.current_fps = app.current_fps;
    app.ui_state.current_frame_time_ms = app.current_frame_time_ms;
    app.ui_state.cpu_usage = app.sys_info.global_cpu_usage();

    // 9. Output Processing (MODULARIZED)
    sync_output_windows(app, elwt, ui_needs_sync, graph_dirty)?;
    app.last_graph_revision = app.state.module_manager.graph_revision;

    // 10. Periodic Tasks (Auto-save)
    if app.last_autosave.elapsed().as_secs() >= 30 {
        if app.state.dirty {
            if let Some(path) = dirs::data_local_dir().map(|p| p.join("MapFlow").join("autosave.mflow")) {
                let _ = std::fs::create_dir_all(path.parent().unwrap());
                let _ = save_project(&app.state, &path);
            }
        }
        app.last_autosave = std::time::Instant::now();
    }

    // FPS Calculation
    let frame_time_ms = dt * 1000.0;
    app.fps_samples.push_back(frame_time_ms);
    if app.fps_samples.len() > 60 { app.fps_samples.pop_front(); }
    if !app.fps_samples.is_empty() {
        let avg: f32 = app.fps_samples.iter().sum::<f32>() / app.fps_samples.len() as f32;
        app.current_frame_time_ms = avg;
        app.current_fps = if avg > 0.0 { 1000.0 / avg } else { 0.0 };
    }

    Ok(())
}
