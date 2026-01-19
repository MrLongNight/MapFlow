// Color Adjust Effect Shader
// Adjusts brightness, contrast, and saturation

struct Uniforms {
    time: f32,
    intensity: f32,
    param_a: f32,     // brightness (-1 to 1)
    param_b: f32,     // contrast (0 to 2)
    param_c: vec2<f32>, // saturation (x component, 0 to 2)
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
    var color = textureSample(input_texture, input_sampler, input.uv);

    let brightness = uniforms.param_a;
    let contrast = uniforms.param_b;
    let saturation = uniforms.param_c.x;

    // Apply brightness
    color = vec4<f32>(color.rgb + brightness, color.a);

    // Apply contrast
    color = vec4<f32>((color.rgb - 0.5) * contrast + 0.5, color.a);

    // Apply saturation
    let gray = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
    color = vec4<f32>(mix(vec3<f32>(gray), color.rgb, saturation), color.a);

    // Mix with original based on intensity
    let original = textureSample(input_texture, input_sampler, input.uv);
    return mix(original, color, uniforms.intensity);
}
