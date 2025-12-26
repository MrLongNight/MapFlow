//! MapFlow - Open source Vj Projection Mapping Software
//!
//! This is the main application crate for MapFlow.

#![warn(missing_docs)]

mod window_manager;

use anyhow::Result;
use egui_wgpu::Renderer;
use egui_winit::State;
#[cfg(feature = "midi")]
use mapmap_control::midi::{MidiInputHandler, MidiMessage};
use mapmap_control::{shortcuts::Action, ControlManager};
use mapmap_core::{
    audio::{backend::cpal_backend::CpalBackend, backend::AudioBackend, AudioAnalyzer},
    AppState, OutputId,
};

use mapmap_mcp::{McpAction, McpServer};
// Define McpAction locally or import if we move it to core later -> Removed local definition

use crossbeam_channel::{unbounded, Receiver};
use mapmap_io::{load_project, save_project};
use mapmap_render::{
    Compositor, EffectChainRenderer, MeshRenderer, OscillatorRenderer, QuadRenderer, TexturePool,
    WgpuBackend,
};
use mapmap_ui::{menu_bar, AppUI, EdgeBlendAction};
use rfd::FileDialog;
use std::path::PathBuf;
use std::thread;
use tracing::{error, info};
use window_manager::WindowManager;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};

/// The main application state.
struct App {
    /// Manages all application windows.
    window_manager: WindowManager,

    /// The UI state.
    ui_state: AppUI,
    /// The application's render backend.
    backend: WgpuBackend,
    /// Texture pool for intermediate textures.
    texture_pool: TexturePool,
    /// The main compositor.
    compositor: Compositor,
    /// The effect chain renderer.
    effect_chain_renderer: EffectChainRenderer,
    /// The mesh renderer.
    #[allow(dead_code)]
    mesh_renderer: MeshRenderer,
    /// Quad renderer for passthrough.
    quad_renderer: QuadRenderer,
    /// Final composite texture before output processing.
    composite_texture: String,
    /// Ping-pong textures for layer composition.
    layer_ping_pong: [String; 2],
    /// The application state (project data).
    state: AppState,
    /// The audio backend.
    audio_backend: Option<CpalBackend>,
    /// The audio analyzer.
    audio_analyzer: AudioAnalyzer,
    /// List of available audio devices.
    audio_devices: Vec<String>,
    /// The egui context.
    egui_context: egui::Context,
    /// The egui state.
    egui_state: State,
    /// The egui renderer.
    egui_renderer: Renderer,
    /// Last autosave timestamp.
    last_autosave: std::time::Instant,
    /// Last update timestamp for delta time calculation.
    last_update: std::time::Instant,
    /// Application start time.
    start_time: std::time::Instant,
    /// Receiver for MCP commands
    mcp_receiver: Receiver<McpAction>,
    /// Unified control manager
    control_manager: ControlManager,
    /// Flag to track if exit was requested
    exit_requested: bool,
    /// The oscillator distortion renderer.
    oscillator_renderer: Option<OscillatorRenderer>,
    /// A dummy texture used as input for effects when no other source is available.
    dummy_texture: Option<wgpu::Texture>,
    /// A view of the dummy texture.
    dummy_view: Option<wgpu::TextureView>,
    /// FPS calculation: accumulated frame times
    fps_samples: Vec<f32>,
    /// Current calculated FPS
    current_fps: f32,
    /// Current frame time in ms
    current_frame_time_ms: f32,
    /// System info for CPU/RAM monitoring
    sys_info: sysinfo::System,
    /// Last system refresh time
    last_sysinfo_refresh: std::time::Instant,
    /// MIDI Input Handler
    #[cfg(feature = "midi")]
    midi_handler: Option<MidiInputHandler>,
}

