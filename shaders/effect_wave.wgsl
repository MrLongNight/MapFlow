// Wave distortion effect shader
// Applies a wave distortion to the image

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
    frequency: f32, // param_a - wave frequency
    amplitude: f32, // param_b - wave amplitude
    direction: vec2<f32>, // param_c - wave direction
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
    
    // Wave parameters
    let frequency = max(params.frequency, 1.0);
    let amplitude = params.amplitude * params.intensity * 0.1;
    let time = params.time;
    
    // Calculate wave offset
    let wave_x = sin(uv.y * frequency + time * 2.0) * amplitude;
    let wave_y = cos(uv.x * frequency + time * 2.0) * amplitude;
    
    // Apply distortion
    let distorted_uv = vec2<f32>(
        uv.x + wave_x,
        uv.y + wave_y
    );
    
    // Clamp to valid range
    let clamped_uv = clamp(distorted_uv, vec2<f32>(0.0), vec2<f32>(1.0));
    
    return textureSample(input_texture, texture_sampler, clamped_uv);
}
