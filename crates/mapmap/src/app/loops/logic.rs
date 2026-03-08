use crate::app::actions::{handle_mcp_actions, handle_ui_actions};
use crate::app::core::app_struct::App;
use crate::orchestration::media::{sync_media_players, update_media_players};
use crate::orchestration::outputs::sync_output_windows;
use anyhow::Result;
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
    // Optimization: Only collect keys if we have content
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

    // 5. Media & Animation Updates
    sync_media_players(app);
    update_media_players(app, dt);
    let _param_updates = app.state.effect_animator_mut().update(dt as f64);

    // 6. Evaluation Logic
    let graph_dirty = app.state.module_manager.graph_revision != app.last_graph_revision;
    
    // For now we re-evaluate every frame if modules exist to ensure reactivity (LFOs, Audio, etc.)
    // but we avoid redundant Bevy structural updates.
    let needs_re_eval = true; 

    if needs_re_eval {
        app.render_ops.clear();
        let mut node_triggers = std::collections::HashMap::new();
        
        let show_module_id = app.ui_state.timeline_panel.runtime_show_module(
            app.state.effect_animator.get_current_time() as f32,
            app.state.effect_animator.is_playing(),
            &all_module_ids,
        );
        
        let modules_for_eval: Vec<u64> = if let Some(mid) = show_module_id {
            vec![mid]
        } else {
            all_module_ids
        };

        for module_id in &modules_for_eval {
            if let Some(module_ref) = app.state.module_manager.get_module(*module_id) {
                // Only sync graph structure to Bevy if it actually changed
                if graph_dirty {
                    if let Some(runner) = &mut app.bevy_runner {
                        runner.apply_graph_state(module_ref);
                    }
                }

                let eval_result = app.module_evaluator.evaluate(
                    module_ref,
                    &app.state.module_manager.shared_media,
                    app.state.module_manager.graph_revision,
                );

                for (part_id, values) in &eval_result.trigger_values {
                    let max_val = values.iter().cloned().fold(0.0, f32::max);
                    node_triggers.insert((*module_id, *part_id), max_val);
                }

                app.render_ops.extend(
                    eval_result.render_ops.iter().cloned().map(|op| (*module_id, op)),
                );
            }
        }

        // Sync with Bevy (only if runner exists)
        if let Some(runner) = &mut app.bevy_runner {
            let analysis = app.audio_analyzer.get_latest_analysis();
            let trigger_data = mapmap_core::audio_reactive::AudioTriggerData {
                band_energies: {
                    let mut b = [0.0; 9];
                    for i in 0..9.min(analysis.band_energies.len()) { b[i] = analysis.band_energies[i]; }
                    b
                },
                rms_volume: analysis.rms_volume,
                peak_volume: analysis.peak_volume,
                beat_detected: analysis.beat_detected,
                beat_strength: analysis.beat_strength,
                bpm: analysis.tempo_bpm,
            };
            runner.update(&trigger_data, &node_triggers);
        }
    }

    // 7. UI Sync
    let analysis = app.audio_analyzer.get_latest_analysis();
    app.ui_state.current_audio_level = analysis.rms_volume;
    app.ui_state.current_bpm = analysis.tempo_bpm;
    app.ui_state.dashboard.set_audio_analysis(analysis.clone());
    app.ui_state.dashboard.set_audio_devices(app.audio_devices.clone());

    // 8. Output Processing
    {
        let current_output_ids: HashSet<u64> = app.window_manager.iter()
            .filter(|wc| wc.output_id != 0).map(|wc| wc.output_id).collect();

        if ui_needs_sync || current_output_ids != app.last_output_ids || graph_dirty {
            let ops_only: Vec<mapmap_core::module_eval::RenderOp> =
                app.render_ops.iter().map(|(_, op)| op.clone()).collect();
            let _ = sync_output_windows(app, elwt, &ops_only, None);
            app.last_output_ids = current_output_ids;
        }
        app.last_graph_revision = app.state.module_manager.graph_revision;
    }

    // 9. Periodic Tasks (Auto-save, Sys-info, GC)
    if app.last_autosave.elapsed().as_secs() >= 30 {
        if app.state.dirty {
            if let Some(path) = dirs::data_local_dir().map(|p| p.join("MapFlow").join("autosave.mflow")) {
                let _ = std::fs::create_dir_all(path.parent().unwrap());
                let _ = save_project(&app.state, &path);
            }
        }
        app.last_autosave = std::time::Instant::now();
    }

    // FPS & Performance
    let frame_time_ms = dt * 1000.0;
    app.fps_samples.push_back(frame_time_ms);
    if app.fps_samples.len() > 60 { app.fps_samples.pop_front(); }
    if !app.fps_samples.is_empty() {
        let avg: f32 = app.fps_samples.iter().sum::<f32>() / app.fps_samples.len() as f32;
        app.current_frame_time_ms = avg;
        app.current_fps = if avg > 0.0 { 1000.0 / avg } else { 0.0 };
    }

    app.ui_state.current_fps = app.current_fps;
    app.ui_state.current_frame_time_ms = app.current_frame_time_ms;
    app.ui_state.cpu_usage = app.sys_info.global_cpu_usage();

    Ok(())
}