impl App {
    /// Creates a new `App`.
    async fn new(event_loop: &EventLoop<()>) -> Result<Self> {
        let backend = WgpuBackend::new().await?;

        // Initialize renderers
        let texture_pool = TexturePool::new(backend.device.clone());
        let compositor = Compositor::new(backend.device.clone(), backend.surface_format())?;
        let effect_chain_renderer = EffectChainRenderer::new(
            backend.device.clone(),
            backend.queue.clone(),
            backend.surface_format(),
        )?;
        let mesh_renderer = MeshRenderer::new(backend.device.clone(), backend.surface_format())?;
        let quad_renderer = QuadRenderer::new(&backend.device, backend.surface_format())?;

        let mut window_manager = WindowManager::new();

        // Create main window
        let main_window_id = window_manager.create_main_window(event_loop, &backend)?;

        let (width, height, format, main_window_for_egui) = {
            let main_window_context = window_manager.get(main_window_id).unwrap();
            (
                main_window_context.surface_config.width,
                main_window_context.surface_config.height,
                main_window_context.surface_config.format,
                main_window_context.window.clone(),
            )
        };

        // Create textures for rendering pipeline
        let composite_texture = texture_pool.create(
            "composite",
            width,
            height,
            backend.surface_format(),
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );

        let layer_ping_pong = [
            texture_pool.create(
                "layer_pong_0",
                width,
                height,
                backend.surface_format(),
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            ),
            texture_pool.create(
                "layer_pong_1",
                width,
                height,
                backend.surface_format(),
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            ),
        ];

        let mut ui_state = AppUI::default();

        let state = AppState::new("New Project");

        let audio_devices = match CpalBackend::list_devices() {
            Ok(Some(devices)) => devices,
            Ok(None) => vec![],
            Err(e) => {
                error!("Failed to list audio devices: {}", e);
                vec![]
            }
        };
        ui_state.audio_devices = audio_devices.clone();

        let mut audio_backend = match CpalBackend::new(None) {
            Ok(backend) => Some(backend),
            Err(e) => {
                error!("Failed to initialize audio backend: {}", e);
                None
            }
        };

        if let Some(backend) = &mut audio_backend {
            if let Err(e) = backend.start() {
                error!("Failed to start audio stream: {}", e);
                audio_backend = None;
            }
        }

        // Initialize Audio Analyzer
        let audio_analyzer = AudioAnalyzer::new(state.audio_config.clone());

        // Start MCP Server in a separate thread
        let (mcp_sender, mcp_receiver) = unbounded();

        thread::spawn(move || {
            // Create a Tokio runtime for the MCP server
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async {
                let server = McpServer::new(Some(mcp_sender));
                if let Err(e) = server.run_stdio().await {
                    error!("MCP Server error: {}", e);
                }
            });
        });

        // Initialize egui
        let egui_context = egui::Context::default();
        let egui_state = State::new(
            egui_context.clone(),
            egui::ViewportId::default(),
            &main_window_for_egui,
            None,
            None,
        );
        let egui_renderer = Renderer::new(&backend.device, format, None, 1);

        let oscillator_renderer = match OscillatorRenderer::new(
            backend.device.clone(),
            backend.queue.clone(),
            format,
            &state.oscillator_config,
        ) {
            Ok(mut renderer) => {
                renderer.initialize_phases(state.oscillator_config.phase_init_mode);
                Some(renderer)
            }
            Err(e) => {
                error!("Failed to create oscillator renderer: {}", e);
                None
            }
        };

        // Initialize icons from assets directory
        let assets_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("..")
            .join("..")
            .join("assets");

        // Try alternative paths for development
        let assets_path = if assets_dir.exists() {
            assets_dir
        } else {
            std::path::PathBuf::from("assets")
        };

        ui_state.initialize_icons(&egui_context, &assets_path);

        let mut app = Self {
            window_manager,
            ui_state,
            backend,
            texture_pool,
            compositor,
            effect_chain_renderer,
            mesh_renderer,
            quad_renderer,
            composite_texture,
            layer_ping_pong,
            state,
            audio_backend,
            audio_analyzer,
            audio_devices,
            egui_context,
            egui_state,
            egui_renderer,
            last_autosave: std::time::Instant::now(),
            last_update: std::time::Instant::now(),
            start_time: std::time::Instant::now(),
            mcp_receiver,
            control_manager: ControlManager::new(),
            exit_requested: false,
            oscillator_renderer,
            dummy_texture: None,
            dummy_view: None,
            fps_samples: Vec::with_capacity(60),
            current_fps: 60.0,
            current_frame_time_ms: 16.6,
            sys_info: sysinfo::System::new_all(),
            last_sysinfo_refresh: std::time::Instant::now(),
            #[cfg(feature = "midi")]
            midi_handler: {
                match MidiInputHandler::new() {
                    Ok(mut handler) => {
                        info!("MIDI initialized");
                        if let Ok(ports) = MidiInputHandler::list_ports() {
                            info!("Available MIDI ports: {:?}", ports);
                            // Auto-connect/Reconnect logic could go here
                            // For now just log them
                        }
                        Some(handler)
                    }
                    Err(e) => {
                        error!("Failed to init MIDI: {}", e);
                        None
                    }
                }
            },
        };

