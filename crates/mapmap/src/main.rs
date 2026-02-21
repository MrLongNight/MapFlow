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

use mapmap_core::OutputId;
use mapmap_media::PlaybackCommand;
use mapmap_ui::types::MediaPlaybackCommand;

use tracing::{error, info, warn};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::WindowId;

use crate::app::core::app_struct::App;

struct MapFlowApp {
    app: Option<App>,
}

impl ApplicationHandler for MapFlowApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.app.is_none() {
            info!("Initializing MapFlow...");
            self.app = Some(
                pollster::block_on(App::new(event_loop)).expect("Failed to initialize application"),
            );
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(app) = &mut self.app {
            let _ = app.handle_event(
                winit::event::Event::WindowEvent { window_id, event },
                event_loop,
            );
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(app) = &mut self.app {
            let _ = app.handle_event(winit::event::Event::AboutToWait, event_loop);
        }
    }
}

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
                    _ => (),
                }
            }
            winit::event::Event::AboutToWait => {
                // Logic update at a fixed rate (e.g. 60Hz)
                let now = std::time::Instant::now();
                let dt = now.duration_since(self.last_update).as_secs_f32();

                // Cap dt to avoid huge jumps
                let dt = dt.min(0.1);

                if dt >= 1.0 / 60.0 {
                    if let Err(e) = self.update(elwt, dt) {
                        error!("Update error: {}", e);
                    }
                    self.last_update = now;
                }

                // Request redraw for all windows
                for context in self.window_manager.iter() {
                    context.window.request_redraw();
                }
            }
            _ => (),
        }

        Ok(())
    }

    /// Global render update
    pub fn render(&mut self, output_id: OutputId) -> Result<()> {
        // Run modularized render loop
        crate::app::loops::render::render(self, output_id)
    }

    /// Global logic update
    pub fn update(&mut self, elwt: &winit::event_loop::ActiveEventLoop, dt: f32) -> Result<()> {
        // Run modularized update loop
        crate::app::loops::logic::update(self, elwt, dt)?;

        // Special handling for MediaPlaybackCommands from UI
        // (These are consumed here to avoid complex cross-crate dependencies in logic.rs)
        let commands = self.ui_state.module_canvas.take_playback_commands();
        for (part_id, cmd) in commands {
            // Find module owner
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
                    && cmd != MediaPlaybackCommand::Reload
                {
                    info!(
                        "Player doesn't exist for part_id={}, attempting to create...",
                        part_id
                    );
                    // Find the source path
                    if let Some(module) = self.state.module_manager.get_module(mod_id) {
                        if let Some(part) = module.parts.iter().find(|p| p.id == part_id) {
                            if let mapmap_core::module::ModulePartType::Source(
                                mapmap_core::module::SourceType::MediaFile { ref path, .. },
                            ) = &part.part_type
                            {
                                info!("Found media path: '{}' in module '{}'", path, module.name);
                                if !path.is_empty() {
                                    match mapmap_media::open_path(path) {
                                        Ok(player) => {
                                            info!("Successfully created player for '{}'", path);
                                            self.media_players
                                                .insert(player_key, (path.clone(), player));
                                        }
                                        Err(e) => {
                                            error!("Failed to load media '{}': {}", path, e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some((_, player)) = self.media_players.get_mut(&player_key) {
                    match cmd {
                        MediaPlaybackCommand::Play => {
                            let _ = player.command_sender().send(PlaybackCommand::Play);
                        }
                        MediaPlaybackCommand::Pause => {
                            let _ = player.command_sender().send(PlaybackCommand::Pause);
                        }
                        MediaPlaybackCommand::Stop => {
                            let _ = player.command_sender().send(PlaybackCommand::Stop);
                        }
                        MediaPlaybackCommand::Reload => {
                            info!("Reloading media player for part_id={}", part_id);
                            // Player removal handled below
                        }
                        MediaPlaybackCommand::SetSpeed(speed) => {
                            info!("Setting speed to {} for part_id={}", speed, part_id);
                            let _ = handle_media_speed(player, speed);
                        }
                        MediaPlaybackCommand::SetLoop(enabled) => {
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
                        MediaPlaybackCommand::Seek(position) => {
                            info!("Seeking to {} for part_id={}", position, part_id);
                            let _ = player.command_sender().send(PlaybackCommand::Seek(
                                std::time::Duration::from_secs_f64(position),
                            ));
                        }
                        MediaPlaybackCommand::SetReverse(reverse) => {
                            info!(
                                "Setting reverse playback to {} for part_id={} (NOT IMPLEMENTED)",
                                reverse, part_id
                            );
                        }
                    }
                }

                // Handle Reload by removing player and immediately recreating
                if cmd == MediaPlaybackCommand::Reload {
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
                                mapmap_core::module::SourceType::MediaFile { ref path, .. },
                            ) = &part.part_type
                            {
                                if !path.is_empty() {
                                    match mapmap_media::open_path(path) {
                                        Ok(player) => {
                                            info!("Recreated player for '{}' after reload", path);
                                            // Auto-play after reload
                                            let _ =
                                                player.command_sender().send(PlaybackCommand::Play);
                                            self.media_players
                                                .insert(player_key, (path.clone(), player));
                                        }
                                        Err(e) => {
                                            error!("Failed to reload media '{}': {}", path, e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

fn handle_media_speed(player: &mut mapmap_media::player::VideoPlayer, speed: f32) -> Result<()> {
    player
        .command_sender()
        .send(PlaybackCommand::SetSpeed(speed))
        .map_err(|e| anyhow::anyhow!("Failed to send speed command: {}", e))
}

fn main() -> Result<()> {
    // Initialize logging
    let file_appender = tracing_appender::rolling::daily("logs", "mapflow.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::io::stdout) // Print to stdout
        .with_writer(non_blocking) // Also print to file
        .init();

    info!("Starting MapFlow...");

    let event_loop = EventLoop::new()?;
    let mut app_handler = MapFlowApp { app: None };

    event_loop.run_app(&mut app_handler)?;

    Ok(())
}
