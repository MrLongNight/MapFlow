// Kaleidoscope effect shader
// Creates kaleidoscope-style reflections

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
    segments: f32, // param_a - number of segments (4-32)
    rotation: f32, // param_b - rotation speed
    direction: vec2<f32>,
    resolution: vec2<f32>,
};

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(1) @binding(0) var<uniform> params: Params;

const PI: f32 = 3.14159265359;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(input.position, 0.0, 1.0);
    output.uv = input.uv;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Center the UV coordinates
    var uv = input.uv - 0.5;

    let segments = max(params.segments, 4.0);
    let rotation = params.rotation * params.time;

    // Convert to polar coordinates
    var angle = atan2(uv.y, uv.x) + rotation;
    let radius = length(uv);

    // Calculate segment angle
    let segment_angle = 2.0 * PI / segments;

    // Fold the angle into one segment
    angle = abs(((angle % segment_angle) + segment_angle) % segment_angle);
    if angle > segment_angle * 0.5 {
        angle = segment_angle - angle;
    }

    // Convert back to cartesian coordinates
    uv = vec2<f32>(cos(angle), sin(angle)) * radius;

    // Re-center and clamp
    uv = uv + 0.5;
    uv = clamp(uv, vec2<f32>(0.0), vec2<f32>(1.0));

    return textureSample(input_texture, texture_sampler, uv);
}