        // Create initial dummy texture
        app.create_dummy_texture(width, height, format);

        Ok(app)
    }

    /// Creates or recreates the dummy texture for effect input.
    fn create_dummy_texture(&mut self, width: u32, height: u32, format: wgpu::TextureFormat) {
        let texture = self
            .backend
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("Dummy Input Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
        self.dummy_view = Some(texture.create_view(&wgpu::TextureViewDescriptor::default()));
        self.dummy_texture = Some(texture);
    }

    /// Runs the application loop.
    pub fn run(mut self, event_loop: EventLoop<()>) {
        info!("Entering event loop");

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        let _ = event_loop.run(move |event, elwt| {
            // Check if exit was requested
            if self.exit_requested {
                info!("Exiting application...");
                elwt.exit();
                return;
            }

            if let Err(e) = self.handle_event(event, elwt) {
                error!("Error handling event: {}", e);
            }
        });
    }

    /// Handles a single event.
    fn handle_event(
        &mut self,
        event: Event<()>,
        elwt: &winit::event_loop::EventLoopWindowTarget<()>,
    ) -> Result<()> {
        // Pass event to UI first (needs reference to full event)

        if let Event::WindowEvent { event, window_id } = &event {
            if let Some(main_window) = self.window_manager.get(0) {
                if *window_id == main_window.window.id() {
                    let _ = self.egui_state.on_window_event(&main_window.window, event);
                }
            }
        }

        match event {
            Event::WindowEvent {
                event, window_id, ..
            } => {
                let output_id = self
                    .window_manager
                    .get_output_id_from_window_id(window_id)
                    .unwrap_or(0);

                match event {
                    WindowEvent::CloseRequested => {
                        elwt.exit();
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
            Event::AboutToWait => {
                // Poll MIDI
                #[cfg(feature = "midi")]
                if let Some(handler) = &mut self.midi_handler {
                    while let Some(msg) = handler.poll_message() {
                        // Pass to UI Overlay
                        self.ui_state.controller_overlay.process_midi(msg);
                    }
                }

                // Autosave check (every 5 minutes)
                if self.state.dirty
                    && self.last_autosave.elapsed() >= std::time::Duration::from_secs(300)
                {
                    let autosave_path = PathBuf::from(".MapFlowAutoSave");

                    if let Err(e) = save_project(&self.state, &autosave_path) {
                        error!("Autosave failed: {}", e);
                    } else {
                        info!("Autosave successful");
                        self.last_autosave = std::time::Instant::now();
                        // Note: We don't clear dirty flag on autosave, only on explicit save
                    }
                }

                // Process audio
                if let Some(backend) = &mut self.audio_backend {
                    let samples = backend.get_samples();
                    if !samples.is_empty() {
                        let timestamp = self.start_time.elapsed().as_secs_f64();
                        let analysis = self.audio_analyzer.process_samples(&samples, timestamp);
                        // Log periodically (every ~5 seconds based on timestamp)
                        if (timestamp as i64) % 5 == 0 {
                            tracing::debug!(
                                "Audio: {} samples, RMS={:.3}, Peak={:.3}",
                                samples.len(),
                                analysis.rms_volume,
                                analysis.peak_volume
                            );
                        }
                        self.ui_state.dashboard.set_audio_analysis(analysis);
                    }
                }

                // Redraw all windows
                for output_id in self
                    .window_manager
                    .window_ids()
                    .copied()
                    .collect::<Vec<_>>()
                {
                    if let Some(window_context) = self.window_manager.get(output_id) {
                        window_context.window.request_redraw();
                    }
                }
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
                // TODO: Handle other actions (AddLayer, etc.) here or delegating to state
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
        if let Some(action) = self.ui_state.paint_panel.take_action() {
            match action {
                mapmap_ui::paint_panel::PaintPanelAction::AddPaint => {
                    self.state
                        .paint_manager
                        .add_paint(mapmap_core::paint::Paint::color(
                            0,
                            "New Color",
                            [1.0, 1.0, 1.0, 1.0],
                        ));
                    self.state.dirty = true;
                }
                mapmap_ui::paint_panel::PaintPanelAction::RemovePaint(id) => {
                    self.state.paint_manager.remove_paint(id);
                    self.state.dirty = true;
                }
            }
        }

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

    /// Renders a single frame for a given output.
    fn render(&mut self, output_id: OutputId) -> Result<()> {
        let now = std::time::Instant::now();
        let delta_time = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;

        // Calculate FPS with smoothing (rolling average of last 60 frames)
        let frame_time_ms = delta_time * 1000.0;
        self.fps_samples.push(frame_time_ms);
        if self.fps_samples.len() > 60 {
            self.fps_samples.remove(0);
        }
        if !self.fps_samples.is_empty() {
            let avg_frame_time: f32 =
                self.fps_samples.iter().sum::<f32>() / self.fps_samples.len() as f32;
            self.current_frame_time_ms = avg_frame_time;
            self.current_fps = if avg_frame_time > 0.0 {
                1000.0 / avg_frame_time
            } else {
                0.0
            };
        }

        if let Some(renderer) = &mut self.oscillator_renderer {
            if self.state.oscillator_config.enabled {
                renderer.update(delta_time, &self.state.oscillator_config);
            }
        }

        let window_context = self.window_manager.get(output_id).unwrap();

        // Get surface texture and view for final output
        let surface_texture = window_context.surface.get_current_texture()?;
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Encoder vorbereiten
        let mut encoder =
            self.backend
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        if output_id == 0 {
            // --------- ImGui removed (Phase 6 Complete) ----------

            // --------- egui: UI separat zeichnen ---------
            let dashboard_action = None;
            let (tris, screen_descriptor) = {
                let raw_input = self.egui_state.take_egui_input(&window_context.window);
                let full_output = self.egui_context.run(raw_input, |ctx| {
                    // Apply the theme at the beginning of each UI render pass
                    self.ui_state.user_config.theme.apply(ctx);

                    // Update performance and audio values for toolbar display
                    self.ui_state.current_fps = self.current_fps;
                    self.ui_state.current_frame_time_ms = self.current_frame_time_ms;
                    self.ui_state.target_fps = self.ui_state.user_config.target_fps.unwrap_or(60.0);

                    // Refresh system info every 500ms
                    if self.last_sysinfo_refresh.elapsed().as_millis() > 500 {
                        self.sys_info.refresh_cpu_usage();
                        self.sys_info.refresh_memory();
                        self.last_sysinfo_refresh = std::time::Instant::now();
                    }

                    let cpu_count = self.sys_info.cpus().len() as f32;
                    self.ui_state.cpu_usage = if cpu_count > 0.0 {
                        self.sys_info.cpus().iter().map(|c| c.cpu_usage()).sum::<f32>() / cpu_count
                    } else { 0.0 };

                    if let Ok(pid) = sysinfo::get_current_pid() {
                        self.ui_state.ram_usage_mb = self
                            .sys_info
                            .process(pid)
                            .map(|p| p.memory() as f32 / 1024.0 / 1024.0)
                            .unwrap_or(0.0);
                    }

                    let fps_ratio = (self.current_fps / self.ui_state.target_fps).clamp(0.0, 1.0);
                    self.ui_state.gpu_usage = ((1.0 - fps_ratio) * 100.0 * 1.2).clamp(0.0, 100.0);

                    let audio_analysis = self.audio_analyzer.get_latest_analysis();
                    self.ui_state.current_audio_level = audio_analysis.rms_volume;

                    self.ui_state.current_audio_level = audio_analysis.rms_volume;

                    // MIDI Controller Overlay
                    self.ui_state.controller_overlay.show(
                        ctx,
                        &mut self.ui_state.show_controller_overlay,
                    );

                    // === 1. TOP PANEL: Menu Bar + Toolbar ===
                    let menu_actions = menu_bar::show(ctx, &mut self.ui_state);
                    self.ui_state.actions.extend(menu_actions);

                    // === 2. BOTTOM PANEL: Timeline (FULL WIDTH - rendered before side panels!) ===
                    if self.ui_state.show_timeline {
                        egui::TopBottomPanel::bottom("timeline_panel")
                            .resizable(true)
                            .default_height(180.0)
                            .min_height(100.0)
                            .max_height(350.0)
                            .show(ctx, |ui| {
                                ui.horizontal(|ui| {
                                    ui.heading("Timeline");
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button("✕").clicked() {
                                            self.ui_state.show_timeline = false;
                                        }
                                    });
                                });
                                ui.separator();
                                let _ = self.ui_state.timeline_panel.ui(ui);
                            });
                    }

                    // === 3. LEFT SIDEBAR (collapsible, contains all controls) ===
                    if self.ui_state.show_left_sidebar {
                        egui::SidePanel::left("left_sidebar")
                            .resizable(true)
                            .default_width(280.0)
                            .min_width(200.0)
                            .max_width(450.0)
                            .show(ctx, |ui| {
                                ui.horizontal(|ui| {
                                    ui.heading("Controls");
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button("◀").on_hover_text("Sidebar einklappen").clicked() {
                                            self.ui_state.show_left_sidebar = false;
                                        }
                                    });
                                });
                                ui.separator();

                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    // Dashboard Section
                                    egui::CollapsingHeader::new("📊 Dashboard")
                                        .default_open(true)
                                        .show(ui, |ui| {
                                            // Playback controls inline
                                            ui.horizontal(|ui| {
                                                if ui.button("▶").clicked() {
                                                    self.ui_state.actions.push(mapmap_ui::UIAction::Play);
                                                }
                                                if ui.button("⏸").clicked() {
                                                    self.ui_state.actions.push(mapmap_ui::UIAction::Pause);
                                                }
                                                if ui.button("⏹").clicked() {
                                                    self.ui_state.actions.push(mapmap_ui::UIAction::Stop);
                                                }
                                            });
                                        });

                                    // Layers Section
                                    egui::CollapsingHeader::new("📑 Layers")
                                        .default_open(true)
                                        .show(ui, |ui| {
                                            // Simple layer list
                                            ui.horizontal(|ui| {
                                                if ui.button("+ Add").clicked() {
                                                    self.ui_state.actions.push(mapmap_ui::UIAction::AddLayer);
                                                }
                                            });
                                            let layers: Vec<_> = self.state.layer_manager.layers().iter()
                                                .map(|l| (l.id, l.name.clone()))
                                                .collect();

                                            for (id, name) in layers {
                                                let selected = self.ui_state.selected_layer_id == Some(id);
                                                if ui.selectable_label(selected, &name).clicked() {
                                                    self.ui_state.selected_layer_id = Some(id);
                                                }
                                            }
                                        });


                                    // Master Controls Section
                                    egui::CollapsingHeader::new("🎚️ Master")
                                        .default_open(false)
                                        .show(ui, |ui| {
                                            let comp = &mut self.state.layer_manager.composition;
                                            ui.add(egui::Slider::new(&mut comp.master_opacity, 0.0..=1.0).text("Opacity"));
                                            ui.add(egui::Slider::new(&mut comp.master_speed, 0.1..=4.0).text("Speed"));
                                        });

                                    // Media Browser Section
                                    egui::CollapsingHeader::new("📁 Media")
                                        .default_open(false)
                                        .show(ui, |ui| {
                                            let _ = self.ui_state.media_browser.ui(
                                                ui,
                                                &self.ui_state.i18n,
                                                self.ui_state.icon_manager.as_ref(),
                                            );
                                        });

                                    // Audio Section
                                    egui::CollapsingHeader::new("🔊 Audio")
                                        .default_open(false)
                                        .show(ui, |ui| {
                                            let analysis = self.audio_analyzer.get_latest_analysis();
                                            if let Some(action) = self.ui_state.audio_panel.ui(
                                                ui,
                                                &self.ui_state.i18n,
                                                Some(&analysis),
                                                &self.state.audio_config,
                                                &self.audio_devices,
                                                &mut self.ui_state.selected_audio_device,
                                            ) {
                                                match action {
                                                    mapmap_ui::audio_panel::AudioPanelAction::DeviceChanged(device) => {
                                                        info!("Audio device changed to: {}", device);
                                                        if let Some(backend) = &mut self.audio_backend {
                                                            backend.stop();
                                                        }
                                                        self.audio_backend = None;
                                                        match CpalBackend::new(Some(device.clone())) {
                                                            Ok(mut backend) => {
                                                                if let Err(e) = backend.start() {
                                                                    error!("Failed to start audio backend: {}", e);
                                                                } else {
                                                                    info!("Audio backend started successfully");
                                                                }
                                                                self.audio_backend = Some(backend);
                                                            }
                                                            Err(e) => {
                                                                error!("Failed to create audio backend for device '{}': {}", device, e);
                                                            }
                                                        }
                                                    }
                                                    mapmap_ui::audio_panel::AudioPanelAction::ConfigChanged(cfg) => {
                                                        self.audio_analyzer.update_config(cfg.clone());
                                                        self.state.audio_config = cfg;
                                                    }
                                                }
                                            }
                                        });

                                    // Effects Section
                                    egui::CollapsingHeader::new("✨ Effects")
                                        .default_open(false)
                                        .show(ui, |ui| {
                                            // Simplified effects UI
                                            ui.label("Effect Chain");
                                            ui.separator();
                                            if ui.button("+ Add Effect").clicked() {
                                                // TODO: Add effect action
                                            }
                                        });

                                });
                            });
                    } else {
                        // Collapsed sidebar - just show expand button
                        egui::SidePanel::left("left_sidebar_collapsed")
                            .exact_width(28.0)
                            .resizable(false)
                            .show(ctx, |ui| {
                                if ui.button("▶").on_hover_text("Sidebar ausklappen").clicked() {
                                    self.ui_state.show_left_sidebar = true;
                                }
                            });
                    }

                    // === 4. RIGHT PANEL: Inspector ===
                    if self.ui_state.show_inspector {
                        let inspector_context = if let Some(layer_id) = self.ui_state.selected_layer_id {
                            if let Some(layer) = self.state.layer_manager.get_layer(layer_id) {
                                mapmap_ui::InspectorContext::Layer { layer, transform: &layer.transform }
                            } else {
                                mapmap_ui::InspectorContext::None
                            }
                        } else if let Some(output_id) = self.ui_state.selected_output_id {
                            if let Some(output) = self.state.output_manager.get_output(output_id) {
                                mapmap_ui::InspectorContext::Output(output)
                            } else {
                                mapmap_ui::InspectorContext::None
                            }
                        } else {
                            mapmap_ui::InspectorContext::None
                        };

                        self.ui_state.inspector_panel.show(
                            ctx,
                            inspector_context,
                            &self.ui_state.i18n,
                            self.ui_state.icon_manager.as_ref(),
                        );
                    }

                    // === 5. CENTRAL PANEL: Module Canvas ===
                    egui::CentralPanel::default().show(ctx, |ui| {
                        if self.ui_state.show_module_canvas {
                            self.ui_state.module_canvas.show(
                                ui,
                                &mut self.state.module_manager,
                                &self.ui_state.i18n,
                            );
                        } else {
                            // Placeholder for normal canvas/viewport
                            ui.centered_and_justified(|ui| {
                                ui.label("Canvas - Module Canvas deaktiviert (View → Module Canvas)");
                            });
                        }
                    });

                    // === Settings Window (only modal allowed) ===
                    if self.ui_state.show_settings {
                        let mut close_settings = false;
                        egui::Window::new(self.ui_state.i18n.t("menu-file-settings"))
                            .collapsible(false)
                            .resizable(true)
                            .default_size([400.0, 300.0])
                            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                            .show(ctx, |ui| {
                                // Project Settings
                                egui::CollapsingHeader::new(format!("🎬 {}", self.ui_state.i18n.t("settings-project")))
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(format!("{}:", self.ui_state.i18n.t("settings-frame-rate")));
                                            let fps_options = [(24.0, "24"), (25.0, "25"), (30.0, "30"), (50.0, "50"), (60.0, "60"), (120.0, "120")];
                                            let current = self.ui_state.user_config.target_fps.unwrap_or(60.0);
                                            egui::ComboBox::from_id_source("fps_select")
                                                .selected_text(format!("{:.0} FPS", current))
                                                .show_ui(ui, |ui| {
                                                    for (fps, label) in fps_options {
                                                        if ui.selectable_label((current - fps).abs() < 0.1, label).clicked() {
                                                            self.ui_state.user_config.target_fps = Some(fps);
                                                            let _ = self.ui_state.user_config.save();
                                                        }
                                                    }
                                                });
                                        });
                                    });

                                ui.separator();

                                // App Settings
                                egui::CollapsingHeader::new(format!("⚙️ {}", self.ui_state.i18n.t("settings-app")))
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(format!("{}:", self.ui_state.i18n.t("settings-language")));
                                            if ui.button("English").clicked() {
                                                self.ui_state.actions.push(mapmap_ui::UIAction::SetLanguage("en".to_string()));
                                            }
                                            if ui.button("Deutsch").clicked() {
                                                self.ui_state.actions.push(mapmap_ui::UIAction::SetLanguage("de".to_string()));
                                            }
                                        });

                                        ui.horizontal(|ui| {
                                            ui.label("Audio Meter:");
                                            let current = self.ui_state.user_config.meter_style;
                                            egui::ComboBox::from_id_source("meter_style_select")
                                                .selected_text(format!("{}", current))
                                                .show_ui(ui, |ui| {
                                                    let styles = [
                                                        mapmap_ui::config::AudioMeterStyle::Retro,
                                                        mapmap_ui::config::AudioMeterStyle::Digital,
                                                    ];
                                                    for style in styles {
                                                        if ui.selectable_value(&mut self.ui_state.user_config.meter_style, style, format!("{}", style)).clicked() {
                                                            let _ = self.ui_state.user_config.save();
                                                        }
                                                    }
                                                });
                                        });
                                    });

                                ui.separator();
                                if ui.button("✕ Schließen").clicked() {
                                    close_settings = true;
                                }
                            });
                        if close_settings {
                            self.ui_state.show_settings = false;
                        }
                    }
                });

                self.egui_state
                    .handle_platform_output(&window_context.window, full_output.platform_output);

                let tris = self
                    .egui_context
                    .tessellate(full_output.shapes.clone(), full_output.pixels_per_point);

                for (id, image_delta) in &full_output.textures_delta.set {
                    self.egui_renderer.update_texture(
                        &self.backend.device,
                        &self.backend.queue,
                        *id,
                        image_delta,
                    );
                }
                for id in &full_output.textures_delta.free {
                    self.egui_renderer.free_texture(id);
                }

                let screen_descriptor = egui_wgpu::ScreenDescriptor {
                    size_in_pixels: [
                        window_context.surface_config.width,
                        window_context.surface_config.height,
                    ],
                    pixels_per_point: window_context.window.scale_factor() as f32,
                };

                self.egui_renderer.update_buffers(
                    &self.backend.device,
                    &self.backend.queue,
                    &mut encoder,
                    &tris,
                    &screen_descriptor,
                );

                (tris, screen_descriptor)
            };

            // Handle Dashboard actions
            if let Some(action) = dashboard_action {
                match action {
                    mapmap_ui::DashboardAction::ToggleAudioPanel => {
                        self.ui_state.show_audio = !self.ui_state.show_audio;
                    }
                    mapmap_ui::DashboardAction::AudioDeviceChanged(_device) => {}
                    mapmap_ui::DashboardAction::SendCommand(_cmd) => {
                        // TODO: Implement playback commands if not handled elsewhere
                        // Currently PlaybackCommand handling seems missing in main.rs or handled via Mcp?
                        // "McpAction::MediaPlay" has TODO.
                        // This suggests buttons in Dashboard might do nothing currently!
                        // But fixing playback is not my task.
                    }
                }
            }

            // Handle TransformPanel actions
            if let Some(action) = self.ui_state.transform_panel.take_action() {
                if let Some(selected_id) = self.ui_state.selected_layer_id {
                    match action {
                        mapmap_ui::TransformAction::UpdateTransform(values) => {
                            if let Some(layer) = self.state.layer_manager.get_layer_mut(selected_id)
                            {
                                layer.transform.position.x = values.position.0;
                                layer.transform.position.y = values.position.1;
                                layer.transform.rotation.z = values.rotation.to_radians();
                                layer.transform.scale.x = values.scale.0;
                                layer.transform.scale.y = values.scale.1;
                                layer.transform.anchor.x = values.anchor.0;
                                layer.transform.anchor.y = values.anchor.1;
                                self.state.dirty = true;
                            }
                        }
                        mapmap_ui::TransformAction::ResetTransform => {
                            if let Some(layer) = self.state.layer_manager.get_layer_mut(selected_id)
                            {
                                layer.transform = mapmap_core::Transform::default();
                                self.state.dirty = true;
                            }
                        }
                        mapmap_ui::TransformAction::ApplyResizeMode(mode) => {
                            self.ui_state
                                .actions
                                .push(mapmap_ui::UIAction::ApplyResizeMode(selected_id, mode));
                        }
                    }
                }
            }

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Egui Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                self.egui_renderer
                    .render(&mut render_pass, &tris, &screen_descriptor);
            }

            // Post-render logic for egui actions
        } else {
            // Ensure textures are the correct size
            self.texture_pool.resize_if_needed(
                &self.composite_texture,
                window_context.surface_config.width,
                window_context.surface_config.height,
            );
            self.texture_pool.resize_if_needed(
                &self.layer_ping_pong[0],
                window_context.surface_config.width,
                window_context.surface_config.height,
            );
            self.texture_pool.resize_if_needed(
                &self.layer_ping_pong[1],
                window_context.surface_config.width,
                window_context.surface_config.height,
            );

            // Get layer textures
            let composite_view = self.texture_pool.get_view(&self.composite_texture);
            let layer_pp_views = [
                self.texture_pool.get_view(&self.layer_ping_pong[0]),
                self.texture_pool.get_view(&self.layer_ping_pong[1]),
            ];

            let mut current_target_idx = 0;

            // Clear the initial composite target
            {
                let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Clear Composite Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &layer_pp_views[0],
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });
            }

            // Main layer composition loop
            for layer in self.state.layer_manager.visible_layers() {
                // TODO: Render layer content (mesh, media) into a temp texture
                // For now, let's just use a solid color from the paint manager
                let temp_layer_content = self.texture_pool.create(
                    "temp_layer_content",
                    window_context.surface_config.width,
                    window_context.surface_config.height,
                    self.backend.surface_format(),
                    wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                );
                let temp_layer_content_view = self.texture_pool.get_view(&temp_layer_content);

                // Placeholder: Clear to a debug color
                {
                    let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Layer Content Placeholder"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &temp_layer_content_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLUE), // Debug color
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });
                }

                // Apply effect chain to the layer content
                self.effect_chain_renderer.apply_chain(
                    &mut encoder,
                    &temp_layer_content_view,
                    &composite_view, // Output of effects goes to composite texture
                    &layer.effect_chain,
                    self.start_time.elapsed().as_secs_f32(),
                    window_context.surface_config.width,
                    window_context.surface_config.height,
                );

                // Composite the result with the previous layers
                let current_base_view = &layer_pp_views[current_target_idx];
                let next_target_view = &layer_pp_views[1 - current_target_idx];

                let bind_group = self
                    .compositor
                    .create_bind_group(current_base_view, &composite_view);
                let uniform_buffer = self
                    .compositor
                    .create_uniform_buffer(layer.blend_mode, layer.opacity);
                let uniform_bind_group = self.compositor.create_uniform_bind_group(&uniform_buffer);

                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Layer Composite Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: next_target_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    self.compositor.composite(
                        &mut rpass,
                        self.quad_renderer.vertex_buffer(),
                        self.quad_renderer.index_buffer(),
                        &bind_group,
                        &uniform_bind_group,
                    );
                }

                // Swap for next iteration
                current_target_idx = 1 - current_target_idx;
                self.texture_pool.release(&temp_layer_content);
            }

            // TODO: Apply output transforms (edge blend, color calibration) here

            // Final copy to the screen
            let final_texture_view = &layer_pp_views[current_target_idx];
            let bind_group = self
                .quad_renderer
                .create_bind_group(&self.backend.device, final_texture_view);
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Final Blit to Screen"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.quad_renderer.draw(&mut rpass, &bind_group);
        }

        self.backend.queue.submit(Some(encoder.finish()));
        surface_texture.present();

        Ok(())
    }
}

/// The main entry point for the application.
fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    info!("Starting MapFlow...");

    // Create the event loop
    let event_loop = EventLoop::new()?;

    // Create the app
    let app = pollster::block_on(App::new(&event_loop))?;

    // Run the app
    app.run(event_loop);

    Ok(())
}
