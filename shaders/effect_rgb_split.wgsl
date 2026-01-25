// RGB Split effect shader
// Separates RGB channels with offset

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct Params {
    time: f32,
    intensity: f32,
    offset_x: f32, // param_a
    offset_y: f32, // param_b
    direction: vec2<f32>,
    resolution: vec2<f32>,
};

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(1) @binding(0) var<uniform> params: Params;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.position, 0.0, 1.0);
    output.uv = input.uv;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let uv = input.uv;
    let intensity = params.intensity;

    // Calculate offset based on intensity
    let offset_x = params.offset_x * intensity * 0.02;
    let offset_y = params.offset_y * intensity * 0.02;

    // Sample each channel with different offsets
    let r = textureSample(input_texture, texture_sampler, uv + vec2<f32>(offset_x, offset_y)).r;
    let g = textureSample(input_texture, texture_sampler, uv).g;
    let b = textureSample(input_texture, texture_sampler, uv - vec2<f32>(offset_x, offset_y)).b;
    let a = textureSample(input_texture, texture_sampler, uv).a;

    return vec4<f32>(r, g, b, a);
}
