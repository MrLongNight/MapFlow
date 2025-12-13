//! MapMap - Open source Vj Projection Mapping Software
//!
//! This is the main application crate for MapMap.

#![warn(missing_docs)]

mod window_manager;

use anyhow::Result;
use mapmap_core::{OutputId, OutputManager};
use mapmap_render::WgpuBackend;
use mapmap_ui::{AppUI, ImGuiContext};
use tracing::{error, info};
use window_manager::WindowManager;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
};

const INITIAL_WIDTH: u32 = 1920;
const INITIAL_HEIGHT: u32 = 1080;

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
    /// The output manager.
    #[allow(dead_code)] // TODO: Prüfen, ob dieses Feld dauerhaft benötigt wird!
    output_manager: OutputManager,
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

        let ui_state = AppUI::default();

        Ok(Self {
            window_manager,
            imgui_context,
            ui_state,
            backend,
            output_manager: OutputManager::new((INITIAL_WIDTH, INITIAL_HEIGHT)),
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

        Ok(())
    }

    /// Renders a single frame for a given output.
    fn render(&mut self, output_id: OutputId) -> Result<()> {
        let window_context = self.window_manager.get(output_id).unwrap();

        let surface_texture = window_context.surface.get_current_texture()?;

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.backend
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        if output_id == 0 {
            // Render the main window (UI)
            self.imgui_context.render(
                &window_context.window,
                &self.backend.device,
                &self.backend.queue,
                &mut encoder,
                &view,
                |ui| {
                    self.ui_state.render_menu_bar(ui);
                    self.ui_state.render_controls(ui);
                    self.ui_state.render_stats(ui, 60.0, 16.6); // Fake stats for now
                                                                // TODO: Render other panels
                },
            );
        } else {
            // Render output window (Clear to black)
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
