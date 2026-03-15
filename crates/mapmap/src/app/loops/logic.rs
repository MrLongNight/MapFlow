use crate::app::actions::{handle_mcp_actions, handle_ui_actions};
use crate::app::core::app_struct::App;
use crate::orchestration::evaluation::perform_evaluation;
use crate::orchestration::media::{sync_media_players, update_media_players};
use crate::orchestration::outputs::sync_output_windows;
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
    let all_module_ids: Vec<u64> = app
        .state
        .module_manager
        .modules()
        .iter()
        .map(|m| m.id)
        .collect();

    // Determine which modules to evaluate based on timeline
    let show_module_id = app.ui_state.timeline_panel.runtime_show_module(
        app.state.effect_animator.get_current_time() as f32,
        app.state.effect_animator.is_playing(),
        &all_module_ids,
    );
    if let Some(active_module_id) = show_module_id {
        app.ui_state
            .module_canvas
            .set_active_module(Some(active_module_id));
    }
    let modules_for_eval: Vec<u64> = if let Some(module_id) = show_module_id {
        vec![module_id]
    } else {
        all_module_ids.clone()
    };

    // --- Performance Optimization: Early return if idle ---
    if all_module_ids.is_empty() {
        app.ui_state.current_fps = app.current_fps;
        app.ui_state.current_frame_time_ms = app.current_frame_time_ms;
        app.last_graph_revision = app.state.module_manager.graph_revision;
        return Ok(());
    }

    // 4. Update evaluator with reaktive events
    app.module_evaluator.set_delta_time(dt);

    let active_keys: HashSet<String> = app
        .egui_context
        .input(|i| i.keys_down.iter().map(|k| format!("{:?}", k)).collect());
    app.module_evaluator.update_keys(&active_keys);

    // --- Control System Update ---
    let (midi_events, osc_packets) = app.control_manager.update();

    // Update shared media state with active events for trigger nodes
    {
        let shared = &mut app.state.module_manager_mut().shared_media;
        shared.active_midi_events.clear();
        shared.active_midi_cc.clear();
        shared.active_osc_messages.clear();

        for msg in &midi_events {
            match msg {
                mapmap_control::midi::MidiMessage::NoteOn {
                    channel,
                    note,
                    velocity,
                } => {
                    shared.active_midi_events.push((*channel, *note, *velocity));
                }
                mapmap_control::midi::MidiMessage::ControlChange {
                    channel,
                    controller,
                    value,
                } => {
                    shared
                        .active_midi_cc
                        .insert((*channel, *controller), *value);
                }
                _ => {}
            }
        }

        for packet in &osc_packets {
            if let rosc::OscPacket::Message(msg) = packet {
                let vals: Vec<f32> = msg
                    .args
                    .iter()
                    .filter_map(|a| match a {
                        rosc::OscType::Float(f) => Some(*f),
                        rosc::OscType::Double(d) => Some(*d as f32),
                        rosc::OscType::Int(i) => Some(*i as f32),
                        _ => None,
                    })
                    .collect();
                shared.active_osc_messages.insert(msg.addr.clone(), vals);
            }
        }
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
    perform_evaluation(app, &modules_for_eval, &analysis_v1, graph_dirty);

    // 8. UI State Sync
    app.ui_state.current_audio_level = analysis_v1.rms_volume;
    app.ui_state.current_bpm = analysis_v1.tempo_bpm;
    app.ui_state
        .dashboard
        .set_audio_analysis(analysis_v1.clone());
    app.ui_state
        .dashboard
        .set_audio_devices(app.audio_devices.clone());
    app.ui_state.current_fps = app.current_fps;
    app.ui_state.current_frame_time_ms = app.current_frame_time_ms;

    if app.last_sysinfo_refresh.elapsed() >= std::time::Duration::from_millis(1000) {
        // Only refresh CPU usage and current process to save performance
        app.sys_info.refresh_cpu_usage();
        app.sys_info.refresh_memory();

        // Refresh only the current process
        if let Ok(pid) = sysinfo::get_current_pid() {
            app.sys_info
                .refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
        }

        app.last_sysinfo_refresh = std::time::Instant::now();
    }

    // Get process-specific CPU usage if available
    if let Ok(pid) = sysinfo::get_current_pid() {
        if let Some(process) = app.sys_info.process(pid) {
            // sysinfo returns sum of all core percentages, divide by core count to match Task Manager
            let num_cpus = app.sys_info.cpus().len() as f32;
            app.ui_state.cpu_usage = process.cpu_usage() / num_cpus.max(1.0);
        } else {
            app.ui_state.cpu_usage = 0.0;
        }
    } else {
        // Fallback to global if we can't get PID (unlikely)
        app.ui_state.cpu_usage = app.sys_info.global_cpu_usage();
    }
    app.ui_state.ram_usage_mb = app.sys_info.used_memory() as f32 / (1024.0 * 1024.0);

    // 9. Output Processing (MODULARIZED)
    sync_output_windows(app, elwt, ui_needs_sync, graph_dirty)?;
    app.last_graph_revision = app.state.module_manager.graph_revision;

    // 10. Periodic Tasks (Auto-save)
    if app.last_autosave.elapsed().as_secs() >= 30 {
        if app.state.dirty {
            if let Some(path) =
                dirs::data_local_dir().map(|p| p.join("MapFlow").join("autosave.mflow"))
            {
                let _ = std::fs::create_dir_all(path.parent().unwrap());
                let _ = save_project(&app.state, &path);
            }
        }
        app.last_autosave = std::time::Instant::now();
    }

    // FPS Calculation
    let frame_time_ms = dt * 1000.0;
    app.fps_samples.push_back(frame_time_ms);
    if app.fps_samples.len() > 60 {
        app.fps_samples.pop_front();
    }
    if !app.fps_samples.is_empty() {
        let avg: f32 = app.fps_samples.iter().sum::<f32>() / app.fps_samples.len() as f32;
        app.current_frame_time_ms = avg;
        app.current_fps = if avg > 0.0 { 1000.0 / avg } else { 0.0 };
    }

    Ok(())
}
