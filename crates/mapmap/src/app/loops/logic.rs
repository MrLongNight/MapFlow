use crate::app::actions::{handle_mcp_actions, handle_ui_actions};
use crate::app::core::app_struct::App;
use crate::orchestration::media::{sync_media_players, update_media_players};
use crate::orchestration::outputs::sync_output_windows;
use anyhow::Result;
use mapmap_core::module::{ModulePartType, OutputType};
use mapmap_io::save_project;
use mapmap_media::PlaybackCommand;
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
    let param_updates = app.state.effect_animator.update(dt as f64);
    if !param_updates.is_empty() {
        tracing::trace!("Effect updates: {}", param_updates.len());
    }

    // Update Evaluator State
    let analysis = app.audio_analyzer.get_latest_analysis();
    app.module_evaluator.update_audio(&analysis);
    app.module_evaluator.update_keys(&app.ui_state.active_keys);

    // --- Bevy Runner Update ---
    if let Some(runner) = &mut app.bevy_runner {
        // First sync graph state
        for module in app.state.module_manager.iter_modules() {
            runner.apply_graph_state(module);
        }

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
    app.module_evaluator.clear_render_ops();
    for module in app.state.module_manager.iter_modules() {
        let result = app
            .module_evaluator
            .evaluate_append(module, &app.state.module_manager.shared_media);

        // Update UI Trigger Visualization (only for active module)
        if Some(module.id) == app.ui_state.module_canvas.active_module_id {
            app.ui_state.module_canvas.last_trigger_values = result
                .trigger_values
                .iter()
                .map(|(k, v)| (*k, v.iter().copied().fold(0.0, f32::max)))
                .collect();
        }

        // Handle Source Commands
        for (part_id, cmd) in &result.source_commands {
            match cmd {
                mapmap_core::SourceCommand::PlayMedia { trigger_value, .. } => {
                    let player_key = (module.id, *part_id);
                    if let Some((_, player)) = app.media_players.get_mut(&player_key) {
                        if *trigger_value > 0.1 {
                            let _ = player.command_sender().send(PlaybackCommand::Play);
                        }
                    }
                }
                mapmap_core::SourceCommand::NdiInput {
                    source_name: Some(_src_name),
                    ..
                } => {
                    #[cfg(feature = "ndi")]
                    {
                        let receiver = app.ndi_receivers.entry(*part_id).or_insert_with(|| {
                            mapmap_io::ndi::NdiReceiver::new()
                                .expect("Failed to create NDI receiver")
                        });
                        if let Ok(Some(frame)) =
                            receiver.receive(std::time::Duration::from_millis(0))
                        {
                            if let mapmap_io::format::FrameData::Cpu(data) = &frame.data {
                                let tex_name = format!("part_{}_{}", module.id, part_id);
                                app.texture_pool.upload_data(
                                    &app.backend.queue,
                                    &tex_name,
                                    data,
                                    frame.format.width,
                                    frame.format.height,
                                );
                            }
                        }
                    }
                }
                mapmap_core::SourceCommand::HueOutput {
                    brightness,
                    hue,
                    saturation,
                    strobe,
                    ids,
                } => {
                    app.hue_controller.update_from_command(
                        ids.as_deref(),
                        *brightness,
                        *hue,
                        *saturation,
                        *strobe,
                    );
                }
                _ => {}
            }
        }
    }

    // Update Output Assignments for Preview/Window Mapping
    app.output_assignments.clear();
    for op in app.module_evaluator.render_ops() {
        if let mapmap_core::module::OutputType::Projector { id, .. } = &op.output_type {
            if let Some(source_id) = op.source_part_id {
                let tex_name = format!("part_{}_{}", op.module_id, source_id);
                app.output_assignments
                    .entry(*id)
                    .or_default()
                    .push(tex_name.clone());
            }
        }
    }

    // Update UI (BPM and Dashboard) using the analysis obtained earlier
    // (Note: analysis variable from Bevy scope is dropped, so we get it again or hoist it)
    // Actually, 'analysis' is in 'Bevy Runner Update' block.
    // If Bevy runner is None, we don't get analysis?
    // We should hoist analysis fetching outside Bevy block.

    // Update UI state using analysis
    app.ui_state.current_bpm = analysis.tempo_bpm;

    let legacy_analysis = mapmap_core::audio::AudioAnalysis {
        timestamp: analysis.timestamp,
        fft_magnitudes: analysis.fft_magnitudes,
        band_energies: [
            analysis.band_energies[0],
            analysis.band_energies[1],
            analysis.band_energies[2],
            analysis.band_energies[3],
            analysis.band_energies[4],
            analysis.band_energies[6],
            analysis.band_energies[8],
        ],
        rms_volume: analysis.rms_volume,
        peak_volume: analysis.peak_volume,
        beat_detected: analysis.beat_detected,
        beat_strength: analysis.beat_strength,
        onset_detected: false,
        tempo_bpm: analysis.tempo_bpm,
        waveform: analysis.waveform,
    };
    app.ui_state.dashboard.set_audio_analysis(legacy_analysis);

    // Sync output windows based on MODULE GRAPH STRUCTURE (stable),
    // NOT render_ops (which can be empty/fluctuate).
    let current_output_ids: HashSet<u64> = app
        .state
        .module_manager
        .iter_modules()
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
        // Pass empty render ops as they are unused in sync_output_windows
        if let Err(e) = sync_output_windows(app, elwt, &[], None) {
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

    // Periodic Cleanups (every 600 frames ~ 10s at 60fps)
    if (app.backend.queue.get_timestamp_period() as u64).is_multiple_of(600) { // Just using a periodic check via backend or frame count if available
         // ... (This logic was implicit in main loop, here we might need frame count or timer)
    }

    Ok(())
}
