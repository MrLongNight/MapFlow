//! Application initialization logic.

use super::app_struct::App;
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
    audio::{
        analyzer_v2::{AudioAnalyzerV2, AudioAnalyzerV2Config},
        backend::{cpal_backend::CpalBackend, AudioBackend},
    },
    media_library::MediaLibrary,
    AppState, ModuleEvaluator,
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
    pub async fn new(elwt: &winit::event_loop::ActiveEventLoop) -> Result<Self> {
        // Load user config early to get preferences
        let saved_config = mapmap_ui::config::UserConfig::load();

        let backend = WgpuBackend::new(saved_config.preferred_gpu.as_deref()).await?;

        // Version marker to confirm correct build is running
        tracing::info!(">>> BUILD VERSION: 2026-02-16-FIX-BEVY-HEADLESS <<<");

        // Initialize renderers
        let texture_pool = TexturePool::new(backend.device.clone());
        let compositor = Compositor::new(backend.device.clone(), backend.surface_format())?;
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

        // Create main window with saved geometry
        let main_window_id = window_manager.create_main_window_with_geometry(
            elwt,
            &backend,
            saved_config.window_width,
            saved_config.window_height,
            saved_config.window_x,
            saved_config.window_y,
            saved_config.window_maximized,
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

            // --- SELF-HEAL: Ensure Output Windows exist for active Projector nodes ---
            let existing_output_ids: std::collections::HashSet<u64> = state
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

            // --- SELF-HEAL: Remove dangling connections ---
            let mut graph_fixed = false;
            for module in state.module_manager_mut().modules_mut() {
                let part_ids: std::collections::HashSet<u64> =
                    module.parts.iter().map(|p| p.id).collect();
                info!(
                    "Self-Heal: Module '{}' has nodes: {:?}",
                    module.name, part_ids
                );

                let initial_count = module.connections.len();
                module.connections.retain(|c| {
                    let from_exists = part_ids.contains(&c.from_part);
                    let to_exists = part_ids.contains(&c.to_part);
                    if !from_exists {
                        warn!("Self-Heal: Removing connection from non-existent node {} in module '{}'", c.from_part, module.name);
                    }
                    if !to_exists {
                        warn!("Self-Heal: Removing connection to non-existent node {} in module '{}'", c.to_part, module.name);
                    }
                    from_exists && to_exists
                });
                let final_count = module.connections.len();
                if initial_count != final_count {
                    info!(
                        "Self-Heal: Cleaned {} dangling connections in module '{}'",
                        initial_count - final_count,
                        module.name
                    );
                    graph_fixed = true;
                }
            }
            if graph_fixed {
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
                // Top-Left (0, 0) -> UV 0,0
                mapmap_core::MeshVertex::new(glam::Vec2::new(0.0, 0.0), glam::Vec2::new(0.0, 0.0)),
                // Top-Right (1, 0) -> UV 1,0
                mapmap_core::MeshVertex::new(glam::Vec2::new(1.0, 0.0), glam::Vec2::new(1.0, 0.0)),
                // Bottom-Right (1, 1) -> UV 1,1
                mapmap_core::MeshVertex::new(glam::Vec2::new(1.0, 1.0), glam::Vec2::new(1.0, 1.0)),
                // Bottom-Left (0, 1) -> UV 0,1
                mapmap_core::MeshVertex::new(glam::Vec2::new(0.0, 1.0), glam::Vec2::new(0.0, 1.0)),
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
            let view =
                std::sync::Arc::new(texture.create_view(&wgpu::TextureViewDescriptor::default()));
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
            mcp_receiver,
            action_sender,
            control_manager,
            exit_requested: false,
            restart_requested: false,
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
            render_ops: Vec::new(),
            edge_blend_renderer,
            color_calibration_renderer,
            output_temp_textures: std::collections::HashMap::new(),
            preview_texture_cache: HashMap::new(),
            output_preview_cache: HashMap::new(),
            preview_quad_buffers,
            hue_controller,
            tokio_runtime,
            media_manager_ui: MediaManagerUI::new(),
            media_library: {
                let mut lib = MediaLibrary::new();
                // Add default search paths for media
                if let Some(video_dir) = dirs::video_dir() {
                    lib.add_scan_path(video_dir);
                }
                // Also add project relative media dir if it exists
                let project_media = std::path::PathBuf::from("resources/app_videos");
                if project_media.exists() {
                    lib.add_scan_path(project_media);
                }
                lib
            },
            bevy_runner: Some(mapmap_bevy::BevyRunner::new()),
        };

        // --- INITIALIZATION STATUS REPORT ---
        info!("==========================================");
        info!("   MapFlow Initialization Status Report   ");
        info!("------------------------------------------");
        info!(
            "- Render Backend: {} ({:?})",
            app.backend.adapter_info().name,
            app.backend.adapter_info().backend
        );
        info!(
            "- Edge Blend:     {}",
            if app.edge_blend_renderer.is_some() {
                "ENABLED"
            } else {
                "DISABLED"
            }
        );
        info!(
            "- Color Calib:    {}",
            if app.color_calibration_renderer.is_some() {
                "ENABLED"
            } else {
                "DISABLED"
            }
        );
        info!("- Bevy Engine:    INITIALIZED");

        #[cfg(feature = "midi")]
        info!(
            "- MIDI System:    {}",
            if app.midi_handler.is_some() {
                "CONNECTED"
            } else {
                "DISCONNECTED"
            }
        );

        info!(
            "- Hue System:     {}",
            if !app.ui_state.user_config.hue_config.bridge_ip.is_empty() {
                "CONFIGURED"
            } else {
                "UNCONFIGURED"
            }
        );
        info!("- Media Library:  {} items", app.media_library.items.len());
        info!("==========================================");

        Ok(app)
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
