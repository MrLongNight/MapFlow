//! MapFlow - Open source Vj Projection Mapping Software
//!
//! This is the main application crate for MapFlow.
//! VERSION: 2026-02-21-FIX-WINIT-RUN-APP-V2

#![warn(missing_docs)]

pub mod app;
/// UI components.
/// CLI arguments parsing and types.
pub mod cli;
mod media_manager_ui;
/// Orchestration and node evaluation logic.
pub mod orchestration;
/// Player modes.
pub mod player;
/// UI components.
pub mod ui;
mod window_manager;

use anyhow::Result;

use mapmap_core::OutputId;
use mapmap_media::PlaybackCommand;
use mapmap_ui::types::MediaPlaybackCommand;

use tracing::{error, info};
use tracing_subscriber::prelude::*;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::WindowId;

use crate::app::core::app_struct::App;

use crate::cli::{CliArgs, Mode};
use clap::Parser;

struct MapFlowApp {
    app: Option<App>,
    is_automation: bool,
    fixture: Option<String>,
    exit_after_frames: Option<u64>,
    screenshot_dir: Option<String>,
}

impl ApplicationHandler for MapFlowApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.app.is_none() {
            info!("Initializing MapFlow...");
            let mut app = pollster::block_on(App::new(event_loop, self.is_automation))
                .expect("Failed to initialize application");

            // Automation mode: load fixture if specified
            if self.is_automation {
                if let Some(fixture_path) = &self.fixture {
                    info!("Automation mode: Loading fixture {}", fixture_path);
                    match mapmap_io::load_project(std::path::Path::new(fixture_path)) {
                        Ok(loaded_state) => {
                            app.state = loaded_state;
                            app.state.dirty = true;
                            info!("Fixture loaded successfully.");
                        }
                        Err(e) => {
                            error!("Automation mode: Failed to load fixture: {}", e);
                            event_loop.exit();
                        }
                    }
                }
            }

            self.app = Some(app);
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

            if self.is_automation {
                if let Some(exit_frames) = self.exit_after_frames {
                    if app.frame_counter >= exit_frames {
                        if let Some(dir) = &self.screenshot_dir {
                            let path = std::path::PathBuf::from(dir);
                            std::fs::create_dir_all(&path).unwrap_or_else(|e| {
                                error!("Failed to create screenshot directory: {}", e);
                            });

                            let file_path =
                                path.join(format!("automation_frame_{}.png", exit_frames));
                            info!("Automation mode: Saving screenshot to {:?}", file_path);

                            // Trigger capture
                            let main_window_context = app.window_manager.get(0).unwrap();
                            let format = main_window_context.surface_config.format;
                            let width = main_window_context.surface_config.width;
                            let height = main_window_context.surface_config.height;

                            let mut encoder = app.backend.device.create_command_encoder(
                                &wgpu::CommandEncoderDescriptor {
                                    label: Some("Automation Screenshot Encoder"),
                                },
                            );

                            let texture = app
                                .texture_pool
                                .get_texture("composite")
                                .expect("Could not find composite texture for automation capture");

                            let bytes_per_pixel = 4;
                            let unpadded_bytes_per_row = width * bytes_per_pixel;
                            let padded_bytes_per_row = unpadded_bytes_per_row
                                .div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
                                * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

                            let buffer =
                                app.backend.device.create_buffer(&wgpu::BufferDescriptor {
                                    label: Some("Automation Readback Buffer"),
                                    size: (padded_bytes_per_row * height) as u64,
                                    usage: wgpu::BufferUsages::COPY_DST
                                        | wgpu::BufferUsages::MAP_READ,
                                    mapped_at_creation: false,
                                });

                            encoder.copy_texture_to_buffer(
                                wgpu::TexelCopyTextureInfo {
                                    texture: &texture,
                                    mip_level: 0,
                                    origin: wgpu::Origin3d::ZERO,
                                    aspect: wgpu::TextureAspect::All,
                                },
                                wgpu::TexelCopyBufferInfo {
                                    buffer: &buffer,
                                    layout: wgpu::TexelCopyBufferLayout {
                                        offset: 0,
                                        bytes_per_row: Some(padded_bytes_per_row),
                                        rows_per_image: Some(height),
                                    },
                                },
                                wgpu::Extent3d {
                                    width,
                                    height,
                                    depth_or_array_layers: 1,
                                },
                            );

                            app.backend.queue.submit(std::iter::once(encoder.finish()));

                            let slice = buffer.slice(..);
                            slice.map_async(wgpu::MapMode::Read, |_| {});

                            app.backend
                                .device
                                .poll(wgpu::PollType::Wait {
                                    submission_index: None,
                                    timeout: None,
                                })
                                .unwrap();

                            let mapped = slice.get_mapped_range();
                            let mut rgba = Vec::with_capacity((width * height * 4) as usize);

                            for row in mapped
                                .chunks_exact(padded_bytes_per_row as usize)
                                .take(height as usize)
                            {
                                for pixel in row[..(width * 4) as usize].chunks_exact(4) {
                                    match format {
                                        wgpu::TextureFormat::Bgra8Unorm
                                        | wgpu::TextureFormat::Bgra8UnormSrgb => {
                                            rgba.extend_from_slice(&[
                                                pixel[2], pixel[1], pixel[0], pixel[3],
                                            ]);
                                        }
                                        _ => rgba.extend_from_slice(pixel),
                                    }
                                }
                            }
                            drop(mapped);
                            buffer.unmap();

                            let img = image::RgbaImage::from_raw(width, height, rgba).unwrap();
                            img.save(&file_path).unwrap();
                        }

                        info!(
                            "Automation mode: Reached frame limit ({}). Exiting.",
                            exit_frames
                        );
                        event_loop.exit();
                    }
                }
            }
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

