use criterion::{criterion_group, criterion_main, Criterion};
use mapmap_render::MeshRenderer;
use std::sync::Arc;
use wgpu::Instance;

fn mesh_renderer_benchmark(c: &mut Criterion) {
    let instance = Instance::new(&wgpu::InstanceDescriptor::default());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        force_fallback_adapter: true,
        compatible_surface: None,
    }))
    .expect("Failed to find an appropriate adapter");

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
            ..Default::default()
        },
    ))
    .expect("Failed to create device");

    let device = Arc::new(device);
    let mut renderer = MeshRenderer::new(device.clone(), wgpu::TextureFormat::Rgba8Unorm).unwrap();
    let transform = glam::Mat4::IDENTITY;

    let mut group = c.benchmark_group("MeshRenderer");
    group.bench_function("get_uniform_bind_group_redundant", |b| {
        b.iter(|| {
            renderer.begin_frame();
            for _ in 0..10 {
                let _ = renderer.get_uniform_bind_group(
                    &queue,
                    std::hint::black_box(transform),
                    std::hint::black_box(1.0),
                );
            }
        })
    });
    group.finish();
}

criterion_group!(benches, mesh_renderer_benchmark);
criterion_main!(benches);




