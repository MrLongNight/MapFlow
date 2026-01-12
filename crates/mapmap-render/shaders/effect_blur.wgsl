// Blur Effect Shader
// Simple box blur with configurable radius

struct Uniforms {
    time: f32,
    intensity: f32,
    param_a: f32,     // radius
    param_b: f32,     // samples (not used in this simple version)
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
    let radius = uniforms.param_a * uniforms.intensity;
    let pixel_size = vec2<f32>(1.0) / uniforms.resolution;

    var color = vec4<f32>(0.0);
    var samples = 0.0;

    // 9-tap box blur
    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            let offset = vec2<f32>(f32(x), f32(y)) * pixel_size * radius;
            color += textureSample(input_texture, input_sampler, input.uv + offset);
            samples += 1.0;
        }
    }

    let blurred = color / samples;
    let original = textureSample(input_texture, input_sampler, input.uv);

    return mix(original, blurred, uniforms.intensity);
}
