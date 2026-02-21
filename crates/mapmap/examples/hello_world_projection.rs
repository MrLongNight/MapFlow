//! Hello World Projection Mapping Example
//!
#![allow(deprecated)]

use mapmap_core::Paint;
use mapmap_render::{QuadRenderer, RenderBackend, TextureDescriptor, WgpuBackend};
use std::sync::Arc;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::WindowAttributes;

fn main() {
    println!("MapFlow - Hello World Projection Mapping Example");
    println!("===============================================\n");

    let event_loop = EventLoop::new().unwrap();
    let window_attributes = WindowAttributes::default()
        .with_title("Hello World Projection")
        .with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0));
    let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

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

    let texture = backend.create_texture(tex_desc).unwrap();
    let texture_data = create_hello_world_texture(512, 512, paint.color);
    backend
        .upload_texture(texture.clone(), &texture_data)
        .unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    #[allow(deprecated)]
    event_loop
        .run(move |event, elwt| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                elwt.exit();
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                logical_key: Key::Named(NamedKey::Escape),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                elwt.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
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
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        })
        .unwrap();
}

fn create_hello_world_texture(width: u32, height: u32, base_color: [f32; 4]) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            let center_x = width as f32 / 2.0;
            let center_y = height as f32 / 2.0;
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let distance = (dx * dx + dy * dy).sqrt();
            let max_distance = (center_x * center_x + center_y * center_y).sqrt();
            let gradient = 1.0 - (distance / max_distance).min(1.0);

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
