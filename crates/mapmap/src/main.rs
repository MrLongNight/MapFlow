//! MapFlow - Open source Vj Projection Mapping Software
//!
//! This is the main application crate for MapFlow.

#![warn(missing_docs)]

mod window_manager;

use anyhow::Result;
use egui_wgpu::Renderer;
use egui_winit::State;
use mapmap_control::hue::controller::HueController;
#[cfg(feature = "midi")]
use mapmap_control::midi::MidiInputHandler;
use mapmap_control::{shortcuts::Action, ControlManager};
use mapmap_core::{
    audio::{
        analyzer_v2::{AudioAnalyzerV2, AudioAnalyzerV2Config},
        backend::cpal_backend::CpalBackend,
        backend::AudioBackend,
    },
    audio_reactive::AudioTriggerData,
    AppState, ModuleEvaluator, OutputId, RenderOp,
};

use mapmap_mcp::{McpAction, McpServer};
// Define McpAction locally or import if we move it to core later -> Removed local definition

use crossbeam_channel::{unbounded, Receiver};
use mapmap_core::module::{ModulePartId, ModulePartType, SourceType};
use mapmap_io::{load_project, save_project};
use mapmap_media::player::{PlaybackCommand, VideoPlayer};
use mapmap_render::{
    ColorCalibrationRenderer, Compositor, EdgeBlendRenderer, EffectChainRenderer, MeshBufferCache,
    MeshRenderer, OscillatorRenderer, QuadRenderer, TexturePool, WgpuBackend,
};
use mapmap_ui::{menu_bar, AppUI, EdgeBlendAction};
use rfd::FileDialog;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::thread;
use tracing::{debug, error, info, warn};

use window_manager::WindowManager;
use winit::{event::WindowEvent, event_loop::EventLoop};

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
    /// Sender for internal actions (async -> sync)
    action_sender: crossbeam_channel::Sender<McpAction>,
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
    /// Active media players for source nodes ((ModuleID, PartID) -> Player)
    media_players: HashMap<(ModulePartId, ModulePartId), (String, VideoPlayer)>,
    /// FPS calculation: accumulated frame times
    fps_samples: VecDeque<f32>,
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
    /// Output assignments (OutputID -> List of Texture Names)
    output_assignments: std::collections::HashMap<u64, Vec<String>>,
    /// Recent Effect Configurations (User Prefs)
    recent_effect_configs: mapmap_core::RecentEffectConfigs,
    /// Render Operations from Module Evaluator ((ModuleID, RenderOp))
    render_ops: Vec<(ModulePartId, RenderOp)>,
    /// Edge blend renderer for output windows
    edge_blend_renderer: Option<EdgeBlendRenderer>,
    /// Color calibration renderer for output windows
    color_calibration_renderer: Option<ColorCalibrationRenderer>,
    /// Temporary textures for output rendering (OutputID -> Texture)
    output_temp_textures: std::collections::HashMap<u64, wgpu::Texture>,
    /// Cache for egui textures to avoid re-registering every frame (PartId -> (EguiId, View))
    preview_texture_cache: HashMap<u64, (egui::TextureId, std::sync::Arc<wgpu::TextureView>)>,
    /// Cache for output preview textures (OutputID -> (EguiTextureId, View))
    output_preview_cache: HashMap<u64, (egui::TextureId, std::sync::Arc<wgpu::TextureView>)>,
    /// Unit Quad buffers for preview rendering (Vertex, Index, IndexCount)
    preview_quad_buffers: (wgpu::Buffer, wgpu::Buffer, u32),
    /// Philips Hue Controller
    hue_controller: HueController,
    /// Tokio runtime for async operations
    tokio_runtime: tokio::runtime::Runtime,
}

