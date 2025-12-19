//! MapMap - Open source Vj Projection Mapping Software
//!
//! This is the main application crate for MapMap.

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
use mapmap_render::WgpuBackend;
use mapmap_ui::{AppUI, ImGuiContext};
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
    /// The ImGui rendering context.
    imgui_context: ImGuiContext,
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
    /// The egui context.
    egui_context: egui::Context,
    /// The egui state.
    egui_state: State,
    /// The egui renderer.
    egui_renderer: Renderer,
    /// Last autosave timestamp.
    last_autosave: std::time::Instant,
    /// Receiver for MCP commands
    mcp_receiver: Receiver<McpAction>,
    /// Unified control manager
    control_manager: ControlManager,
}

impl App {
    /// Creates a new `App`.
    async fn new(event_loop: &EventLoop<()>) -> Result<Self> {
        let backend = WgpuBackend::new().await?;
        let mut window_manager = WindowManager::new();

        // Create main window
        let main_window_id = window_manager.create_main_window(event_loop, &backend)?;
        let main_window_context = window_manager.get(main_window_id).unwrap();

        // Initialize UI
        let imgui_context = ImGuiContext::new(
            &main_window_context.window,
            &backend.device,
            &backend.queue,
            main_window_context.surface_config.format,
        );

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
        ui_state.dashboard.set_audio_devices(audio_devices.clone());

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
            &main_window_context.window,
            None,
            None,
        );
        let egui_renderer = Renderer::new(
            &backend.device,
            main_window_context.surface_config.format,
            None,
            1,
        );

        Ok(Self {
            window_manager,
            imgui_context,
            ui_state,
            backend,
            state,
            audio_backend,
            audio_analyzer,
            egui_context,
            egui_state,
            egui_renderer,
            last_autosave: std::time::Instant::now(),
            mcp_receiver,
            control_manager: ControlManager::new(),
        })
    }

    /// Runs the application loop.
    pub fn run(mut self, event_loop: EventLoop<()>) {
        info!("Entering event loop");

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        let _ = event_loop.run(move |event, elwt| {
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
        if let Event::WindowEvent { window_id, .. } = &event {
            let output_id = self
                .window_manager
                .get_output_id_from_window_id(*window_id)
                .unwrap_or(0);

            if let Some(window_context) = self.window_manager.get(output_id) {
                if output_id == 0 {
                    self.imgui_context
                        .handle_event(&window_context.window, &event);
                }
            }
        }

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
                        if let Some(window_context) = self.window_manager.get_mut(output_id) {
                            if size.width > 0 && size.height > 0 {
                                window_context.surface_config.width = size.width;
                                window_context.surface_config.height = size.height;
                                window_context.surface.configure(
                                    &self.backend.device,
                                    &window_context.surface_config,
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
                mapmap_ui::UIAction::SaveProject(path_str) => {
                    let path = if path_str.is_empty() {
                        if let Some(path) = FileDialog::new()
                            .add_filter("MapMap Project", &["mapmap", "ron", "json"])
                            .set_file_name("project.mapmap")
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
                            .add_filter("MapMap Project", &["mapmap", "ron", "json"])
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
                    self.ui_state.recent_files.insert(0, p);
                    // Limit to 10
                    if self.ui_state.recent_files.len() > 10 {
                        self.ui_state.recent_files.pop();
                    }
                }
            }
            Err(e) => error!("Failed to load project: {}", e),
        }
    }

    /// Renders a single frame for a given output.
    fn render(&mut self, output_id: OutputId) -> Result<()> {
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
            // --------- ImGui: UI zeichnen (mutable Borrow ist hier auf das Minimum begrenzt) ----------
            self.imgui_context
                .prepare_frame(&window_context.window, |ui| {
                    self.ui_state.render_menu_bar(ui);
                    self.ui_state.render_controls(ui);
                    self.ui_state.render_stats(ui, 60.0, 16.6);

                    // Panels
                    self.ui_state
                        .render_layer_panel(ui, &mut self.state.layer_manager);
                    self.ui_state
                        .render_paint_panel(ui, &mut self.state.paint_manager);
                    self.ui_state
                        .render_mapping_panel(ui, &mut self.state.mapping_manager);
                    self.ui_state
                        .render_transform_panel(ui, &mut self.state.layer_manager);
                    self.ui_state
                        .render_master_controls(ui, &mut self.state.layer_manager);
                });

            // --------- egui: UI separat zeichnen ---------
            let mut dashboard_action = None;
            let (tris, screen_descriptor) = {
                let raw_input = self.egui_state.take_egui_input(&window_context.window);
                let full_output = self.egui_context.run(raw_input, |ctx| {
                    egui::Window::new("Dashboard").show(ctx, |ui| {
                        dashboard_action = self.ui_state.dashboard.ui(ui, &self.ui_state.i18n);
                    });

                    // Render Effect Chain Panel
                    self.ui_state
                        .effect_chain_panel
                        .ui(ctx, &self.ui_state.i18n);
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

            self.imgui_context.render_frame(
                &self.backend.device,
                &self.backend.queue,
                &mut encoder,
                &view,
            );

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
            // Output-Fenster Management: Clear To Black oder dein Mapping-Rendering!
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Output Render Pass"),
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

        self.backend.queue.submit(Some(encoder.finish()));
        surface_texture.present();

        Ok(())
    }
}

/// The main entry point for the application.
fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    info!("Starting MapMap...");

    // Create the event loop
    let event_loop = EventLoop::new()?;

    // Create the app
    let app = pollster::block_on(App::new(&event_loop))?;

    // Run the app
    app.run(event_loop);

    Ok(())
}
