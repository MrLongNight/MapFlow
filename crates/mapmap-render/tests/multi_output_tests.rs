use mapmap_render::backend::WgpuBackend;
use mapmap_render::{RenderBackend, TextureDescriptor, TextureHandle};
use std::sync::Arc;
use wgpu::{Device, Queue, TextureFormat, TextureUsages};

struct TestContext {
    backend: WgpuBackend,
    device: Arc<Device>,
    queue: Arc<Queue>,
}

impl TestContext {
    async fn new() -> Option<Self> {
        let backend = WgpuBackend::new().await.ok()?;
        let device = backend.device.clone();
        let queue = backend.queue.clone();
        Some(Self {
            backend,
            device,
            queue,
        })
    }

    fn create_render_target(&mut self, width: u32, height: u32) -> TextureHandle {
        self.backend
            .create_texture(TextureDescriptor {
                width,
                height,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::RENDER_ATTACHMENT
                    | TextureUsages::COPY_SRC
                    | TextureUsages::TEXTURE_BINDING,
                ..Default::default()
            })
            .unwrap()
    }
}

// Helper function to initialize the context for tests.
// Skips the test if a backend cannot be created (e.g., in a CI environment without a GPU).
fn setup_test_context() -> Option<TestContext> {
    pollster::block_on(TestContext::new())
}

#[test]
fn test_create_multiple_outputs() {
    let mut context = match setup_test_context() {
        Some(context) => context,
        None => {
            eprintln!("SKIP: Could not create test context (no GPU backend?).");
            return;
        }
    };

    let output1 = context.create_render_target(128, 128);
    let output2 = context.create_render_target(256, 256);

    assert_eq!(output1.width, 128);
    assert_eq!(output1.height, 128);
    assert_ne!(output1.id, output2.id);

    assert_eq!(output2.width, 256);
    assert_eq!(output2.height, 256);
}

#[test]
fn test_render_to_distinct_targets() {
    let mut context = match setup_test_context() {
        Some(context) => context,
        None => {
            eprintln!("SKIP: Could not create test context.");
            return;
        }
    };

    let output1 = context.create_render_target(1, 1);
    let output2 = context.create_render_target(1, 1);

    // Render pass for the first output (red)
    let mut encoder1 = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let _ = encoder1.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass 1"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output1.texture.create_view(&Default::default()),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::RED),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }
    context.queue.submit(Some(encoder1.finish()));

    // Render pass for the second output (blue)
    let mut encoder2 = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let _ = encoder2.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass 2"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output2.texture.create_view(&Default::default()),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }
    context.queue.submit(Some(encoder2.finish()));

    // Verify the contents of both textures
    assert_pixel_color(&context, &output1, &[255, 0, 0, 255]);
    assert_pixel_color(&context, &output2, &[0, 0, 255, 255]);
}

#[test]
fn test_output_resolutions() {
    let mut context = match setup_test_context() {
        Some(context) => context,
        None => {
            eprintln!("SKIP: Could not create test context.");
            return;
        }
    };

    let output_hd = context.create_render_target(1920, 1080);
    let output_sd = context.create_render_target(640, 480);

    // Render green to the HD target
    let mut encoder_hd = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let _ = encoder_hd.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("HD Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output_hd.texture.create_view(&Default::default()),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
    }
    context.queue.submit(Some(encoder_hd.finish()));

    // Render red to the SD target
    let mut encoder_sd = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let _ = encoder_sd.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("SD Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output_sd.texture.create_view(&Default::default()),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::RED),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
    }
    context.queue.submit(Some(encoder_sd.finish()));

    // Verify the first pixel of each texture.
    assert_pixel_color(&context, &output_hd, &[0, 255, 0, 255]);
    assert_pixel_color(&context, &output_sd, &[255, 0, 0, 255]);
}

