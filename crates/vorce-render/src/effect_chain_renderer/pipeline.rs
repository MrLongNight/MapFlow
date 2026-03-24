use super::EffectChainRenderer;
use crate::Result;
use vorce_core::EffectType;

impl EffectChainRenderer {
    /// Create a render pipeline for a specific effect type
    pub(crate) fn create_effect_pipeline(
        device: &wgpu::Device,
        bind_group_layout: &wgpu::BindGroupLayout,
        uniform_bind_group_layout: &wgpu::BindGroupLayout,
        lut_bind_group_layout: &wgpu::BindGroupLayout,
        target_format: wgpu::TextureFormat,
        effect_type: &EffectType,
    ) -> Result<wgpu::RenderPipeline> {
        let shader_source = Self::get_effect_shader_source(effect_type);

        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!("Effect Shader: {:?}", effect_type)),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let mut bind_group_layouts = vec![bind_group_layout, uniform_bind_group_layout];
        if let EffectType::LoadLUT { .. } = effect_type {
            bind_group_layouts.push(lut_bind_group_layout);
        }

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("Effect Pipeline Layout: {:?}", effect_type)),
            bind_group_layouts: &bind_group_layouts,
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("Effect Pipeline: {:?}", effect_type)),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 16,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Ok(pipeline)
    }

    /// Get the WGSL shader source for an effect type
    pub(crate) fn get_effect_shader_source(effect_type: &EffectType) -> &'static str {
        match effect_type {
            EffectType::LoadLUT { .. } => include_str!("../../../../shaders/lut_color_grade.wgsl"),
            EffectType::ColorAdjust => include_str!("../../../../shaders/effect_color_adjust.wgsl"),
            EffectType::Blur => include_str!("../../../../shaders/effect_blur.wgsl"),
            EffectType::ChromaticAberration => {
                include_str!("../../../../shaders/effect_chromatic_aberration.wgsl")
            }
            EffectType::EdgeDetect => include_str!("../../../../shaders/effect_edge_detect.wgsl"),
            EffectType::Invert => include_str!("../../../../shaders/effect_invert.wgsl"),
            EffectType::Pixelate => include_str!("../../../../shaders/effect_pixelate.wgsl"),
            EffectType::Vignette => include_str!("../../../../shaders/effect_vignette.wgsl"),
            EffectType::FilmGrain => include_str!("../../../../shaders/effect_film_grain.wgsl"),
            EffectType::Wave => include_str!("../../../../shaders/effect_wave.wgsl"),
            EffectType::Glitch => include_str!("../../../../shaders/effect_glitch.wgsl"),
            EffectType::RgbSplit => include_str!("../../../../shaders/effect_rgb_split.wgsl"),
            EffectType::Mirror => include_str!("../../../../shaders/effect_mirror.wgsl"),
            EffectType::HueShift => include_str!("../../../../shaders/effect_hue_shift.wgsl"),
            EffectType::Kaleidoscope => {
                include_str!("../../../../shaders/effect_kaleidoscope.wgsl")
            }
            EffectType::Voronoi => include_str!("../../../../shaders/effect_voronoi.wgsl"),
            EffectType::Tunnel => include_str!("../../../../shaders/effect_tunnel.wgsl"),
            EffectType::Galaxy => include_str!("../../../../shaders/effect_galaxy.wgsl"),
            _ => include_str!("../../../../shaders/effect_passthrough.wgsl"),
        }
    }
}
