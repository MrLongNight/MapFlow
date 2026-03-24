//! Application initialization logic.

use super::app_struct::{App, InitializationConfig};
use crate::media_manager_ui::MediaManagerUI;
use crate::window_manager::WindowManager;
use anyhow::Result;
use crossbeam_channel::unbounded;
use egui_wgpu::Renderer;
use egui_winit::State;
use mapmap_control::hue::controller::HueController;
#[cfg(feature = "midi")]
use mapmap_control::midi::MidiInputHandler;
use mapmap_control::ControlManager;
use mapmap_core::{
    audio::backend::{cpal_backend::CpalBackend, AudioBackend},
    media_library::MediaLibrary,
    runtime_paths, AppState, ModuleEvaluator,
};
use mapmap_io::load_project;
use mapmap_mcp::McpServer;
use mapmap_render::{
    ColorCalibrationRenderer, Compositor, EdgeBlendRenderer, EffectChainRenderer, MeshBufferCache,
    MeshRenderer, OscillatorRenderer, QuadRenderer, TexturePool, WgpuBackend,
};
use mapmap_ui::AppUI;
use std::collections::{HashMap, VecDeque};
use std::thread;
use tracing::{error, info, warn};

impl App {
    /// Creates a new `App`.
    pub async fn new(
        elwt: &winit::event_loop::ActiveEventLoop,
        config: InitializationConfig,
    ) -> Result<Self> {
        // Load user config early to get preferences
        let saved_config = mapmap_ui::config::UserConfig::load();

        let backend = WgpuBackend::new(saved_config.preferred_gpu.as_deref()).await?;

        // Version marker to confirm correct build is running
        tracing::info!(">>> BUILD VERSION: 2026-03-19-VISUAL-TEST-READY <<<");

        // Initialize renderers
        let texture_pool = TexturePool::new(backend.device.clone());
        let compositor = Compositor::new(backend.device.clone(), backend.surface_format())?;
        let (effect_chain_renderer, preview_effect_chain_renderer) =
            Self::init_renderers(&backend)?;
        let mesh_renderer = MeshRenderer::new(backend.device.clone(), backend.surface_format())?;
        let mesh_buffer_cache = MeshBufferCache::new();
        let quad_renderer = QuadRenderer::new(&backend.device, backend.surface_format())?;

        // Initialize advanced output renderers
        let (edge_blend_renderer, color_calibration_renderer) =
            Self::init_advanced_renderers(&backend);

        let mut window_manager = WindowManager::new();

        // Create Tokio runtime
        let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime");

        // Create main window with saved geometry
        let main_window_id = window_manager.create_main_window_with_geometry(
            elwt,
            &backend,
            if config.is_automation {
                Some(1280)
            } else {
                saved_config.window_width
            },
            if config.is_automation {
                Some(720)
            } else {
                saved_config.window_height
            },
            saved_config.window_x,
            saved_config.window_y,
            if config.is_automation {
                false
            } else {
                saved_config.window_maximized
            },
            saved_config.vsync_mode,
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
            wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
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
        Self::init_ui_assets(&mut ui_state);

        // Initialize state, trying to load autosave first unless skipped
        let mut state = AppState::new("New Project");
        state.settings_mut().log_config.level = saved_config.log_level.as_str().to_string();

        if !config.skip_autosave {
            Self::load_autosave(&mut state, &saved_config);
            Self::run_self_heal(&mut state);
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

        let audio_backend = if config.skip_audio {
            info!("Initialization: Skipping audio backend");
            None
        } else {
            Self::init_audio(device_to_use)
        };

        // Initialize Audio Analyzer
        let audio_analyzer = mapmap_core::audio::AudioAnalyzer::new(state.audio_config.clone());

        // Start MCP Server
        let (mcp_sender, mcp_receiver) = unbounded();
        let action_sender = mcp_sender.clone();

        if !config.skip_mcp {
            Self::start_mcp_server(mcp_sender);
        }

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
        let oscillator_renderer = Self::init_oscillator_renderer(&backend, format, &state);

        // Initialize icons from assets directory
        let assets_path = runtime_paths::assets_dir();
        ui_state.initialize_icons(&egui_context, &assets_path);
        ui_state.user_config.theme.apply(&egui_context);

        // Initialize preview quad buffers
        let preview_quad_buffers = Self::init_preview_quad_buffers(&mesh_renderer);

        let mut hue_controller = Self::init_hue_controller(&ui_state);
        if !config.skip_hue {
            Self::connect_hue(&mut hue_controller, &ui_state, &tokio_runtime);
        }

        let mut control_manager = ControlManager::new();

        #[cfg(feature = "http-api")]
        if ui_state.user_config.web_api_enabled {
            let web_config =
                mapmap_control::web::WebServerConfig::new(ui_state.user_config.web_api_port);
            if let Err(e) = control_manager.init_web_server(web_config) {
                error!("Failed to initialize Web API: {}", e);
            }
        }

        let sys_info = sysinfo::System::new_all();
        let (dummy_texture, dummy_view) =
            Self::create_initial_dummy_texture(&backend, width, height, format);

        #[cfg(feature = "midi")]
        let midi_handler = if config.skip_midi {
            info!("Initialization: Skipping MIDI");
            None
        } else {
            Self::init_midi()
        };

        let app = Self {
            window_manager,
            ui_state,
            backend,
            texture_pool: std::sync::Arc::new(texture_pool),
            _compositor: compositor,
            effect_chain_renderer,
            preview_effect_chain_renderer,
            mesh_renderer,
            mesh_buffer_cache,
            _quad_renderer: quad_renderer,
            _composite_texture: composite_texture,
            layer_ping_pong,
            state,
            history: mapmap_core::History::default(),
            audio_backend,
            audio_analyzer,
            audio_devices,
            egui_context,
            egui_state,
            egui_renderer,
            last_autosave: std::time::Instant::now(),
            last_update: std::time::Instant::now(),
            start_time: std::time::Instant::now(),
            startup_animation: crate::app::core::app_struct::StartupAnimationState::default(),
            last_texture_gc: std::time::Instant::now(),
            mcp_receiver,
            action_sender,
            control_manager,
            exit_requested: false,
            restart_requested: false,
            oscillator_renderer,
            dummy_texture: Some(dummy_texture),
            dummy_view: Some(dummy_view),
            module_evaluator: ModuleEvaluator::new(),
            last_graph_revision: 0,
            cached_output_infos: Vec::new(),
            frame_counter: 0,
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
                Some(0)
            },
            #[cfg(feature = "ndi")]
            ndi_receivers: std::collections::HashMap::new(),
            #[cfg(feature = "ndi")]
            ndi_senders: std::collections::HashMap::new(),
            #[cfg(feature = "ndi")]
            ndi_readbacks: std::collections::HashMap::new(),

            output_assignments: std::collections::HashMap::new(),
            shader_graph_manager: mapmap_render::ShaderGraphManager::new(),
            recent_effect_configs: mapmap_core::RecentEffectConfigs::with_persistence(
                dirs::data_dir()
                    .unwrap_or(std::path::PathBuf::from("."))
                    .join("MapFlow")
                    .join("recent_effect_configs.json"),
            ),
            render_queue: crate::app::core::app_struct::RuntimeRenderQueue::default(),
            edge_blend_renderer,
            color_calibration_renderer,
            edge_blend_cache: std::collections::HashMap::new(),
            edge_blend_texture_cache: std::collections::HashMap::new(),
            output_temp_textures: std::collections::HashMap::new(),
            preview_texture_cache: HashMap::new(),
            output_preview_cache: HashMap::new(),
            video_diagnostic_log_times: HashMap::new(),
            preview_quad_buffers,
            hue_controller,
            tokio_runtime,
            media_manager_ui: MediaManagerUI::new(),
            media_library: {
                let mut lib = MediaLibrary::new();
                if let Some(video_dir) = dirs::video_dir() {
                    lib.add_scan_path(video_dir);
                }
                if let Some(project_media) = runtime_paths::existing_resource_path("app_videos") {
                    lib.add_scan_path(project_media);
                }
                lib
            },
            bevy_runner: Some(mapmap_bevy::BevyRunner::new()),
        };

        app.print_init_report();

        Ok(app)
    }

    // Helper methods for modular initialization

    fn init_renderers(backend: &WgpuBackend) -> Result<(EffectChainRenderer, EffectChainRenderer)> {
        let effect_chain_renderer = EffectChainRenderer::new(
            backend.device.clone(),
            backend.queue.clone(),
            backend.surface_format(),
        )?;
        let preview_effect_chain_renderer = EffectChainRenderer::new(
            backend.device.clone(),
            backend.queue.clone(),
            backend.surface_format(),
        )?;
        Ok((effect_chain_renderer, preview_effect_chain_renderer))
    }

    fn init_advanced_renderers(
        backend: &WgpuBackend,
    ) -> (Option<EdgeBlendRenderer>, Option<ColorCalibrationRenderer>) {
        let edge_blend_renderer =
            EdgeBlendRenderer::new(backend.device.clone(), backend.surface_format())
                .map_err(|e| {
                    warn!("Failed to create edge blend renderer: {}", e);
                    e
                })
                .ok();

        let color_calibration_renderer =
            ColorCalibrationRenderer::new(backend.device.clone(), backend.surface_format())
                .map_err(|e| {
                    warn!("Failed to create color calibration renderer: {}", e);
                    e
                })
                .ok();

        (edge_blend_renderer, color_calibration_renderer)
    }

    fn init_ui_assets(ui_state: &mut AppUI) {
        #[cfg(feature = "midi")]
        {
            if let Some(path) =
                runtime_paths::existing_resource_path("controllers/ecler_nuo4/elements.json")
            {
                match std::fs::read_to_string(&path) {
                    Ok(json) => {
                        if let Err(e) = ui_state.controller_overlay.load_elements(&json) {
                            error!("Failed to parse elements.json: {}", e);
                        } else {
                            info!("Loaded controller elements from {:?}", path);
                        }
                    }
                    Err(e) => {
                        error!("Failed to read elements.json from {:?}: {}", path, e)
                    }
                }
            }
        }
    }

    fn load_autosave(state: &mut AppState, saved_config: &mapmap_ui::config::UserConfig) {
        let autosave_path =
            dirs::data_local_dir().map(|p| p.join("MapFlow").join("autosave.mflow"));
        if let Some(path) = &autosave_path {
            if path.exists() {
                info!("Found autosave at {:?}, attempting to load...", path);
                match load_project(path) {
                    Ok(loaded_state) => {
                        info!("Successfully loaded autosave.");
                        *state = loaded_state;
                        state.settings_mut().log_config.level =
                            saved_config.log_level.as_str().to_string();
                    }
                    Err(e) => {
                        error!("Failed to load autosave: {}", e);
                    }
                }
            } else {
                info!("No autosave found at {:?}, starting new project.", path);
            }
        }
    }

    fn run_self_heal(state: &mut AppState) {
        // --- Reconcile Output IDs ---
        let valid_outputs: HashMap<String, u64> = state
            .output_manager
            .outputs()
            .iter()
            .map(|o| (o.name.clone(), o.id))
            .collect();
        let valid_ids: Vec<u64> = valid_outputs.values().cloned().collect();

        let mut fixed_count = 0;
        for module in state.module_manager_mut().modules_mut() {
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
                        }
                    }
                }
            }
        }
        if fixed_count > 0 {
            state.dirty = true;
        }

        // --- Ensure Output Windows exist ---
        let existing_output_ids: Vec<u64> = state
            .output_manager
            .outputs()
            .iter()
            .map(|o| o.id)
            .collect();
        let mut missing_outputs = Vec::new();
        for module in state.module_manager.modules() {
            for part in &module.parts {
                if let mapmap_core::module::ModulePartType::Output(
                    mapmap_core::module::OutputType::Projector { id, name, .. },
                ) = &part.part_type
                {
                    if !existing_output_ids.contains(id) {
                        missing_outputs.push((*id, name.clone()));
                    }
                }
            }
        }
        for (id, name) in missing_outputs {
            info!(
                "Self-Heal: Creating missing Output Window '{}' (ID {})",
                name, id
            );
            state.output_manager_mut().add_output(
                name,
                mapmap_core::output::CanvasRegion::new(0.0, 0.0, 1.0, 1.0),
                (1920, 1080),
            );
        }

        // --- Remove dangling connections ---
        for module in state.module_manager_mut().modules_mut() {
            let part_ids: Vec<u64> = module.parts.iter().map(|p| p.id).collect();
            module
                .connections
                .retain(|c| part_ids.contains(&c.from_part) && part_ids.contains(&c.to_part));
        }
    }

    fn init_audio(device_to_use: Option<String>) -> Option<CpalBackend> {
        let mut backend = match CpalBackend::new(device_to_use) {
            Ok(backend) => Some(backend),
            Err(e) => {
                error!("Failed to initialize audio backend: {}", e);
                None
            }
        };
        if let Some(b) = &mut backend {
            if let Err(e) = b.start() {
                error!("Failed to start audio stream: {}", e);
                return None;
            }
        }
        backend
    }

    fn start_mcp_server(mcp_sender: crossbeam_channel::Sender<mapmap_mcp::McpAction>) {
        thread::spawn(move || {
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
    }

    fn init_oscillator_renderer(
        backend: &WgpuBackend,
        format: wgpu::TextureFormat,
        state: &AppState,
    ) -> Option<OscillatorRenderer> {
        match OscillatorRenderer::new(
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
        }
    }

    fn init_preview_quad_buffers(
        mesh_renderer: &MeshRenderer,
    ) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let preview_mesh = mapmap_core::Mesh {
            mesh_type: mapmap_core::MeshType::Quad,
            vertices: vec![
                mapmap_core::MeshVertex::new(glam::Vec2::new(0.0, 0.0), glam::Vec2::new(0.0, 0.0)),
                mapmap_core::MeshVertex::new(glam::Vec2::new(1.0, 0.0), glam::Vec2::new(1.0, 0.0)),
                mapmap_core::MeshVertex::new(glam::Vec2::new(1.0, 1.0), glam::Vec2::new(1.0, 1.0)),
                mapmap_core::MeshVertex::new(glam::Vec2::new(0.0, 1.0), glam::Vec2::new(0.0, 1.0)),
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
            revision: 0,
        };
        let (vb, ib) = mesh_renderer.create_mesh_buffers(&preview_mesh);
        (vb, ib, preview_mesh.indices.len() as u32)
    }

    fn init_hue_controller(ui_state: &AppUI) -> HueController {
        let ui_hue_conf = &ui_state.user_config.hue_config;
        HueController::new(mapmap_control::hue::models::HueConfig {
            bridge_ip: ui_hue_conf.bridge_ip.clone(),
            username: ui_hue_conf.username.clone(),
            client_key: ui_hue_conf.client_key.clone(),
            application_id: String::new(),
            entertainment_group_id: ui_hue_conf.entertainment_area.clone(),
        })
    }

    fn connect_hue(controller: &mut HueController, ui_state: &AppUI, rt: &tokio::runtime::Runtime) {
        if !ui_state.user_config.hue_config.bridge_ip.is_empty()
            && ui_state.user_config.hue_config.auto_connect
        {
            info!("Initializing Hue Controller...");
            if let Err(e) = rt.block_on(controller.connect()) {
                warn!("Hue Controller initial connection failed: {}", e);
            }
        }
    }

    fn create_initial_dummy_texture(
        backend: &WgpuBackend,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> (wgpu::Texture, std::sync::Arc<wgpu::TextureView>) {
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
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view =
            std::sync::Arc::new(texture.create_view(&wgpu::TextureViewDescriptor::default()));
        (texture, view)
    }

    #[cfg(feature = "midi")]
    fn init_midi() -> Option<MidiInputHandler> {
        match MidiInputHandler::new() {
            Ok(mut handler) => {
                info!("MIDI initialized");
                if let Ok(ports) = MidiInputHandler::list_ports() {
                    if !ports.is_empty() {
                        let _ = handler.connect(0);
                    }
                }
                Some(handler)
            }
            Err(e) => {
                error!("Failed to init MIDI: {}", e);
                None
            }
        }
    }

    fn print_init_report(&self) {
        info!("==========================================");
        info!("   MapFlow Initialization Status Report   ");
        info!("------------------------------------------");
        info!(
            "- Render Backend: {} ({:?})",
            self.backend.adapter_info().name,
            self.backend.adapter_info().backend
        );
        info!(
            "- Edge Blend:     {}",
            if self.edge_blend_renderer.is_some() {
                "ENABLED"
            } else {
                "DISABLED"
            }
        );
        info!(
            "- Color Calib:    {}",
            if self.color_calibration_renderer.is_some() {
                "ENABLED"
            } else {
                "DISABLED"
            }
        );
        info!("- Bevy Engine:    INITIALIZED");
        #[cfg(feature = "midi")]
        info!(
            "- MIDI System:    {}",
            if self.midi_handler.is_some() {
                "CONNECTED"
            } else {
                "DISCONNECTED"
            }
        );
        info!(
            "- Hue System:     {}",
            if !self.ui_state.user_config.hue_config.bridge_ip.is_empty() {
                "CONFIGURED"
            } else {
                "UNCONFIGURED"
            }
        );
        info!("- Media Library:  {} items", self.media_library.items.len());
        info!("==========================================");
    }

    /// Creates or recreates the dummy texture for effect input.
    pub fn create_dummy_texture(&mut self, width: u32, height: u32, format: wgpu::TextureFormat) {
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
        self.dummy_view = Some(std::sync::Arc::new(
            texture.create_view(&wgpu::TextureViewDescriptor::default()),
        ));
        self.dummy_texture = Some(texture);
    }
}
