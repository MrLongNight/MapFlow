//! Simple Render Example
//!
#![allow(deprecated)]

//! This example demonstrates the basics of rendering with mapmap_render.

use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

struct SimpleApp {
    window: Option<Arc<Window>>,
    backend: Option<WgpuBackend>,
    surface: Option<wgpu::Surface<'static>>,
    quad_renderer: Option<QuadRenderer>,
    texture: Option<Arc<wgpu::Texture>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
}

impl ApplicationHandler for SimpleApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = WindowAttributes::default()
                .with_title("MapFlow - Simple Render")
                .with_inner_size(winit::dpi::PhysicalSize::new(800, 600));
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            self.window = Some(window.clone());

            let mut backend = pollster::block_on(WgpuBackend::new(None)).unwrap();
            let surface = backend.create_surface(window.clone()).unwrap();

            let surface_config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8Unorm,
                width: 800,
                height: 600,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };

            surface.configure(backend.device(), &surface_config);

            let quad_renderer = QuadRenderer::new(backend.device(), surface_config.format).unwrap();

            // Create a dummy texture
            let tex_desc = TextureDescriptor {
                width: 256,
                height: 256,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                mip_levels: 1,
            };

            let texture = Arc::new(backend.create_texture(tex_desc).unwrap());
            let data = vec![255; 256 * 256 * 4];
            backend.upload_texture(texture.clone(), &data).unwrap();

            self.backend = Some(backend);
            self.surface = Some(surface);
            self.quad_renderer = Some(quad_renderer);
            self.texture = Some(texture);
            self.surface_config = Some(surface_config);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                let (surface, backend, quad_renderer, texture) = match (
                    &self.surface,
                    &self.backend,
                    &self.quad_renderer,
                    &self.texture,
                ) {
                    (Some(s), Some(b), Some(q), Some(t)) => (s, b, q, t),
                    _ => return,
                };

                let frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(_) => return,
                };

                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder =
                    backend
                        .device()
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Render Encoder"),
                        });

                let texture_view = texture.create_view();
                let bind_group = quad_renderer.create_bind_group(backend.device(), &texture_view);
                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Main Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            depth_slice: None,
                            view: &view,
                            resolve_target: None,

                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });

                    quad_renderer.draw(&mut render_pass, &bind_group);
                }

                backend.queue().submit(Some(encoder.finish()));
                frame.present();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    println!("MapFlow - Simple Render Example");
    println!("==============================\n");

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = SimpleApp {
        window: None,
        backend: None,
        surface: None,
        quad_renderer: None,
        texture: None,
        surface_config: None,
    };

    event_loop.run_app(&mut app).unwrap();
}
}
