//! MapMap - Open source Vj Projection Mapping Software
//!
//! This is the main application crate for MapMap.

#![warn(missing_docs)]

mod window_manager;

use anyhow::Result;
use egui_wgpu::wgpu;
use egui_winit::winit;
use mapmap_core::{OutputId, OutputManager};
use mapmap_render::{Color, Pass, RenderContext, Renderer, Texture, TextureHandle, Vertex};
use mapmap_ui::UI;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;
use wgpu::TextureView;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowId,
};
use window_manager::{WindowContext, WindowManager};

const INITIAL_WIDTH: u32 = 1920;
const INITIAL_HEIGHT: u32 = 1080;

/// The main application state.
struct App<'a> {
    /// Manages all application windows.
    window_manager: WindowManager<'a>,
    /// The main UI.
    ui: UI,
    /// The application's render context.
    render_context: RenderContext,
    /// The main renderer for the application.
    renderer: Renderer,
    /// The output manager.
    output_manager: OutputManager,
    /// A map of paint IDs to their corresponding textures.
    paint_textures: HashMap<u64, TextureHandle>,
    /// A map of layer IDs to their corresponding textures.
    layer_textures: HashMap<u64, TextureHandle>,
    /// A map of output IDs to their intermediate textures.
    intermediate_textures: HashMap<OutputId, TextureHandle>,
}

impl App<'_> {
    /// Creates a new `App`.
    async fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Result<Self> {
        let mut window_manager = WindowManager::new();

        let main_backend = mapmap_render::WgpuBackend::new().await?;
        let _main_window_id = window_manager.create_main_window(event_loop, &main_backend)?;

        let renderer = Renderer::new(main_backend);
        let render_context = RenderContext::new();

        let ui = UI::new();
        let output_manager = OutputManager::new((INITIAL_WIDTH, INITIAL_HEIGHT));

        let mut control_manager = mapmap_control::ControlManager::new();
        if let Err(e) = control_manager.init_osc_server(8000) {
            error!("Failed to start OSC server: {}", e);
        }
        if let Err(e) = control_manager
            .osc_mapping
            .load_from_file("osc_mappings.json")
        {
            error!("Could not load OSC mappings: {}", e);
        }

        Ok(Self {
            window_manager,
            ui,
            render_context,
            renderer,
            output_manager,
            paint_textures: HashMap::new(),
            layer_textures: HashMap::new(),
            intermediate_textures: HashMap::new(),
        })
    }

    /// Runs the main application event loop.
    fn run(mut self, event_loop: EventLoop<()>) {
        event_loop
            .run(move |event, elwt| {
                // Handle events
                if let Err(e) = self.handle_event(event, elwt) {
                    tracing::error!("An error occurred: {}", e);
                }
            })
            .unwrap();
    }

    /// Handles a single event.
    fn handle_event(
        &mut self,
        event: Event<()>,
        elwt: &winit::event_loop::EventLoopWindowTarget<()>,
    ) -> Result<()> {
        match event {
            Event::WindowEvent {
                event, window_id, ..
            } => {
                let output_id = self
                    .window_manager
                    .get_output_id_from_window_id(window_id)
                    .unwrap_or(0);

                if let Some(window_context) = self.window_manager.get(output_id) {
                    self.ui.handle_event(&window_context.window, &event);
                }

                match event {
                    WindowEvent::CloseRequested => {
                        elwt.exit();
                    }
                    WindowEvent::Resized(size) => {
                        if let Some(window_context) = self.window_manager.get_mut(output_id) {
                            self.renderer.resize(size);
                            window_context.surface_config.width = size.width;
                            window_context.surface_config.height = size.height;
                            window_context
                                .surface
                                .configure(&self.renderer.device, &window_context.surface_config);
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        self.render(output_id)?;
                    }
                    _ => (),
                }
            }
            Event::AboutToWait => {
                // Redraw all windows
                for output_id in self.window_manager.window_ids().copied().collect::<Vec<_>>() {
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

        let surface_texture = window_context
            .surface
            .get_current_texture()?
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        if output_id == 0 {
            // Render the main window (UI)
            self.ui.render(
                &window_context.window,
                &self.renderer.device,
                &self.renderer.queue,
                &surface_texture,
            );
        } else {
            // Render an output window
            let mut pass = Pass::new(Some(Color::BLACK));
            pass.add_geometry(
                &[
                    Vertex {
                        position: [-0.5, -0.5, 0.0],
                        tex_coords: [0.0, 1.0],
                    },
                    Vertex {
                        position: [0.5, -0.5, 0.0],
                        tex_coords: [1.0, 1.0],
                    },
                    Vertex {
                        position: [0.5, 0.5, 0.0],
                        tex_coords: [1.0, 0.0],
                    },
                    Vertex {
                        position: [-0.5, 0.5, 0.0],
                        tex_coords: [0.0, 0.0],
                    },
                ],
                &[0, 1, 2, 0, 2, 3],
                &Texture::from_color(
                    &self.renderer.device,
                    &self.renderer.queue,
                    [255, 0, 0, 255],
                    1,
                    1,
                ),
                glam::Mat4::IDENTITY,
            );

            self.renderer
                .render_to_view(&mut self.render_context, vec![pass], &surface_texture);
        }

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