impl App {
    /// Creates a new `App`.
    pub async fn new(elwt: &winit::event_loop::ActiveEventLoop) -> Result<Self> {
        let backend = WgpuBackend::new().await?;

        // Version marker to confirm correct build is running
        tracing::info!(">>> BUILD VERSION: 2026-01-04-FIX-RENDER-CHECK <<<");

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

        // Create Tokio runtime
        let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        // Load user config to get saved window geometry
        let saved_config = mapmap_ui::config::UserConfig::load();

        // Create main window with saved geometry
        let main_window_id = window_manager.create_main_window_with_geometry(
            elwt,
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

            // --- SELF-HEAL: Reconcile Output IDs ---
            // Ensure Output Nodes in the graph point to valid IDs in OutputManager.
            // If ID mismatch but NAME match, update the ID.
            let valid_outputs: HashMap<String, u64> = state
                .output_manager
                .outputs()
                .iter()
                .map(|o| (o.name.clone(), o.id))
                .collect();
            let valid_ids: Vec<u64> = valid_outputs.values().cloned().collect();

            let mut fixed_count = 0;
            for module in state.module_manager.modules_mut() {
                for part in &mut module.parts {
                    if let mapmap_core::module::ModulePartType::Output(
                        mapmap_core::module::OutputType::Projector {
                            ref mut id,
                            ref name,
                            ..
                        },
                    ) = &mut part.part_type
                    {
                        if !valid_ids.contains(id) {
                            if let Some(new_id) = valid_outputs.get(name) {
                                info!(
                                    "Self-Heal: Relinking Output '{}' from ID {} to {}.",
                                    name, id, new_id
                                );
                                *id = *new_id;
                                fixed_count += 1;
                            } else {
                                warn!(
                                    "Self-Heal: Output '{}' (ID {}) has no matching Output Window.",
                                    name, id
                                );
                            }
                        }
                    }
                }
            }
            if fixed_count > 0 {
                info!("Self-Heal: Fixed {} output connections.", fixed_count);
                state.dirty = true;
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
        let action_sender = mcp_sender.clone();

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
            egui::viewport::ViewportId::ROOT,
            &main_window_for_egui,
            None,
            None,
            None,
        );
        let egui_renderer = Renderer::new(
            &backend.device,
            format,
            egui_wgpu::RendererOptions::default(),
        );
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

        // Initialize preview quad buffers
        // Use manual construction to ensure -1..1 NDC range coverage for full viewport
        let preview_mesh = mapmap_core::Mesh {
            mesh_type: mapmap_core::MeshType::Quad,
            vertices: vec![
                // Top-Left (NDC -1, 1) -> UV 0,0
                mapmap_core::MeshVertex::new(glam::Vec2::new(-1.0, 1.0), glam::Vec2::new(0.0, 0.0)),
                // Top-Right (NDC 1, 1) -> UV 1,0
                mapmap_core::MeshVertex::new(glam::Vec2::new(1.0, 1.0), glam::Vec2::new(1.0, 0.0)),
                // Bottom-Right (NDC 1, -1) -> UV 1,1
                mapmap_core::MeshVertex::new(glam::Vec2::new(1.0, -1.0), glam::Vec2::new(1.0, 1.0)),
                // Bottom-Left (NDC -1, -1) -> UV 0,1
                mapmap_core::MeshVertex::new(
                    glam::Vec2::new(-1.0, -1.0),
                    glam::Vec2::new(0.0, 1.0),
                ),
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
            revision: 0,
        };
        let preview_quad_buffers = {
            let (vb, ib) = mesh_renderer.create_mesh_buffers(&preview_mesh);
            (vb, ib, preview_mesh.indices.len() as u32)
        };

        // Initialize Hue Controller
        let ui_hue_conf = &ui_state.user_config.hue_config;
        let control_hue_conf = mapmap_control::hue::models::HueConfig {
            bridge_ip: ui_hue_conf.bridge_ip.clone(),
            username: ui_hue_conf.username.clone(),
            client_key: ui_hue_conf.client_key.clone(),
            application_id: String::new(), // Will be fetched if needed
            entertainment_group_id: ui_hue_conf.entertainment_area.clone(),
        };

        let mut hue_controller = HueController::new(control_hue_conf);

        // Try to connect if IP is set and auto-connect is enabled
        if !ui_state.user_config.hue_config.bridge_ip.is_empty()
            && ui_state.user_config.hue_config.auto_connect
        {
            info!("Initializing Hue Controller...");
            if let Err(e) = tokio_runtime.block_on(hue_controller.connect()) {
                warn!("Hue Controller initial connection failed: {}", e);
            }
        }

        let control_manager = ControlManager::new();
        let sys_info = sysinfo::System::new_all();
        let (dummy_texture, dummy_view) = {
            let texture = backend.device.create_texture(&wgpu::TextureDescriptor {
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
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            (texture, view)
        };

        #[cfg(feature = "midi")]
        let midi_handler = {
            match MidiInputHandler::new() {
                Ok(mut handler) => {
                    info!("MIDI initialized");
                    if let Ok(ports) = MidiInputHandler::list_ports() {
                        info!("Available MIDI ports: {:?}", ports);
                        // Auto-connect to first port if available
                        if !ports.is_empty() {
                            if let Err(e) = handler.connect(0) {
                                error!("Failed to auto-connect MIDI: {}", e);
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
        };

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
            action_sender,
            control_manager,
            exit_requested: false,
            oscillator_renderer,
            dummy_texture: Some(dummy_texture),
            dummy_view: Some(dummy_view),
            module_evaluator: ModuleEvaluator::new(),
            media_players: HashMap::new(),
            fps_samples: VecDeque::new(),
            current_fps: 0.0,
            current_frame_time_ms: 0.0,
            sys_info,
            last_sysinfo_refresh: std::time::Instant::now(),
            #[cfg(feature = "midi")]
            midi_handler,
            #[cfg(feature = "midi")]
            midi_ports: MidiInputHandler::list_ports().unwrap_or_default(),
            #[cfg(feature = "midi")]
            selected_midi_port: if MidiInputHandler::list_ports()
                .unwrap_or_default()
                .is_empty()
            {
                None
            } else {
                Some(0) // Assuming auto-connect to first port succeeded
            },
            #[cfg(feature = "ndi")]
            ndi_receivers: std::collections::HashMap::new(),
            #[cfg(feature = "ndi")]
            ndi_senders: std::collections::HashMap::new(),

            output_assignments: std::collections::HashMap::new(),
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
            preview_texture_cache: HashMap::new(),
            output_preview_cache: HashMap::new(),
            preview_quad_buffers,
            hue_controller,
            tokio_runtime,
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
    /// Handles a window event.
    pub fn handle_event(
        &mut self,
        event: winit::event::Event<()>,
        elwt: &winit::event_loop::ActiveEventLoop,
    ) -> Result<()> {
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

                    let result = self.module_evaluator.evaluate(module);

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
                    for (part_id, cmd) in &result.source_commands {
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
                    .set_audio_data(AudioTriggerData {
                        band_energies: analysis_v2.band_energies,
                        rms_volume: analysis_v2.rms_volume,
                        peak_volume: analysis_v2.peak_volume,
                        beat_detected: analysis_v2.beat_detected,
                        beat_strength: analysis_v2.beat_strength,
                        bpm: analysis_v2.tempo_bpm, // BPM from beat tracking!
                    });

                // Update BPM in toolbar
                // Convert V2 analysis to legacy format for UI compatibility
                // analyzer_v2 produces 9 bands, legacy AudioAnalysis expects 7.
                let legacy_analysis = mapmap_core::audio::AudioAnalysis {
                    timestamp: analysis_v2.timestamp,
                    fft_magnitudes: analysis_v2.fft_magnitudes.clone(),
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
                    waveform: analysis_v2.waveform.clone(),
                };

                self.ui_state.dashboard.set_audio_analysis(legacy_analysis);

                // Update module canvas with audio trigger data
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

                // Update BPM in toolbar
                self.ui_state.current_bpm = analysis_v2.tempo_bpm;

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

        self.output_assignments.clear();

        // 1. Iterate over ALL modules to collect required outputs
        for module in self.state.module_manager.list_modules() {
            if let Some(module_ref) = self.state.module_manager.get_module(module.id) {
                for part in &module_ref.parts {
                    if let mapmap_core::module::ModulePartType::Output(output_type) = &part.part_type {
                        // Use part.id for consistency with render pipeline
                        let output_id = part.id;

                        match output_type {
                            OutputType::Projector {
                                id: projector_id,
                                name,
                                fullscreen,
                                hide_cursor,
                                target_screen,
                                show_in_preview_panel: _,
                                extra_preview_window,
                                ..
                            } => {
                                // 1. Primary Window - Use Logical ID (projector_id) not Part ID
                                let window_id = *projector_id;
                                active_window_ids.insert(window_id);

                                if let Some(window_context) = self.window_manager.get(window_id)
                                {
                                    // Update existing
                                    let is_fullscreen =
                                        window_context.window.fullscreen().is_some();
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
                                        elwt,
                                        &self.backend,
                                        window_id,
                                        name,
                                        *fullscreen,
                                        *hide_cursor,
                                        *target_screen,
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
                                        )?;
                                        info!(
                                            "Created preview window for output {}",
                                            window_id
                                        );
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
                                                pixel_format:
                                                    mapmap_io::format::PixelFormat::BGRA8,
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

    /// Synchronize media players with active source modules
    /// Synchronize media players with active source modules
    fn sync_media_players(&mut self) {
        let mut active_sources = std::collections::HashSet::new();

        // Identify active media files
        for module in self.state.module_manager.modules() {
            for part in &module.parts {
                if let ModulePartType::Source(SourceType::MediaFile { path, .. }) = &part.part_type
                {
                    if !path.is_empty() {
                        let key = (module.id, part.id);
                        active_sources.insert(key);

                        // Create player if not exists
                        match self.media_players.entry(key) {
                            std::collections::hash_map::Entry::Vacant(e) => {
                                match mapmap_media::open_path(path) {
                                    Ok(mut player) => {
                                        if let Err(e) = player.play() {
                                            error!(
                                                "Failed to start playback for source {}:{} : {}",
                                                module.id, part.id, e
                                            );
                                        }
                                        e.insert((path.clone(), player));
                                    }
                                    Err(e) => {
                                        error!(
                                            "Failed to create video player for source {}:{} : {}",
                                            module.id, part.id, e
                                        );
                                    }
                                }
                            }
                            std::collections::hash_map::Entry::Occupied(mut e) => {
                                // Check if path changed
                                let (current_path, player) = e.get_mut();
                                if current_path != path {
                                    info!(
                                        "Path changed for source {}:{} -> loading {}",
                                        module.id, part.id, path
                                    );
                                    // Load new media
                                    match mapmap_media::open_path(path) {
                                        Ok(mut new_player) => {
                                            if let Err(err) = new_player.play() {
                                                error!("Failed to start playback: {}", err);
                                            }
                                            *current_path = path.clone();
                                            *player = new_player;
                                        }
                                        Err(err) => {
                                            error!("Failed to load new media: {}", err);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Cleanup removed players
        self.media_players
            .retain(|key, _| active_sources.contains(key));
    }

    /// Update all media players and upload frames to texture pool
    /// Update all media players and upload frames to texture pool
    fn update_media_players(&mut self, dt: f32) {
        static FRAME_LOG_COUNTER: std::sync::atomic::AtomicU32 =
            std::sync::atomic::AtomicU32::new(0);
        let log_this_frame = FRAME_LOG_COUNTER
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            .is_multiple_of(60);

        let texture_pool = &mut self.texture_pool;
        let queue = &self.backend.queue;
        let ui_state = &mut self.ui_state;

        for ((mod_id, part_id), (_, player)) in &mut self.media_players {
            // Update player logic
            if let Some(frame) = player.update(std::time::Duration::from_secs_f32(dt)) {
                let tex_name = format!("part_{}_{}", mod_id, part_id);

                // Upload to GPU if data is on CPU
                if let mapmap_io::format::FrameData::Cpu(data) = &frame.data {
                    if log_this_frame {
                        tracing::info!(
                            "Frame upload: mod={} part={} size={}x{}",
                            mod_id,
                            part_id,
                            frame.format.width,
                            frame.format.height
                        );
                    }
                    texture_pool.upload_data(
                        queue,
                        &tex_name,
                        data,
                        frame.format.width,
                        frame.format.height,
                    );
                }
            } else if log_this_frame {
                // tracing::warn!("Media player {}:{} returned no frame", mod_id, part_id);
            }

            // Sync player info to UI for timeline display
            // Only if this is the active module to avoid polluting global state map?
            // Actually ModuleCanvas has a map PartID -> Info. This assumes uniqueness.
            // Since UI only shows ONE active module, we should only populate if mod_id == active_module_id
            if let Some(active_id) = ui_state.module_canvas.active_module_id {
                if *mod_id == active_id {
                    ui_state.module_canvas.player_info.insert(
                        *part_id,
                        mapmap_ui::MediaPlayerInfo {
                            current_time: player.current_time().as_secs_f64(),
                            duration: player.duration().as_secs_f64(),
                            is_playing: matches!(
                                player.state(),
                                mapmap_media::PlaybackState::Playing
                            ),
                        },
                    );
                }
            }
        }
    }

    fn prepare_texture_previews(&mut self, _encoder: &mut wgpu::CommandEncoder) {
        // Sync Texture Previews for Module Canvas
        // Identify active sources and gather their properties
        let mut active_preview_sources = Vec::new();
        for module in self.state.module_manager.modules() {
            for part in &module.parts {
                if let mapmap_core::module::ModulePartType::Source(
                    mapmap_core::module::SourceType::MediaFile {
                        brightness,
                        contrast,
                        saturation,
                        hue_shift,
                        flip_horizontal,
                        flip_vertical,
                        rotation,
                        scale_x,
                        scale_y,
                        offset_x,
                        offset_y,
                        ..
                    },
                ) = &part.part_type
                {
                    // Find source connection
                    if let Some(conn) = module.connections.iter().find(|c| c.to_part == part.id) {
                        active_preview_sources.push((
                            module.id,
                            part.id,        // Target (Output Node)
                            conn.from_part, // Source (The plugged in layer/media)
                            *brightness,
                            *contrast,
                            *saturation,
                            *hue_shift,
                            *flip_horizontal,
                            *flip_vertical,
                            *rotation,
                            *scale_x,
                            *scale_y,
                            *offset_x,
                            *offset_y,
                        ));
                    }
                } else if let mapmap_core::module::ModulePartType::Output(
                    mapmap_core::module::OutputType::Projector {
                        show_in_preview_panel,
                        ..
                    },
                ) = &part.part_type
                {
                    if *show_in_preview_panel {
                        // Find connected input
                        // We need to look at module.connections
                        // Find connection where to_part == part.id
                        // For Projector, input is usually socket 0 ("Layer In")
                        if let Some(conn) = module.connections.iter().find(|c| c.to_part == part.id)
                        {
                            active_preview_sources.push((
                                module.id,
                                part.id,        // Target (Output Node)
                                conn.from_part, // Source (The plugged in layer/media)
                                0.0,            // Brightness default
                                1.0,            // Contrast default
                                1.0,            // Saturation default
                                0.0,            // Hue default
                                false,          // Flip H
                                false,          // Flip V
                                0.0,            // Rotation
                                1.0,            // Scale X
                                1.0,            // Scale Y
                                0.0,            // Offset X
                                0.0,            // Offset Y
                            ));
                        }
                    }
                }
            }
        }

        // Render previews with effects
        let mut current_frame_previews = std::collections::HashMap::new();

        // ⚡ Bolt Optimization: Batch all preview render passes into a single encoder submission
        // This avoids creating N encoders and submitting N command buffers to the queue per frame.
        self.mesh_renderer.begin_frame(); // Reset uniform buffer cache index for this batch

        let mut encoder =
            self.backend
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Preview Encoder (Batched)"),
                });

        let mut _has_preview_work = false;

        for (
            module_id,
            target_part_id,
            source_part_id,
            brightness,
            contrast,
            saturation,
            hue_shift,
            flip_h,
            flip_v,
            rotation,
            scale_x,
            scale_y,
            offset_x,
            offset_y,
        ) in active_preview_sources
        {
            let raw_tex_name = format!("part_{}_{}", module_id, source_part_id);
            if self.texture_pool.has_texture(&raw_tex_name) {
                let raw_view = self.texture_pool.get_view(&raw_tex_name);

                // Create/Get preview texture (fixed small resolution)
                let preview_tex_name = format!("preview_{}", target_part_id);
                // Ensure it exists with correct size
                self.texture_pool.ensure_texture(
                    &preview_tex_name,
                    320,
                    180,
                    self.backend.surface_format(),
                    wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                );
                let preview_view = self.texture_pool.get_view(&preview_tex_name);

                // Calculate Transform Matrix based on source properties
                let transform_mat = glam::Mat4::from_scale_rotation_translation(
                    glam::Vec3::new(scale_x, scale_y, 1.0),
                    glam::Quat::from_rotation_z(rotation.to_radians()),
                    glam::Vec3::new(offset_x, offset_y, 0.0),
                );

                // Prepare Uniforms
                // Note: queue.write_buffer operations here are scheduled on the queue
                // effectively "before" the command buffer execution, maintaining correctness.
                let uniform_bg = self.mesh_renderer.get_uniform_bind_group_with_source_props(
                    &self.backend.queue,
                    transform_mat,
                    1.0,
                    flip_h,
                    flip_v,
                    brightness,
                    contrast,
                    saturation,
                    hue_shift,
                );

                let texture_bg = self.mesh_renderer.create_texture_bind_group(&raw_view);

                // Render Pass - Scope limits lifetime of render_pass borrow on encoder
                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Preview Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &preview_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    // Use the pre-allocated quad buffers
                    let (vb, ib, index_count) = &self.preview_quad_buffers;

                    self.mesh_renderer.draw(
                        &mut render_pass,
                        vb,
                        ib,
                        *index_count,
                        &uniform_bg,
                        &texture_bg,
                        false,
                    );
                }

                // Register the PROCESSED preview texture for UI
                let texture_id = match self.preview_texture_cache.entry(target_part_id) {
                    std::collections::hash_map::Entry::Occupied(mut entry) => {
                        let (cached_id, cached_view) = entry.get();
                        if std::sync::Arc::ptr_eq(cached_view, &preview_view) {
                            *cached_id
                        } else {
                            self.egui_renderer.free_texture(cached_id);
                            let new_id = self.egui_renderer.register_native_texture(
                                &self.backend.device,
                                &preview_view,
                                wgpu::FilterMode::Linear,
                            );
                            entry.insert((new_id, preview_view.clone()));
                            new_id
                        }
                    }
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        let new_id = self.egui_renderer.register_native_texture(
                            &self.backend.device,
                            &preview_view,
                            wgpu::FilterMode::Linear,
                        );
                        entry.insert((new_id, preview_view.clone()));
                        new_id
                    }
                };

                current_frame_previews.insert(target_part_id, texture_id);
            }
        }

        // Cleanup stale cache entries
        self.preview_texture_cache.retain(|id, (tex_id, _)| {
            if !current_frame_previews.contains_key(id) {
                self.egui_renderer.free_texture(tex_id);
                false
            } else {
                true
            }
        });

        // Update UI state map
        self.ui_state.module_canvas.node_previews = current_frame_previews;

        // Register Output Preview Textures
        let mut current_output_previews: std::collections::HashMap<u64, egui::TextureId> =
            std::collections::HashMap::new();
        for (output_id, tex_names) in &self.output_assignments {
            // Use the last assigned texture for preview (topmost layer)
            if let Some(tex_name) = tex_names.last() {
                if self.texture_pool.has_texture(tex_name) {
                    let tex_view = self.texture_pool.get_view(tex_name);

                    // ⚡ Bolt Optimization: Use Arc::ptr_eq to avoid unnecessary re-registration
                    let texture_id = match self.output_preview_cache.entry(*output_id) {
                        std::collections::hash_map::Entry::Occupied(mut entry) => {
                            let (cached_id, cached_view) = entry.get();
                            if std::sync::Arc::ptr_eq(cached_view, &tex_view) {
                                *cached_id
                            } else {
                                self.egui_renderer.free_texture(cached_id);
                                let new_id = self.egui_renderer.register_native_texture(
                                    &self.backend.device,
                                    &tex_view,
                                    wgpu::FilterMode::Linear,
                                );
                                entry.insert((new_id, tex_view.clone()));
                                new_id
                            }
                        }
                        std::collections::hash_map::Entry::Vacant(entry) => {
                            let new_id = self.egui_renderer.register_native_texture(
                                &self.backend.device,
                                &tex_view,
                                wgpu::FilterMode::Linear,
                            );
                            entry.insert((new_id, tex_view.clone()));
                            new_id
                        }
                    };

                    current_output_previews.insert(*output_id, texture_id);
                }
            }
        }

        self.output_preview_cache.retain(|id, (tex_id, _)| {
            if !current_output_previews.contains_key(id) {
                self.egui_renderer.free_texture(tex_id);
                false
            } else {
                true
            }
        });
    }

    /// Global update loop (physics/logic), independent of render rate per window.
    fn update(&mut self, elwt: &winit::event_loop::ActiveEventLoop, dt: f32) {
        let _ = self.handle_ui_actions();

        // --- Media Player Update ---
        self.sync_media_players();
        self.update_media_players(dt);

        // --- Effect Animator Update ---
        // --- Effect Animator Update ---
        let param_updates = self.state.effect_animator.update(dt as f64);
        // Note: param_updates is Vec, so just iterate
        if !param_updates.is_empty() {
            // TODO: Apply updates to Active Module
            tracing::trace!("Effect updates: {}", param_updates.len());
        }

        // --- Module Graph Evaluation ---
        // Evaluate ALL modules and merge render_ops for multi-output support
        self.render_ops.clear();
        for module in self.state.module_manager.list_modules() {
            let module_id = module.id;
            if let Some(module_ref) = self.state.module_manager.get_module(module_id) {
                let eval_result = self.module_evaluator.evaluate(module_ref);
                // Push (ModuleId, RenderOp) tuple
                self.render_ops.extend(
                    eval_result
                        .render_ops
                        .iter()
                        .cloned()
                        .map(|op| (module_id, op)),
                );
            }
        }

        // Sync output windows based on MODULE GRAPH STRUCTURE (stable),
        // NOT render_ops (which can be empty/fluctuate).
        // Extract Output part IDs from all modules' output parts.
        let current_output_ids: std::collections::HashSet<u64> = self
            .state
            .module_manager
            .list_modules()
            .iter()
            .flat_map(|m| m.parts.iter())
            .filter_map(|part| {
                if let mapmap_core::module::ModulePartType::Output(
                    mapmap_core::module::OutputType::Projector { .. },
                ) = &part.part_type
                {
                    Some(part.id) // Use part.id for consistency with render pipeline
                } else {
                    None
                }
            })
            .collect();

        // Get current window IDs (excluding main window 0)
        let prev_output_ids: std::collections::HashSet<u64> = self
            .window_manager
            .iter()
            .filter(|wc| wc.output_id != 0)
            .map(|wc| wc.output_id)
            .collect();

        // Only sync if module graph's projector set changed
        // Only sync if module graph's projector set changed
        if current_output_ids != prev_output_ids {
            tracing::info!(
                "Output set changed: {:?} -> {:?}",
                prev_output_ids,
                current_output_ids
            );
            // Create temp list of ops for sync (stripping module ID)
            let ops_only: Vec<mapmap_core::module_eval::RenderOp> =
                self.render_ops.iter().map(|(_, op)| op.clone()).collect();
            if let Err(e) = self.sync_output_windows(elwt, &ops_only, None) {
                tracing::error!("Failed to sync output windows: {}", e);
            }
        }

        // --- Oscillator Update ---
        if let Some(renderer) = &mut self.oscillator_renderer {
            if self.state.oscillator_config.enabled {
                renderer.update(dt, &self.state.oscillator_config);
            }
        }

        // --- FPS Calculation ---
        let frame_time_ms = dt * 1000.0;
        self.fps_samples.push_back(frame_time_ms);
        if self.fps_samples.len() > 60 {
            self.fps_samples.pop_front();
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
    }
    /// Handle global UI actions
    fn handle_ui_actions(&mut self) -> Result<()> {
        let actions = self.ui_state.take_actions();
        for action in actions {
            match action {
                mapmap_ui::UIAction::NodeAction(node_action) => {
                    self.handle_node_action(node_action)?;
                }
                mapmap_ui::UIAction::OpenShaderGraph(graph_id) => {
                    self.ui_state.show_shader_graph = true;
                    if let Some(graph) = self.state.shader_graphs.get(&graph_id) {
                        self.ui_state.node_editor_panel.load_graph(graph);
                    } else {
                        // Create if not exists (lazy creation for testing)
                        // Or log warning
                        // For Phase 6 demo: Create a default graph if ID not found?
                        // Better: Ensure graph exists via other means (Graph Manager UI).
                        // For now we assume call is valid or we create empty.
                        if let std::collections::hash_map::Entry::Vacant(e) =
                            self.state.shader_graphs.entry(graph_id)
                        {
                            let new_graph = mapmap_core::shader_graph::ShaderGraph::new(
                                graph_id,
                                "New Graph".to_string(),
                            );
                            e.insert(new_graph.clone());
                            self.ui_state.node_editor_panel.load_graph(&new_graph);
                        }
                    }
                }
                _ => {
                    // Ignore other actions or let them fall through if not handled here
                    // Ideally we should have a centralized handler. Both AppUI and App might handle different actions.
                    // But AppUI actions are usually handled by the caller (Main).
                }
            }
        }
        Ok(())
    }

    /// Handle Node Editor actions
    fn handle_node_action(&mut self, action: mapmap_ui::NodeEditorAction) -> Result<()> {
        if let Some(graph_id) = self.ui_state.node_editor_panel.graph_id {
            if let Some(graph) = self.state.shader_graphs.get_mut(&graph_id) {
                use mapmap_ui::NodeEditorAction;
                let mut needs_update = false;

                match action {
                    NodeEditorAction::AddNode(node_type, pos) => {
                        let _id = graph.add_node(node_type);
                        // TODO: Update position in Core logic if possible.
                        // Core ShaderNode has `position: (f32, f32)`.
                        if let Some(node) = graph.nodes.get_mut(&_id) {
                            node.position = (pos.x, pos.y);
                        }
                        needs_update = true;
                    }
                    NodeEditorAction::RemoveNode(node_id) => {
                        graph.remove_node(node_id);
                        needs_update = true;
                    }
                    NodeEditorAction::SelectNode(_) => {
                        // Selection is handled in UI state mostly.
                    }
                    NodeEditorAction::AddConnection(_from, from_socket, to, to_socket) => {
                        // Note: UI NodeEditorAction provides NodeId and socket name
                        if let Err(e) = graph.connect(_from, &from_socket, to, &to_socket) {
                            tracing::warn!("Failed to connect nodes: {}", e);
                        } else {
                            needs_update = true;
                        }
                    }
                    NodeEditorAction::RemoveConnection(_from, _sub_idx, to, to_socket) => {
                        if let Err(e) = graph.disconnect(to, &to_socket) {
                            tracing::warn!("Failed to disconnect nodes: {}", e);
                        } else {
                            needs_update = true;
                        }
                    }
                    NodeEditorAction::UpdateGraph(_) => {
                        needs_update = true;
                    }
                }

                if needs_update {
                    // Sync back to UI to maintain consistency
                    self.ui_state.node_editor_panel.load_graph(graph);
                    self.state.dirty = true;

                    // Compile Graph
                    if let Err(e) = self
                        .effect_chain_renderer
                        .update_shader_graph(&mut self.shader_graph_manager, graph_id)
                    {
                        tracing::error!("Shader Compile Error: {}", e);
                    } else {
                        tracing::info!("Shader Graph {} compiled successfully", graph_id);
                    }
                }
            }
        }
        Ok(())
    }

    fn render(&mut self, output_id: OutputId) -> Result<()> {
        // Clone device Arc to create encoder without borrowing self
        let device = self.backend.device.clone();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // ⚡ Bolt Optimization: Batch render passes.
        // We call begin_frame() once here to reset the uniform cache index for the entire batch.
        self.mesh_renderer.begin_frame();

        if output_id == 0 {
            // Sync Texture Previews for Module Canvas (renders into preview textures using main encoder)
            self.prepare_texture_previews(&mut encoder);
        }

        let window_context = self.window_manager.get(output_id).unwrap();

        // Get surface texture and view for final output
        let surface_texture = window_context.surface.get_current_texture()?;
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut egui_render_data = None;

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
                                    UIEffectType::Wave => RenderEffectType::Wave,
                                    UIEffectType::Glitch => RenderEffectType::Glitch,
                                    UIEffectType::RgbSplit => RenderEffectType::RgbSplit,
                                    UIEffectType::Mirror => RenderEffectType::Mirror,
                                    UIEffectType::HueShift => RenderEffectType::HueShift,
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
                                    UIEffectType::Wave => RenderEffectType::Wave,
                                    UIEffectType::Glitch => RenderEffectType::Glitch,
                                    UIEffectType::RgbSplit => RenderEffectType::RgbSplit,
                                    UIEffectType::Mirror => RenderEffectType::Mirror,
                                    UIEffectType::HueShift => RenderEffectType::HueShift,
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

                    // === UNIFIED LEFT SIDEBAR ===
                    // Two independent collapsible panels: Controls (top) and Preview (bottom)
                    if self.ui_state.show_left_sidebar {
                        egui::SidePanel::left("unified_left_sidebar")
                            .resizable(true)
                            .default_width(280.0)
                            .min_width(200.0)
                            .max_width(500.0)
                            .show(ctx, |ui| {
                                // Sidebar header with collapse button
                                ui.horizontal(|ui| {
                                    ui.heading("Sidebar");
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button("◀").on_hover_text("Sidebar einklappen").clicked() {
                                            self.ui_state.show_left_sidebar = false;
                                        }
                                    });
                                });
                                ui.separator();

                                // === CONTROLS PANEL (Top) ===
                                // Header with toggle button
                                ui.horizontal(|ui| {
                                    let arrow = if self.ui_state.show_control_panel { "▼" } else { "▶" };
                                    if ui.button(arrow).on_hover_text("Controls ein-/ausklappen").clicked() {
                                        self.ui_state.show_control_panel = !self.ui_state.show_control_panel;
                                    }
                                    ui.heading("⚙ Controls");
                                });

                                if self.ui_state.show_control_panel {
                                    // Use fixed height when both panels are open
                                    let use_fixed_height = self.ui_state.show_preview_panel;

                                    if use_fixed_height {
                                        ui.allocate_ui_with_layout(
                                            egui::vec2(ui.available_width(), self.ui_state.control_panel_height),
                                            egui::Layout::top_down(egui::Align::LEFT),
                                            |ui| {
                                                egui::ScrollArea::vertical().id_salt("controls_scroll").show(ui, |ui| {
                                                    // Media Browser Section
                                                    egui::CollapsingHeader::new("📁 Media")
                                                        .default_open(false)
                                                        .show(ui, |ui| {
                                                            if let Some(action) = self.ui_state.media_browser.ui(
                                                                ui,
                                                                &self.ui_state.i18n,
                                                                self.ui_state.icon_manager.as_ref(),
                                                            ) {
                                                                use mapmap_ui::media_browser::MediaBrowserAction;
                                                                match action {
                                                                    MediaBrowserAction::FileSelected(path) | MediaBrowserAction::FileDoubleClicked(path) => {
                                                                        // Update active part if one is being edited
                                                                        if let (Some(module_id), Some(part_id)) = (
                                                                            self.ui_state.module_canvas.active_module_id,
                                                                            self.ui_state.module_canvas.editing_part_id
                                                                        ) {
                                                                            self.ui_state.actions.push(mapmap_ui::UIAction::PickMediaFile(
                                                                                module_id,
                                                                                part_id,
                                                                                path.to_string_lossy().to_string()
                                                                            ));
                                                                        }
                                                                    }
                                                                    _ => {}
                                                                }
                                                            }
                                                        });

                                                    // Audio Section
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
                                                                        self.ui_state.user_config.set_audio_device(Some(device.clone()));
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
                                                                                error!("Failed to create audio backend for device '{}': {}", device, e);
                                                                            }
                                                                        }
                                                                    }
                                                                    mapmap_ui::audio_panel::AudioPanelAction::ConfigChanged(cfg) => {
                                                                        self.audio_analyzer.update_config(AudioAnalyzerV2Config {
                                                                            sample_rate: cfg.sample_rate,
                                                                            fft_size: cfg.fft_size,
                                                                            overlap: cfg.overlap,
                                                                            smoothing: cfg.smoothing,
                                                                        });
                                                                        self.state.audio_config = cfg;
                                                                    }
                                                                }
                                                            }
                                                        });
                                                });
                                            },
                                        );

                                        // Splitter (resize handle)
                                        let splitter_response = ui.add(egui::Separator::default().spacing(8.0));
                                        if splitter_response.dragged() {
                                            self.ui_state.control_panel_height += splitter_response.drag_delta().y;
                                            self.ui_state.control_panel_height = self.ui_state.control_panel_height.clamp(50.0, 500.0);
                                        }
                                        if splitter_response.hovered() {
                                            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeVertical);
                                        }
                                    } else {
                                        // Controls only - full height
                                        egui::ScrollArea::vertical().id_salt("inspector_scroll_full").show(ui, |ui| {
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
                                                                self.ui_state.user_config.set_audio_device(Some(device.clone()));
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
                                                                        error!("Failed to create audio backend for device '{}': {}", device, e);
                                                                    }
                                                                }
                                                            }
                                                            mapmap_ui::audio_panel::AudioPanelAction::ConfigChanged(cfg) => {
                                                                self.audio_analyzer.update_config(AudioAnalyzerV2Config {
                                                                    sample_rate: cfg.sample_rate,
                                                                    fft_size: cfg.fft_size,
                                                                    overlap: cfg.overlap,
                                                                    smoothing: cfg.smoothing,
                                                                });
                                                                self.state.audio_config = cfg;
                                                            }
                                                        }
                                                    }
                                                });
                                        });
                                    }
                                }

                                // === PREVIEW PANEL (Bottom) ===
                                // Header with toggle button
                                ui.horizontal(|ui| {
                                    let arrow = if self.ui_state.show_preview_panel { "▼" } else { "▶" };
                                    if ui.button(arrow).on_hover_text("Preview ein-/ausklappen").clicked() {
                                        self.ui_state.show_preview_panel = !self.ui_state.show_preview_panel;
                                    }
                                    ui.heading("👁 Preview");
                                });

                                if self.ui_state.show_preview_panel {
                                    let output_infos: Vec<mapmap_ui::OutputPreviewInfo> = self
                                        .state
                                        .module_manager
                                        .modules()
                                        .iter()
                                        .flat_map(|module| {
                                            module.parts.iter().filter_map(|part| {
                                                if let mapmap_core::module::ModulePartType::Output(output_type) = &part.part_type {
                                                    match output_type {
                                                        mapmap_core::module::OutputType::Projector { ref id, ref name, ref show_in_preview_panel, .. } => {
                                                            Some(mapmap_ui::OutputPreviewInfo {
                                                                id: *id,
                                                                name: name.clone(),
                                                                show_in_panel: *show_in_preview_panel,
                                                                texture_name: self.output_assignments.get(id).and_then(|v| v.last().cloned()),
                                                                texture_id: self.output_preview_cache.get(id).map(|(id, _)| *id),
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
                                    // Ensure continuous repaint for live preview
                                    if self.ui_state.show_preview_panel {
                                        ctx.request_repaint();
                                    }
                                    self.ui_state.preview_panel.show(ui);
                                }
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

                    // === RIGHT PANEL: Inspector ===
                    self.ui_state.render_inspector(
                        ctx,
                        &mut self.state.module_manager,
                        &self.state.layer_manager,
                        &self.state.output_manager,
                    );

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

                    // === 6. Node Editor (Phase 6b) ===
                    self.ui_state.render_node_editor(ctx);

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
                                            egui::ComboBox::from_id_salt("fps_select")
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
                                            egui::ComboBox::from_id_salt("meter_style_select")
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
                                            if ui.add(egui::DragValue::new(&mut output_count).speed(1).range(1..=8)).changed() {
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
                                            egui::ComboBox::from_id_salt("log_level_select")
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
                                            if ui.add(egui::DragValue::new(&mut log_config.max_files).speed(1).range(1..=100)).changed() {
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

                                // Philips Hue Settings
                                let body_returned = egui::CollapsingHeader::new(format!("💡 {}", "Philips Hue"))
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        let mut changed = false;
                                        let mut connect_clicked = false;
                                        let mut register_clicked = false;
                                        let disconnect_clicked = false;
                                        let mut discover_clicked = false;
                                        let hue_conf = &mut self.ui_state.user_config.hue_config;

                                        ui.horizontal(|ui| {
                                            ui.label("Bridge IP:");
                                            if ui.text_edit_singleline(&mut hue_conf.bridge_ip).changed() {
                                                changed = true;
                                            }

                                            if ui.button("Discover").clicked() {
                                                discover_clicked = true;
                                            }
                                        });

                                        if !self.ui_state.discovered_hue_bridges.is_empty() {
                                            egui::ComboBox::from_id_salt("discovered_bridges")
                                                .selected_text("Select Discovered Bridge...")
                                                .show_ui(ui, |ui| {
                                                    for bridge in &self.ui_state.discovered_hue_bridges {
                                                        if ui.selectable_label(hue_conf.bridge_ip == bridge.ip, format!("{} ({})", bridge.ip, bridge.id)).clicked() {
                                                            hue_conf.bridge_ip = bridge.ip.clone();
                                                            changed = true;
                                                        }
                                                    }
                                                });
                                        }

                                        ui.horizontal(|ui| {
                                            ui.label("App Key (User):");
                                            ui.add_enabled(false, egui::TextEdit::singleline(&mut hue_conf.username));
                                        });

                                        ui.horizontal(|ui| {
                                            ui.label("Client Key:");
                                            ui.add_enabled(false, egui::TextEdit::singleline(&mut hue_conf.client_key).password(true));
                                        });

                                        ui.horizontal(|ui| {
                                            ui.label("Entertainment Group:");
                                            let current_id = &mut hue_conf.entertainment_area;

                                            // ComboBox for Group Selection
                                            let selected_text = self
                                                .ui_state
                                                .available_hue_groups
                                                .iter()
                                                .find(|(id, _)| id == current_id)
                                                .map(|(_, name)| name.as_str())
                                                .unwrap_or(current_id.as_str());

                                            egui::ComboBox::from_id_salt("hue_group_select")
                                                .selected_text(selected_text)
                                                .show_ui(ui, |ui| {
                                                    for (id, name) in
                                                        &self.ui_state.available_hue_groups
                                                    {
                                                        if ui
                                                            .selectable_label(current_id == id, name)
                                                            .clicked()
                                                        {
                                                            *current_id = id.clone();
                                                            changed = true;
                                                        }
                                                    }
                                                });

                                            // Refresh Button
                                            if ui.button("🔄").on_hover_text("Refresh Groups").clicked()
                                            {
                                                self.ui_state
                                                    .actions
                                                    .push(mapmap_ui::UIAction::FetchHueGroups);
                                            }
                                        });

                                        ui.horizontal(|ui| {
                                            if ui.checkbox(&mut hue_conf.auto_connect, "Auto Connect via Main Settings").changed() {
                                                changed = true;
                                            }
                                        });

                                        ui.add_space(5.0);

                                        ui.horizontal(|ui| {
                                            if ui.button("Verbinden (Sync)").clicked() {
                                                connect_clicked = true;
                                            }

                                            // Link Bridge Button
                                            if ui.button("Link Bridge").on_hover_text("Press the button on your Hue Bridge, then click this to link.").clicked() {
                                                register_clicked = true;
                                            }

                                            // Connection Status Display
                                            // Using self.hue_controller is disjoint from self.ui_state (hue_conf)
                                            if self.hue_controller.is_connected() {
                                                 ui.colored_label(egui::Color32::GREEN, "Connected");
                                            } else {
                                                 ui.colored_label(egui::Color32::RED, "Disconnected");
                                            }
                                        });

                                        ui.label(egui::RichText::new("Note: Press Link Button on Bridge before linking/connecting for the first time.").small());
                                        (changed, connect_clicked, disconnect_clicked, discover_clicked, register_clicked)
                                    })
                                    .body_returned;

                                    if let Some((changed, connect, disconnect, discover, register)) = body_returned {
                                        if register {
                                            self.ui_state.actions.push(mapmap_ui::UIAction::RegisterHue);
                                        }
                                        if changed {
                                            let _ = self.ui_state.user_config.save();
                                            // Update controller config with correct types
                                            let ui_hue = &self.ui_state.user_config.hue_config;
                                            let control_hue = mapmap_control::hue::models::HueConfig {
                                                bridge_ip: ui_hue.bridge_ip.clone(),
                                                username: ui_hue.username.clone(),
                                                client_key: ui_hue.client_key.clone(),
                                                application_id: String::new(),
                                                entertainment_group_id: ui_hue.entertainment_area.clone(),
                                            };
                                            self.hue_controller.update_config(control_hue);
                                        }
                                        if connect {
                                            self.ui_state.actions.push(mapmap_ui::UIAction::ConnectHue);
                                        }
                                        if disconnect {
                                            self.ui_state.actions.push(mapmap_ui::UIAction::DisconnectHue);
                                        }
                                        if discover {
                                            self.ui_state.actions.push(mapmap_ui::UIAction::DiscoverHueBridges);
                                        }
                                    }

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

                                            egui::ComboBox::from_id_salt("midi_port_select")
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

            // Store for final rendering
            egui_render_data = Some((tris, screen_descriptor));

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

            // Post-render logic for egui actions
        } else {
            // === Node-Based Rendering Pipeline ===

            // 1. Find ALL RenderOps for this output to support composition
            const PREVIEW_FLAG: u64 = 1u64 << 63;
            let real_output_id = output_id & !PREVIEW_FLAG;

            let target_ops: Vec<(u64, &mapmap_core::module_eval::RenderOp)> = self
                .render_ops
                .iter()
                .filter(|(_, op)| match &op.output_type {
                    mapmap_core::module::OutputType::Projector { id, .. } => *id == real_output_id,
                    _ => op.output_part_id == real_output_id, /* Use real_output_id for generic outputs too */
                })
                .map(|(mid, op)| (*mid, op))
                .collect();

            // Debug: Log number of ops per output
            if target_ops.len() > 1 {
                tracing::info!(
                    "Multi-Output active: Output {} rendering {} layers",
                    output_id,
                    target_ops.len()
                );
            }

            if target_ops.is_empty() {
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
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });
            } else {
                // Shared Configuration for Output
                let output_config_opt = self.state.output_manager.get_output(output_id);
                let use_edge_blend =
                    output_config_opt.is_some() && self.edge_blend_renderer.is_some();
                let use_color_calib =
                    output_config_opt.is_some() && self.color_calibration_renderer.is_some();
                let needs_post_processing = use_edge_blend || use_color_calib;

                // A. Prepare Mesh Target
                let mesh_target_view_ref;
                let mesh_output_tex_name = &self.layer_ping_pong[1];
                let mut _mesh_intermediate_view: Option<std::sync::Arc<wgpu::TextureView>> = None;

                if needs_post_processing {
                    let width = window_context.surface_config.width;
                    let height = window_context.surface_config.height;
                    self.texture_pool
                        .resize_if_needed(mesh_output_tex_name, width, height);
                    _mesh_intermediate_view =
                        Some(self.texture_pool.get_view(mesh_output_tex_name));
                    mesh_target_view_ref = _mesh_intermediate_view.as_deref().unwrap();
                } else {
                    mesh_target_view_ref = &view;
                }

                // B. Clear Mesh Target (Once per frame, before accumulation)
                {
                    let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Mesh Target Clear Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: mesh_target_view_ref,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                            depth_slice: None,
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });
                }

                // C. Process and Render Each Op (Accumulate)
                self.mesh_renderer.begin_frame();
                for (module_id, op) in target_ops {
                    // Determine source texture view
                    let owned_source_view = if let Some(src_id) = op.source_part_id {
                        let tex_name = format!("part_{}_{}", module_id, src_id);
                        if self.texture_pool.has_texture(&tex_name) {
                            Some(self.texture_pool.get_view(&tex_name))
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    let source_view_ref = owned_source_view.as_deref();
                    let effective_view = source_view_ref.or(self.dummy_view.as_ref());

                    if let Some(src_view) = effective_view {
                        // --- Effect Chain Processing ---
                        let mut final_view = src_view;
                        let mut _temp_view_holder: Option<std::sync::Arc<wgpu::TextureView>> = None;

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
                                        | mapmap_core::module::EffectType::Saturation
                                        | mapmap_core::module::EffectType::Colorize => {
                                            Some(mapmap_core::effects::EffectType::ColorAdjust)
                                        }
                                        mapmap_core::module::EffectType::HueShift => {
                                            Some(mapmap_core::effects::EffectType::HueShift)
                                        }
                                        mapmap_core::module::EffectType::ChromaticAberration
                                        | mapmap_core::module::EffectType::RgbSplit => Some(
                                            mapmap_core::effects::EffectType::ChromaticAberration,
                                        ),
                                        mapmap_core::module::EffectType::EdgeDetect => {
                                            Some(mapmap_core::effects::EffectType::EdgeDetect)
                                        }
                                        mapmap_core::module::EffectType::FilmGrain
                                        | mapmap_core::module::EffectType::VHS => {
                                            Some(mapmap_core::effects::EffectType::FilmGrain)
                                        }
                                        mapmap_core::module::EffectType::Vignette => {
                                            Some(mapmap_core::effects::EffectType::Vignette)
                                        }
                                        mapmap_core::module::EffectType::Kaleidoscope => {
                                            Some(mapmap_core::effects::EffectType::Kaleidoscope)
                                        }
                                        mapmap_core::module::EffectType::Wave => {
                                            Some(mapmap_core::effects::EffectType::Wave)
                                        }
                                        mapmap_core::module::EffectType::Glitch => {
                                            Some(mapmap_core::effects::EffectType::Glitch)
                                        }
                                        mapmap_core::module::EffectType::Mirror => {
                                            Some(mapmap_core::effects::EffectType::Mirror)
                                        }
                                        mapmap_core::module::EffectType::ShaderGraph(id) => {
                                            Some(mapmap_core::effects::EffectType::ShaderGraph(*id))
                                        }
                                        _ => {
                                            tracing::warn!(
                                                "Effect {:?} not implemented",
                                                mod_effect
                                            );
                                            None
                                        }
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
                                    &self.shader_graph_manager,
                                    time,
                                    w,
                                    h,
                                );
                                _temp_view_holder = Some(target_view);
                                final_view = _temp_view_holder.as_ref().unwrap();
                            }
                        }

                        // --- Render Mesh (Warping) ---
                        {
                            let (vertex_buffer, index_buffer, index_count) =
                                self.mesh_buffer_cache.get_buffers(
                                    &self.backend.device,
                                    op.layer_part_id,
                                    &op.mesh.to_mesh(),
                                );

                            // No manual transform needed - MeshRenderer handles [0,1] -> [-1,1] conversion internally
                            let transform = glam::Mat4::IDENTITY;
                            let uniform_bind_group =
                                self.mesh_renderer.get_uniform_bind_group_with_source_props(
                                    &self.backend.queue,
                                    transform,
                                    op.opacity * op.source_props.opacity,
                                    op.source_props.flip_horizontal,
                                    op.source_props.flip_vertical,
                                    op.source_props.brightness,
                                    op.source_props.contrast,
                                    op.source_props.saturation,
                                    op.source_props.hue_shift,
                                );
                            let texture_bind_group =
                                self.mesh_renderer.create_texture_bind_group(final_view);

                            let mut render_pass =
                                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                    label: Some("Mesh Render Pass"),
                                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                        view: mesh_target_view_ref,
                                        resolve_target: None,
                                        ops: wgpu::Operations {
                                            load: wgpu::LoadOp::Load, // ACCUMULATE
                                            store: wgpu::StoreOp::Store,
                                        },
                                        depth_slice: None,
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
                    } else {
                        // Log missing view?
                    }
                } // End Loop

                // D. Post-Processing
                if needs_post_processing {
                    let post_input_view = mesh_target_view_ref;
                    let need_double_pass = use_edge_blend && use_color_calib;
                    let mut blend_output_temp_view_opt: Option<wgpu::TextureView> = None;

                    if need_double_pass {
                        let width = window_context.surface_config.width;
                        let height = window_context.surface_config.height;
                        // Optimization: reuse text logic from original... for brevity assume recreation or fetch
                        let recreate = if let Some(tex) = self.output_temp_textures.get(&output_id)
                        {
                            tex.width() != width || tex.height() != height
                        } else {
                            true
                        };

                        if recreate {
                            // Create tex... logic abbreviated but necessary
                            let texture =
                                self.backend
                                    .device
                                    .create_texture(&wgpu::TextureDescriptor {
                                        label: Some("Blend Temp"),
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
                        blend_output_temp_view_opt = Some(
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
                            blend_output_temp_view_opt.as_ref().unwrap()
                        } else {
                            &view
                        };
                        let bind_group = renderer.create_texture_bind_group(post_input_view);
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
                                    depth_slice: None,
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
                            blend_output_temp_view_opt.as_ref().unwrap()
                        } else {
                            post_input_view
                        };
                        // Output is always Surface
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
                                    depth_slice: None,
                                })],
                                depth_stencil_attachment: None,
                                occlusion_query_set: None,
                                timestamp_writes: None,
                            });
                        renderer.render(&mut render_pass, &bind_group, &uniform_bind_group);
                    }
                }
            }
        }

        // 1. Submit Main Rendering Commands
        // We merged egui submission into this one

        // 2. Egui Render Pass (Sequential - using shared encoder)
        if let Some((tris, screen_descriptor)) = egui_render_data {
            let egui_renderer = &self.egui_renderer;

            // Use the main encoder instead of creating a new one
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Egui Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

                // SAFETY: We transmute BOTH the renderer and the render pass reference to break
                // the lifetime dependency inferred by the compiler.
                let renderer_static: &'static egui_wgpu::Renderer =
                    unsafe { std::mem::transmute(egui_renderer) };

                let render_pass_static: &mut wgpu::RenderPass<'static> =
                    unsafe { std::mem::transmute(&mut render_pass) };

                renderer_static.render(render_pass_static, &tris, &screen_descriptor);
            }
        }

        // Final single submit
        let command_buffer = encoder.finish();
        self.backend.queue.submit(Some(command_buffer));

        surface_texture.present();

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