#[test]
fn test_edge_blending() {
    use mapmap_core::EdgeBlendConfig;
    use mapmap_render::edge_blend_renderer::EdgeBlendRenderer;

    let mut context = match setup_test_context() {
        Some(context) => context,
        None => {
            eprintln!("SKIP: Could not create test context.");
            return;
        }
    };

    let width = 256;
    let height = 256;
    let blend_width = 0.5; // 50% blend

    // Create a source texture with a solid white color.
    let source_texture = context.create_render_target(width, height);
    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Source Texture Clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &source_texture.texture.create_view(&Default::default()),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
    }
    context.queue.submit(Some(encoder.finish()));

    // Create the target for the edge blend output.
    let target_texture = context.create_render_target(width, height);

    // Setup edge blend renderer and config.
    let edge_blend_renderer =
        EdgeBlendRenderer::new(context.device.clone(), target_texture.format).unwrap();
    let blend_config = EdgeBlendConfig {
        left: mapmap_core::output::EdgeBlendZone {
            enabled: true,
            width: blend_width,
            offset: 0.0,
        },
        gamma: 1.0, // Linear gamma for predictable test results.
        ..Default::default()
    };

    // Create uniforms and bind groups.
    let uniform_buffer = edge_blend_renderer.create_uniform_buffer(&blend_config);
    let uniform_bind_group = edge_blend_renderer.create_uniform_bind_group(&uniform_buffer);
    let texture_bind_group = edge_blend_renderer
        .create_texture_bind_group(&source_texture.texture.create_view(&Default::default()));

    // Render the edge blend.
    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let target_view = target_texture.texture.create_view(&Default::default());
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Edge Blend Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        edge_blend_renderer.render(&mut rpass, &texture_bind_group, &uniform_bind_group);
    }
    context.queue.submit(Some(encoder.finish()));

    // Verify the pixel color at the center of the blend region.
    // It should be roughly 50% gray.
    assert_pixel_color_approx(
        &context,
        &target_texture,
        width / 4,
        height / 2,
        &[188, 188, 188, 255],
        5,
    );
}

/// Helper to read a single pixel and assert its color is within a tolerance.
fn assert_pixel_color_approx(
    context: &TestContext,
    texture: &TextureHandle,
    x: u32,
    y: u32,
    expected_color: &[u8; 4],
    tolerance: u8,
) {
    const ALIGNMENT: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let bytes_per_pixel = 4;
    let padded_bytes_per_row = (texture.width * bytes_per_pixel + ALIGNMENT - 1) & !(ALIGNMENT - 1);
    let buffer_size = (padded_bytes_per_row * texture.height) as u64;

    let buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Readback Buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    encoder.copy_texture_to_buffer(
        texture.texture.as_image_copy(),
        wgpu::ImageCopyBuffer {
            buffer: &buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(texture.height),
            },
        },
        wgpu::Extent3d {
            width: texture.width,
            height: texture.height,
            depth_or_array_layers: 1,
        },
    );

    context.queue.submit(Some(encoder.finish()));

    let buffer_slice = buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).unwrap();
    });

    context.device.poll(wgpu::Maintain::Wait);

    if let Ok(Ok(())) = rx.recv() {
        let data = buffer_slice.get_mapped_range();
        let offset = (y * padded_bytes_per_row + x * bytes_per_pixel) as usize;
        let pixel = &data[offset..offset + 4];

        for i in 0..4 {
            let diff = (pixel[i] as i16 - expected_color[i] as i16).abs() as u8;
            assert!(
                diff <= tolerance,
                "Pixel color mismatch at ({}, {}). Expected: {:?}, Got: {:?}, Diff: {}",
                x,
                y,
                expected_color,
                pixel,
                diff
            );
        }
    } else {
        panic!("Failed to map buffer to read texture data.");
    }
}

