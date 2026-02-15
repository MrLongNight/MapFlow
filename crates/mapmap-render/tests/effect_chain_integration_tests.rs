//! Integration tests for the EffectChainRenderer

use mapmap_core::{EffectChain, EffectType};
use mapmap_render::{EffectChainRenderer, WgpuBackend};
use wgpu::util::DeviceExt;
use wgpu::{
    CommandEncoderDescriptor, Extent3d, ImageCopyBuffer, ImageDataLayout, TextureDescriptor,
    TextureUsages,
};

// Helper function to run a test with a given texture setup
async fn run_test_with_texture<F>(
    width: u32,
    height: u32,
    input_data: Vec<u8>,
    test_fn: F,
) -> Vec<u8>
where
    F: FnOnce(&mut EffectChainRenderer, &wgpu::TextureView, &wgpu::TextureView),
{
    let backend = WgpuBackend::new(None).await.unwrap();
    let device = &backend.device;
    let queue = &backend.queue;
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;

    // Create input texture
    let input_texture = device.create_texture_with_data(
        queue,
        &TextureDescriptor {
            label: Some("Input Test Texture"),
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
    let input_view = input_texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Create output texture
    let output_texture = device.create_texture(&TextureDescriptor {
        label: Some("Output Test Texture"),
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
    let output_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Create renderer
    let mut effect_chain_renderer =
        EffectChainRenderer::new(device.clone(), queue.clone(), format).unwrap();

    // Run the provided test function
    test_fn(&mut effect_chain_renderer, &input_view, &output_view);

    // Read back the data from the output texture
    let bytes_per_pixel = 4;
    let buffer_size = (width * height * bytes_per_pixel) as u64;
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output Readback Buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("Readback Encoder"),
    });

    let bytes_per_row = {
        let alignment = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let unaligned_bytes_per_row = width * bytes_per_pixel;
        (unaligned_bytes_per_row + alignment - 1) & !(alignment - 1)
    };

    encoder.copy_texture_to_buffer(
        output_texture.as_image_copy(),
        ImageCopyBuffer {
            buffer: &output_buffer,
            layout: ImageDataLayout {
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

<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
    let _index = queue.submit(Some(encoder.finish()));
=======
    queue.submit(Some(encoder.finish()));
>>>>>>> origin/jules-ui-cyber-effect-panel-12215865592445160390
=======
    let _index = queue.submit(Some(encoder.finish()));
>>>>>>> origin/jules-7419576384359145166-df4ba129
=======
    let _index = queue.submit(Some(encoder.finish()));
>>>>>>> origin/bolt/optimize-audio-analysis-allocation-16814173430346295439
=======
    let _index = queue.submit(Some(encoder.finish()));
>>>>>>> origin/tracker-update-roadmap-changelog-1-62932857170364482
=======
    let _index = queue.submit(Some(encoder.finish()));
>>>>>>> origin/lina-ui-oscillator-polish-9227819679485877388

    // Add a small delay to give the GPU time to process the command buffer.
    // This is a workaround for potential race conditions in headless environments.
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Map the buffer and get the data
    let slice = output_buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    // Use Maintain::Wait to ensure all GPU operations are complete before reading back.
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
<<<<<<< HEAD
    // device.poll(wgpu::Maintain::WaitForSubmissionIndex(index));
=======
    device.poll(wgpu::Maintain::Wait);
>>>>>>> origin/ux/accessibility-custom-widgets-3047621584255357057
=======
    device.poll(wgpu::Maintain::Wait);
>>>>>>> origin/jules-ui-cyber-effect-panel-12215865592445160390
=======
    device
<<<<<<< HEAD
        .poll(wgpu::Maintain::Wait)
        .panic_on_timeout();
<<<<<<< HEAD
>>>>>>> origin/jules-7419576384359145166-df4ba129
=======
>>>>>>> origin/bolt/optimize-audio-analysis-allocation-16814173430346295439
=======
    device.poll(wgpu::Maintain::Wait);
>>>>>>> origin/tracker-update-roadmap-changelog-1-62932857170364482
=======
        .poll(wgpu::Maintain::WaitForSubmissionIndex(index))
        .panic_on_timeout();
>>>>>>> origin/ux/safety-shield-standardization-4591480983604393855
=======
    device.poll(wgpu::Maintain::Wait);
>>>>>>> origin/lina-ui-oscillator-polish-9227819679485877388
    let data = {
        let view = slice.get_mapped_range();
        view.chunks_exact(bytes_per_row as usize)
            .flat_map(|row| &row[..(width * bytes_per_pixel) as usize])
            .copied()
            .collect::<Vec<u8>>()
    };
    output_buffer.unmap();

    data
}

#[tokio::test]
#[ignore = "GPU tests are unstable in headless CI environment"]
async fn test_passthrough_no_effects() {
    let input_color = [255, 0, 0, 255]; // Red
    let output_data =
        run_test_with_texture(1, 1, input_color.to_vec(), |renderer, input, output| {
            let chain = EffectChain::new();
            let mut encoder = renderer
                .device()
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Test Encoder"),
                });
            let shader_graph_manager = mapmap_render::ShaderGraphManager::new();
            renderer.apply_chain(
                &mut encoder,
                input,
                output,
                &chain,
                &shader_graph_manager,
                0.0,
                1,
                1,
            );
            renderer.queue().submit(Some(encoder.finish()));
        })
        .await;

    assert_eq!(output_data, input_color);
}

#[tokio::test]
#[ignore = "GPU tests are unstable in headless CI environment"]
async fn test_single_invert_effect() {
    let input_color = [255, 128, 0, 255]; // Orange
    let expected_color = [0, 127, 255, 255]; // Inverted Orange (approx)

    let output_data =
        run_test_with_texture(1, 1, input_color.to_vec(), |renderer, input, output| {
            let mut chain = EffectChain::new();
            chain.add_effect(EffectType::Invert);

            let mut encoder = renderer
                .device()
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Test Encoder"),
                });
            let shader_graph_manager = mapmap_render::ShaderGraphManager::new();
            renderer.apply_chain(
                &mut encoder,
                input,
                output,
                &chain,
                &shader_graph_manager,
                0.0,
                1,
                1,
            );
            renderer.queue().submit(Some(encoder.finish()));
        })
        .await;

    // Allow for small differences due to GPU interpolation/precision
    assert!(output_data[0] < 5); // R
    assert!((output_data[1] as i16 - expected_color[1] as i16).abs() < 5); // G
    assert!(output_data[2] > 250); // B
    assert_eq!(output_data[3], 255); // A
}

#[tokio::test]
#[ignore = "GPU tests are unstable in headless CI environment"]
async fn test_multiple_effects() {
    let input_color = [255, 255, 255, 255]; // White
                                            // Invert -> Black [0,0,0,255]
                                            // Then ColorAdjust (brightness +0.5) -> Grey [127,127,127,255]
    let expected_color = [127, 127, 127, 255];

    let output_data =
        run_test_with_texture(1, 1, input_color.to_vec(), |renderer, input, output| {
            let mut chain = EffectChain::new();
            chain.add_effect(EffectType::Invert);
            let color_adjust_id = chain.add_effect(EffectType::ColorAdjust);

            let effect = chain.get_effect_mut(color_adjust_id).unwrap();
            effect.set_param("brightness", 0.5);

            let mut encoder = renderer
                .device()
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Test Encoder"),
                });
            let shader_graph_manager = mapmap_render::ShaderGraphManager::new();
            renderer.apply_chain(
                &mut encoder,
                input,
                output,
                &chain,
                &shader_graph_manager,
                0.0,
                1,
                1,
            );
            renderer.queue().submit(Some(encoder.finish()));
        })
        .await;

    assert!((output_data[0] as i16 - expected_color[0] as i16).abs() < 5);
    assert!((output_data[1] as i16 - expected_color[1] as i16).abs() < 5);
    assert!((output_data[2] as i16 - expected_color[2] as i16).abs() < 5);
    assert_eq!(output_data[3], 255);
}

#[tokio::test]
#[ignore = "GPU tests are unstable in headless CI environment"]
async fn test_stability_multiple_frames() {
    let input_color = [255, 0, 0, 255]; // Red
    run_test_with_texture(1, 1, input_color.to_vec(), |renderer, input, output| {
        let mut chain = EffectChain::new();
        chain.add_effect(EffectType::Blur);
        chain.add_effect(EffectType::FilmGrain);

        let shader_graph_manager = mapmap_render::ShaderGraphManager::new();
        for i in 0..10 {
            let mut encoder = renderer
                .device()
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some(&format!("Test Encoder Frame {}", i)),
                });
            renderer.apply_chain(
                &mut encoder,
                input,
                output,
                &chain,
                &shader_graph_manager,
                i as f32,
                1,
                1,
            );
            renderer.queue().submit(Some(encoder.finish()));
        }
    })
    .await;

    // The test passes if it doesn't panic.
}
