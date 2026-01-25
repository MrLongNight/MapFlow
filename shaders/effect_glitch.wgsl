// Glitch effect shader
// Applies digital glitch distortion

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
    block_size: f32, // param_a
    color_shift: f32, // param_b
    direction: vec2<f32>,
    resolution: vec2<f32>,
};

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(1) @binding(0) var<uniform> params: Params;

fn rand(co: vec2<f32>) -> f32 {
    return fract(sin(dot(co, vec2<f32>(12.9898, 78.233))) * 43758.5453);
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
    var uv = input.uv;
    let time = params.time;
    let intensity = params.intensity;
    
    // Block size for glitch
    let block_size = max(params.block_size, 8.0);
    
    // Calculate block position
    let block_y = floor(uv.y * params.resolution.y / block_size);
    let block_time = floor(time * 10.0);
    
    // Random offset per block
    let rand_val = rand(vec2<f32>(block_y, block_time));
    
    // Apply horizontal shift if random threshold met
    if rand_val > (1.0 - intensity * 0.3) {
        let shift = (rand(vec2<f32>(block_y + 1.0, block_time)) - 0.5) * intensity * 0.2;
        uv.x = uv.x + shift;
    }
    
    // RGB splitting for color glitch
    let color_shift = params.color_shift * intensity * 0.01;
    let r = textureSample(input_texture, texture_sampler, uv + vec2<f32>(color_shift, 0.0)).r;
    let g = textureSample(input_texture, texture_sampler, uv).g;
    let b = textureSample(input_texture, texture_sampler, uv - vec2<f32>(color_shift, 0.0)).b;
    let a = textureSample(input_texture, texture_sampler, uv).a;
    
    return vec4<f32>(r, g, b, a);
}
