use crate::app::actions::{handle_mcp_actions, handle_ui_actions};
use crate::app::core::app_struct::App;
use crate::orchestration::evaluation::perform_evaluation;
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

    let current_theme = app.ui_state.user_config.theme.theme;
    let ui_needs_sync = handle_ui_actions(app).unwrap_or(false);
    if app.ui_state.user_config.theme.theme != current_theme {
        app.ui_state.user_config.theme.apply(&app.egui_context);
    }

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

    if app.last_sysinfo_refresh.elapsed() >= std::time::Duration::from_millis(500) {
        app.sys_info.refresh_cpu_usage();
        app.sys_info.refresh_memory();
        app.last_sysinfo_refresh = std::time::Instant::now();
    }
    app.ui_state.cpu_usage = app.sys_info.global_cpu_usage();
    app.ui_state.ram_usage_mb = app.sys_info.used_memory() as f32 / (1024.0 * 1024.0);

    // --- Bevy Runner Update ---
    if let Some(runner) = &mut app.bevy_runner {
        let runner: &mut mapmap_bevy::BevyRunner = runner;
        let mut node_triggers = std::collections::HashMap::new();

        for module_id in &modules_for_eval {
            if let Some(module_ref) = app.state.module_manager.get_module(*module_id) {
                // OPTIMIZATION: Only apply structural graph state to Bevy if changed
                if graph_dirty {
                    runner.apply_graph_state(module_ref);
                }

                let eval_result = app.module_evaluator.evaluate(
                    module_ref,
                    &app.state.module_manager.shared_media,
                    app.state.module_manager.graph_revision,
                );

                for (part_id, values) in &eval_result.trigger_values {
                    if let Some(last_val) = values.last() {
                        node_triggers.insert((*module_id, *part_id), *last_val);
                    }
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

        // SYNC WITH UI
        app.ui_state
            .module_canvas
            .set_audio_data(trigger_data.clone());
        app.ui_state.current_audio_level = trigger_data.rms_volume;
        app.ui_state.current_bpm = trigger_data.bpm;
    } else {
        // Fallback for when Bevy is disabled: still need to evaluate for render_ops
        for module_id in &modules_for_eval {
            if let Some(module_ref) = app.state.module_manager.get_module(*module_id) {
                let eval_result = app.module_evaluator.evaluate(
                    module_ref,
                    &app.state.module_manager.shared_media,
                    app.state.module_manager.graph_revision,
                );
                app.render_ops.extend(
                    eval_result
                        .render_ops
                        .iter()
                        .cloned()
                        .map(|op| (*module_id, op)),
                );
            }
        }
    }

    // --- Rebuild output_assignments from render_ops ---
    // Maps each Projector output ID to the source texture names that feed into it.
    // Without this, prepare_texture_previews cannot find textures and no video is shown.
    app.output_assignments.clear();
    for (module_id, op) in &app.render_ops {
        if let OutputType::Projector { id, .. } = &op.output_type {
            if let Some(src_id) = op.source_part_id {
                let tex_name = format!("part_{}_{}", module_id, src_id);
                app.output_assignments
                    .entry(*id)
                    .or_default()
                    .push(tex_name);
            }
        }
    }

    // --- Output Window Sync (Optimized) ---
    if ui_needs_sync || graph_dirty {
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

        let prev_output_ids: HashSet<u64> = app
            .window_manager
            .iter()
            .filter(|wc| wc.output_id != 0)
            .map(|wc| wc.output_id)
            .collect();

        if ui_needs_sync || current_output_ids != prev_output_ids {
            info!(
                "Output set changed: {:?} -> {:?}",
                prev_output_ids, current_output_ids
            );
            let ops_only: Vec<mapmap_core::module_eval::RenderOp> =
                app.render_ops.iter().map(|(_, op)| op.clone()).collect();
            if let Err(e) = sync_output_windows(app, elwt, &ops_only, None) {
                tracing::error!("Failed to sync output windows: {}", e);
            }
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

    // Check auto-save (every 30s)
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
