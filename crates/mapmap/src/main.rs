//! MapFlow - Open source Vj Projection Mapping Software
//!
//! This is the main application crate for MapFlow.

#![warn(missing_docs)]

mod window_manager;

use anyhow::Result;
use egui_wgpu::Renderer;
use egui_winit::State;
use mapmap_control::{shortcuts::Action, ControlManager};
use mapmap_core::{
    audio::{backend::cpal_backend::CpalBackend, backend::AudioBackend, AudioAnalyzer},
    AppState, OutputId,
};

use mapmap_mcp::{McpAction, McpServer};
// Define McpAction locally or import if we move it to core later -> Removed local definition

use crossbeam_channel::{unbounded, Receiver};
use mapmap_io::{load_project, save_project};
use mapmap_render::{OscillatorRenderer, WgpuBackend};
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
    /// Last frame update timestamp.
    last_update: std::time::Instant,
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
}

impl App {
    /// Creates a new `App`.
    async fn new(event_loop: &EventLoop<()>) -> Result<Self> {
        let backend = WgpuBackend::new().await?;
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
        let egui_renderer = Renderer::new(
            &backend.device,
            format,
            None,
            1,
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

        let mut app = Self {
            window_manager,
            ui_state,
            backend,
            state,
            audio_backend,
            audio_analyzer,
            audio_devices,
            egui_context,
            egui_state,
            egui_renderer,
            last_autosave: std::time::Instant::now(),
            last_update: std::time::Instant::now(),
            mcp_receiver,
            control_manager: ControlManager::new(),
            exit_requested: false,
            oscillator_renderer,
            dummy_texture: None,
            dummy_view: None,
        };

        // Create initial dummy texture
        app.create_dummy_texture(width, height, format);

        Ok(app)
    }

    /// Creates or recreates the dummy texture for effect input.
    fn create_dummy_texture(&mut self, width: u32, height: u32, format: wgpu::TextureFormat) {
        let texture = self.backend.device.create_texture(&wgpu::TextureDescriptor {
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
                        let new_size = if let Some(window_context) =
                            self.window_manager.get_mut(output_id)
                        {
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
                        if let Some((width, height, format)) = new_size {
                            self.create_dummy_texture(width, height, format);
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
                // Autosave check (every 5 minutes)
                if self.state.dirty
                    && self.last_autosave.elapsed() >= std::time::Duration::from_secs(300)
                {
                    let autosave_path = PathBuf::from(".mapmap_autosave");
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
                        let analysis = self.audio_analyzer.process_samples(&samples, 0.0);
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
                mapmap_ui::UIAction::Exit => {
                    info!("Exit requested via menu");
                    self.exit_requested = true;
                }
                mapmap_ui::UIAction::OpenSettings => {
                    info!("Settings requested");
                    self.ui_state.show_settings = true;
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
            let mut dashboard_action = None;
            let (tris, screen_descriptor) = {
                let raw_input = self.egui_state.take_egui_input(&window_context.window);
                let full_output = self.egui_context.run(raw_input, |ctx| {
                    let menu_actions = menu_bar::show(ctx, &mut self.ui_state);
                    self.ui_state.actions.extend(menu_actions);

                    // Render Dashboard
                    dashboard_action = self.ui_state.dashboard.ui(ctx, &self.ui_state.i18n);

                    // Migrated Panels Integration (Controls, Stats, Master, Cue)
                    self.ui_state.render_controls(ctx);
                    self.ui_state.render_stats(ctx, 60.0, 16.6);
                    self.ui_state
                        .render_master_controls(ctx, &mut self.state.layer_manager);
                    self.ui_state.cue_panel.show(
                        ctx,
                        &mut self.control_manager,
                        &self.ui_state.i18n,
                        &mut self.ui_state.actions,
                    );

                    // Render Audio Panel
                    if self.ui_state.show_audio {
                        let analysis = self.audio_analyzer.get_latest_analysis();
                        egui::Window::new(self.ui_state.i18n.t("audio-panel-title")).show(
                            ctx,
                            |ui| {
                                if let Some(action) = self.ui_state.audio_panel.ui(
                                    ui,
                                    &self.ui_state.i18n,
                                    Some(&analysis),
                                    &self.audio_devices,
                                    &mut self.ui_state.selected_audio_device,
                                ) {
                                    // Handle device change
                                    if let Some(backend) = &mut self.audio_backend {
                                        backend.stop();
                                    }
                                    self.audio_backend = None;

                                    match CpalBackend::new(Some(action)) {
                                        Ok(mut backend) => {
                                            if let Err(e) = backend.start() {
                                                error!("Failed to start audio stream: {}", e);
                                            } else {
                                                self.audio_backend = Some(backend);
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to initialize audio backend: {}", e);
                                        }
                                    }
                                }
                            },
                        );
                    }

                    // Render Effect Chain Panel
                    self.ui_state
                        .effect_chain_panel
                        .ui(ctx, &self.ui_state.i18n);

                    // Render Layer Panel
                    self.ui_state.layer_panel.show(
                        ctx,
                        &mut self.state.layer_manager,
                        &mut self.ui_state.selected_layer_id,
                        &mut self.ui_state.actions,
                        &self.ui_state.i18n,
                    );

                    // Render Paint Panel
                    self.ui_state.paint_panel.render(
                        ctx,
                        &self.ui_state.i18n,
                        &mut self.state.paint_manager,
                    );

                    // Render Mapping Panel
                    self.ui_state.mapping_panel.show(
                        ctx,
                        &mut self.state.mapping_manager,
                        &mut self.ui_state.actions,
                        &self.ui_state.i18n,
                    );

                    // Render Output Panel
                    self.ui_state.output_panel.show(
                        ctx,
                        &mut self.state.output_manager,
                        &mut self.ui_state.selected_output_id,
                        &mut self.ui_state.actions,
                        &self.ui_state.i18n,
                    );

                    // Render Timeline
                    egui::Window::new("Timeline")
                        .open(&mut self.ui_state.show_timeline)
                        .default_size([800.0, 300.0])
                        .show(ctx, |ui| {
                            let _ = self.ui_state.timeline_panel.ui(ui);
                        });

                    // Render Shader Graph
                    egui::Window::new("Shader Graph")
                        .open(&mut self.ui_state.show_shader_graph)
                        .default_size([800.0, 600.0])
                        .show(ctx, |ui| {
                            let _ = self.ui_state.node_editor_panel.ui(ui, &self.ui_state.i18n);
                        });

                    // Update and render Transform Panel
                    if let Some(selected_id) = self.ui_state.selected_layer_id {
                        if let Some(layer) = self.state.layer_manager.get_layer(selected_id) {
                            self.ui_state
                                .transform_panel
                                .set_transform(&layer.name, &layer.transform);
                        }
                    } else {
                        self.ui_state.transform_panel.clear_selection();
                    }
                    self.ui_state
                        .transform_panel
                        .render(ctx, &self.ui_state.i18n);

                    // Update and show the edge blend panel
                    if let Some(output_id) = self.ui_state.selected_output_id {
                        if let Some(output) = self.state.output_manager.get_output(output_id) {
                            self.ui_state.edge_blend_panel.set_selected_output(output);
                        }
                    } else {
                        self.ui_state.edge_blend_panel.clear_selection();
                    }
                    self.ui_state
                        .edge_blend_panel
                        .show(ctx, &self.ui_state.i18n);

                    // Render Oscillator Panel
                    self.ui_state.oscillator_panel.render(
                        ctx,
                        &self.ui_state.i18n,
                        &mut self.state.oscillator_config,
                    );

                    // Render Settings Window
                    if self.ui_state.show_settings {
                        let mut close_settings = false;
                        egui::Window::new(self.ui_state.i18n.t("menu-file-settings"))
                            .collapsible(false)
                            .resizable(true)
                            .default_size([400.0, 300.0])
                            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                            .show(ctx, |ui| {
                                ui.heading(self.ui_state.i18n.t("menu-file-settings"));
                                ui.separator();

                                // Language selection
                                ui.horizontal(|ui| {
                                    ui.label("Language / Sprache:");
                                    if ui.button("English").clicked() {
                                        self.ui_state.actions.push(
                                            mapmap_ui::UIAction::SetLanguage("en".to_string()),
                                        );
                                    }
                                    if ui.button("Deutsch").clicked() {
                                        self.ui_state.actions.push(
                                            mapmap_ui::UIAction::SetLanguage("de".to_string()),
                                        );
                                    }
                                });

                                ui.separator();

                                // Close button
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::TOP),
                                    |ui| {
                                        if ui.button("✕ Close / Schließen").clicked() {
                                            close_settings = true;
                                        }
                                    },
                                );
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
            if let Some(mapmap_ui::DashboardAction::AudioDeviceChanged(device)) = dashboard_action {
                if let Some(backend) = &mut self.audio_backend {
                    backend.stop();
                }
                self.audio_backend = None;

                match CpalBackend::new(Some(device)) {
                    Ok(mut backend) => {
                        if let Err(e) = backend.start() {
                            error!("Failed to start audio stream: {}", e);
                        } else {
                            self.audio_backend = Some(backend);
                        }
                    }
                    Err(e) => {
                        error!("Failed to initialize audio backend: {}", e);
                    }
                }
            }
        } else {
            // Output-Fenster Management
            if let Some(renderer) = &mut self.oscillator_renderer {
                if self.state.oscillator_config.enabled {
                    if let Some(dummy_view) = &self.dummy_view {
                        renderer.render(
                            &mut encoder,
                            dummy_view,
                            &view,
                            window_context.surface_config.width,
                            window_context.surface_config.height,
                            &self.state.oscillator_config,
                        );
                    }
                } else {
                    // Just clear to black if effect is disabled
                    let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
            }
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
