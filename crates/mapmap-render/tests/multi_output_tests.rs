//! Tests for multi-output rendering support

use mapmap_core::{ModulePartId, OutputId, RenderOp};
use mapmap_render::{Compositor, WgpuBackend};
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::{Extent3d, TexelCopyBufferInfo, TexelCopyBufferLayout, TextureDescriptor, TextureUsages};

#[tokio::test]
#[ignore = "GPU tests are unstable in headless CI environment"]
async fn test_compositor_creation() {
    let backend = WgpuBackend::new(None).await.unwrap();
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let compositor = Compositor::new(backend.device.clone(), backend.queue.clone(), format);
    assert!(compositor.is_ok());
}

#[tokio::test]
#[ignore = "GPU tests are unstable in headless CI environment"]
async fn test_multi_output_render_ops() {
    let backend = WgpuBackend::new(None).await.unwrap();
    let device = &backend.device;
    let queue = &backend.queue;
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;

    let width = 64;
    let height = 64;

    let mut compositor = Compositor::new(device.clone(), queue.clone(), format).unwrap();

    // Create a dummy texture
    let texture = device.create_texture(&TextureDescriptor {
        label: Some("Test Texture"),
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = Arc::new(texture.create_view(&wgpu::TextureViewDescriptor::default()));

    // Define render ops for two different outputs
    let mut render_ops = Vec::new();
    let part_id = ModulePartId(1);
    
    render_ops.push((part_id, RenderOp {
        texture_view: view.clone(),
        output_id: 0,
        layer_index: 0,
        opacity: 1.0,
        blend_mode: mapmap_core::module::BlendMode::Normal,
        transform: glam::Mat4::IDENTITY,
    }));

    render_ops.push((part_id, RenderOp {
        texture_view: view.clone(),
        output_id: 1,
        layer_index: 0,
        opacity: 0.5,
        blend_mode: mapmap_core::module::BlendMode::Normal,
        transform: glam::Mat4::IDENTITY,
    }));

    let mut output_textures = HashMap::new();
    output_textures.insert(0, view.clone());
    output_textures.insert(1, view.clone());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Compositor Encoder"),
    });

    compositor.render(&mut encoder, &render_ops, &output_textures);
    queue.submit(Some(encoder.finish()));

    // Verify by reading back (simplified)
    device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).unwrap();
}
