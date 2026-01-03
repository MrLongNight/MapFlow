//! MapFlow - Open source Vj Projection Mapping Software
//!
//! This is the main application crate for MapFlow.

#![warn(missing_docs)]

mod window_manager;

use anyhow::Result;
use egui_wgpu::Renderer;
use egui_winit::State;
#[cfg(feature = "midi")]
use mapmap_control::midi::MidiInputHandler;
use mapmap_control::{shortcuts::Action, ControlManager};
use mapmap_core::{
    audio::{
        analyzer_v2::{AudioAnalyzerV2, AudioAnalyzerV2Config},
        backend::cpal_backend::CpalBackend,
        backend::AudioBackend,
    },
    AppState, ModuleEvaluator, OutputId, RenderOp,
};

use mapmap_mcp::{McpAction, McpServer};
// Define McpAction locally or import if we move it to core later -> Removed local definition

use crossbeam_channel::{unbounded, Receiver};
use mapmap_core::module::ModulePartId;
use mapmap_io::{load_project, save_project};
use mapmap_media::player::{PlaybackCommand, VideoPlayer};
use mapmap_render::{
    ColorCalibrationRenderer, Compositor, EdgeBlendRenderer, EffectChainRenderer, MeshBufferCache,
    MeshRenderer, OscillatorRenderer, QuadRenderer, TexturePool, WgpuBackend,
};
use mapmap_ui::{menu_bar, AppUI, EdgeBlendAction};
use rfd::FileDialog;
use std::collections::HashMap;
use std::path::PathBuf;
use std::thread;
use tracing::{debug, error, info, warn};
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
    _compositor: Compositor,
    /// The effect chain renderer.
    effect_chain_renderer: EffectChainRenderer,
    /// The mesh renderer.
    mesh_renderer: MeshRenderer,
    /// Cache for mesh GPU buffers
    mesh_buffer_cache: MeshBufferCache,
    /// Quad renderer for passthrough.
    _quad_renderer: QuadRenderer,
    /// Final composite texture before output processing.
    _composite_texture: String,
    /// Ping-pong textures for layer composition.
    layer_ping_pong: [String; 2],
    /// The application state (project data).
    state: AppState,
    /// The audio backend.
    audio_backend: Option<CpalBackend>,
    /// The audio analyzer.
    audio_analyzer: AudioAnalyzerV2,
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
    /// Module evaluator
    module_evaluator: ModuleEvaluator,
    /// Active media players for source nodes (PartID -> Player)
    media_players: HashMap<ModulePartId, VideoPlayer>,
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
    /// Available MIDI ports
    #[cfg(feature = "midi")]
    midi_ports: Vec<String>,
    /// Selected MIDI port index
    #[cfg(feature = "midi")]
    selected_midi_port: Option<usize>,
    /// NDI Receivers for module sources
    #[cfg(feature = "ndi")]
    ndi_receivers:
        std::collections::HashMap<mapmap_core::module::ModulePartId, mapmap_io::ndi::NdiReceiver>,
    /// NDI Senders for module outputs
    #[cfg(feature = "ndi")]
    ndi_senders:
        std::collections::HashMap<mapmap_core::module::ModulePartId, mapmap_io::ndi::NdiSender>,

    /// Shader Graph Manager (Runtime)
    #[allow(dead_code)]
    shader_graph_manager: mapmap_render::ShaderGraphManager,
    /// Output assignments (OutputID -> Texture Name)
    output_assignments: std::collections::HashMap<u64, String>,
    /// Recent Effect Configurations (User Prefs)
    recent_effect_configs: mapmap_core::RecentEffectConfigs,
    /// Render Operations from Module Evaluator
    render_ops: Vec<RenderOp>,
    /// Edge blend renderer for output windows
    edge_blend_renderer: Option<EdgeBlendRenderer>,
    /// Color calibration renderer for output windows
    color_calibration_renderer: Option<ColorCalibrationRenderer>,
    /// Temporary textures for output rendering (OutputID -> Texture)
    output_temp_textures: std::collections::HashMap<u64, wgpu::Texture>,
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
        let mesh_buffer_cache = MeshBufferCache::new();
        let quad_renderer = QuadRenderer::new(&backend.device, backend.surface_format())?;

        // Initialize advanced output renderers
        let edge_blend_renderer =
            EdgeBlendRenderer::new(backend.device.clone(), backend.surface_format())
                .map_err(|e| {
                    tracing::warn!("Failed to create edge blend renderer: {}", e);
                    e
                })
                .ok();

        let color_calibration_renderer =
            ColorCalibrationRenderer::new(backend.device.clone(), backend.surface_format())
                .map_err(|e| {
                    tracing::warn!("Failed to create color calibration renderer: {}", e);
                    e
                })
                .ok();

        let mut window_manager = WindowManager::new();

        // Load user config to get saved window geometry
        let saved_config = mapmap_ui::config::UserConfig::load();

        // Create main window with saved geometry
        let main_window_id = window_manager.create_main_window_with_geometry(
            event_loop,
            &backend,
            saved_config.window_width,
            saved_config.window_height,
            saved_config.window_x,
            saved_config.window_y,
            saved_config.window_maximized,
        )?;

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

        #[cfg(feature = "midi")]
        {
            let paths = [
                "resources/controllers/ecler_nuo4/elements.json",
                "../resources/controllers/ecler_nuo4/elements.json",
                r"C:\Users\Vinyl\Desktop\VJMapper\VjMapper\resources\controllers\ecler_nuo4\elements.json",
            ];
            for path_str in paths {
                let path = std::path::Path::new(path_str);
                if path.exists() {
                    match std::fs::read_to_string(path) {
                        Ok(json) => {
                            if let Err(e) = ui_state.controller_overlay.load_elements(&json) {
                                tracing::error!("Failed to parse elements.json: {}", e);
                            } else {
                                tracing::info!("Loaded controller elements from {:?}", path);
                                break;
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to read elements.json from {:?}: {}", path, e)
                        }
                    }
                }
            }
        }

        // Initialize state, trying to load autosave first
        let mut state = AppState::new("New Project");

        let autosave_path =
            dirs::data_local_dir().map(|p| p.join("MapFlow").join("autosave.mflow"));

        if let Some(path) = &autosave_path {
            if path.exists() {
                info!("Found autosave at {:?}, attempting to load...", path);
                match load_project(path) {
                    Ok(loaded_state) => {
                        info!("Successfully loaded autosave.");
                        state = loaded_state;
                    }
                    Err(e) => {
                        error!("Failed to load autosave: {}", e);
                        // Fallback to new project is automatic as state is already initialized
                    }
                }
            } else {
                info!("No autosave found at {:?}, starting new project.", path);
            }
        } else {
            warn!("Could not determine data local directory for autosave.");
        }

        let audio_devices = match CpalBackend::list_devices() {
            Ok(Some(devices)) => devices,
            Ok(None) => vec![],
            Err(e) => {
                error!("Failed to list audio devices: {}", e);
                vec![]
            }
        };
        ui_state.audio_devices = audio_devices.clone();

        // Load saved audio device from user config
        let saved_device = ui_state.user_config.selected_audio_device.clone();
        let device_to_use = if let Some(ref dev) = saved_device {
            // Check if saved device still exists
            if audio_devices.contains(dev) {
                info!("Restoring saved audio device: {}", dev);
                Some(dev.clone())
            } else {
                info!(
                    "Saved audio device '{}' no longer available, using default",
                    dev
                );
                None
            }
        } else {
            None
        };

        // Set the selected device in UI state
        ui_state.selected_audio_device = device_to_use.clone();

        let mut audio_backend = match CpalBackend::new(device_to_use) {
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

        // Initialize Audio Analyzer V2 (new implementation)
        let audio_analyzer = AudioAnalyzerV2::new(AudioAnalyzerV2Config {
            sample_rate: state.audio_config.sample_rate,
            fft_size: state.audio_config.fft_size,
            overlap: state.audio_config.overlap,
            smoothing: state.audio_config.smoothing,
        });

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
            _compositor: compositor,
            effect_chain_renderer,
            mesh_renderer,
            mesh_buffer_cache,
            _quad_renderer: quad_renderer,
            _composite_texture: composite_texture,
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
            module_evaluator: ModuleEvaluator::new(),
            media_players: HashMap::new(),
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
                            // Auto-connect to first port if available
                            if !ports.is_empty() {
                                if let Err(e) = handler.connect(0) {
                                    error!("Failed to auto-connect MIDI: {}", e);
                                } else {
                                    info!("Auto-connected to MIDI port: {}", ports[0]);
                                }
                            }
                        }
                        Some(handler)
                    }
                    Err(e) => {
                        error!("Failed to init MIDI: {}", e);
                        None
                    }
                }
            },
            #[cfg(feature = "midi")]
            midi_ports: MidiInputHandler::list_ports().unwrap_or_default(),
            #[cfg(feature = "midi")]
            selected_midi_port: if MidiInputHandler::list_ports()
                .unwrap_or_default()
                .is_empty()
            {
                None
            } else {
                Some(0)
            },
            #[cfg(feature = "ndi")]
            ndi_receivers: std::collections::HashMap::new(),
            #[cfg(feature = "ndi")]
            ndi_senders: std::collections::HashMap::new(),
            output_assignments: HashMap::new(),
            shader_graph_manager: mapmap_render::ShaderGraphManager::new(),
            recent_effect_configs: mapmap_core::RecentEffectConfigs::with_persistence(
                dirs::data_dir()
                    .unwrap_or(std::path::PathBuf::from("."))
                    .join("MapFlow")
                    .join("recent_effect_configs.json"),
            ),
            render_ops: Vec::new(),
            edge_blend_renderer,
            color_calibration_renderer,
            output_temp_textures: std::collections::HashMap::new(),
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

                // Save window geometry and settings before exit
                if let Some(main_window) = self.window_manager.get(0) {
                    let window = &main_window.window;

                    // Get window size
                    let size = window.inner_size();
                    self.ui_state.user_config.window_width = Some(size.width);
                    self.ui_state.user_config.window_height = Some(size.height);

                    // Get window position
                    if let Ok(pos) = window.outer_position() {
                        self.ui_state.user_config.window_x = Some(pos.x);
                        self.ui_state.user_config.window_y = Some(pos.y);
                    }

                    // Check if maximized
                    self.ui_state.user_config.window_maximized = window.is_maximized();
                }

                // Save panel visibility states
                self.ui_state.user_config.show_left_sidebar = self.ui_state.show_left_sidebar;
                self.ui_state.user_config.show_inspector = self.ui_state.show_inspector;
                self.ui_state.user_config.show_timeline = self.ui_state.show_timeline;
                self.ui_state.user_config.show_media_browser = self.ui_state.show_media_browser;
                self.ui_state.user_config.show_module_canvas = self.ui_state.show_module_canvas;
                self.ui_state.user_config.show_controller_overlay =
                    self.ui_state.show_controller_overlay;

                // Save audio device selection
                self.ui_state.user_config.selected_audio_device =
                    self.ui_state.selected_audio_device.clone();

                // Save target FPS
                self.ui_state.user_config.target_fps = Some(self.ui_state.target_fps);

                // Save the config
                if let Err(e) = self.ui_state.user_config.save() {
                    error!("Failed to save user config on exit: {}", e);
                } else {
                    info!("User settings saved successfully");
                }

                if let Err(e) = self.recent_effect_configs.save() {
                    error!("Failed to save recent configs: {}", e);
                }

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
            Event::LoopExiting => {
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
                    // Note: Width/Height/X/Y are typically tracked during move/resize, not just at exit,
                    // but we could explicitly query inner_size here if needed.
                    // For now, maximization and flags are key.
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
            }
            Event::AboutToWait => {
                // Poll MIDI
                #[cfg(feature = "midi")]
                if let Some(handler) = &mut self.midi_handler {
                    while let Some(msg) = handler.poll_message() {
                        // Pass to UI Overlay
                        self.ui_state.controller_overlay.process_midi(msg);
                        // Pass to Module Canvas for MIDI Learn
                        self.ui_state.module_canvas.process_midi_message(msg);
                    }
                }

                // Autosave check (every 5 minutes)
                if self.state.dirty
                    && self.last_autosave.elapsed() >= std::time::Duration::from_secs(300)
                {
                    // Use data directory for autosave
                    let autosave_path = dirs::data_local_dir()
                        .unwrap_or_else(|| PathBuf::from("."))
                        .join("MapFlow")
                        .join("autosave.mflow");

                    // Ensure directory exists
                    if let Some(parent) = autosave_path.parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }

                    if let Err(e) = save_project(&self.state, &autosave_path) {
                        error!("Autosave failed: {}", e);
                    } else {
                        info!("Autosave successful to {:?}", autosave_path);
                        self.last_autosave = std::time::Instant::now();
                        // Note: We don't clear dirty flag on autosave, only on explicit save
                    }
                }

                // Process audio
                // Process audio
                let timestamp = self.start_time.elapsed().as_secs_f64();
                let mut samples_len = 0;

                if let Some(backend) = &mut self.audio_backend {
                    let samples = backend.get_samples();
                    samples_len = samples.len();
                    if !samples.is_empty() {
                        self.audio_analyzer.process_samples(&samples, timestamp);
                    }
                }

                // Get analysis results
                let analysis_v2 = self.audio_analyzer.get_latest_analysis();

                // --- MODULE EVALUATION ---
                self.module_evaluator.update_audio(&analysis_v2);

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
                    // If player doesn't exist and we get a Play command, try to create it
                    if !self.media_players.contains_key(&part_id) {
                        if let mapmap_ui::MediaPlaybackCommand::Play = &cmd {
                            info!(
                                "Player doesn't exist for part_id={}, attempting to create...",
                                part_id
                            );
                            // Find the source path from the module manager
                            if let Some(active_module_id) =
                                self.ui_state.module_canvas.active_module_id
                            {
                                if let Some(module) =
                                    self.state.module_manager.get_module(active_module_id)
                                {
                                    if let Some(part) =
                                        module.parts.iter().find(|p| p.id == part_id)
                                    {
                                        if let mapmap_core::module::ModulePartType::Source(
                                            mapmap_core::module::SourceType::MediaFile {
                                                ref path,
                                                ..
                                            },
                                        ) = &part.part_type
                                        {
                                            info!("Found media path: '{}'", path);
                                            if !path.is_empty() {
                                                match mapmap_media::open_path(path) {
                                                    Ok(player) => {
                                                        info!(
                                                            "Successfully created player for '{}'",
                                                            path
                                                        );
                                                        self.media_players.insert(part_id, player);
                                                    }
                                                    Err(e) => {
                                                        error!(
                                                            "Failed to load media '{}': {}",
                                                            path, e
                                                        );
                                                    }
                                                }
                                            }
                                        } else {
                                            warn!("Part {} is not a MediaFile source", part_id);
                                        }
                                    } else {
                                        warn!("Part {} not found in module", part_id);
                                    }
                                }
                            }
                        }
                    }

                    if let Some(player) = self.media_players.get_mut(&part_id) {
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
                        }
                    }
                }

                // Update all active media players and upload frames to texture pool
                // This ensures previews work even without triggers connected
                let player_ids: Vec<u64> = self.media_players.keys().cloned().collect();
                if !player_ids.is_empty() {
                    debug!("Updating {} active media players", player_ids.len());
                }
                for part_id in player_ids {
                    if let Some(player) = self.media_players.get_mut(&part_id) {
                        debug!(
                            "Updating player for part_id={}, state={:?}",
                            part_id,
                            player.state()
                        );
                        if let Some(frame) = player.update(std::time::Duration::from_millis(16)) {
                            debug!(
                                "Got frame for part_id={}, size={}x{}",
                                part_id, frame.format.width, frame.format.height
                            );
                            if let mapmap_io::format::FrameData::Cpu(data) = &frame.data {
                                let tex_name = format!("part_{}", part_id);
                                debug!(
                                    "Uploading texture '{}' with {} bytes",
                                    tex_name,
                                    data.len()
                                );
                                self.texture_pool.upload_data(
                                    &self.backend.queue,
                                    &tex_name,
                                    data,
                                    frame.format.width,
                                    frame.format.height,
                                );
                            } else {
                                debug!("Frame data is GPU-based, not CPU");
                            }
                        }
                    }
                }

                if let Some(active_module_id) = self.ui_state.module_canvas.active_module_id {
                    if let Some(module) = self.state.module_manager.get_module(active_module_id) {
                        let result = self.module_evaluator.evaluate(module);

                        // 1. Handle Source Commands
                        for (part_id, cmd) in result.source_commands {
                            match cmd {
                                mapmap_core::SourceCommand::PlayMedia {
                                    path,
                                    trigger_value,
                                } => {
                                    if path.is_empty() {
                                        continue;
                                    }

                                    // Check if player exists or needs creation
                                    let player_exists = self.media_players.contains_key(&part_id);

                                    if !player_exists {
                                        match mapmap_media::open_path(&path) {
                                            Ok(player) => {
                                                self.media_players.insert(part_id, player);
                                            }
                                            Err(e) => {
                                                // Log error only once per failure to avoid spam?
                                                // For now just error
                                                error!("Failed to load media '{}': {}", path, e);
                                                continue;
                                            }
                                        }
                                    }

                                    if let Some(player) = self.media_players.get_mut(&part_id) {
                                        // Trigger update
                                        if trigger_value > 0.1 {
                                            let _ =
                                                player.command_sender().send(PlaybackCommand::Play);
                                        }

                                        // Update player with fixed DT for now (should use real DT)
                                        if let Some(frame) =
                                            player.update(std::time::Duration::from_millis(16))
                                        {
                                            // Upload to texture pool
                                            if let mapmap_io::format::FrameData::Cpu(data) =
                                                &frame.data
                                            {
                                                let tex_name = format!("part_{}", part_id);
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
                                    trigger_value: _,
                                } => {
                                    #[cfg(feature = "ndi")]
                                    {
                                        if let Some(src_name) = _source_name {
                                            let receiver = self.ndi_receivers.entry(part_id).or_insert_with(|| {
                                                info!("Creating NDI receiver for part {} source {}", part_id, src_name);
                                                 mapmap_io::ndi::NdiReceiver::new().expect("Failed to create NDI receiver")
                                            });

                                            if let Ok(Some(frame)) = receiver
                                                .receive(std::time::Duration::from_millis(0))
                                            {
                                                if let mapmap_io::format::FrameData::Cpu(data) =
                                                    &frame.data
                                                {
                                                    let tex_name = format!("part_{}", part_id);
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
                                }
                                _ => {}
                            }
                        }

                        // 2. Handle Render Ops (New System)
                        self.render_ops = result.render_ops;

                        // Update Output Assignments for Preview
                        self.output_assignments.clear();
                        for op in &self.render_ops {
                            if let mapmap_core::module::OutputType::Projector { id, .. } =
                                &op.output_type
                            {
                                if let Some(source_id) = op.source_part_id {
                                    let tex_name = format!("part_{}", source_id);
                                    self.output_assignments.insert(*id, tex_name);
                                }
                            }
                        }

                        // 3. Sync output windows with evaluation result
                        let render_ops_clone = self.render_ops.clone();
                        if let Err(e) = self.sync_output_windows(elwt, &render_ops_clone) {
                            error!("Failed to sync output windows: {}", e);
                        }
                    }
                }

                // Log every second for debugging
                static mut LAST_LOG_SEC: i64 = 0;
                let current_sec = timestamp as i64;
                unsafe {
                    if current_sec != LAST_LOG_SEC {
                        LAST_LOG_SEC = current_sec;
                        tracing::debug!(
                            "AudioV2: {} samples, RMS={:.4}, Peak={:.4}, Bands[0..3]={:?}",
                            samples_len,
                            analysis_v2.rms_volume,
                            analysis_v2.peak_volume,
                            &analysis_v2.band_energies[..3]
                        );
                    }
                }

                // Convert V2 analysis to legacy format for UI compatibility
                let legacy_analysis = mapmap_core::audio::AudioAnalysis {
                    timestamp: analysis_v2.timestamp,
                    fft_magnitudes: analysis_v2.fft_magnitudes.clone(),
                    band_energies: [
                        analysis_v2.band_energies[0], // SubBass
                        analysis_v2.band_energies[1], // Bass
                        analysis_v2.band_energies[2], // LowMid
                        analysis_v2.band_energies[3], // Mid
                        analysis_v2.band_energies[4], // HighMid
                        analysis_v2.band_energies[5], // UpperMid (Presence in V1)
                        analysis_v2.band_energies[6], // Presence (Brilliance in V1)
                    ],
                    rms_volume: analysis_v2.rms_volume,
                    peak_volume: analysis_v2.peak_volume,
                    beat_detected: analysis_v2.beat_detected,
                    beat_strength: analysis_v2.beat_strength,
                    onset_detected: false, // Not implemented in V2 yet
                    tempo_bpm: analysis_v2.tempo_bpm, // Now from AudioAnalyzerV2!
                    waveform: analysis_v2.waveform.clone(),
                };

                self.ui_state.dashboard.set_audio_analysis(legacy_analysis);

                // Update module canvas with audio trigger data
                self.ui_state
                    .module_canvas
                    .set_audio_data(mapmap_ui::AudioTriggerData {
                        band_energies: analysis_v2.band_energies,
                        rms_volume: analysis_v2.rms_volume,
                        peak_volume: analysis_v2.peak_volume,
                        beat_detected: analysis_v2.beat_detected,
                        beat_strength: analysis_v2.beat_strength,
                        bpm: analysis_v2.tempo_bpm, // BPM from beat tracking!
                    });

                // Update BPM in toolbar
                self.ui_state.current_bpm = analysis_v2.tempo_bpm;

                // Update Effect Automation
                let now = std::time::Instant::now();
                let delta_time = now.duration_since(self.last_update).as_secs_f64();
                self.last_update = now;

                let _param_updates = self.state.effect_animator.update(delta_time);
                // TODO: Apply param_updates to renderer (EffectChainRenderer needs update_params method)

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

    /// Synchronizes output windows with the current module evaluation result.
    ///
    /// Creates windows for new output assignments and removes windows that are no longer needed.
    /// Synchronizes output windows and NDI senders with the current module graph output nodes.
    fn sync_output_windows<T>(
        &mut self,
        event_loop: &winit::event_loop::EventLoopWindowTarget<T>,
        render_ops: &[mapmap_core::module_eval::RenderOp],
    ) -> Result<()> {
        use mapmap_core::module::OutputType;
        const PREVIEW_FLAG: u64 = 1u64 << 63;

        // Track active IDs for cleanup
        let mut active_window_ids = std::collections::HashSet::new();
        let mut active_sender_ids = std::collections::HashSet::new();

        // 1. Process RenderOps
        for op in render_ops {
            let output_id = op.output_part_id;

            // -- Projector Logic --
            match &op.output_type {
                OutputType::Projector {
                    id: _,
                    name,
                    fullscreen,
                    hide_cursor,
                    target_screen,
                    show_in_preview_panel: _,
                    extra_preview_window,
                } => {
                    // 1. Primary Window
                    active_window_ids.insert(output_id);

                    if let Some(window_context) = self.window_manager.get(output_id) {
                        // Update existing
                        let is_fullscreen = window_context.window.fullscreen().is_some();
                        if is_fullscreen != *fullscreen {
                            window_context.window.set_fullscreen(if *fullscreen {
                                Some(winit::window::Fullscreen::Borderless(None))
                            } else {
                                None
                            });
                        }
                        window_context.window.set_cursor_visible(!*hide_cursor);
                    } else {
                        // Create new
                        self.window_manager.create_projector_window(
                            event_loop,
                            &self.backend,
                            output_id,
                            name,
                            *fullscreen,
                            *hide_cursor,
                            *target_screen,
                        )?;
                        info!("Created projector window for output {}", output_id);
                    }

                    // 2. Extra Preview Window
                    if *extra_preview_window {
                        let preview_id = output_id | PREVIEW_FLAG;
                        active_window_ids.insert(preview_id);

                        // Ensure render assignment exists for preview
                        self.output_assignments.insert(
                            preview_id,
                            op.source_part_id
                                .map(|id| format!("part_{}", id))
                                .unwrap_or_default(),
                        );

                        if self.window_manager.get(preview_id).is_none() {
                            self.window_manager.create_projector_window(
                                event_loop,
                                &self.backend,
                                preview_id,
                                &format!("Preview: {}", name),
                                false, // Always windowed
                                false, // Show cursor
                                0,     // Default screen (0)
                            )?;
                            info!("Created preview window for output {}", output_id);
                        }
                    }
                }
                OutputType::NdiOutput { name: _name } => {
                    // -- NDI Logic --
                    active_sender_ids.insert(output_id);

                    #[cfg(feature = "ndi")]
                    {
                        if !self.ndi_senders.contains_key(&output_id) {
                            // Create NDI Sender
                            let width = 1920; // TODO: Dynamic Res
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
                                Err(e) => error!("Failed to create NDI sender {}: {}", _name, e),
                            }
                        }
                    }
                }
                #[cfg(target_os = "windows")]
                OutputType::Spout { .. } => {
                    // TODO: Spout Sender
                }
            }
        }

        // 2. Cleanup Windows
        let window_ids: Vec<u64> = self.window_manager.window_ids().cloned().collect();
        for id in window_ids {
            if id != 0 && !active_window_ids.contains(&id) {
                self.window_manager.remove_window(id);
                // Also remove assignment if it was a preview
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

    /// Renders the controls panel content (Media and Audio sections)
    fn render_controls_content(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("📁 Media")
            .default_open(false)
            .show(ui, |ui| {
                let _ = self.ui_state.media_browser.ui(
                    ui,
                    &self.ui_state.i18n,
                    self.ui_state.icon_manager.as_ref(),
                );
            });
        egui::CollapsingHeader::new("🔊 Audio")
            .default_open(false)
            .show(ui, |ui| {
                let analysis_v2 = self.audio_analyzer.get_latest_analysis();
                let legacy_analysis = if self.audio_backend.is_some() {
                    Some(mapmap_core::audio::AudioAnalysis {
                        timestamp: analysis_v2.timestamp,
                        fft_magnitudes: analysis_v2.fft_magnitudes.clone(),
                        band_energies: [
                            analysis_v2.band_energies[0],
                            analysis_v2.band_energies[1],
                            analysis_v2.band_energies[2],
                            analysis_v2.band_energies[3],
                            analysis_v2.band_energies[4],
                            analysis_v2.band_energies[5],
                            analysis_v2.band_energies[6],
                        ],
                        rms_volume: analysis_v2.rms_volume,
                        peak_volume: analysis_v2.peak_volume,
                        beat_detected: analysis_v2.beat_detected,
                        beat_strength: analysis_v2.beat_strength,
                        onset_detected: false,
                        tempo_bpm: None,
                        waveform: analysis_v2.waveform.clone(),
                    })
                } else {
                    None
                };

                if let Some(action) = self.ui_state.audio_panel.ui(
                    ui,
                    &self.ui_state.i18n,
                    legacy_analysis.as_ref(),
                    &self.state.audio_config,
                    &self.audio_devices,
                    &mut self.ui_state.selected_audio_device,
                ) {
                    match action {
                        mapmap_ui::audio_panel::AudioPanelAction::DeviceChanged(device) => {
                            info!("Audio device changed to: {}", device);
                            self.ui_state
                                .user_config
                                .set_audio_device(Some(device.clone()));
                            self.audio_analyzer.reset();
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
                                    error!(
                                        "Failed to create audio backend for device '{}': {}",
                                        device, e
                                    );
                                }
                            }
                        }
                        mapmap_ui::audio_panel::AudioPanelAction::ConfigChanged(cfg) => {
                            self.audio_analyzer.update_config(
                                mapmap_core::audio::analyzer_v2::AudioAnalyzerV2Config {
                                    sample_rate: cfg.sample_rate,
                                    fft_size: cfg.fft_size,
                                    overlap: cfg.overlap,
                                    smoothing: cfg.smoothing,
                                },
                            );
                            self.state.audio_config = cfg;
                        }
                    }
                }
            });
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

            // Sync Texture Previews for Module Canvas
            {
                // Free old textures
                let ids_to_free: Vec<egui::TextureId> = self
                    .ui_state
                    .module_canvas
                    .node_previews
                    .values()
                    .cloned()
                    .collect();
                for id in ids_to_free {
                    self.egui_renderer.free_texture(&id);
                }
                self.ui_state.module_canvas.node_previews.clear();

                // Register new textures for active sources
                let active_part_ids: Vec<u64> = self
                    .state
                    .module_manager
                    .modules()
                    .iter()
                    .flat_map(|m| &m.parts)
                    .filter_map(|p| {
                        if let mapmap_core::module::ModulePartType::Source(_) = p.part_type {
                            Some(p.id)
                        } else {
                            None
                        }
                    })
                    .collect();

                for part_id in active_part_ids {
                    let tex_name = format!("part_{}", part_id);
                    if self.texture_pool.has_texture(&tex_name) {
                        let view = self.texture_pool.get_view(&tex_name);
                        let tex_id = self.egui_renderer.register_native_texture(
                            &self.backend.device,
                            &view,
                            wgpu::FilterMode::Linear,
                        );
                        self.ui_state
                            .module_canvas
                            .node_previews
                            .insert(part_id, tex_id);
                    }
                }
            }

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

                    // MIDI Controller Overlay
                    #[cfg(feature = "midi")]
                    {
                        let midi_connected = self.midi_handler.as_ref().map(|h| h.is_connected()).unwrap_or(false);
                        self.ui_state.controller_overlay.show(ctx, self.ui_state.show_controller_overlay, midi_connected, &mut self.ui_state.user_config);
                    }

                    // === 1. TOP PANEL: Menu Bar + Toolbar ===
                    let menu_actions = menu_bar::show(ctx, &mut self.ui_state);
                    self.ui_state.actions.extend(menu_actions);

                    // === Effect Chain Panel ===
                    self.ui_state.effect_chain_panel.ui(
                        ctx,
                        &self.ui_state.i18n,
                        self.ui_state.icon_manager.as_ref(),
                        Some(&mut self.recent_effect_configs),
                    );

                    // Render Oscillator Panel
                    self.ui_state.oscillator_panel.render(
                        ctx,
                        &self.ui_state.i18n,
                        &mut self.state.oscillator_config,
                    );

                    // Handle Effect Chain Actions
                    for action in self.ui_state.effect_chain_panel.take_actions() {
                        use mapmap_ui::effect_chain_panel::{EffectChainAction, EffectType as UIEffectType};
                        use mapmap_core::EffectType as RenderEffectType;

                        match action {
                            EffectChainAction::AddEffectWithParams(ui_type, params) => {
                                let render_type = match ui_type {
                                    UIEffectType::Blur => RenderEffectType::Blur,
                                    UIEffectType::ColorAdjust => RenderEffectType::ColorAdjust,
                                    UIEffectType::ChromaticAberration => RenderEffectType::ChromaticAberration,
                                    UIEffectType::EdgeDetect => RenderEffectType::EdgeDetect,
                                    UIEffectType::Glow => RenderEffectType::Glow,
                                    UIEffectType::Kaleidoscope => RenderEffectType::Kaleidoscope,
                                    UIEffectType::Invert => RenderEffectType::Invert,
                                    UIEffectType::Pixelate => RenderEffectType::Pixelate,
                                    UIEffectType::Vignette => RenderEffectType::Vignette,
                                    UIEffectType::FilmGrain => RenderEffectType::FilmGrain,
                                    UIEffectType::Custom => RenderEffectType::Custom,
                                };

                                let id = self.state.effect_chain.add_effect(render_type);
                                if let Some(effect) = self.state.effect_chain.get_effect_mut(id) {
                                    for (k, v) in &params {
                                        effect.set_param(k, *v);
                                    }
                                }

                                self.recent_effect_configs.add_float_config(&format!("{:?}", ui_type), params);
                            }
                            EffectChainAction::AddEffect(ui_type) => {
                                let render_type = match ui_type {
                                    UIEffectType::Blur => RenderEffectType::Blur,
                                    UIEffectType::ColorAdjust => RenderEffectType::ColorAdjust,
                                    UIEffectType::ChromaticAberration => RenderEffectType::ChromaticAberration,
                                    UIEffectType::EdgeDetect => RenderEffectType::EdgeDetect,
                                    UIEffectType::Glow => RenderEffectType::Glow,
                                    UIEffectType::Kaleidoscope => RenderEffectType::Kaleidoscope,
                                    UIEffectType::Invert => RenderEffectType::Invert,
                                    UIEffectType::Pixelate => RenderEffectType::Pixelate,
                                    UIEffectType::Vignette => RenderEffectType::Vignette,
                                    UIEffectType::FilmGrain => RenderEffectType::FilmGrain,
                                    UIEffectType::Custom => RenderEffectType::Custom,
                                };
                                self.state.effect_chain.add_effect(render_type);
                            }
                            EffectChainAction::ClearAll => {
                                self.state.effect_chain.effects.clear();
                            }
                            EffectChainAction::RemoveEffect(id) => {
                                self.state.effect_chain.remove_effect(id);
                            }
                            EffectChainAction::MoveUp(id) => {
                                self.state.effect_chain.move_up(id);
                            }
                            EffectChainAction::MoveDown(id) => {
                                self.state.effect_chain.move_down(id);
                            }
                            EffectChainAction::ToggleEnabled(id) => {
                                if let Some(effect) = self.state.effect_chain.get_effect_mut(id) {
                                    effect.enabled = !effect.enabled;
                                }
                            }
                            EffectChainAction::SetIntensity(id, val) => {
                                if let Some(effect) = self.state.effect_chain.get_effect_mut(id) {
                                    effect.intensity = val;
                                }
                            }
                            EffectChainAction::SetParameter(id, name, val) => {
                                if let Some(effect) = self.state.effect_chain.get_effect_mut(id) {
                                    effect.set_param(&name, val);
                                }
                            }
                            _ => {}
                        }
                    }

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

                                if let Some(action) = self.ui_state.timeline_panel.ui(ui, &mut self.state.effect_animator) {
                                     use mapmap_ui::timeline_v2::TimelineAction;
                                     match action {
                                         TimelineAction::Play => self.state.effect_animator.play(),
                                         TimelineAction::Pause => self.state.effect_animator.pause(),
                                         TimelineAction::Stop => self.state.effect_animator.stop(),
                                         TimelineAction::Seek(t) => self.state.effect_animator.seek(t as f64),
                                     }
                                }
                            });
                    }

                    // === 3. LEFT SIDEBAR (Combined: Controls + Preview) ===
                    let show_controls = self.ui_state.show_left_sidebar;
                    let show_preview = self.ui_state.show_preview_panel;

                    if show_controls || show_preview {
                        egui::SidePanel::left("main_combined_sidebar")
                            .resizable(true)
                            .default_width(300.0)
                            .min_width(200.0)
                            .max_width(500.0)
                            .show(ctx, |ui| {
                                // 1. CONTROLS PANEL (Top)
                                if show_controls {
                                    // If preview is also visible, wrap controls in a resizable top panel
                                    if show_preview {
                                        egui::TopBottomPanel::top("controls_sub_panel")
                                            .resizable(true)
                                            .min_height(200.0)
                                            .default_height(ui.available_height() * 0.6)
                                            .show_inside(ui, |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.heading("Controls");
                                                    ui.with_layout(
                                                        egui::Layout::right_to_left(
                                                            egui::Align::Center,
                                                        ),
                                                        |ui| {
                                                            if ui
                                                                .button("◀")
                                                                .on_hover_text(
                                                                    "Controls ausblenden",
                                                                )
                                                                .clicked()
                                                            {
                                                                self.ui_state.show_left_sidebar =
                                                                    false;
                                                            }
                                                        },
                                                    );
                                                });
                                                ui.separator();

                                                // INLINED CONTROLS CONTENT (Split View)
                                                egui::ScrollArea::vertical()
                                                    .id_source("controls_sidebar_scroll_split")
                                                    .show(ui, |ui| {
                                                        self.render_controls_content(ui);
                                                    });
                                            });

                                        ui.add_space(4.0); // Spacing between panels
                                    } else {
                                        // Controls takes full height
                                        ui.horizontal(|ui| {
                                            ui.heading("Controls");
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    if ui
                                                        .button("◀")
                                                        .on_hover_text("Sidebar einklappen")
                                                        .clicked()
                                                    {
                                                        self.ui_state.show_left_sidebar = false;
                                                    }
                                                },
                                            );
                                        });
                                        ui.separator();

                                        // INLINED CONTROLS CONTENT (Full View)
                                        egui::ScrollArea::vertical()
                                            .id_source("controls_sidebar_scroll_full")
                                            .show(ui, |ui| {
                                                self.render_controls_content(ui);
                                            });
                                    }
                                }

                                // 2. PREVIEW PANEL (Bottom)
                                if show_preview {
                                    if show_controls {
                                        ui.separator();
                                    }

                                    // Update preview info logic
                                    let output_infos: Vec<mapmap_ui::OutputPreviewInfo> = self
                                        .state
                                        .module_manager
                                        .modules()
                                        .iter()
                                        .flat_map(|module| {
                                            module.parts.iter().filter_map(|part| {
                                                if let mapmap_core::module::ModulePartType::Output(
                                                    output_type,
                                                ) = &part.part_type
                                                {
                                                    match output_type {
                                                        mapmap_core::module::OutputType::Projector {
                                                            ref id,
                                                            ref name,
                                                            ref show_in_preview_panel,
                                                            ..
                                                        } => {
                                                            Some(mapmap_ui::OutputPreviewInfo {
                                                                id: *id,
                                                                name: name.clone(),
                                                                show_in_panel: *show_in_preview_panel,
                                                                texture_name: self
                                                                    .output_assignments
                                                                    .get(id)
                                                                    .cloned(),
                                                            })
                                                        }
                                                        _ => None,
                                                    }
                                                } else {
                                                    None
                                                }
                                            })
                                        })
                                        .collect();

                                    self.ui_state.preview_panel.update_outputs(output_infos);
                                    self.ui_state.preview_panel.show(ui);
                                }
                            });
                    } else {
                        // Collapsed Sidebar Strip
                        egui::SidePanel::left("left_sidebar_collapsed")
                            .exact_width(28.0)
                            .resizable(false)
                            .show(ctx, |ui| {
                                ui.vertical_centered(|ui| {
                                    if ui
                                        .button("C")
                                        .on_hover_text("Controls öffnen")
                                        .clicked()
                                    {
                                        self.ui_state.show_left_sidebar = true;
                                    }
                                    ui.add_space(8.0);
                                    if ui
                                        .button("P")
                                        .on_hover_text("Preview öffnen")
                                        .clicked()
                                    {
                                        self.ui_state.show_preview_panel = true;
                                    }
                                });
                            });
                    }

                    // === 5. CENTRAL PANEL: Module Canvas ===
                    egui::CentralPanel::default().show(ctx, |ui| {
                        if self.ui_state.show_module_canvas {
                            // Update available outputs for the ModuleCanvas dropdown
                            self.ui_state.module_canvas.available_outputs = self
                                .state
                                .output_manager
                                .outputs()
                                .iter()
                                .map(|o| (o.id, o.name.clone()))
                                .collect();

                            self.ui_state.module_canvas.show(
                                ui,
                                &mut self.state.module_manager,
                                &self.ui_state.i18n,
                                &mut self.ui_state.actions,
                            );
                        } else {
                            // Placeholder for normal canvas/viewport
                            ui.centered_and_justified(|ui| {
                                ui.label("Canvas - Module Canvas deaktiviert (View → Module Canvas)");
                            });
                        }
                    });

                    // === Settings Window (only modal allowed) ===
                    let mut show_settings = self.ui_state.show_settings;
                    let mut explicit_close = false;

                    if show_settings {
                        egui::Window::new(self.ui_state.i18n.t("menu-file-settings"))
                            .id(egui::Id::new("app_settings_window"))
                            .collapsible(false)
                            .resizable(true)
                            .default_size([400.0, 300.0])
                            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                            .open(&mut show_settings)
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

                                // Output/Projector Settings
                                egui::CollapsingHeader::new(format!("📽️ {}", self.ui_state.i18n.t("settings-outputs")))
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label("Number of Outputs (Projectors):");
                                            let mut output_count = self.state.settings.output_count;
                                            if ui.add(egui::DragValue::new(&mut output_count).clamp_range(1..=8)).changed() {
                                                self.state.settings.output_count = output_count;
                                                self.state.dirty = true;
                                            }
                                        });

                                        ui.label("💡 Each output can be assigned to a different screen/projector");

                                        // List current outputs if any
                                        let output_count = self.state.output_manager.outputs().len();
                                        if output_count > 0 {
                                            ui.add_space(8.0);
                                            ui.label(format!("Currently configured: {} outputs", output_count));
                                            for output in self.state.output_manager.outputs() {
                                                ui.label(format!("  • {} (ID: {})", output.name, output.id));
                                            }
                                        } else {
                                            ui.add_space(8.0);
                                            ui.label("⚠️ No outputs configured yet. Add an Output node in the Module Canvas.");
                                        }
                                    });

                                ui.separator();

                                // Logging Settings
                                egui::CollapsingHeader::new(format!("📝 {}", self.ui_state.i18n.t("settings-logging")))
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        let log_config = &mut self.state.settings.log_config;

                                        ui.horizontal(|ui| {
                                            ui.label("Log Level:");
                                            let levels = ["trace", "debug", "info", "warn", "error"];
                                            egui::ComboBox::from_id_source("log_level_select")
                                                .selected_text(&log_config.level)
                                                .show_ui(ui, |ui| {
                                                    for level in levels {
                                                        if ui.selectable_label(log_config.level == level, level).clicked() {
                                                            log_config.level = level.to_string();
                                                            self.state.dirty = true;
                                                        }
                                                    }
                                                });
                                        });

                                        ui.horizontal(|ui| {
                                            ui.label("Log Path:");
                                            let path_str = log_config.log_path.to_string_lossy().to_string();
                                            let mut path_edit = path_str.clone();
                                            if ui.text_edit_singleline(&mut path_edit).changed() {
                                                log_config.log_path = std::path::PathBuf::from(path_edit);
                                                self.state.dirty = true;
                                            }
                                            if ui.button("📂").clicked() {
                                                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                                    log_config.log_path = folder;
                                                    self.state.dirty = true;
                                                }
                                            }
                                        });

                                        ui.horizontal(|ui| {
                                            ui.label("Max Log Files:");
                                            if ui.add(egui::DragValue::new(&mut log_config.max_files).speed(1).clamp_range(1..=100)).changed() {
                                                self.state.dirty = true;
                                            }
                                        });

                                        ui.horizontal(|ui| {
                                            if ui.checkbox(&mut log_config.console_output, "Console Output").changed() {
                                                self.state.dirty = true;
                                            }
                                            if ui.checkbox(&mut log_config.file_output, "File Output").changed() {
                                                self.state.dirty = true;
                                            }
                                        });
                                    });

                                ui.separator();

                                // MIDI Settings
                                #[cfg(feature = "midi")]
                                egui::CollapsingHeader::new(format!("🎹 {}", "MIDI"))
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        // Connection status
                                        let is_connected = self.midi_handler.as_ref().map(|h| h.is_connected()).unwrap_or(false);
                                        ui.horizontal(|ui| {
                                            ui.label("Status:");
                                            if is_connected {
                                                ui.colored_label(egui::Color32::GREEN, "🟢 Connected");
                                            } else {
                                                ui.colored_label(egui::Color32::RED, "🔴 Disconnected");
                                            }
                                        });

                                        // Port selection
                                        ui.horizontal(|ui| {
                                            ui.label("MIDI Port:");
                                            let current_port = self.selected_midi_port
                                                .and_then(|i| self.midi_ports.get(i))
                                                .cloned()
                                                .unwrap_or_else(|| "None".to_string());

                                            egui::ComboBox::from_id_source("midi_port_select")
                                                .selected_text(&current_port)
                                                .show_ui(ui, |ui| {
                                                    for (i, port) in self.midi_ports.iter().enumerate() {
                                                        if ui.selectable_label(self.selected_midi_port == Some(i), port).clicked() {
                                                            self.selected_midi_port = Some(i);
                                                            // Connect to the selected port
                                                            if let Some(handler) = &mut self.midi_handler {
                                                                handler.disconnect();
                                                                match handler.connect(i) {
                                                                    Ok(()) => {
                                                                        info!("Connected to MIDI port: {}", port);
                                                                    }
                                                                    Err(e) => {
                                                                        error!("Failed to connect to MIDI port: {}", e);
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                });
                                        });

                                        // Refresh ports button
                                        if ui.button("🔄 Refresh Ports").clicked() {
                                            if let Ok(ports) = MidiInputHandler::list_ports() {
                                                self.midi_ports = ports;
                                                info!("Refreshed MIDI ports: {:?}", self.midi_ports);
                                            }
                                        }

                                        // Show available ports count
                                        ui.label(format!("{} port(s) available", self.midi_ports.len()));
                                    });

                                ui.separator();
                                if ui.button("✕ Schließen").clicked() {
                                    explicit_close = true;
                                }
                            });
                    }

                    if explicit_close {
                        show_settings = false;
                    }
                    self.ui_state.show_settings = show_settings;
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

            // Handle Global UI Actions
            for action in self.ui_state.take_actions() {
                // TODO: Handle Play, Pause, etc.
                if let mapmap_ui::UIAction::SetMidiAssignment(element_id, target_id) = action {
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
            // === Node-Based Rendering Pipeline ===

            // 1. Find the RenderOp for this output
            let target_op = self.render_ops.iter().find(|op| {
                if let mapmap_core::module::OutputType::Projector { id, .. } = &op.output_type {
                    *id == output_id
                } else {
                    false
                }
            });

            if let Some(op) = target_op {
                // Determine source texture view
                let owned_source_view = if let Some(src_id) = op.source_part_id {
                    let tex_name = format!("part_{}", src_id);
                    if self.texture_pool.has_texture(&tex_name) {
                        Some(self.texture_pool.get_view(&tex_name))
                    } else {
                        None
                    }
                } else {
                    None
                };
                let source_view_ref = owned_source_view.as_ref();
                let effective_view = source_view_ref.or(self.dummy_view.as_ref());

                if let Some(src_view) = effective_view {
                    // --- 1. Effect Chain Processing (Common) ---
                    let mut final_view = src_view;
                    let mut _temp_view_holder: Option<wgpu::TextureView> = None;

                    if !op.effects.is_empty() {
                        let time = self.start_time.elapsed().as_secs_f32();
                        let mut chain = mapmap_core::EffectChain::new();

                        for modulizer in &op.effects {
                            if let mapmap_core::module::ModulizerType::Effect {
                                effect_type: mod_effect,
                                params,
                            } = modulizer
                            {
                                let core_effect = match mod_effect {
                                    mapmap_core::module::EffectType::Blur => {
                                        Some(mapmap_core::effects::EffectType::Blur)
                                    }
                                    mapmap_core::module::EffectType::Invert => {
                                        Some(mapmap_core::effects::EffectType::Invert)
                                    }
                                    mapmap_core::module::EffectType::Pixelate => {
                                        Some(mapmap_core::effects::EffectType::Pixelate)
                                    }
                                    mapmap_core::module::EffectType::Brightness
                                    | mapmap_core::module::EffectType::Contrast
                                    | mapmap_core::module::EffectType::Saturation => {
                                        Some(mapmap_core::effects::EffectType::ColorAdjust)
                                    }
                                    mapmap_core::module::EffectType::ChromaticAberration => {
                                        Some(mapmap_core::effects::EffectType::ChromaticAberration)
                                    }
                                    mapmap_core::module::EffectType::EdgeDetect => {
                                        Some(mapmap_core::effects::EffectType::EdgeDetect)
                                    }
                                    mapmap_core::module::EffectType::FilmGrain => {
                                        Some(mapmap_core::effects::EffectType::FilmGrain)
                                    }
                                    mapmap_core::module::EffectType::Vignette => {
                                        Some(mapmap_core::effects::EffectType::Vignette)
                                    }
                                    _ => None,
                                };
                                if let Some(et) = core_effect {
                                    let effect_id = chain.add_effect(et);
                                    if let Some(effect) = chain.get_effect_mut(effect_id) {
                                        effect.parameters = params.clone();
                                    }
                                }
                            }
                        }

                        if chain.enabled_effects().count() > 0 {
                            let target_tex_name = &self.layer_ping_pong[0];
                            let (w, h) = (
                                window_context.surface_config.width,
                                window_context.surface_config.height,
                            );
                            self.texture_pool.resize_if_needed(target_tex_name, w, h);
                            let target_view = self.texture_pool.get_view(target_tex_name);
                            self.effect_chain_renderer.apply_chain(
                                &mut encoder,
                                src_view,
                                &target_view,
                                &chain,
                                time,
                                w,
                                h,
                            );
                            _temp_view_holder = Some(target_view);
                            final_view = _temp_view_holder.as_ref().unwrap();
                        }
                    }

                    // --- 2. Advanced Output OR Mesh Rendering ---
                    let output_config_opt = self.state.output_manager.get_output(output_id);
                    let use_edge_blend =
                        output_config_opt.is_some() && self.edge_blend_renderer.is_some();
                    let use_color_calib =
                        output_config_opt.is_some() && self.color_calibration_renderer.is_some();

                    if use_edge_blend || use_color_calib {
                        // === ADVANCED RENDERING PIPELINE ===
                        let need_temp = use_edge_blend && use_color_calib;
                        let mut temp_view_opt: Option<wgpu::TextureView> = None;

                        if need_temp {
                            let width = window_context.surface_config.width;
                            let height = window_context.surface_config.height;
                            let recreate =
                                if let Some(tex) = self.output_temp_textures.get(&output_id) {
                                    tex.width() != width || tex.height() != height
                                } else {
                                    true
                                };

                            if recreate {
                                let texture =
                                    self.backend
                                        .device
                                        .create_texture(&wgpu::TextureDescriptor {
                                            label: Some(&format!(
                                                "Output {} Temp Texture",
                                                output_id
                                            )),
                                            size: wgpu::Extent3d {
                                                width,
                                                height,
                                                depth_or_array_layers: 1,
                                            },
                                            mip_level_count: 1,
                                            sample_count: 1,
                                            dimension: wgpu::TextureDimension::D2,
                                            format: self.backend.surface_format(),
                                            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                                                | wgpu::TextureUsages::TEXTURE_BINDING,
                                            view_formats: &[],
                                        });
                                self.output_temp_textures.insert(output_id, texture);
                            }
                            temp_view_opt = Some(
                                self.output_temp_textures
                                    .get(&output_id)
                                    .unwrap()
                                    .create_view(&wgpu::TextureViewDescriptor::default()),
                            );
                        }

                        let config = output_config_opt.unwrap();

                        if use_edge_blend {
                            let renderer = self.edge_blend_renderer.as_ref().unwrap();
                            let target_view = if use_color_calib {
                                temp_view_opt.as_ref().unwrap()
                            } else {
                                &view
                            };
                            let bind_group = renderer.create_texture_bind_group(final_view);
                            let uniform_buffer = renderer.create_uniform_buffer(&config.edge_blend);
                            let uniform_bind_group =
                                renderer.create_uniform_bind_group(&uniform_buffer);

                            let mut render_pass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some("Edge Blend Pass"),
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view: target_view,
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
                            renderer.render(&mut render_pass, &bind_group, &uniform_bind_group);
                        }

                        if use_color_calib {
                            let renderer = self.color_calibration_renderer.as_ref().unwrap();
                            let input_view_for_cc = if use_edge_blend {
                                temp_view_opt.as_ref().unwrap()
                            } else {
                                final_view
                            };
                            let target_view = &view;
                            let bind_group = renderer.create_texture_bind_group(input_view_for_cc);
                            let uniform_buffer =
                                renderer.create_uniform_buffer(&config.color_calibration);
                            let uniform_bind_group =
                                renderer.create_uniform_bind_group(&uniform_buffer);

                            let mut render_pass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some("Color Calibration Pass"),
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view: target_view,
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
                            renderer.render(&mut render_pass, &bind_group, &uniform_bind_group);
                        }
                    } else {
                        // === MESH RENDERING (Default/Keystone) ===
                        {
                            let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label: Some("Output Clear Pass"),
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
                        }
                        self.mesh_renderer.begin_frame();
                        let (vertex_buffer, index_buffer, index_count) =
                            self.mesh_buffer_cache.get_buffers(
                                &self.backend.device,
                                op.layer_part_id,
                                &op.mesh.to_mesh(),
                            );
                        let transform = glam::Mat4::IDENTITY;
                        let uniform_bind_group = self.mesh_renderer.get_uniform_bind_group(
                            &self.backend.queue,
                            transform,
                            op.opacity,
                        );
                        let texture_bind_group =
                            self.mesh_renderer.create_texture_bind_group(final_view);

                        {
                            let mut render_pass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some("Mesh Render Pass"),
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view: &view,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Load,
                                            store: wgpu::StoreOp::Store,
                                        },
                                    })],
                                    depth_stencil_attachment: None,
                                    occlusion_query_set: None,
                                    timestamp_writes: None,
                                });
                            self.mesh_renderer.draw(
                                &mut render_pass,
                                vertex_buffer,
                                index_buffer,
                                index_count,
                                &uniform_bind_group,
                                &texture_bind_group,
                                true,
                            );
                        }
                    }
                }
            } else {
                // No op for this output - Clear to Black
                let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Clear Pass"),
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
            }
        }

        self.backend.queue.submit(Some(encoder.finish()));
        surface_texture.present();

        Ok(())
    }
}

mod logging_setup;

/// The main entry point for the application.
fn main() -> Result<()> {
    // Initialize logging with default configuration
    // This creates a log file in logs/ and outputs to console
    let _log_guard = logging_setup::init(&mapmap_core::logging::LogConfig::default())?;

    info!("==========================================");
    info!("===      MapFlow Session Started       ===");
    info!("==========================================");

    // Create the event loop
    let event_loop = EventLoop::new()?;

    // Create the app
    let app = pollster::block_on(App::new(&event_loop))?;

    // Run the app
    info!("--- Entering Main Event Loop ---");
    app.run(event_loop);

    info!("==========================================");
    info!("===       MapFlow Session Ended        ===");
    info!("==========================================");

    Ok(())
}
