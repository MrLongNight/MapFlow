#![allow(deprecated)]

mod scenarios;

use std::{
    cell::{Cell, RefCell},
    fs,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use image::RgbaImage;
use scenarios::{build_scenario, ScenarioName, ScenarioSpec};
use vorce_render::{QuadRenderer, RenderBackend, TextureDescriptor, WgpuBackend};
use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::WindowAttributes,
};

#[derive(Parser)]
#[command(
    author,
    version,
    about = "Local visible visual regression harness for MapFlow"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Capture {
        #[arg(long, value_enum)]
        scenario: ScenarioName,
        #[arg(long)]
        output: PathBuf,
    },
    Reference {
        #[arg(long, value_enum)]
        scenario: ScenarioName,
        #[arg(long)]
        output: PathBuf,
    },
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Capture { scenario, output } => {
            capture_scenario(build_scenario(scenario), &output)
        }
        Command::Reference { scenario, output } => {
            write_reference_image(&build_scenario(scenario), &output)
        }
    }
}

fn write_reference_image(scenario: &ScenarioSpec, output: &Path) -> Result<()> {
    save_rgba_png(
        scenario.width,
        scenario.height,
        &scenario.expected_pixels,
        output,
    )
}

fn capture_scenario(scenario: ScenarioSpec, output: &Path) -> Result<()> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory {}", parent.display()))?;
    }

    let event_loop = EventLoop::new().context("failed to create event loop")?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let window = Arc::new(
        event_loop
            .create_window(
                WindowAttributes::default()
                    .with_title(scenario.title)
                    .with_inner_size(winit::dpi::PhysicalSize::new(
                        scenario.width,
                        scenario.height,
                    ))
                    .with_resizable(false),
            )
            .context("failed to create visual harness window")?,
    );

    let mut backend = pollster::block_on(WgpuBackend::new(None))
        .context("failed to initialize WGPU backend for visual harness")?;
    let surface = backend
        .create_surface(window.clone())
        .context("failed to create capture surface")?;

    let surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        format: wgpu::TextureFormat::Bgra8Unorm,
        width: scenario.width,
        height: scenario.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Opaque,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(backend.device(), &surface_config);

    let quad_renderer = QuadRenderer::new(backend.device(), surface_config.format)
        .context("failed to create quad renderer for visual harness")?;

    let texture = backend
        .create_texture(TextureDescriptor {
            width: scenario.width,
            height: scenario.height,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            mip_levels: 1,
        })
        .context("failed to create scenario texture")?;
    backend
        .upload_texture(texture.clone(), &scenario.source_pixels)
        .context("failed to upload scenario texture data")?;

    let output_path = output.to_path_buf();
    let render_count = Rc::new(Cell::new(0u32));
    let finished = Rc::new(Cell::new(false));
    let captured_error = Rc::new(RefCell::new(None::<String>));

    let render_count_cl = render_count.clone();
    let finished_cl = finished.clone();
    let captured_error_cl = captured_error.clone();

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
                if !finished_cl.get() {
                    window.request_redraw();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let current_count = render_count_cl.get() + 1;
                render_count_cl.set(current_count);

                if let Err(err) = render_frame(
                    &backend,
                    &surface,
                    &surface_config,
                    &quad_renderer,
                    &texture,
                    current_count >= 2,
                    &output_path,
                ) {
                    *captured_error_cl.borrow_mut() = Some(format!("{err:#}"));
                    finished_cl.set(true);
                    elwt.exit();
                    return;
                }

                if current_count >= 2 {
                    finished_cl.set(true);
                    elwt.exit();
                }
            }
            _ => {}
        })
        .context("visual harness event loop failed")?;

    if let Some(err) = captured_error.borrow_mut().take() {
        return Err(anyhow!(err));
    }

    Ok(())
}

fn render_frame(
    backend: &WgpuBackend,
    surface: &wgpu::Surface<'static>,
    surface_config: &wgpu::SurfaceConfiguration,
    quad_renderer: &QuadRenderer,
    texture: &vorce_render::TextureHandle,
    capture_output: bool,
    output_path: &Path,
) -> Result<()> {
    let frame = surface
        .get_current_texture()
        .context("failed to acquire current surface texture")?;
    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = backend
        .device()
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Visual Harness Encoder"),
        });

    let texture_view = texture.create_view();
    let bind_group = quad_renderer.create_bind_group(backend.device(), &texture_view);

    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Visual Harness Render Pass"),
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

    let readback = if capture_output {
        Some(queue_readback_copy(
            backend.device(),
            &mut encoder,
            &frame.texture,
            surface_config.width,
            surface_config.height,
        ))
    } else {
        None
    };

    backend.queue().submit(Some(encoder.finish()));
    frame.present();

    if let Some((buffer, padded_bytes_per_row)) = readback {
        save_readback_buffer(
            backend.device(),
            buffer,
            surface_config.width,
            surface_config.height,
            padded_bytes_per_row,
            surface_config.format,
            output_path,
        )?;
    }

    Ok(())
}

fn queue_readback_copy(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
) -> (wgpu::Buffer, u32) {
    let bytes_per_pixel = 4;
    let unpadded_bytes_per_row = width * bytes_per_pixel;
    let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
        * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Visual Harness Readback Buffer"),
        size: (padded_bytes_per_row * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    (buffer, padded_bytes_per_row)
}

fn save_readback_buffer(
    device: &wgpu::Device,
    buffer: wgpu::Buffer,
    width: u32,
    height: u32,
    padded_bytes_per_row: u32,
    format: wgpu::TextureFormat,
    output_path: &Path,
) -> Result<()> {
    let slice = buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device
        .poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: None,
        })
        .context("failed to wait for visual harness readback")?;

    let mapped = slice.get_mapped_range();
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);

    for row in mapped
        .chunks_exact(padded_bytes_per_row as usize)
        .take(height as usize)
    {
        for pixel in row[..(width * 4) as usize].chunks_exact(4) {
            match format {
                wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => {
                    rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
                }
                _ => rgba.extend_from_slice(pixel),
            }
        }
    }

    drop(mapped);
    buffer.unmap();

    save_rgba_png(width, height, &rgba, output_path)
}

fn save_rgba_png(width: u32, height: u32, pixels: &[u8], output_path: &Path) -> Result<()> {
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory {}", parent.display()))?;
    }

    let image = RgbaImage::from_raw(width, height, pixels.to_vec())
        .ok_or_else(|| anyhow!("failed to assemble RGBA image buffer"))?;
    image
        .save(output_path)
        .with_context(|| format!("failed to save {}", output_path.display()))?;
    Ok(())
}
