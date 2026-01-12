// YCoCg to RGB conversion shader for HAP Q video codec
//
// HAP Q stores frames in YCoCg-DXT5 format for better quality.
// This shader converts YCoCg colorspace back to RGB for display.

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

@group(0) @binding(0)
var t_ycocg: texture_2d<f32>;
@group(0) @binding(1)
var s_ycocg: sampler;

// For HAP Q Alpha, there's a separate alpha texture
@group(0) @binding(2)
var t_alpha: texture_2d<f32>;
@group(0) @binding(3)
var s_alpha: sampler;

// Uniform for alpha mode
struct Uniforms {
    has_alpha: u32,
    _padding: vec3<u32>,
};

@group(1) @binding(0)
var<uniform> uniforms: Uniforms;

// Full-screen triangle vertex shader
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    // Generate full-screen triangle
    let x = f32(i32(vertex_index & 1u) * 4 - 1);
    let y = f32(i32(vertex_index >> 1u) * 4 - 1);

    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.tex_coord = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);

    return out;
}

// YCoCg to RGB conversion
// Based on the HAP specification and common YCoCg implementations
fn ycocg_to_rgb(ycocg: vec4<f32>) -> vec3<f32> {
    // YCoCg is stored as:
    // R = Co (chrominance orange)
    // G = Y  (luminance)
    // B = Cg (chrominance green)
    // A = Scale factor (for scaled YCoCg-DXT5)

    let scale = (ycocg.a * 255.0 + 0.5) / 255.0;

    // Unscale Co and Cg
    let co = (ycocg.r - 0.5) * scale;
    let cg = (ycocg.b - 0.5) * scale;
    let y = ycocg.g;

    // Convert to RGB
    let r = y + co - cg;
    let g = y + cg;
    let b = y - co - cg;

    return clamp(vec3<f32>(r, g, b), vec3<f32>(0.0), vec3<f32>(1.0));
}

// Alternative scaled YCoCg-R conversion (used by some encoders)
fn ycocg_r_to_rgb(ycocg: vec4<f32>) -> vec3<f32> {
    let y = ycocg.g;
    let co = ycocg.r - 0.5;
    let cg = ycocg.b - 0.5;

    let r = y + co - cg;
    let g = y + cg;
    let b = y - co - cg;

    return clamp(vec3<f32>(r, g, b), vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample YCoCg texture
    let ycocg = textureSample(t_ycocg, s_ycocg, in.tex_coord);

    // Convert YCoCg to RGB
    let rgb = ycocg_to_rgb(ycocg);

    // Get alpha
    var alpha = 1.0;
    if (uniforms.has_alpha != 0u) {
        alpha = textureSample(t_alpha, s_alpha, in.tex_coord).r;
    }

    return vec4<f32>(rgb, alpha);
}

// Simplified fragment shader without separate alpha texture
@fragment
fn fs_main_no_alpha(in: VertexOutput) -> @location(0) vec4<f32> {
    let ycocg = textureSample(t_ycocg, s_ycocg, in.tex_coord);
    let rgb = ycocg_to_rgb(ycocg);
    return vec4<f32>(rgb, 1.0);
}
