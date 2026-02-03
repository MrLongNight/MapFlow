//! MapFlow - Open source Vj Projection Mapping Software
//!
//! This is the main application crate for MapFlow.

#![warn(missing_docs)]

pub mod app;
mod media_manager_ui;
pub mod orchestration;
/// UI components.
pub mod ui;
mod window_manager;

use anyhow::Result;
use mapmap_core::audio::backend::AudioBackend;
#[cfg(feature = "midi")]
use mapmap_core::audio_reactive::AudioTriggerData;

// Define McpAction locally or import if we move it to core later -> Removed local definition

use mapmap_io::{load_project, save_project};

use mapmap_control::Action;
use mapmap_core::OutputId;
use mapmap_mcp::McpAction;
use mapmap_media::PlaybackCommand;
use mapmap_ui::EdgeBlendAction;

use rfd::FileDialog;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

use winit::{event::WindowEvent, event_loop::EventLoop};

use crate::app::core::app_struct::App;

impl App {
    /// Handles a window event.
    pub fn handle_event(
        &mut self,
        event: winit::event::Event<()>,
        elwt: &winit::event_loop::ActiveEventLoop,
    ) -> Result<()> {
        if self.exit_requested {
            elwt.exit();
        }

        match &event {
            winit::event::Event::WindowEvent { event, window_id } => {
                if let Some(main_window) = self.window_manager.get(0) {
                    if *window_id == main_window.window.id() {
                        let _ = self.egui_state.on_window_event(&main_window.window, event);
                    }
                }

                let output_id = self
                    .window_manager
                    .get_output_id_from_window_id(*window_id)
                    .unwrap_or(0);

                match event {
                    WindowEvent::CloseRequested => {
                        if output_id == 0 {
                            elwt.exit();
                        }
                    }
                    WindowEvent::Resized(size) => {
                        let new_size =
                            if let Some(window_context) = self.window_manager.get_mut(output_id) {
                                if size.width > 0 && size.height > 0 {
                                    window_context.surface_config.width = size.width;
                                    window_context.surface_config.height = size.height;
                                    window_context.surface.configure(
                                        &self.backend.device,
                                        &window_context.surface_config,
                                    );
                                    Some((
                                        size.width,
                                        size.height,
                                        window_context.surface_config.format,
                                    ))
                                } else {
                                    None
                                }
                            } else {
                                None
                            };
                        // Recreate dummy texture for the new size
                        match new_size {
                            Some((width, height, format)) => {
                                self.create_dummy_texture(width, height, format);
                            }
                            None => {
                                tracing::warn!(
                                    "Resize event received but no valid new size was determined."
                                );
                            }
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        if let Err(e) = self.render(output_id) {
                            error!("Render error on output {}: {}", output_id, e);
                        }
                    }
                    // Handle keyboard input for Shortcut triggers
                    WindowEvent::KeyboardInput { event, .. } => {
                        if let winit::keyboard::PhysicalKey::Code(key_code) = event.physical_key {
                            let key_name = format!("{:?}", key_code);
                            match event.state {
                                winit::event::ElementState::Pressed => {
                                    self.ui_state.active_keys.insert(key_name);
                                }
                                winit::event::ElementState::Released => {
                                    self.ui_state.active_keys.remove(&key_name);
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }
            winit::event::Event::LoopExiting => {
                info!("Application exiting, saving autosave and config...");

                // 1. Save User Config (UI State)
                self.ui_state.user_config.show_left_sidebar = self.ui_state.show_left_sidebar;
                self.ui_state.user_config.show_inspector = self.ui_state.show_inspector;
                self.ui_state.user_config.show_timeline = self.ui_state.show_timeline;
                self.ui_state.user_config.show_media_browser = self.ui_state.show_media_browser;
                self.ui_state.user_config.show_module_canvas = self.ui_state.show_module_canvas;
                self.ui_state.user_config.show_controller_overlay =
                    self.ui_state.show_controller_overlay;

                // Get main window maximization state
                if let Some(main_window) = self.window_manager.get(0) {
                    self.ui_state.user_config.window_maximized = main_window.window.is_maximized();
                }

                if let Err(e) = self.ui_state.user_config.save() {
                    error!("Failed to save user config: {}", e);
                } else {
                    info!("User config saved successfully.");
                }

                // 2. Save Project Autosave
                let autosave_path = dirs::data_local_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("MapFlow")
                    .join("autosave.mflow");

                if let Some(parent) = autosave_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }

                if let Err(e) = save_project(&self.state, &autosave_path) {
                    error!("Exit autosave failed: {}", e);
                } else {
                    info!("Exit autosave successful to {:?}", autosave_path);
                }

                if self.restart_requested {
                    info!("Restarting application...");
                    if let Ok(current_exe) = std::env::current_exe() {
                        let args: Vec<String> = std::env::args().collect();
                        // Spawn new process detached
                        match std::process::Command::new(current_exe)
                            .args(&args[1..])
                            .spawn()
                        {
                            Ok(_) => info!("Restart process spawned successfully."),
                            Err(e) => error!("Failed to restart application: {}", e),
                        }
                    }
                }
            }
            winit::event::Event::AboutToWait => {
                // --- Non-blocking Frame Limiter ---
                let target_fps = self.ui_state.user_config.target_fps.unwrap_or(60.0);
                let cap_fps = if target_fps <= 0.0 { 60.0 } else { target_fps };
                let frame_target = std::time::Duration::from_secs_f64(1.0 / cap_fps as f64);
                let time_since_last = std::time::Instant::now().duration_since(self.last_update);

                // Skip frame if too early (non-blocking)
                if time_since_last < frame_target {
                    // Don't block - use Poll to immediately re-check
                    elwt.set_control_flow(winit::event_loop::ControlFlow::Poll);
                    return Ok(());
                }

                // Always use Poll mode - VJ software needs continuous updates
                elwt.set_control_flow(winit::event_loop::ControlFlow::Poll);

                // --- Update State (Physics/Media) ---
                let actual_dt = time_since_last.as_secs_f32();
                self.update(elwt, actual_dt);
                self.last_update = std::time::Instant::now();

                // Poll MIDI
                #[cfg(feature = "midi")]
                if let Some(handler) = &mut self.midi_handler {
                    while let Some(msg) = handler.poll_message() {
                        self.ui_state.controller_overlay.process_midi(msg);
                        self.ui_state.module_canvas.process_midi_message(msg);
                    }
                }

                // Autosave check (every 5 minutes)
                if self.state.dirty
                    && self.last_autosave.elapsed() >= std::time::Duration::from_secs(300)
                {
                    let autosave_path = dirs::data_local_dir()
                        .unwrap_or_else(|| PathBuf::from("."))
                        .join("MapFlow")
                        .join("autosave.mflow");
                    if let Some(parent) = autosave_path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }
                    if let Err(e) = save_project(&self.state, &autosave_path) {
                        error!("Autosave failed: {}", e);
                    } else {
                        info!("Autosave successful to {:?}", autosave_path);
                        self.last_autosave = std::time::Instant::now();
                    }
                }

                // --- CRITICAL: Render all windows DIRECTLY (not via event queue) ---
                // This ensures output windows update immediately, not after event dispatch
                let output_ids: Vec<u64> =
                    self.window_manager.iter().map(|wc| wc.output_id).collect();
                for output_id in output_ids {
                    if let Err(e) = self.render(output_id) {
                        error!("Render error on output {}: {}", output_id, e);
                    }
                }

                // Process audio
                // Process audio
                let timestamp = self.start_time.elapsed().as_secs_f64();
                if let Some(backend) = &mut self.audio_backend {
                    let samples = backend.get_samples();
                    if !samples.is_empty() {
                        self.audio_analyzer.process_samples(&samples, timestamp);
                    }
                }

                // Get analysis results
                let analysis_v2 = self.audio_analyzer.get_latest_analysis();

                // --- MODULE EVALUATION ---
                self.module_evaluator.update_audio(&analysis_v2);
                self.module_evaluator
                    .update_keys(&self.ui_state.active_keys);

                // Process pending playback commands from UI
                for (part_id, cmd) in self
                    .ui_state
                    .module_canvas
                    .pending_playback_commands
                    .drain(..)
                {
                    info!(
                        "Processing playback command {:?} for part_id={}",
                        cmd, part_id
                    );
                    // If player doesn't exist and we get any command (except Reload), try to create it
                    // Find the module that owns this part to construct the key
                    let mut target_module_id = None;
                    for module in self.state.module_manager.modules() {
                        if module.parts.iter().any(|p| p.id == part_id) {
                            target_module_id = Some(module.id);
                            break;
                        }
                    }

                    if let Some(mod_id) = target_module_id {
                        let player_key = (mod_id, part_id);

                        // If player doesn't exist and we get any command (except Reload), try to create it
                        if !self.media_players.contains_key(&player_key)
                            && cmd != mapmap_ui::MediaPlaybackCommand::Reload
                        {
                            info!(
                                "Player doesn't exist for part_id={}, attempting to create...",
                                part_id
                            );
                            // Find the source path
                            if let Some(module) = self.state.module_manager.get_module(mod_id) {
                                if let Some(part) = module.parts.iter().find(|p| p.id == part_id) {
                                    if let mapmap_core::module::ModulePartType::Source(
                                        mapmap_core::module::SourceType::MediaFile {
                                            ref path, ..
                                        },
                                    ) = &part.part_type
                                    {
                                        info!(
                                            "Found media path: '{}' in module '{}'",
                                            path, module.name
                                        );
                                        if !path.is_empty() {
                                            match mapmap_media::open_path(path) {
                                                Ok(player) => {
                                                    info!(
                                                        "Successfully created player for '{}'",
                                                        path
                                                    );
                                                    self.media_players
                                                        .insert(player_key, (path.clone(), player));
                                                }
                                                Err(e) => {
                                                    error!(
                                                        "Failed to load media '{}': {}",
                                                        path, e
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if let Some((_, player)) = self.media_players.get_mut(&player_key) {
                            match cmd {
                                mapmap_ui::MediaPlaybackCommand::Play => {
                                    let _ = player.command_sender().send(PlaybackCommand::Play);
                                }
                                mapmap_ui::MediaPlaybackCommand::Pause => {
                                    let _ = player.command_sender().send(PlaybackCommand::Pause);
                                }
                                mapmap_ui::MediaPlaybackCommand::Stop => {
                                    let _ = player.command_sender().send(PlaybackCommand::Stop);
                                }
                                mapmap_ui::MediaPlaybackCommand::Reload => {
                                    // Remove existing player - it will be recreated with new path
                                    info!("Reloading media player for part_id={}", part_id);
                                    // (Player removal handled below)
                                }
                                mapmap_ui::MediaPlaybackCommand::SetSpeed(speed) => {
                                    info!("Setting speed to {} for part_id={}", speed, part_id);
                                    let _ = player
                                        .command_sender()
                                        .send(PlaybackCommand::SetSpeed(speed));
                                }
                                mapmap_ui::MediaPlaybackCommand::SetLoop(enabled) => {
                                    info!("Setting loop to {} for part_id={}", enabled, part_id);
                                    let mode = if enabled {
                                        mapmap_media::LoopMode::Loop
                                    } else {
                                        mapmap_media::LoopMode::PlayOnce
                                    };
                                    let _ = player
                                        .command_sender()
                                        .send(PlaybackCommand::SetLoopMode(mode));
                                }
                                mapmap_ui::MediaPlaybackCommand::Seek(position) => {
                                    info!("Seeking to {} for part_id={}", position, part_id);
                                    let _ = player.command_sender().send(PlaybackCommand::Seek(
                                        std::time::Duration::from_secs_f64(position),
                                    ));
                                }
                            }
                        }

                        // Handle Reload by removing player and immediately recreating
                        if cmd == mapmap_ui::MediaPlaybackCommand::Reload {
                            if self.media_players.remove(&player_key).is_some() {
                                info!(
                                    "Removed old media player for part_id={} for reload",
                                    part_id
                                );
                            }
                            // Immediately recreate the player
                            if let Some(module) = self.state.module_manager.get_module(mod_id) {
                                if let Some(part) = module.parts.iter().find(|p| p.id == part_id) {
                                    if let mapmap_core::module::ModulePartType::Source(
                                        mapmap_core::module::SourceType::MediaFile {
                                            ref path, ..
                                        },
                                    ) = &part.part_type
                                    {
                                        if !path.is_empty() {
                                            match mapmap_media::open_path(path) {
                                                Ok(player) => {
                                                    info!(
                                                        "Recreated player for '{}' after reload",
                                                        path
                                                    );
                                                    // Auto-play after reload
                                                    let _ = player
                                                        .command_sender()
                                                        .send(PlaybackCommand::Play);
                                                    self.media_players
                                                        .insert(player_key, (path.clone(), player));
                                                }
                                                Err(e) => {
                                                    error!(
                                                        "Failed to reload media '{}': {}",
                                                        path, e
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        warn!("Could not find module owner for part_id={}", part_id);
                    }
                }

                // Update all active media players and upload frames to texture pool
                // This ensures previews work even without triggers connected

                // (Redundant media player update removed - handled in regular update_media_players path)

                // CLEAR render ops for the new frame
                self.render_ops.clear();

                // Evaluate ALL modules to support parallel output
                for module in self.state.module_manager.list_modules() {
                    // DEBUG: Log which module we're evaluating
                    debug!(
                        "Evaluating module '{}' (ID {}): parts={}, connections={}",
                        module.name,
                        module.id,
                        module.parts.len(),
                        module.connections.len()
                    );

                    let result = self
                        .module_evaluator
                        .evaluate(module, &self.state.module_manager.shared_media);

                    // Accumulate Render Ops
                    let mut module_ops: Vec<(u64, mapmap_core::module_eval::RenderOp)> = result
                        .render_ops
                        .iter()
                        .cloned()
                        .map(|op| (module.id, op))
                        .collect();
                    self.render_ops.append(&mut module_ops);

                    // Update UI Trigger Visualization (only for active module)
                    if Some(module.id) == self.ui_state.module_canvas.active_module_id {
                        self.ui_state.module_canvas.last_trigger_values = result
                            .trigger_values
                            .iter()
                            .map(|(k, v)| (*k, v.iter().copied().fold(0.0, f32::max)))
                            .collect();
                    }

                    // 1. Handle Source Commands for this module
                    #[allow(clippy::for_kv_map)]
                    for (part_id, cmd) in &result.source_commands {
                        #[allow(clippy::single_match)]
                        match cmd {
                            mapmap_core::SourceCommand::PlayMedia {
                                path,
                                trigger_value,
                            } => {
                                let path = path.clone();
                                let part_id = *part_id;
                                let player_key = (module.id, part_id);
                                if path.is_empty() {
                                    continue;
                                }

                                let player_needs_reload = if let Some((current_path, _)) =
                                    self.media_players.get(&player_key)
                                {
                                    current_path != &path
                                } else {
                                    true
                                };

                                if player_needs_reload {
                                    match mapmap_media::open_path(&path) {
                                        Ok(player) => {
                                            self.media_players
                                                .insert(player_key, (path.clone(), player));
                                        }
                                        Err(e) => {
                                            error!("Failed to load media '{}': {}", path, e);
                                            continue;
                                        }
                                    }
                                }

                                if let Some((_, player)) = self.media_players.get_mut(&player_key) {
                                    if *trigger_value > 0.1 {
                                        let _ = player.command_sender().send(PlaybackCommand::Play);
                                    }
                                    if let Some(frame) =
                                        player.update(std::time::Duration::from_millis(16))
                                    {
                                        if let mapmap_io::format::FrameData::Cpu(data) = &frame.data
                                        {
                                            let tex_name =
                                                format!("part_{}_{}", module.id, part_id);
                                            self.texture_pool.upload_data(
                                                &self.backend.queue,
                                                &tex_name,
                                                data,
                                                frame.format.width,
                                                frame.format.height,
                                            );
                                        }
                                    }
                                }
                            }
                            mapmap_core::SourceCommand::NdiInput {
                                source_name: _source_name,
                                ..
                            } =>
                            {
                                #[cfg(feature = "ndi")]
                                if let Some(src_name) = _source_name {
                                    let receiver =
                                        self.ndi_receivers.entry(*part_id).or_insert_with(|| {
                                            mapmap_io::ndi::NdiReceiver::new()
                                                .expect("Failed to create NDI receiver")
                                        });
                                    if let Ok(Some(frame)) =
                                        receiver.receive(std::time::Duration::from_millis(0))
                                    {
                                        if let mapmap_io::format::FrameData::Cpu(data) = &frame.data
                                        {
                                            let tex_name =
                                                format!("part_{}_{}", module.id, part_id);
                                            self.texture_pool.upload_data(
                                                &self.backend.queue,
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
                                self.hue_controller.update_from_command(
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

                // Global render log (once per second)
                static mut LAST_RENDER_LOG: u64 = 0;
                let now_ms = (timestamp * 1000.0) as u64;
                unsafe {
                    if now_ms / 1000 > LAST_RENDER_LOG {
                        LAST_RENDER_LOG = now_ms / 1000;
                        debug!("=== Render Pipeline Status ===");
                        debug!("  render_ops count: {}", self.render_ops.len());
                        for (i, (mid, op)) in self.render_ops.iter().enumerate() {
                            debug!(
                                "  Op[{}]: mod={} source_part_id={:?}, output={:?}",
                                i, mid, op.source_part_id, op.output_type
                            );
                        }
                    }
                }

                // 2. Update Output Assignments for Preview/Window Mapping
                self.output_assignments.clear();
                for (mid, op) in &self.render_ops {
                    if let mapmap_core::module::OutputType::Projector { id, .. } = &op.output_type {
                        if let Some(source_id) = op.source_part_id {
                            let tex_name = format!("part_{}_{}", mid, source_id);
                            // Insert both for UI panel and window manager
                            self.output_assignments
                                .entry(*id)
                                .or_default()
                                .push(tex_name.clone());
                        }
                    }
                }

                // 3. Sync output windows with evaluation result
                let render_ops_temp = std::mem::take(&mut self.render_ops);
                let ops_only: Vec<mapmap_core::module_eval::RenderOp> =
                    render_ops_temp.iter().map(|(_, op)| op.clone()).collect();
                if let Err(e) = self.sync_output_windows(
                    elwt,
                    &ops_only,
                    self.ui_state.module_canvas.active_module_id,
                ) {
                    error!("Failed to sync output windows: {}", e);
                }
                self.render_ops = render_ops_temp;

                // Update BPM in toolbar
                self.ui_state.current_bpm = analysis_v2.tempo_bpm;

                // Update module canvas with audio trigger data
                #[cfg(feature = "midi")]
                self.ui_state
                    .module_canvas
                    .set_audio_data(AudioTriggerData {
                        band_energies: analysis_v2.band_energies,
                        rms_volume: analysis_v2.rms_volume,
                        peak_volume: analysis_v2.peak_volume,
                        beat_detected: analysis_v2.beat_detected,
                        beat_strength: analysis_v2.beat_strength,
                        bpm: analysis_v2.tempo_bpm,
                    });

                // Convert V2 analysis to legacy format for UI compatibility
                // analyzer_v2 produces 9 bands, legacy AudioAnalysis expects 7.
                // ⚡ Bolt: Moved vectors instead of cloning to reduce allocations
                let legacy_analysis = mapmap_core::audio::AudioAnalysis {
                    timestamp: analysis_v2.timestamp,
                    fft_magnitudes: analysis_v2.fft_magnitudes, // Move
                    band_energies: [
                        analysis_v2.band_energies[0], // SubBass
                        analysis_v2.band_energies[1], // Bass
                        analysis_v2.band_energies[2], // LowMid
                        analysis_v2.band_energies[3], // Mid
                        analysis_v2.band_energies[4], // HighMid
                        analysis_v2.band_energies[6], // Presence (V2 Presence)
                        analysis_v2.band_energies[8], // Brilliance (V2 Air)
                    ],
                    rms_volume: analysis_v2.rms_volume,
                    peak_volume: analysis_v2.peak_volume,
                    beat_detected: analysis_v2.beat_detected,
                    beat_strength: analysis_v2.beat_strength,
                    onset_detected: false, // Not implemented in V2 yet
                    tempo_bpm: analysis_v2.tempo_bpm,
                    waveform: analysis_v2.waveform, // Move
                };

                self.ui_state.dashboard.set_audio_analysis(legacy_analysis);

                // Update Effect Automation
                // Redraw all windows - Optimized to avoid allocation
                for window_context in self.window_manager.iter() {
                    window_context.window.request_redraw();
                }

                // Also ensure egui updates for previews
                self.egui_context.request_repaint();
            }
            _ => (),
        }

        // Process UI actions
        let actions = self.ui_state.take_actions();
        for action in actions {
            match action {
                mapmap_ui::UIAction::SaveProjectAs => {
                    if let Some(path) = FileDialog::new()
                        .add_filter("MapFlow Project", &["mflow", "mapmap", "ron", "json"])
                        .set_file_name("project.mflow")
                        .save_file()
                    {
                        if let Err(e) = save_project(&self.state, &path) {
                            error!("Failed to save project: {}", e);
                        } else {
                            info!("Project saved to {:?}", path);
                        }
                    }
                }
                mapmap_ui::UIAction::SaveProject(path_str) => {
                    let path = if path_str.is_empty() {
                        if let Some(path) = FileDialog::new()
                            .add_filter("MapFlow Project", &["mflow", "mapmap", "ron", "json"])
                            .set_file_name("project.mflow")
                            .save_file()
                        {
                            path
                        } else {
                            // Cancelled
                            PathBuf::new()
                        }
                    } else {
                        PathBuf::from(path_str)
                    };

                    if !path.as_os_str().is_empty() {
                        if let Err(e) = save_project(&self.state, &path) {
                            error!("Failed to save project: {}", e);
                        } else {
                            info!("Project saved to {:?}", path);
                        }
                    }
                }
                mapmap_ui::UIAction::PickMediaFile(module_id, part_id, path_str) => {
                    self.ui_state.module_canvas.active_module_id = Some(module_id);
                    self.ui_state.module_canvas.editing_part_id = Some(part_id);
                    if !path_str.is_empty() {
                        let _ = self.action_sender.send(McpAction::SetModuleSourcePath(
                            module_id,
                            part_id,
                            std::path::PathBuf::from(path_str),
                        ));
                    } else {
                        let sender = self.action_sender.clone();
                        self.tokio_runtime.spawn(async move {
                            if let Some(handle) = rfd::AsyncFileDialog::new()
                                .add_filter(
                                    "Media",
                                    &[
                                        "mp4", "mov", "avi", "mkv", "webm", "gif", "png", "jpg",
                                        "jpeg",
                                    ],
                                )
                                .pick_file()
                                .await
                            {
                                let path = handle.path().to_path_buf();
                                let _ = sender
                                    .send(McpAction::SetModuleSourcePath(module_id, part_id, path));
                            }
                        });
                    }
                }
                mapmap_ui::UIAction::SetMediaFile(module_id, part_id, path) => {
                    let _ = self.action_sender.send(McpAction::SetModuleSourcePath(
                        module_id,
                        part_id,
                        PathBuf::from(path),
                    ));
                }

                mapmap_ui::UIAction::LoadProject(path_str) => {
                    let path = if path_str.is_empty() {
                        if let Some(path) = FileDialog::new()
                            .add_filter("MapFlow Project", &["mflow", "mapmap", "ron", "json"])
                            .pick_file()
                        {
                            path
                        } else {
                            // Cancelled
                            PathBuf::new()
                        }
                    } else {
                        PathBuf::from(path_str)
                    };

                    if !path.as_os_str().is_empty() {
                        self.load_project_file(&path);
                    }
                }
                mapmap_ui::UIAction::LoadRecentProject(path_str) => {
                    let path = PathBuf::from(path_str);
                    self.load_project_file(&path);
                }
                mapmap_ui::UIAction::SetLanguage(lang_code) => {
                    self.state.settings.language = lang_code.clone();
                    self.state.dirty = true;
                    self.ui_state.i18n.set_locale(&lang_code);
                    info!("Language switched to: {}", lang_code);
                }
                mapmap_ui::UIAction::ToggleModuleCanvas => {
                    self.ui_state.show_module_canvas = !self.ui_state.show_module_canvas;
                }
                mapmap_ui::UIAction::Exit => {
                    info!("Exit requested via menu");
                    self.exit_requested = true;
                }
                mapmap_ui::UIAction::OpenSettings => {
                    info!("Settings requested");
                    self.ui_state.show_settings = true;
                }
                mapmap_ui::UIAction::ToggleControllerOverlay => {
                    self.ui_state.show_controller_overlay = !self.ui_state.show_controller_overlay;
                }
                #[cfg(feature = "ndi")]
                mapmap_ui::UIAction::ConnectNdiSource { part_id, source } => {
                    let receiver = self.ndi_receivers.entry(part_id).or_insert_with(|| {
                        info!("Creating new NdiReceiver for part {}", part_id);
                        mapmap_io::ndi::NdiReceiver::new().expect("Failed to create NDI receiver")
                    });
                    info!(
                        "Connecting part {} to NDI source '{}'",
                        part_id, source.name
                    );
                    if let Err(e) = receiver.connect(&source) {
                        error!("Failed to connect to NDI source: {}", e);
                    }
                }
                mapmap_ui::UIAction::SetMidiAssignment(element_id, target_id) => {
                    #[cfg(feature = "midi")]
                    {
                        use mapmap_ui::config::MidiAssignmentTarget;
                        self.ui_state.user_config.set_midi_assignment(
                            &element_id,
                            MidiAssignmentTarget::MapFlow(target_id.clone()),
                        );
                        tracing::info!(
                            "MIDI Assignment set via Global Learn: {} -> {}",
                            element_id,
                            target_id
                        );
                    }
                    #[cfg(not(feature = "midi"))]
                    {
                        let _ = element_id;
                        let _ = target_id;
                    }
                }
                mapmap_ui::UIAction::RegisterHue => {
                    info!("Linking with Philips Hue Bridge...");
                    let ip = self.ui_state.user_config.hue_config.bridge_ip.clone();
                    if ip.is_empty() {
                        error!("Cannot link: No Bridge IP specified.");
                    } else {
                        match self
                            .tokio_runtime
                            .block_on(self.hue_controller.register(&ip))
                        {
                            Ok(new_config) => {
                                info!("Successfully linked with Hue Bridge!");
                                self.ui_state.user_config.hue_config.username = new_config.username;
                                self.ui_state.user_config.hue_config.client_key =
                                    new_config.client_key;
                                let _ = self.ui_state.user_config.save();
                            }
                            Err(e) => {
                                error!("Failed to link with Hue Bridge: {}", e);
                            }
                        }
                    }
                }
                mapmap_ui::UIAction::FetchHueGroups => {
                    info!("Fetching Hue Entertainment Groups...");
                    let bridge_ip = self.ui_state.user_config.hue_config.bridge_ip.clone();
                    let username = self.ui_state.user_config.hue_config.username.clone();

                    info!(
                        "Bridge IP: '{}', Username: '{}'",
                        bridge_ip,
                        if username.is_empty() {
                            "(empty)"
                        } else {
                            "(set)"
                        }
                    );

                    if bridge_ip.is_empty() || username.is_empty() {
                        error!("Cannot fetch groups: Bridge IP or Username missing");
                    } else {
                        // Construct a temp config to fetch groups
                        let config = mapmap_control::hue::models::HueConfig {
                            bridge_ip: bridge_ip.clone(),
                            username: username.clone(),
                            ..Default::default()
                        };

                        info!("Calling get_entertainment_groups API...");
                        // Blocking call
                        match self.tokio_runtime.block_on(
                            mapmap_control::hue::api::groups::get_entertainment_groups(&config),
                        ) {
                            Ok(groups) => {
                                info!("Successfully fetched {} entertainment groups", groups.len());
                                for g in &groups {
                                    info!("  - Group: id='{}', name='{}'", g.id, g.name);
                                }
                                self.ui_state.available_hue_groups =
                                    groups.into_iter().map(|g| (g.id, g.name)).collect();
                            }
                            Err(e) => {
                                error!("Failed to fetch groups: {:?}", e);
                            }
                        }
                    }
                }
                mapmap_ui::UIAction::ConnectHue => {
                    info!("Connecting to Philips Hue Bridge...");

                    // Sync configuration from UI to Controller
                    let ui_hue = &self.ui_state.user_config.hue_config;
                    let control_hue = mapmap_control::hue::models::HueConfig {
                        bridge_ip: ui_hue.bridge_ip.clone(),
                        username: ui_hue.username.clone(),
                        client_key: ui_hue.client_key.clone(),
                        application_id: String::new(), // TODO: This should be retrieved from bridge during pairing
                        entertainment_group_id: ui_hue.entertainment_area.clone(),
                    };
                    self.hue_controller.update_config(control_hue);

                    if let Err(e) = self.tokio_runtime.block_on(self.hue_controller.connect()) {
                        error!("Failed to connect to Hue Bridge: {}", e);
                    } else {
                        info!("Successfully connected to Hue Bridge");
                    }
                }
                mapmap_ui::UIAction::DisconnectHue => {
                    info!("Disconnecting from Philips Hue Bridge...");
                    self.tokio_runtime
                        .block_on(self.hue_controller.disconnect());
                }
                mapmap_ui::UIAction::DiscoverHueBridges => {
                    info!("Discovering Philips Hue Bridges...");
                    // Discovery is async but meethue.com is usually fast.
                    match self
                        .tokio_runtime
                        .block_on(mapmap_control::hue::api::discovery::discover_bridges())
                    {
                        Ok(bridges) => {
                            info!("Discovered {} bridges", bridges.len());
                            self.ui_state.discovered_hue_bridges = bridges;
                        }
                        Err(e) => {
                            error!("Bridge discovery failed: {}", e);
                        }
                    }
                }
                mapmap_ui::UIAction::SetLayerOpacity(id, opacity) => {
                    if let Some(layer) = self.state.layer_manager.get_layer_mut(id) {
                        layer.opacity = opacity;
                        self.state.dirty = true;
                    }
                }
                mapmap_ui::UIAction::SetLayerBlendMode(id, mode) => {
                    if let Some(layer) = self.state.layer_manager.get_layer_mut(id) {
                        layer.blend_mode = mode;
                        self.state.dirty = true;
                    }
                }
                mapmap_ui::UIAction::SetLayerVisibility(id, visible) => {
                    if let Some(layer) = self.state.layer_manager.get_layer_mut(id) {
                        layer.visible = visible;
                        self.state.dirty = true;
                    }
                }
                mapmap_ui::UIAction::AddLayer => {
                    let count = self.state.layer_manager.len();
                    self.state
                        .layer_manager
                        .create_layer(format!("Layer {}", count + 1));
                    self.state.dirty = true;
                }
                mapmap_ui::UIAction::CreateGroup => {
                    let count = self.state.layer_manager.len();
                    self.state
                        .layer_manager
                        .create_group(format!("Group {}", count + 1));
                    self.state.dirty = true;
                }
                mapmap_ui::UIAction::ReparentLayer(id, parent_id) => {
                    self.state.layer_manager.reparent_layer(id, parent_id);
                    self.state.dirty = true;
                }
                mapmap_ui::UIAction::SwapLayers(id1, id2) => {
                    self.state.layer_manager.swap_layers(id1, id2);
                    self.state.dirty = true;
                }
                mapmap_ui::UIAction::ToggleGroupCollapsed(id) => {
                    if let Some(layer) = self.state.layer_manager.get_layer_mut(id) {
                        layer.collapsed = !layer.collapsed;
                        self.state.dirty = true;
                    }
                }
                mapmap_ui::UIAction::RemoveLayer(id) => {
                    self.state.layer_manager.remove_layer(id);
                    self.state.dirty = true;
                    // Deselect if removed
                    if self.ui_state.selected_layer_id == Some(id) {
                        self.ui_state.selected_layer_id = None;
                    }
                }
                mapmap_ui::UIAction::DuplicateLayer(id) => {
                    if let Some(new_id) = self.state.layer_manager.duplicate_layer(id) {
                        self.ui_state.selected_layer_id = Some(new_id);
                        self.state.dirty = true;
                    }
                }
                mapmap_ui::UIAction::RenameLayer(id, name) => {
                    if self.state.layer_manager.rename_layer(id, name) {
                        self.state.dirty = true;
                    }
                }
                mapmap_ui::UIAction::ToggleLayerSolo(id) => {
                    if let Some(layer) = self.state.layer_manager.get_layer_mut(id) {
                        layer.toggle_solo();
                        self.state.dirty = true;
                    }
                }
                mapmap_ui::UIAction::ToggleLayerBypass(id) => {
                    if let Some(layer) = self.state.layer_manager.get_layer_mut(id) {
                        layer.toggle_bypass();
                        self.state.dirty = true;
                    }
                }
                mapmap_ui::UIAction::EjectAllLayers => {
                    self.state.layer_manager.eject_all();
                    self.state.dirty = true;
                }
                mapmap_ui::UIAction::SetLayerTransform(id, transform) => {
                    if let Some(layer) = self.state.layer_manager.get_layer_mut(id) {
                        layer.transform = transform;
                        self.state.dirty = true;
                    }
                }
                mapmap_ui::UIAction::ApplyResizeMode(id, mode) => {
                    // We need source content size and target size.
                    // For now, let's assume composition size as target.
                    // Source size is trickier without paint info, but we might check paint manager.
                    let target_size = mapmap_core::Vec2::new(
                        self.state.layer_manager.composition.size.0 as f32,
                        self.state.layer_manager.composition.size.1 as f32,
                    );

                    // We need a way to get source size.
                    // Layer has paint_id. PaintManager has paints. Paint has source.
                    let mut source_size = mapmap_core::Vec2::ONE; // Default
                    if let Some(layer) = self.state.layer_manager.get_layer(id) {
                        if let Some(paint_id) = layer.paint_id {
                            if let Some(_paint) = self.state.paint_manager.get_paint(paint_id) {
                                // This requires Paint to have size info or we fetch it from source
                                // For now, let's just use target_size as placeholder or 1920x1080 if not known
                                // TODO: Fetch actual media size
                                source_size = target_size;
                            }
                        }
                    }

                    if let Some(layer) = self.state.layer_manager.get_layer_mut(id) {
                        layer.set_transform_with_resize(mode, source_size, target_size);
                        self.state.dirty = true;
                    }
                }
                mapmap_ui::UIAction::SetMasterOpacity(val) => {
                    self.state.layer_manager.composition.set_master_opacity(val);
                    self.state.dirty = true;
                }
                mapmap_ui::UIAction::SetMasterSpeed(val) => {
                    self.state.layer_manager.composition.set_master_speed(val);
                    self.state.dirty = true;
                }
                mapmap_ui::UIAction::SetCompositionName(name) => {
                    self.state.layer_manager.composition.name = name;
                    self.state.dirty = true;
                }
                // Phase 2 output actions (placeholders for now)
                mapmap_ui::UIAction::AddOutput(..) => {
                    warn!("AddOutput not implemented in main");
                }
                mapmap_ui::UIAction::RemoveOutput(..) => {
                    warn!("RemoveOutput not implemented in main");
                }
                mapmap_ui::UIAction::ConfigureOutput(id, config) => {
                    // Collect sync for fullscreen
                    let fs = config.fullscreen;

                    // Update the target output
                    self.state.output_manager.update_output(id, config.clone());

                    // SYNC Logic: If this is an output node, we might want to sync fullscreen across all projectors
                    // This prevents desync where one projector is FS and another is windowed.
                    let all_ids: Vec<_> = self
                        .state
                        .output_manager
                        .list_outputs()
                        .iter()
                        .map(|o| o.id)
                        .collect();
                    for oid in all_ids {
                        if let Some(other) = self.state.output_manager.get_output_mut(oid) {
                            if other.fullscreen != fs {
                                other.fullscreen = fs;
                                info!("Syncing fullscreen state for output {} -> {}", oid, fs);
                            }
                        }
                    }

                    self.state.dirty = true;
                }

                mapmap_ui::UIAction::MediaCommand(part_id, command) => {
                    self.ui_state
                        .module_canvas
                        .pending_playback_commands
                        .push((part_id, command));
                }
                // Fallback
                _ => {}
            }
        }

        // Poll MCP commands
        while let Ok(action) = self.mcp_receiver.try_recv() {
            match action {
                McpAction::SaveProject(path) => {
                    info!("MCP: Saving project to {:?}", path);
                    if let Err(e) = save_project(&self.state, &path) {
                        error!("MCP: Failed to save project: {}", e);
                    }
                }
                McpAction::LoadProject(path) => {
                    info!("MCP: Loading project from {:?}", path);
                    self.load_project_file(&path);
                }
                McpAction::AddLayer(name) => {
                    info!("MCP: Adding layer '{}'", name);
                    self.state.layer_manager.create_layer(name);
                    self.state.dirty = true;
                }
                McpAction::RemoveLayer(id) => {
                    info!("MCP: Removing layer {}", id);
                    self.state.layer_manager.remove_layer(id);
                    self.state.dirty = true;
                }
                McpAction::TriggerCue(id) => {
                    info!("MCP: Triggering cue {}", id);
                    self.control_manager
                        .execute_action(Action::GotoCue(id as u32));
                }
                McpAction::NextCue => {
                    info!("MCP: Next cue");
                    self.control_manager.execute_action(Action::NextCue);
                }
                McpAction::PrevCue => {
                    info!("MCP: Prev cue");
                    println!("Triggering PrevCue"); // Debug print as per earlier pattern
                    self.control_manager.execute_action(Action::PrevCue);
                }
                McpAction::MediaPlay => {
                    info!("MCP: Media Play");
                    // TODO: Integrate with media player when available
                }
                McpAction::MediaPause => {
                    info!("MCP: Media Pause");
                    // TODO: Integrate with media player when available
                }
                McpAction::MediaStop => {
                    info!("MCP: Media Stop");
                    // TODO: Integrate with media player when available
                }
                McpAction::SetModuleSourcePath(module_id, part_id, path) => {
                    info!(
                        "MCP: Setting source path for part {} in module {} to {:?}",
                        part_id, module_id, path
                    );
                    // Update module part in specific module
                    if let Some(module) = self.state.module_manager.get_module_mut(module_id) {
                        if let Some(part) = module.parts.iter_mut().find(|p| p.id == part_id) {
                            if let mapmap_core::module::ModulePartType::Source(
                                mapmap_core::module::SourceType::MediaFile {
                                    path: ref mut p, ..
                                },
                            ) = &mut part.part_type
                            {
                                *p = path.to_string_lossy().to_string();
                                self.state.dirty = true;
                                // Trigger reload
                                self.ui_state
                                    .module_canvas
                                    .pending_playback_commands
                                    .push((part_id, mapmap_ui::MediaPlaybackCommand::Reload));
                            }
                        }
                    }
                }
                McpAction::SetLayerOpacity(id, opacity) => {
                    info!("MCP: Set layer {} opacity to {}", id, opacity);
                    // TODO: Implement layer opacity update
                }
                McpAction::SetLayerVisibility(id, visible) => {
                    info!("MCP: Set layer {} visibility to {}", id, visible);
                    // TODO: Implement layer visibility update
                }
                _ => {
                    info!("MCP: Unimplemented action received: {:?}", action);
                }
            }
        }

        // Process egui panel actions
        ui::panels::paint::handle_actions(&mut self.ui_state, &mut self.state);

        if let Some(action) = self.ui_state.edge_blend_panel.take_action() {
            match action {
                EdgeBlendAction::UpdateEdgeBlend(id, values) => {
                    if let Some(output) = self.state.output_manager.get_output_mut(id) {
                        output.edge_blend.left.enabled = values.left_enabled;
                        output.edge_blend.left.width = values.left_width;
                        output.edge_blend.left.offset = values.left_offset;
                        output.edge_blend.right.enabled = values.right_enabled;
                        output.edge_blend.right.width = values.right_width;
                        output.edge_blend.right.offset = values.right_offset;
                        output.edge_blend.top.enabled = values.top_enabled;
                        output.edge_blend.top.width = values.top_width;
                        output.edge_blend.top.offset = values.top_offset;
                        output.edge_blend.bottom.enabled = values.bottom_enabled;
                        output.edge_blend.bottom.width = values.bottom_width;
                        output.edge_blend.bottom.offset = values.bottom_offset;
                        output.edge_blend.gamma = values.gamma;
                        self.state.dirty = true;
                    }
                }
                EdgeBlendAction::UpdateColorCalibration(id, values) => {
                    if let Some(output) = self.state.output_manager.get_output_mut(id) {
                        output.color_calibration.brightness = values.brightness;
                        output.color_calibration.contrast = values.contrast;
                        output.color_calibration.gamma.x = values.gamma_r;
                        output.color_calibration.gamma.y = values.gamma_g;
                        output.color_calibration.gamma_b = values.gamma_b;
                        output.color_calibration.saturation = values.saturation;
                        output.color_calibration.color_temp = values.color_temp;
                        self.state.dirty = true;
                    }
                }
                EdgeBlendAction::ResetEdgeBlend(id) => {
                    if let Some(output) = self.state.output_manager.get_output_mut(id) {
                        output.edge_blend = Default::default();
                        self.state.dirty = true;
                    }
                }
                EdgeBlendAction::ResetColorCalibration(id) => {
                    if let Some(output) = self.state.output_manager.get_output_mut(id) {
                        output.color_calibration = Default::default();
                        self.state.dirty = true;
                    }
                }
            }
        }

        // Request redraw for all windows to ensure continuous rendering
        for window_context in self.window_manager.iter() {
            window_context.window.request_redraw();
        }

        Ok(())
    }

    /// Helper to load a project file and update state
    fn load_project_file(&mut self, path: &PathBuf) {
        match load_project(path) {
            Ok(new_state) => {
                self.state = new_state;
                // Sync language to UI
                self.ui_state.i18n.set_locale(&self.state.settings.language);

                info!("Project loaded from {:?}", path);

                // Add to recent files
                if let Some(path_str) = path.to_str() {
                    let p = path_str.to_string();
                    // Remove if exists to move to top
                    if let Some(pos) = self.ui_state.recent_files.iter().position(|x| x == &p) {
                        self.ui_state.recent_files.remove(pos);
                    }
                    self.ui_state.recent_files.insert(0, p.clone());
                    // Limit to 10
                    if self.ui_state.recent_files.len() > 10 {
                        self.ui_state.recent_files.pop();
                    }
                    // Persist to user config
                    self.ui_state.user_config.add_recent_file(&p);
                }
            }
            Err(e) => error!("Failed to load project: {}", e),
        }
    }

    /// Synchronizes output windows with the current module evaluation result.
    ///
    /// Creates windows for new output assignments and removes windows that are no longer needed.
    /// Synchronizes output windows and NDI senders with the current module graph output nodes.
    fn sync_output_windows(
        &mut self,
        elwt: &winit::event_loop::ActiveEventLoop,
        _render_ops: &[mapmap_core::module_eval::RenderOp],
        _active_module_id: Option<mapmap_core::module::ModuleId>,
    ) -> Result<()> {
        use mapmap_core::module::OutputType;
        const PREVIEW_FLAG: u64 = 1u64 << 63;

        // Track active IDs for cleanup
        let mut active_window_ids = std::collections::HashSet::new();
        let mut active_sender_ids = std::collections::HashSet::new();
        let global_fullscreen = self.ui_state.user_config.global_fullscreen;

        // 1. Iterate over ALL modules to collect required outputs
        for module in self.state.module_manager.list_modules() {
            if let Some(module_ref) = self.state.module_manager.get_module(module.id) {
                for part in &module_ref.parts {
                    if let mapmap_core::module::ModulePartType::Output(output_type) =
                        &part.part_type
                    {
                        // Use part.id for consistency with render pipeline
                        let output_id = part.id;

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

                                if let Some(window_context) = self.window_manager.get(window_id) {
                                    // Update existing
                                    let is_fullscreen =
                                        window_context.window.fullscreen().is_some();
                                    if is_fullscreen != global_fullscreen {
                                        info!(
                                            "Toggling fullscreen for window {}: {}",
                                            window_id, global_fullscreen
                                        );
                                        window_context.window.set_fullscreen(
                                            if global_fullscreen {
                                                Some(winit::window::Fullscreen::Borderless(None))
                                            } else {
                                                None
                                            },
                                        );
                                    }
                                    window_context.window.set_cursor_visible(!*hide_cursor);
                                } else {
                                    // Create new
                                    self.window_manager.create_projector_window(
                                        elwt,
                                        &self.backend,
                                        window_id,
                                        name,
                                        global_fullscreen,
                                        *hide_cursor,
                                        *target_screen,
                                        self.ui_state.user_config.vsync_mode,
                                    )?;
                                    info!(
                                        "Created projector window for output {} (Part {})",
                                        window_id, output_id
                                    );
                                }

                                // 2. Extra Preview Window
                                if *extra_preview_window {
                                    let preview_id = window_id | PREVIEW_FLAG;
                                    active_window_ids.insert(preview_id);

                                    if self.window_manager.get(preview_id).is_none() {
                                        self.window_manager.create_projector_window(
                                            elwt,
                                            &self.backend,
                                            preview_id,
                                            &format!("Preview: {}", name),
                                            false, // Always windowed
                                            false, // Show cursor
                                            0,     // Default screen (0)
                                            self.ui_state.user_config.vsync_mode,
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
                                    if !self.ndi_senders.contains_key(&output_id) {
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
                                                info!("Created NDI sender: {}", _name);
                                                self.ndi_senders.insert(output_id, sender);
                                            }
                                            Err(e) => error!(
                                                "Failed to create NDI sender {}: {}",
                                                _name, e
                                            ),
                                        }
                                    }
                                }
                            }
                            #[cfg(target_os = "windows")]
                            OutputType::Spout { .. } => {
                                // TODO: Spout Sender
                            }
                            OutputType::Hue { .. } => {
                                // Hue integration handled via separate controller, no window needed
                            }
                        }
                    }
                }
            }
        }

        // 2. Cleanup Windows (only close if projector was removed from graph)
        let window_ids: Vec<u64> = self.window_manager.window_ids().cloned().collect();
        for id in window_ids {
            if id != 0 && !active_window_ids.contains(&id) {
                self.window_manager.remove_window(id);
                if (id & PREVIEW_FLAG) != 0 {
                    self.output_assignments.remove(&id);
                }
                info!("Closed window {}", id);
            }
        }

        // 3. Cleanup NDI Senders
        #[cfg(feature = "ndi")]
        {
            let sender_ids: Vec<u64> = self.ndi_senders.keys().cloned().collect();
            for id in sender_ids {
                if !active_sender_ids.contains(&id) {
                    self.ndi_senders.remove(&id);
                    info!("Removed NDI sender {}", id);
                }
            }
        }

        Ok(())
    }

    // Process pending MCP actions (e.g. from UI or external clients)

    /// Global update loop (physics/logic), independent of render rate per window.
    fn update(&mut self, elwt: &winit::event_loop::ActiveEventLoop, dt: f32) {
        if let Err(e) = crate::app::loops::logic::update(self, elwt, dt) {
            tracing::error!("Update error: {}", e);
        }
    }
    /// Handle global UI actions
    /// Handle Node Editor actions
    fn render(&mut self, output_id: OutputId) -> Result<()> {
        #[cfg(feature = "midi")]
        return crate::app::loops::render::render(self, output_id);
        #[cfg(not(feature = "midi"))]
        Ok(())
    }
}

mod logging_setup;

/// The main entry point for the application.
fn main() -> Result<()> {
    // Initialize logging with default configuration.
    // This creates a log file in logs/ and outputs to console
    let _log_guard = logging_setup::init(&mapmap_core::logging::LogConfig::default())?;

    info!("==========================================");
    info!("===      MapFlow Session Started       ===");
    info!("==========================================");

    // Start the application loop
    let event_loop = EventLoop::new().unwrap();
    let mut app: Option<App> = None;

    #[allow(deprecated)]
    event_loop.run(move |event, elwt| {
        if app.is_none() {
            app = Some(pollster::block_on(App::new(elwt)).expect("Failed to create App"));
            info!("--- Entering Main Event Loop ---");
        }

        if let Some(app_ref) = &mut app {
            if let Err(e) = app_ref.handle_event(event, elwt) {
                error!("Application error: {}", e);
                elwt.exit();
            }
        }
    })?;

    Ok(())
}
// Force CI trigger
