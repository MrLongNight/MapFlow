//! Hello World Projection Mapping Example
//!
#![allow(deprecated)]
//! This example demonstrates the basics of projection mapping:
//! 1. Creating a Paint (media source)
//! 2. Creating a Mesh (warping geometry)
//! 3. Creating a Mapping (connecting Paint to Mesh)
//! 4. Rendering the result

use glam::Vec2;
use mapmap_core::{Mapping, Mesh, Paint};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

struct ProjectionApp {
    window: Option<Arc<Window>>,
    backend: Option<WgpuBackend>,
    surface: Option<wgpu::Surface<'static>>,
    quad_renderer: Option<QuadRenderer>,
    texture: Option<Arc<wgpu::Texture>>,
    paint: Option<Paint>,
}

impl ApplicationHandler for ProjectionApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = WindowAttributes::default()
                .with_title("Hello World Projection")
                .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));
            let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
            self.window = Some(window.clone());

            let mut backend = pollster::block_on(WgpuBackend::new(None)).unwrap();
            let surface = backend.create_surface(window.clone()).unwrap();

            let surface_config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8Unorm,
                width: 1280,
                height: 720,
                present_mode: wgpu::PresentMode::Fifo,
                alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };

            surface.configure(backend.device(), &surface_config);

            let quad_renderer = QuadRenderer::new(backend.device(), surface_config.format).unwrap();

            let paint = Paint::color(1, "Hello World Paint", [0.2, 0.6, 1.0, 1.0]);

            let tex_desc = TextureDescriptor {
                width: 512,
                height: 512,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                mip_levels: 1,
            };

            let texture = Arc::new(backend.create_texture(tex_desc).unwrap());
            let texture_data = create_hello_world_texture(512, 512, paint.color);
            backend
                .upload_texture(texture.clone(), &texture_data)
                .unwrap();

            self.backend = Some(backend);
            self.surface = Some(surface);
            self.quad_renderer = Some(quad_renderer);
            self.texture = Some(texture);
            self.paint = Some(paint);
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
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
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
    println!("MapFlow - Hello World Projection Mapping Example");
    println!("===============================================\n");

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = ProjectionApp {
        window: None,
        backend: None,
        surface: None,
        quad_renderer: None,
        texture: None,
        paint: None,
    };

    event_loop.run_app(&mut app).unwrap();
}
}

/// Creates a "Hello World" texture with a gradient pattern
fn create_hello_world_texture(width: u32, height: u32, base_color: [f32; 4]) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            // Create a radial gradient effect
            let center_x = width as f32 / 2.0;
            let center_y = height as f32 / 2.0;
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let distance = (dx * dx + dy * dy).sqrt();
            let max_distance = (center_x * center_x + center_y * center_y).sqrt();
            let gradient = 1.0 - (distance / max_distance).min(1.0);

            // Apply gradient to base color
            let r = (base_color[0] * gradient * 255.0) as u8;
            let g = (base_color[1] * gradient * 255.0) as u8;
            let b = (base_color[2] * gradient * 255.0) as u8;
            let a = (base_color[3] * 255.0) as u8;

            data.push(r);
            data.push(g);
            data.push(b);
            data.push(a);
        }
    }

    data
}
