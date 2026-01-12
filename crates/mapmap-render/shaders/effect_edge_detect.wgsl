// Edge Detect Effect Shader
// Sobel edge detection filter

struct Uniforms {
    time: f32,
    intensity: f32,
    param_a: f32,     // threshold
    param_b: f32,
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

fn luminance(color: vec3<f32>) -> f32 {
    return dot(color, vec3<f32>(0.299, 0.587, 0.114));
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel_size = vec2<f32>(1.0) / uniforms.resolution;

    // Sample 3x3 neighborhood
    let tl = luminance(textureSample(input_texture, input_sampler, input.uv + vec2<f32>(-1.0, -1.0) * pixel_size).rgb);
    let t  = luminance(textureSample(input_texture, input_sampler, input.uv + vec2<f32>( 0.0, -1.0) * pixel_size).rgb);
    let tr = luminance(textureSample(input_texture, input_sampler, input.uv + vec2<f32>( 1.0, -1.0) * pixel_size).rgb);
    let l  = luminance(textureSample(input_texture, input_sampler, input.uv + vec2<f32>(-1.0,  0.0) * pixel_size).rgb);
    let r  = luminance(textureSample(input_texture, input_sampler, input.uv + vec2<f32>( 1.0,  0.0) * pixel_size).rgb);
    let bl = luminance(textureSample(input_texture, input_sampler, input.uv + vec2<f32>(-1.0,  1.0) * pixel_size).rgb);
    let b  = luminance(textureSample(input_texture, input_sampler, input.uv + vec2<f32>( 0.0,  1.0) * pixel_size).rgb);
    let br = luminance(textureSample(input_texture, input_sampler, input.uv + vec2<f32>( 1.0,  1.0) * pixel_size).rgb);

    // Sobel operators
    let gx = -tl - 2.0 * l - bl + tr + 2.0 * r + br;
    let gy = -tl - 2.0 * t - tr + bl + 2.0 * b + br;

    let edge = sqrt(gx * gx + gy * gy);
    let edge_color = vec4<f32>(vec3<f32>(edge), 1.0);

    let original = textureSample(input_texture, input_sampler, input.uv);

    return mix(original, edge_color, uniforms.intensity);
}