#[test]
fn test_independent_layer_visibility() {
    use mapmap_core::BlendMode;
    use mapmap_render::compositor::Compositor;
    use wgpu::util::DeviceExt;

    let mut context = match setup_test_context() {
        Some(context) => context,
        None => {
            eprintln!("SKIP: Could not create test context.");
            return;
        }
    };

    // Create textures for two layers and two outputs
    let layer1 = context.create_render_target(1, 1);
    let layer2 = context.create_render_target(1, 1);
    let output1 = context.create_render_target(1, 1);
    let output2 = context.create_render_target(1, 1);
    let black_texture = context.create_render_target(1, 1);

    // Prepare a black texture to use as a base for compositing, avoiding read/write conflicts.
    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Black Texture Clear"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &black_texture.texture.create_view(&Default::default()),
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
        })],
        ..Default::default()
    });
    // Fill layer 1 with red and layer 2 with green
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Layer 1 Clear"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &layer1.texture.create_view(&Default::default()),
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::RED),
                store: wgpu::StoreOp::Store,
            },
        })],
        ..Default::default()
    });
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("Layer 2 Clear"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &layer2.texture.create_view(&Default::default()),
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                store: wgpu::StoreOp::Store,
            },
        })],
        ..Default::default()
    });
    context.queue.submit(Some(encoder.finish()));

    // Setup compositor
    let compositor = Compositor::new(context.device.clone(), output1.format).unwrap();

    // Create a quad to draw the layers onto
    let vertex_buffer =
        context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Quad Vertex Buffer"),
                contents: bytemuck::cast_slice(&[
                    // Triangle 1
                    -1.0f32, -1.0, 0.0, 0.0, 0.0, 1.0, -1.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0,
                    // Triangle 2
                    -1.0, -1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, -1.0, 1.0, 0.0, 0.0, 1.0,
                ]),
                usage: wgpu::BufferUsages::VERTEX,
            });
    let index_buffer =
        context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Quad Index Buffer"),
                contents: bytemuck::cast_slice(&[0u16, 1, 2, 3, 4, 5]),
                usage: wgpu::BufferUsages::INDEX,
            });

    // --- Render to Output 1 (only layer 1 visible) ---
    let mut encoder1 = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    let uniform_buffer1 = compositor.create_uniform_buffer(BlendMode::Add, 1.0);
    let uniform_bind_group1 = compositor.create_uniform_bind_group(&uniform_buffer1);
    let bind_group1 = compositor.create_bind_group(
        &black_texture.texture.create_view(&Default::default()),
        &layer1.texture.create_view(&Default::default()),
    );
    {
        let output1_view = output1.texture.create_view(&Default::default());
        let mut rpass = encoder1.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Output 1 Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output1_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), // Start with a black background
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        compositor.composite(
            &mut rpass,
            &vertex_buffer,
            &index_buffer,
            &bind_group1,
            &uniform_bind_group1,
        );
    }
    context.queue.submit(Some(encoder1.finish()));

    // --- Render to Output 2 (only layer 2 visible) ---
    let mut encoder2 = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    let uniform_buffer2 = compositor.create_uniform_buffer(BlendMode::Add, 1.0);
    let uniform_bind_group2 = compositor.create_uniform_bind_group(&uniform_buffer2);
    let bind_group2 = compositor.create_bind_group(
        &black_texture.texture.create_view(&Default::default()),
        &layer2.texture.create_view(&Default::default()),
    );
    {
        let output2_view = output2.texture.create_view(&Default::default());
        let mut rpass = encoder2.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Output 2 Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output2_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        compositor.composite(
            &mut rpass,
            &vertex_buffer,
            &index_buffer,
            &bind_group2,
            &uniform_bind_group2,
        );
    }
    context.queue.submit(Some(encoder2.finish()));

    // Verify the outputs
    assert_pixel_color(&context, &output1, &[255, 0, 0, 255]);
    assert_pixel_color(&context, &output2, &[0, 255, 0, 255]);
}

/// Helper to read a 1x1 texture and assert its color.
fn assert_pixel_color(context: &TestContext, texture: &TextureHandle, expected_color: &[u8; 4]) {
    // wgpu requires texture-to-buffer copies to have bytes_per_row aligned to 256.
    const ALIGNMENT: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let bytes_per_pixel = 4;
    let padded_bytes_per_row = (texture.width * bytes_per_pixel + ALIGNMENT - 1) & !(ALIGNMENT - 1);
    let buffer_size = (padded_bytes_per_row * texture.height) as u64;

    let buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Readback Buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    encoder.copy_texture_to_buffer(
        texture.texture.as_image_copy(),
        wgpu::ImageCopyBuffer {
            buffer: &buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(texture.height),
            },
        },
        wgpu::Extent3d {
            width: texture.width,
            height: texture.height,
            depth_or_array_layers: 1,
        },
    );

    context.queue.submit(Some(encoder.finish()));

    let buffer_slice = buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).unwrap();
    });

    context.device.poll(wgpu::Maintain::Wait);

    if let Ok(Ok(())) = rx.recv() {
        let data = buffer_slice.get_mapped_range();
        // Check only the first pixel, ignoring row padding.
        assert_eq!(&data[..bytes_per_pixel as usize], expected_color);
    } else {
        panic!("Failed to map buffer to read texture data.");
    }
}
