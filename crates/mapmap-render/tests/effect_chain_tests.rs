//! Tests for the Effect Chain Renderer
//
// These are integration-style tests that verify the entire effect chain pipeline,
// from data structures to GPU rendering.

use mapmap_render::effect_chain_renderer::*;
use std::sync::Arc;

// Helper to set up a headless wgpu device for testing
async fn setup_headless_renderer() -> Option<(Arc<wgpu::Device>, wgpu::Queue)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await?;

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Test Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .ok()?;

    Some((Arc::new(device), queue))
}

#[tokio::test]
async fn test_headless_renderer_setup() {
    let result = setup_headless_renderer().await;
    assert!(result.is_some(), "Failed to create headless renderer");
}

#[test]
fn test_effect_chain_creation_and_modification() {
    let mut chain = EffectChain::new();
    assert!(chain.effects.is_empty());

    // Add effects
    let blur_id = chain.add_effect(EffectType::Blur);
    let color_id = chain.add_effect(EffectType::ColorAdjust);
    assert_eq!(chain.effects.len(), 2);
    assert_eq!(chain.effects[0].id, blur_id);
    assert_eq!(chain.effects[1].id, color_id);

    // Remove an effect
    let removed = chain.remove_effect(blur_id);
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().id, blur_id);
    assert_eq!(chain.effects.len(), 1);
    assert_eq!(chain.effects[0].id, color_id);
}

#[test]
fn test_effect_chain_reordering() {
    let mut chain = EffectChain::new();
    let id1 = chain.add_effect(EffectType::Blur);
    let id2 = chain.add_effect(EffectType::ColorAdjust);
    let id3 = chain.add_effect(EffectType::Invert);

    // Move id2 up
    chain.move_up(id2);
    assert_eq!(chain.effects.iter().map(|e| e.id).collect::<Vec<_>>(), vec![id2, id1, id3]);

    // Move id3 down (no-op)
    chain.move_down(id3);
    assert_eq!(chain.effects.iter().map(|e| e.id).collect::<Vec<_>>(), vec![id2, id1, id3]);

    // Move id2 down
    chain.move_down(id2);
    assert_eq!(chain.effects.iter().map(|e| e.id).collect::<Vec<_>>(), vec![id1, id2, id3]);
}

#[test]
fn test_effect_chain_bypass() {
    let mut chain = EffectChain::new();
    let blur_id = chain.add_effect(EffectType::Blur);
    let color_id = chain.add_effect(EffectType::ColorAdjust);

    chain.get_effect_mut(blur_id).unwrap().enabled = false;

    let enabled_effects: Vec<_> = chain.enabled_effects().collect();
    assert_eq!(enabled_effects.len(), 1);
    assert_eq!(enabled_effects[0].id, color_id);
}

#[test]
fn test_effect_parameter_modification() {
    let mut effect = Effect::new(1, EffectType::Blur);
    assert_eq!(effect.get_param("radius", 0.0), 5.0);

    effect.set_param("radius", 10.0);
    assert_eq!(effect.get_param("radius", 0.0), 10.0);

    // Test non-existent param
    assert_eq!(effect.get_param("nonexistent", 42.0), 42.0);
}

#[tokio::test]
async fn test_renderer_initialization_and_shader_compilation() {
    let (device, _queue) = setup_headless_renderer().await.unwrap();
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;

    // This will compile all built-in shaders
    let renderer = EffectChainRenderer::new(device, format);
    assert!(renderer.is_ok(), "EffectChainRenderer failed to initialize");
}

#[tokio::test]
async fn test_apply_chain_smoke_test() {
    let (device, queue) = setup_headless_renderer().await.unwrap();
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;

    let mut renderer = EffectChainRenderer::new(device.clone(), format).unwrap();

    // Create dummy textures
    let width = 128;
    let height = 128;
    let texture_desc = wgpu::TextureDescriptor {
        label: Some("Test Texture"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };
    let input_texture = device.create_texture(&texture_desc);
    let output_texture = device.create_texture(&texture_desc);
    let input_view = input_texture.create_view(&Default::default());
    let output_view = output_texture.create_view(&Default::default());

    // Create an effect chain
    let mut chain = EffectChain::new();
    chain.add_effect(EffectType::Invert);
    chain.add_effect(EffectType::Blur); // To test ping-pong

    // Apply the chain
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Test Encoder"),
    });

    renderer.apply_chain(&mut encoder, &input_view, &output_view, &chain, 0.0, width, height);

    // The test passes if this doesn't panic
    queue.submit(Some(encoder.finish()));
    device.poll(wgpu::Maintain::Wait);
}

#[test]
fn test_effect_preset_serialization() {
    let mut chain = EffectChain::new();
    let blur_id = chain.add_effect(EffectType::Blur);
    let color_id = chain.add_effect(EffectType::ColorAdjust);

    // Modify some parameters
    let blur_effect = chain.get_effect_mut(blur_id).unwrap();
    blur_effect.set_param("radius", 20.0);
    blur_effect.intensity = 0.75;

    let color_effect = chain.get_effect_mut(color_id).unwrap();
    color_effect.set_param("contrast", 1.5);
    color_effect.enabled = false;

    // Serialize to JSON
    let serialized_json = serde_json::to_string_pretty(&chain).unwrap();

    // Deserialize back
    let deserialized_chain: EffectChain = serde_json::from_str(&serialized_json).unwrap();

    // Compare
    assert_eq!(chain, deserialized_chain, "Deserialized chain should match the original");
}
