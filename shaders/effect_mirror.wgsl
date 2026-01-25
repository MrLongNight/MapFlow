// Mirror effect shader
// Creates symmetrical reflections

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
    mode: f32, // param_a - 0=horizontal, 1=vertical, 2=both, 3=diagonal
    center: f32, // param_b - mirror center position (0.0-1.0)
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
    var uv = input.uv;
    let mode = i32(params.mode);
    let center = select(0.5, params.center, params.center > 0.0);
    
    // Horizontal mirror (left-right)
    if mode == 0 || mode == 2 {
        if uv.x > center {
            uv.x = center - (uv.x - center);
        }
    }
    
    // Vertical mirror (top-bottom)
    if mode == 1 || mode == 2 {
        if uv.y > center {
            uv.y = center - (uv.y - center);
        }
    }
    
    // Diagonal mirror
    if mode == 3 {
        if uv.x + uv.y > 1.0 {
            let temp = uv.x;
            uv.x = 1.0 - uv.y;
            uv.y = 1.0 - temp;
        }
    }
    
    // Clamp to valid range
    uv = clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0));
    
    return textureSample(input_texture, texture_sampler, uv);
}
