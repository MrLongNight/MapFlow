use mapmap_core::output::{CanvasRegion, OutputConfig};
use mapmap_render::{QuadRenderer, WgpuBackend};
use std::sync::Arc;
use wgpu::{Device, Queue};

struct TestEnvironment {
    device: Arc<Device>,
    queue: Arc<Queue>,
}

async fn setup_test_environment() -> Option<TestEnvironment> {
    WgpuBackend::new()
        .await
        .ok()
        .map(|backend| TestEnvironment {
            device: backend.device.clone(),
            queue: backend.queue.clone(),
        })
}

/// Helper to create a texture with a solid color
fn create_solid_color_texture(
    device: &Device,
    queue: &Queue,
    width: u32,
    height: u32,
    color: [u8; 4],
) -> wgpu::Texture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Solid Color Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    let mut data = Vec::with_capacity((width * height * 4) as usize);
    for _ in 0..(width * height) {
        data.extend_from_slice(&color);
    }

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &data,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * width),
            rows_per_image: Some(height),
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    texture
}

/// Helper to read texture data back to the CPU, handling row padding
async fn read_texture_data(
    device: &Device,
    queue: &Queue,
    texture: &wgpu::Texture,
    width: u32,
    height: u32,
) -> Vec<u8> {
    let bytes_per_pixel = 4;
    let unpadded_bytes_per_row = bytes_per_pixel * width;
    let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_bytes_per_row = (unpadded_bytes_per_row + alignment - 1) & !(alignment - 1);
    let buffer_size = (padded_bytes_per_row * height) as u64;

    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Readback Buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Readback Encoder"),
    });

    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::ImageCopyBuffer {
            buffer: &buffer,
            layout: wgpu::ImageDataLayout {
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

    queue.submit(Some(encoder.finish()));

    let slice = buffer.slice(..);
    let (tx, rx) = futures_channel::oneshot::channel();
    slice.map_async(wgpu::MapMode::Read, |result| {
        tx.send(result).unwrap();
    });
    device.poll(wgpu::Maintain::Wait);
    rx.await.unwrap().unwrap();

    let mut unpadded_data = Vec::with_capacity((unpadded_bytes_per_row * height) as usize);

    {
        let padded_data = slice.get_mapped_range();
        let padded_row_size = padded_bytes_per_row as usize;
        let unpadded_row_size = unpadded_bytes_per_row as usize;

        // Safety check: Ensure we have enough data
        let expected_size = padded_row_size * height as usize;
        if padded_data.len() < expected_size {
            panic!(
                "Readback buffer too small! Expected {} bytes, got {}",
                expected_size,
                padded_data.len()
            );
        }

        for i in 0..height as usize {
            let start = i * padded_row_size;
            let end = start + unpadded_row_size;

            if end > padded_data.len() {
                panic!(
                    "Buffer overrun detected at row {}: start={}, end={}, buffer_len={}",
                    i,
                    start,
                    end,
                    padded_data.len()
                );
            }

            unpadded_data.extend_from_slice(&padded_data[start..end]);
        }
    }
    buffer.unmap();

    unpadded_data
}

#[test]
#[ignore = "GPU tests are unstable in headless CI environment"]
fn test_render_to_multiple_outputs() {
    pollster::block_on(async {
        if let Some(env) = setup_test_environment().await {
            let TestEnvironment { device, queue } = env;
            let width = 64;
            let height = 64;
            let color = [255, 0, 0, 255]; // Red

            // 1. Create a source texture with a solid color
            let source_texture = create_solid_color_texture(&device, &queue, width, height, color);
            let source_view = source_texture.create_view(&wgpu::TextureViewDescriptor::default());

            // 2. Define output configurations (we won't use them directly for rendering,
            // but it's good practice to have them for context)
            let output_configs = vec![
                OutputConfig::new(
                    1,
                    "Output 1".to_string(),
                    CanvasRegion::new(0.0, 0.0, 0.5, 1.0),
                    (width, height),
                ),
                OutputConfig::new(
                    2,
                    "Output 2".to_string(),
                    CanvasRegion::new(0.5, 0.0, 0.5, 1.0),
                    (width, height),
                ),
            ];

            // 3. Create output textures
            let mut output_textures = Vec::new();
            for config in &output_configs {
                let texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some(&format!("Output Texture {}", config.id)),
                    size: wgpu::Extent3d {
                        width: config.resolution.0,
                        height: config.resolution.1,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                });
                output_textures.push(texture);
            }

            // 4. Create a QuadRenderer
            let quad_renderer =
                QuadRenderer::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb).unwrap();
            let bind_group = quad_renderer.create_bind_group(&device, &source_view);

            // 5. Render to each output texture
            for texture in &output_textures {
                let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    quad_renderer.draw(&mut render_pass, &bind_group);
                }
                queue.submit(Some(encoder.finish()));
            }

            // 6. Verify the contents of each output texture
            for (i, texture) in output_textures.iter().enumerate() {
                let data = read_texture_data(&device, &queue, texture, width, height).await;
                for chunk in data.chunks_exact(4) {
                    assert_eq!(chunk, color, "Pixel mismatch in output {}", i + 1);
                }
            }
        } else {
            println!("Skipping test: No suitable GPU adapter found.");
        }
    });
}

