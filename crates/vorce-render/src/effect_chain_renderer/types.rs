use bytemuck::{Pod, Zeroable};
use std::sync::Arc;

/// Parameters for an effect instance
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct EffectParams {
    /// Time in seconds (for animated effects)
    pub time: f32,
    /// Effect intensity (0.0 - 1.0)
    pub intensity: f32,
    /// Parameter A (effect-specific)
    pub param_a: f32,
    /// Parameter B (effect-specific)
    pub param_b: f32,
    /// Parameter C (vec2 packed as xy)
    pub param_c: [f32; 2],
    /// Resolution (width, height)
    pub resolution: [f32; 2],
}

impl Default for EffectParams {
    fn default() -> Self {
        Self {
            time: 0.0,
            intensity: 1.0,
            param_a: 0.0,
            param_b: 0.0,
            param_c: [0.0, 0.0],
            resolution: [1920.0, 1080.0],
        }
    }
}

/// Ping-pong buffer for multi-pass rendering
#[allow(dead_code)]
pub(crate) struct PingPongBuffer {
    pub(crate) textures: [wgpu::Texture; 2],
    pub(crate) views: [Arc<wgpu::TextureView>; 2],
    pub(crate) current: usize,
}

#[allow(dead_code)]
impl PingPongBuffer {
    pub(crate) fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        let create_texture = || {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Effect Chain Ping-Pong Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            })
        };

        let tex_a = create_texture();
        let tex_b = create_texture();

        let view_a = Arc::new(tex_a.create_view(&wgpu::TextureViewDescriptor::default()));
        let view_b = Arc::new(tex_b.create_view(&wgpu::TextureViewDescriptor::default()));

        Self {
            textures: [tex_a, tex_b],
            views: [view_a, view_b],
            current: 0,
        }
    }

    pub(crate) fn current_view(&self) -> &Arc<wgpu::TextureView> {
        &self.views[self.current]
    }

    pub(crate) fn next_view(&self) -> &Arc<wgpu::TextureView> {
        &self.views[1 - self.current]
    }

    pub(crate) fn swap(&mut self) {
        self.current = 1 - self.current;
    }
}
