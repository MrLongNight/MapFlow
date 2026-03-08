use crate::app::actions::{handle_mcp_actions, handle_ui_actions};
use crate::app::core::app_struct::App;
use crate::orchestration::media::{sync_media_players, update_media_players};
use crate::orchestration::outputs::sync_output_windows;
use anyhow::Result;
use mapmap_io::save_project;
use std::collections::HashSet;
use tracing::info;

/// Global update loop (physics/logic), independent of render rate per window.
pub fn update(app: &mut App, elwt: &winit::event_loop::ActiveEventLoop, dt: f32) -> Result<()> {
    // Process internal MCP actions first
    handle_mcp_actions(app);

    let ui_needs_sync = handle_ui_actions(app).unwrap_or(false);

    // Update evaluator with active keys for Shortcut triggers
    let active_keys: HashSet<String> = app
        .egui_context
        .input(|i| i.keys_down.iter().map(|k| format!("{:?}", k)).collect());
    app.module_evaluator.update_keys(&active_keys);

    // Update evaluator with raw MIDI/OSC events
    for (channel, note) in &app.control_manager.raw_midi_events {
        app.module_evaluator.record_midi(*channel, *note);
    }
    for addr in &app.control_manager.raw_osc_events {
        app.module_evaluator.record_osc(addr);
    }

    // --- Media Player Update ---
    sync_media_players(app);
    update_media_players(app, dt);

    // --- Effect Animator Update ---
    let param_updates = app.state.effect_animator_mut().update(dt as f64);

    // Check if we need to re-evaluate the graph
    // We re-evaluate if:
    // 1. Graph structure changed
    // 2. Parameters are animating
    // 3. UI explicitly requested sync
    // 4. We have active modulations (Audio/MIDI/OSC)
    let graph_dirty = app.state.module_manager.graph_revision != app.last_graph_revision;

    // Check for reactive nodes or active modulations
    let mut has_reactive_content = false;
    for module in app.state.module_manager.modules() {
        if !module.parts.is_empty() {
            // If we have any parts, we check if any are triggers or have active modulations
            // For now, if there is ANY content, we keep evaluating to ensure reactivity.
            // In a future update, we could check specifically for ModulePartType::Trigger
            has_reactive_content = true;
            break;
        }
    }

    let needs_re_eval =
        graph_dirty || !param_updates.is_empty() || ui_needs_sync || has_reactive_content;

    let all_module_ids: Vec<u64> = app
        .state
        .module_manager
        .list_modules()
        .iter()
        .map(|m| m.id)
        .collect();
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

    if needs_re_eval {
        app.render_ops.clear();

        // --- Bevy Runner Update ---
        let mut node_triggers = std::collections::HashMap::new();

        for module_id in &modules_for_eval {
            if let Some(module_ref) = app.state.module_manager.get_module(*module_id) {
                // OPTIMIZATION: Only apply structural graph state to Bevy if changed
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

                // Collect render ops while we are already evaluating for triggers
                app.render_ops.extend(
                    eval_result
                        .render_ops
                        .iter()
                        .cloned()
                        .map(|op| (*module_id, op)),
                );
            }
        }

        if let Some(runner) = &mut app.bevy_runner {
            let analysis = app.audio_analyzer.get_latest_analysis();
            let mut bands = [0.0; 9];
            for (i, &energy) in analysis.band_energies.iter().enumerate() {
                if i < 9 {
                    bands[i] = energy;
                }
            }

            let trigger_data = mapmap_core::audio_reactive::AudioTriggerData {
                band_energies: bands,
                rms_volume: analysis.rms_volume,
                peak_volume: analysis.peak_volume,
                beat_detected: analysis.beat_detected,
                beat_strength: analysis.beat_strength,
                bpm: analysis.tempo_bpm,
            };
            runner.update(&trigger_data, &node_triggers);
        }
    }

    // Always sync some UI values
    let analysis = app.audio_analyzer.get_latest_analysis();
    app.ui_state.current_audio_level = analysis.rms_volume;
    app.ui_state.current_bpm = analysis.tempo_bpm;
    app.ui_state
        .module_canvas
        .set_audio_data(mapmap_core::audio_reactive::AudioTriggerData {
            band_energies: {
                let mut b = [0.0; 9];
                for i in 0..9.min(analysis.band_energies.len()) {
                    b[i] = analysis.band_energies[i];
                }
                b
            },
            rms_volume: analysis.rms_volume,
            peak_volume: analysis.peak_volume,
            beat_detected: analysis.beat_detected,
            beat_strength: analysis.beat_strength,
            bpm: analysis.tempo_bpm,
        });

    app.ui_state.dashboard.set_audio_analysis(analysis.clone());
    app.ui_state
        .dashboard
        .set_audio_devices(app.audio_devices.clone());

    // --- Output Processing ---
    {
        let current_output_ids: HashSet<u64> = app
            .window_manager
            .iter()
            .filter(|wc| wc.output_id != 0)
            .map(|wc| wc.output_id)
            .collect();

        if ui_needs_sync || current_output_ids != app.last_output_ids {
            info!(
                "Output set changed: {:?} -> {:?}",
                app.last_output_ids, current_output_ids
            );
            let ops_only: Vec<mapmap_core::module_eval::RenderOp> =
                app.render_ops.iter().map(|(_, op)| op.clone()).collect();
            if let Err(e) = sync_output_windows(app, elwt, &ops_only, None) {
                tracing::error!("Failed to sync output windows: {}", e);
            }
            app.last_output_ids = current_output_ids;
        }

        // Update revision after sync
        app.last_graph_revision = app.state.module_manager.graph_revision;
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

    // --- Sync UI State with App Data ---
    app.ui_state.current_fps = app.current_fps;
    app.ui_state.current_frame_time_ms = app.current_frame_time_ms;
    app.ui_state.cpu_usage = app.sys_info.global_cpu_usage();
    app.ui_state.ram_usage_mb = if let Ok(pid) = sysinfo::get_current_pid() {
        app.sys_info
            .process(pid)
            .map(|p| p.memory() as f32 / 1024.0 / 1024.0)
            .unwrap_or(0.0)
    } else {
        0.0
    };

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
    static PERF_LOG_COUNTER: std::sync::atomic::AtomicUsize =
        std::sync::atomic::AtomicUsize::new(0);
    if PERF_LOG_COUNTER
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        .is_multiple_of(600)
    {
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
