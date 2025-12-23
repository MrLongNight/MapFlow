use mapmap_render::effect_chain_renderer::{EffectChain, EffectType, EffectChainRenderer};
use mapmap_render::WgpuBackend;
use std::sync::Arc;
use wgpu::{Device, Queue};

// --- Test Setup Boilerplate (adapted from multi_output_tests.rs) ---

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

    let data: Vec<u8> = (0..width * height).flat_map(|_| color).collect();

    queue.write_texture(
        texture.as_image_copy(),
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
    // Padded bytes per row must be a multiple of the alignment.
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

    // Map the buffer
    let slice = buffer.slice(..);
    let (tx, rx) = futures_channel::oneshot::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).unwrap();
    });
    device.poll(wgpu::Maintain::Wait);
    rx.await.unwrap().unwrap();

    // The view is a guard that must be dropped before unmap is called.
    let data = {
        let view = slice.get_mapped_range();
        view.chunks_exact(padded_bytes_per_row as usize)
            .flat_map(|row| &row[..unpadded_bytes_per_row as usize])
            .copied()
            .collect::<Vec<u8>>()
    };

    buffer.unmap();

    data
}


// --- Foundational Tests ---

#[test]
fn test_add_and_remove_effects() {
    let mut chain = EffectChain::new();
    assert_eq!(chain.effects.len(), 0);

    let blur_id = chain.add_effect(EffectType::Blur);
    assert_eq!(chain.effects.len(), 1);
    assert_eq!(chain.effects[0].effect_type, EffectType::Blur);

    let color_id = chain.add_effect(EffectType::ColorAdjust);
    assert_eq!(chain.effects.len(), 2);
    assert_eq!(chain.effects[1].effect_type, EffectType::ColorAdjust);

    let removed = chain.remove_effect(blur_id);
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().id, blur_id);
    assert_eq!(chain.effects.len(), 1);
    assert_eq!(chain.effects[0].id, color_id);
}

#[test]
fn test_reorder_effects() {
    let mut chain = EffectChain::new();
    let blur_id = chain.add_effect(EffectType::Blur);
    let color_id = chain.add_effect(EffectType::ColorAdjust);
    let vignette_id = chain.add_effect(EffectType::Vignette);

    assert_eq!(chain.effects[0].id, blur_id);
    assert_eq!(chain.effects[1].id, color_id);
    assert_eq!(chain.effects[2].id, vignette_id);

    // Move color adjust up
    chain.move_up(color_id);
    assert_eq!(chain.effects[0].id, color_id);
    assert_eq!(chain.effects[1].id, blur_id);
    assert_eq!(chain.effects[2].id, vignette_id);

    // Move it up again (should do nothing)
    chain.move_up(color_id);
    assert_eq!(chain.effects[0].id, color_id);

    // Move vignette down (should do nothing)
    chain.move_down(vignette_id);
    assert_eq!(chain.effects[2].id, vignette_id);

    // Move blur down
    chain.move_down(blur_id);
    assert_eq!(chain.effects[0].id, color_id);
    assert_eq!(chain.effects[1].id, vignette_id);
    assert_eq!(chain.effects[2].id, blur_id);
}

#[test]
fn test_toggle_effect() {
    let mut chain = EffectChain::new();
    let blur_id = chain.add_effect(EffectType::Blur);
    chain.add_effect(EffectType::ColorAdjust);

    assert_eq!(chain.enabled_effects().count(), 2);

    let blur_effect = chain.get_effect_mut(blur_id).unwrap();
    assert!(blur_effect.enabled);
    blur_effect.enabled = false;
    assert!(!blur_effect.enabled);

    assert_eq!(chain.enabled_effects().count(), 1);
    assert_eq!(
        chain.enabled_effects().next().unwrap().effect_type,
        EffectType::ColorAdjust
    );
}

#[test]
fn test_modify_effect_parameters() {
    let mut chain = EffectChain::new();
    let blur_id = chain.add_effect(EffectType::Blur);

    let blur_effect = chain.get_effect_mut(blur_id).unwrap();
    assert_eq!(blur_effect.get_param("radius", 0.0), 5.0); // Default value

    blur_effect.set_param("radius", 15.5);
    assert_eq!(blur_effect.get_param("radius", 0.0), 15.5);

    blur_effect.intensity = 0.5;
    assert_eq!(blur_effect.intensity, 0.5);
}

#[test]
fn test_effect_chain_serialization() {
    let mut chain = EffectChain::new();
    let blur_id = chain.add_effect(EffectType::Blur);
    chain.add_effect(EffectType::ColorAdjust);

    let blur_effect = chain.get_effect_mut(blur_id).unwrap();
    blur_effect.intensity = 0.75;
    blur_effect.set_param("radius", 20.0);

    let serialized = serde_json::to_string(&chain).unwrap();
    let deserialized: EffectChain = serde_json::from_str(&serialized).unwrap();

    assert_eq!(chain, deserialized);
    assert_eq!(deserialized.effects.len(), 2);
    assert_eq!(deserialized.effects[0].intensity, 0.75);
}

// --- GPU Integration Tests ---

