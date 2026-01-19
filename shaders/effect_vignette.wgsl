// Vignette Effect Shader
// Darkens edges of the image

struct Uniforms {
    time: f32,
    intensity: f32,
    param_a: f32,     // radius (0.0 - 1.0)
    param_b: f32,     // softness (0.0 - 1.0)
    param_c: vec2<f32>,
    resolution: vec2<f32>,
}

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var input_sampler: sampler;
@group(1) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.position, 0.0, 1.0);
    output.uv = input.uv;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(input_texture, input_sampler, input.uv);

    let radius = uniforms.param_a;
    let softness = max(uniforms.param_b, 0.001);

    // Calculate distance from center
    let center = vec2<f32>(0.5);
    let dist = distance(input.uv, center);

    // Calculate vignette factor
    let vignette = smoothstep(radius, radius - softness, dist);

    let vignetted = vec4<f32>(color.rgb * vignette, color.a);

    return mix(color, vignetted, uniforms.intensity);
}