use glam::Mat4;
use mapmap_core::output::{EdgeBlendConfig, EdgeBlendZone};
use mapmap_core::Mesh;
use mapmap_render::{ColorCalibrationRenderer, EdgeBlendRenderer, MeshRenderer};

// ... (previous helper functions)

use mapmap_core::{MeshType, MeshVertex};

/// Helper to create a fullscreen quad mesh
fn create_fullscreen_quad_mesh() -> Mesh {
    let vertices = vec![
        MeshVertex::new(glam::vec2(-1.0, -1.0), glam::vec2(0.0, 1.0)),
        MeshVertex::new(glam::vec2(1.0, -1.0), glam::vec2(1.0, 1.0)),
        MeshVertex::new(glam::vec2(1.0, 1.0), glam::vec2(1.0, 0.0)),
        MeshVertex::new(glam::vec2(-1.0, 1.0), glam::vec2(0.0, 0.0)),
    ];
    let indices = vec![0, 1, 2, 0, 2, 3];
    Mesh {
        vertices,
        indices,
        mesh_type: MeshType::Quad,
    }
}

#[test]
#[ignore = "GPU tests are unstable in headless CI environment"]
fn test_individual_output_transforms() {
    pollster::block_on(async {
        if let Some(env) = setup_test_environment().await {
            let TestEnvironment { device, queue } = env;
            let width: u32 = 64;
            let height: u32 = 64;
            let color = [0, 255, 0, 255]; // Green

            let source_texture = create_solid_color_texture(&device, &queue, width, height, color);
            let source_view = source_texture.create_view(&wgpu::TextureViewDescriptor::default());

            let output_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Transform Output Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

            let mesh_renderer =
                MeshRenderer::new(device.clone(), wgpu::TextureFormat::Rgba8UnormSrgb).unwrap();
            let mesh = create_fullscreen_quad_mesh();
            let (vertex_buffer, index_buffer) = mesh_renderer.create_mesh_buffers(&mesh);
            let texture_bind_group = mesh_renderer.create_texture_bind_group(&source_view);

            // Scale the quad to half size. It should now occupy the center quadrant.
            let transform = Mat4::from_scale(glam::Vec3::new(0.5, 0.5, 1.0));
            let uniform_buffer = mesh_renderer.create_uniform_buffer(transform, 1.0);
            let uniform_bind_group = mesh_renderer.create_uniform_bind_group(&uniform_buffer);

            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            {
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &output_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                mesh_renderer.draw(
                    &mut rpass,
                    &vertex_buffer,
                    &index_buffer,
                    mesh.indices.len() as u32,
                    &uniform_bind_group,
                    &texture_bind_group,
                    false, // Use simple shader
                );
            }
            queue.submit(Some(encoder.finish()));

            let data = read_texture_data(&device, &queue, &output_texture, width, height).await;

            // Check a pixel that should be green (e.g., center of the scaled quad)
            let center_pixel_index = ((height / 2 * width) + (width / 2)) * 4;
            assert_eq!(
                &data[center_pixel_index as usize..(center_pixel_index + 4) as usize],
                color,
                "Center pixel should be green"
            );

            // Check a pixel that should be black (e.g., top-left corner)
            let corner_pixel_index = 0;
            assert_eq!(
                &data[corner_pixel_index as usize..(corner_pixel_index + 4) as usize],
                [0, 0, 0, 255],
                "Corner pixel should be black"
            );
        } else {
            println!("Skipping test: No suitable GPU adapter found.");
        }
    });
}