#[test]
fn test_empty_chain_is_passthrough() {
    pollster::block_on(async {
        if let Some(env) = setup_test_environment().await {
            let TestEnvironment { device, queue } = env;
            let width = 32;
            let height = 32;
            let color = [255, 0, 0, 255]; // Red
            let format = wgpu::TextureFormat::Rgba8UnormSrgb;

            let source = create_solid_color_texture(&device, &queue, width, height, color);
            let source_view = source.create_view(&Default::default());

            let output = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Output"),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let output_view = output.create_view(&Default::default());

            let mut renderer = EffectChainRenderer::new(device.clone(), format).unwrap();
            let chain = EffectChain::new(); // Empty chain

            let mut encoder = device.create_command_encoder(&Default::default());
            renderer.apply_chain(&mut encoder, &source_view, &output_view, &chain, 0.0, width, height);
            queue.submit(Some(encoder.finish()));

            let data = read_texture_data(&device, &queue, &output, width, height).await;

            // With the fix, an empty chain should copy the source, so the output should be red.
            assert_eq!(&data[0..4], &color);
        }
    });
}

#[test]
fn test_blur_plus_coloradjust_chain() {
    pollster::block_on(async {
        if let Some(env) = setup_test_environment().await {
            let TestEnvironment { device, queue } = env;
            let width = 32;
            let height = 32;
            let color = [0, 0, 255, 255]; // Blue
            let format = wgpu::TextureFormat::Rgba8UnormSrgb;

            // Create a texture that is blue, to check color adjust
            let source = create_solid_color_texture(&device, &queue, width, height, color);
            let source_view = source.create_view(&Default::default());

            let output = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Output"),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let output_view = output.create_view(&Default::default());

            let mut renderer = EffectChainRenderer::new(device.clone(), format).unwrap();
            let mut chain = EffectChain::new();
            let blur_id = chain.add_effect(EffectType::Blur);
            let color_id = chain.add_effect(EffectType::ColorAdjust);

            // Make blur negligible but present
            chain.get_effect_mut(blur_id).unwrap().set_param("radius", 0.0);
            // Drastically reduce saturation
            chain.get_effect_mut(color_id).unwrap().set_param("saturation", 0.0);

            let mut encoder = device.create_command_encoder(&Default::default());
            renderer.apply_chain(&mut encoder, &source_view, &output_view, &chain, 0.0, width, height);
            queue.submit(Some(encoder.finish()));

            let data = read_texture_data(&device, &queue, &output, width, height).await;
            let pixel = &data[0..4];

            // Because saturation is 0, the color should be grayscale.
            // sRGB (0,0,255) -> luma -> sRGB gray is ~81.
            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];
            assert!((r as i16 - g as i16).abs() < 5, "R ({}) and G ({}) should be equal for grayscale", r, g);
            assert!((g as i16 - b as i16).abs() < 5, "G ({}) and B ({}) should be equal for grayscale", g, b);
            assert!(r > 70 && r < 100, "Gray value should be around 81, but was {}", r);
        }
    });
}

#[test]
fn test_vignette_plus_filmgrain_chain() {
    pollster::block_on(async {
         if let Some(env) = setup_test_environment().await {
            let TestEnvironment { device, queue } = env;
            let width = 32;
            let height = 32;
            let color = [255, 255, 255, 255]; // White
            let format = wgpu::TextureFormat::Rgba8UnormSrgb;

            let source = create_solid_color_texture(&device, &queue, width, height, color);
            let source_view = source.create_view(&Default::default());

            let output = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Output"),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let output_view = output.create_view(&Default::default());

            let mut renderer = EffectChainRenderer::new(device.clone(), format).unwrap();
            let mut chain = EffectChain::new();
            let vignette_id = chain.add_effect(EffectType::Vignette);
            let grain_id = chain.add_effect(EffectType::FilmGrain);

            // Set aggressive parameters to make effects obvious
            let vignette = chain.get_effect_mut(vignette_id).unwrap();
            vignette.set_param("radius", 0.2);
            vignette.set_param("softness", 0.2);

            let grain = chain.get_effect_mut(grain_id).unwrap();
            grain.set_param("amount", 0.4);

            let mut encoder = device.create_command_encoder(&Default::default());
            renderer.apply_chain(&mut encoder, &source_view, &output_view, &chain, 1.23, width, height);
            queue.submit(Some(encoder.finish()));

            let data = read_texture_data(&device, &queue, &output, width, height).await;

            // Center pixel should be affected by grain, so not pure white
            let center_idx = ((height / 2 * width) + (width / 2)) * 4;
            let center_pixel = &data[center_idx as usize .. (center_idx + 4) as usize];
            assert_ne!(center_pixel, color, "Center pixel should have grain, not be pure white");

            // Corner pixel should be darker than center due to vignette
            let corner_pixel = &data[0..4];
            let corner_brightness = corner_pixel[0] as u16 + corner_pixel[1] as u16 + corner_pixel[2] as u16;
            let center_brightness = center_pixel[0] as u16 + center_pixel[1] as u16 + center_pixel[2] as u16;
            assert!(corner_brightness < center_brightness, "Corner ({}) should be darker than center ({})", corner_brightness, center_brightness);
        }
    });
}
