//! Tests for the EffectChainRenderer

use mapmap_core::{EffectChain, EffectType};
use mapmap_render::{EffectChainRenderer, WgpuBackend};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use wgpu::{CommandEncoderDescriptor, Extent3d, TextureDescriptor, TextureUsages};

#[tokio::test]
#[ignore = "GPU tests are unstable in headless CI environment"]
async fn test_effect_chain_renderer_creation() {
    let backend = WgpuBackend::new(None).await.unwrap();
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let renderer = EffectChainRenderer::new(backend.device.clone(), backend.queue.clone(), format);
    assert!(renderer.is_ok());
}

#[tokio::test]
#[ignore = "GPU tests are unstable in headless CI environment"]
async fn test_simple_invert() {
    let backend = WgpuBackend::new(None).await.unwrap();
    let device = &backend.device;
    let queue = &backend.queue;
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;

    let width = 64;
    let height = 64;

    // Create a red input texture
    let mut input_data = Vec::new();
    for _ in 0..width * height {
        input_data.extend_from_slice(&[255, 0, 0, 255]);
    }

    let input_texture = device.create_texture_with_data(
        queue,
        &TextureDescriptor {
            label: Some("Input Texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC,
            view_formats: &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        &input_data,
    );
    let input_view = Arc::new(input_texture.create_view(&wgpu::TextureViewDescriptor::default()));

    // Create output texture
    let output_texture = device.create_texture(&TextureDescriptor {
        label: Some("Output Texture"),
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let output_view = Arc::new(output_texture.create_view(&wgpu::TextureViewDescriptor::default()));

    let mut renderer = EffectChainRenderer::new(device.clone(), queue.clone(), format).unwrap();
    let mut chain = EffectChain::new();
    chain.add_effect(EffectType::Invert);

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Test Encoder"),
    });

    let shader_graph_manager = mapmap_render::ShaderGraphManager::new();
    renderer.apply_chain(
        &mut encoder,
        &input_view,
        &output_view,
        &chain,
        &shader_graph_manager,
        0.0,
        width,
        height,
    );

    queue.submit(Some(encoder.finish()));

    // Read back and verify
    let bytes_per_pixel = 4;
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Readback Buffer"),
        size: (width * height * bytes_per_pixel) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Readback Encoder"),
    });

    let bytes_per_row = width * bytes_per_pixel;
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &output_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(height),
            },
        },
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    queue.submit(Some(encoder.finish()));

    let slice = output_buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).unwrap();

    let data = slice.get_mapped_range();
    // First pixel should be cyan (inverted red) [0, 255, 255, 255]
    assert!(data[0] < 10);
    assert!(data[1] > 245);
    assert!(data[2] > 245);
    assert_eq!(data[3], 255);
}
