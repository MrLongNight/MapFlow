//! Simple Render Example
//!
#![allow(deprecated)]

//! This example demonstrates the basics of rendering with mapmap_render.

use mapmap_render::{QuadRenderer, RenderBackend, TextureDescriptor, WgpuBackend};
use std::sync::Arc;
use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::WindowAttributes,
};

fn main() {
    println!("MapFlow - Simple Render Example");
    println!("==============================\n");

    let event_loop = EventLoop::new().unwrap();
    let window_attributes = WindowAttributes::default()
        .with_title("MapFlow - Simple Render")
        .with_inner_size(winit::dpi::PhysicalSize::new(800, 600));
    let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

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

    let texture = backend.create_texture(tex_desc).unwrap();
    let data = vec![255; 256 * 256 * 4];
    backend.upload_texture(texture.clone(), &data).unwrap();

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
            Event::AboutToWait => {
                window.request_redraw();
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
                            view: &view,
                            resolve_target: None,
                            depth_slice: None,
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
        })
        .unwrap();
}
