// Film Grain Effect Shader
// Adds animated noise overlay for a film-like look

struct Uniforms {
    time: f32,
    intensity: f32,
    param_a: f32,     // amount (0.0 - 1.0)
    param_b: f32,     // speed
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

// Hash function for pseudo-random noise
fn hash(p: vec2<f32>) -> f32 {
    let p3 = fract(vec3<f32>(p.xyx) * 0.1031);
    let p3_dot = dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(input_texture, input_sampler, input.uv);

    let amount = uniforms.param_a * uniforms.intensity;
    let speed = uniforms.param_b;

    // Generate animated noise
    let noise_uv = input.uv * uniforms.resolution;
    let noise = hash(noise_uv + vec2<f32>(uniforms.time * speed, 0.0)) * 2.0 - 1.0;

    // Apply grain
    let grain = color.rgb + vec3<f32>(noise * amount);
    let grained = vec4<f32>(grain, color.a);

    return mix(color, grained, uniforms.intensity);
}