#[test]
#[ignore = "GPU tests are unstable in headless CI environment"]
fn test_edge_blending_between_outputs() {
    pollster::block_on(async {
        if let Some(env) = setup_test_environment().await {
            let TestEnvironment { device, queue } = env;
            let width: u32 = 256;
            let height: u32 = 256;
            let color = [255, 0, 0, 255]; // Red

            // Create a solid red texture
            let source_texture = create_solid_color_texture(&device, &queue, width, height, color);
            let source_view = source_texture.create_view(&wgpu::TextureViewDescriptor::default());

            let output_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Edge Blend Output"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

            let edge_blend_renderer =
                EdgeBlendRenderer::new(device.clone(), wgpu::TextureFormat::Rgba8UnormSrgb)
                    .unwrap();

            // Blend the right edge
            let blend_config = EdgeBlendConfig {
                right: EdgeBlendZone {
                    enabled: true,
                    width: 0.5,
                    offset: 0.0,
                },
                ..Default::default()
            };
            let uniform_buffer = edge_blend_renderer.create_uniform_buffer(&blend_config);
            let uniform_bind_group = edge_blend_renderer.create_uniform_bind_group(&uniform_buffer);
            let texture_bind_group = edge_blend_renderer.create_texture_bind_group(&source_view);

            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            {
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &output_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                edge_blend_renderer.render(&mut rpass, &texture_bind_group, &uniform_bind_group);
            }
            queue.submit(Some(encoder.finish()));

            let data = read_texture_data(&device, &queue, &output_texture, width, height).await;

            // Check the pixel values in the blend zone
            let y = height / 2;

            // Pixel just before the blend zone (uv.x slightly < 0.5) should be fully red
            let before_blend_x = width / 2 - 1;
            let before_blend_idx = ((y * width) + before_blend_x) * 4;
            assert_eq!(
                &data[before_blend_idx as usize..(before_blend_idx + 4) as usize],
                color
            );

            // Pixel in the middle of the blend zone (uv.x = 0.75) should be faded
            let mid_blend_x = width * 3 / 4;
            let mid_blend_idx = ((y * width) + mid_blend_x) * 4;
            let mid_red_value = data[mid_blend_idx as usize];
            assert!(
                mid_red_value > 10 && mid_red_value < 200,
                "Expected blended value at center of blend zone, got {}",
                mid_red_value
            );

            // Pixel at the far right edge (uv.x = 1.0) should be black
            let end_x = width - 1;
            let end_idx = ((y * width) + end_x) * 4;
            // Gamma correction means it might not be perfectly 0
            assert!(
                data[end_idx as usize] < 10,
                "Expected near-black value at the edge, got {}",
                data[end_idx as usize]
            );
        } else {
            println!("Skipping test: No suitable GPU adapter found.");
        }
    });
}

#[test]
#[ignore = "GPU tests are unstable in headless CI environment"]
fn test_color_calibration_per_output() {
    pollster::block_on(async {
        if let Some(env) = setup_test_environment().await {
            let TestEnvironment { device, queue } = env;
            let width: u32 = 64;
            let height: u32 = 64;
            let gray_color = [128, 128, 128, 255];

            let source_texture =
                create_solid_color_texture(&device, &queue, width, height, gray_color);
            let source_view = source_texture.create_view(&wgpu::TextureViewDescriptor::default());

            let output_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Color Calib Output"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

            let calib_renderer =
                ColorCalibrationRenderer::new(device.clone(), wgpu::TextureFormat::Rgba8UnormSrgb)
                    .unwrap();
            let texture_bind_group = calib_renderer.create_texture_bind_group(&source_view);

            // Increase brightness and contrast
            let calib_config = mapmap_core::output::ColorCalibration {
                brightness: 0.2, // Make it brighter
                contrast: 1.5,   // Increase contrast
                ..Default::default()
            };
            let uniform_buffer = calib_renderer.create_uniform_buffer(&calib_config);
            let uniform_bind_group = calib_renderer.create_uniform_bind_group(&uniform_buffer);

            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            {
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &output_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                calib_renderer.render(&mut rpass, &texture_bind_group, &uniform_bind_group);
            }
            queue.submit(Some(encoder.finish()));

            let data = read_texture_data(&device, &queue, &output_texture, width, height).await;
            let calibrated_pixel = &data[0..4];

            // Expected value calculation is complex due to sRGB.
            // We expect the gray to be significantly brighter.
            // Original: 128. After brightness/contrast: should be > 128.
            let calibrated_value = calibrated_pixel[0];
            assert!(
                calibrated_value > 150,
                "Expected a brighter gray value, got {}",
                calibrated_value
            );
        } else {
            println!("Skipping test: No suitable GPU adapter found.");
        }
    });
}

#[test]
#[ignore = "GPU tests are unstable in headless CI environment"]
fn test_different_output_resolutions() {
    pollster::block_on(async {
        if let Some(env) = setup_test_environment().await {
            let TestEnvironment { device, queue } = env;
            let color = [0, 0, 255, 255]; // Blue

            let resolutions = [(128, 64), (80, 100)];

            let source_texture = create_solid_color_texture(&device, &queue, 32, 32, color);
            let source_view = source_texture.create_view(&wgpu::TextureViewDescriptor::default());

            let quad_renderer =
                QuadRenderer::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb).unwrap();
            let bind_group = quad_renderer.create_bind_group(&device, &source_view);

            for (width, height) in resolutions {
                let output_texture = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some(&format!("Output {}x{}", width, height)),
                    size: wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                });
                let output_view =
                    output_texture.create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder =
                    device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
                {
                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &output_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    quad_renderer.draw(&mut rpass, &bind_group);
                }
                queue.submit(Some(encoder.finish()));

                let data = read_texture_data(&device, &queue, &output_texture, width, height).await;
                for (i, chunk) in data.chunks_exact(4).enumerate() {
                    assert_eq!(
                        chunk, color,
                        "Pixel {} at res {}x{} was incorrect",
                        i, width, height
                    );
                }
            }
        } else {
            println!("Skipping test: No suitable GPU adapter found.");
        }
    });
}