                let has_modules = !self.state.module_manager.modules().is_empty();
                let has_projector_outputs =
                    self.state.module_manager.modules().iter().any(|module| {
                        module.parts.iter().any(|part| {
                            matches!(
                                part.part_type,
                                mapmap_core::module::ModulePartType::Output(
                                    mapmap_core::module::OutputType::Projector { .. }
                                )
                            )
                        })
                    });
                let is_playing = self.state.effect_animator.is_playing();
                let configured_fps = self.ui_state.target_fps.max(1.0);
                let tick_fps = if !has_modules {
                    10.0
                } else if is_playing {
                    configured_fps
                } else {
                    // Editing/idle mode with modules loaded: lower tick rate to reduce CPU.
                    // Increased from 15 to 30 to avoid 'lagginess' reported by users.
                    configured_fps.min(30.0)
                };
                let target_interval = 1.0 / tick_fps;

                if dt >= target_interval {
                    if let Err(e) = self.update(elwt, dt) {
                        error!("Update error: {}", e);
                    }
                    self.last_update = now;

                    // Avoid expensive continuous redraws while idle.
                    // - During playback: redraw all windows (main + projector outputs)
                    // - With projector outputs present: keep them live while editing so media
                    //   sources do not freeze on stale frames.
                    if has_modules {
                        if is_playing || has_projector_outputs {
                            self.window_manager.request_redraw_all();
                        } else if let Some(main_window) = self.window_manager.get(0) {
                            main_window.window.request_redraw();
                        }
                    }

                    elwt.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(
                        self.last_update + std::time::Duration::from_secs_f32(target_interval),
                    ));
                } else {
                    // Wait until the next frame is due
                    let wait_until =
                        self.last_update + std::time::Duration::from_secs_f32(target_interval);
                    elwt.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(wait_until));
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

                if cmd == MediaPlaybackCommand::Reload {
                    if self.media_players.remove(&player_key).is_some() {
                        info!(
                            "Removed old media player for part_id={} for reload",
                            part_id
                        );
                    }
                    self.texture_pool
                        .release(&format!("part_{}_{}", mod_id, part_id));
                    crate::orchestration::media::sync_media_players(self);
                    continue;
                }

                if !self.media_players.contains_key(&player_key) {
                    crate::orchestration::media::sync_media_players(self);
                }

                if let Some(player) = self.media_players.get_mut(&player_key) {
                    match cmd {
                        MediaPlaybackCommand::Play => {
                            let _ = player.command_tx.send(PlaybackCommand::Play);
                        }
                        MediaPlaybackCommand::Pause => {
                            let _ = player.command_tx.send(PlaybackCommand::Pause);
                        }
                        MediaPlaybackCommand::Stop => {
                            let _ = player.command_tx.send(PlaybackCommand::Stop);
                        }
                        MediaPlaybackCommand::Reload => unreachable!(
                            "MediaPlaybackCommand::Reload is handled before player dispatch"
                        ),
                        MediaPlaybackCommand::SetSpeed(speed) => {
                            info!("Setting speed to {} for part_id={}", speed, part_id);
                            let _ = player.command_tx.send(PlaybackCommand::SetSpeed(speed));
                        }
                        MediaPlaybackCommand::SetLoop(enabled) => {
                            info!("Setting loop to {} for part_id={}", enabled, part_id);
                            let mode = if enabled {
                                mapmap_media::LoopMode::Loop
                            } else {
                                mapmap_media::LoopMode::PlayOnce
                            };
                            let _ = player.command_tx.send(PlaybackCommand::SetLoopMode(mode));
                        }
                        MediaPlaybackCommand::Seek(position) => {
                            info!("Seeking to {} for part_id={}", position, part_id);
                            let _ = player.command_tx.send(PlaybackCommand::Seek(
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
                } else {
                    error!(
                        "Fehler in Videoausgabe: Befehl {:?} fuer Modul {} / Part {} konnte nicht ausgefuehrt werden, weil kein MediaPlayer aktiv ist.",
                        cmd, mod_id, part_id
                    );
                }
            }
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let args = CliArgs::parse();
    let initial_user_config = mapmap_ui::config::UserConfig::load();
    let configured_log_level = match initial_user_config.log_level {
        mapmap_ui::config::AppLogLevel::Info => tracing::Level::INFO,
        mapmap_ui::config::AppLogLevel::Debug => tracing::Level::DEBUG,
    };

    // Initialize logging
    let file_appender = tracing_appender::rolling::daily("logs", "mapflow.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Filter configuration
    let env_filter = tracing_subscriber::EnvFilter::from_default_env()
        .add_directive(configured_log_level.into())
        // Noise reduction from external crates
        .add_directive("wgpu_core=warn".parse().unwrap())
        .add_directive("wgpu_hal=warn".parse().unwrap())
        .add_directive("naga=warn".parse().unwrap())
        .add_directive("winit=info".parse().unwrap());

    // Layer for Console output (pretty and colored)
    let console_layer = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_writer(std::io::stdout);

    // Layer for File output (clean and structured)
    let file_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_target(true)
        .with_writer(non_blocking);

    // Combine everything
    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    info!("Starting MapFlow in {:?} mode...", args.mode);
    info!(
        "Configured log level from user settings: {}",
        initial_user_config.log_level
    );

    match args.mode {
        Mode::Editor => run_editor()?,
        Mode::Automation => run_automation(&args)?,
        Mode::PlayerNdi => run_player_ndi(&args)?,
        Mode::PlayerDist => run_player_dist(&args)?,
        Mode::PlayerLegacy => run_player_legacy(&args)?,
        Mode::PlayerPi => run_player_pi(&args)?,
    }

    Ok(())
}

fn run_editor() -> Result<()> {
    info!("Starting Editor mode...");
    let event_loop = EventLoop::new()?;
    let mut app_handler = MapFlowApp {
        app: None,
        is_automation: false,
        fixture: None,
        exit_after_frames: None,
        screenshot_dir: None,
    };

    event_loop.run_app(&mut app_handler)?;

    Ok(())
}

fn run_automation(args: &CliArgs) -> Result<()> {
    info!("Starting Automation mode...");
    let event_loop = EventLoop::new()?;
    let mut app_handler = MapFlowApp {
        app: None,
        is_automation: true,
        fixture: args.fixture.clone(),
        exit_after_frames: args.exit_after_frames,
        screenshot_dir: args.screenshot_dir.clone(),
    };

    event_loop.run_app(&mut app_handler)?;

    Ok(())
}

fn run_player_ndi(args: &CliArgs) -> Result<()> {
    crate::player::ndi_player::run(args)
}

fn run_player_dist(_args: &CliArgs) -> Result<()> {
    info!("Starting Distributed Player mode...");
    Ok(())
}

fn run_player_legacy(_args: &CliArgs) -> Result<()> {
    info!("Starting Legacy RTSP/H.264 Player mode...");
    Ok(())
}

fn run_player_pi(_args: &CliArgs) -> Result<()> {
    info!("Starting Raspberry Pi Player mode...");
    Ok(())
}
