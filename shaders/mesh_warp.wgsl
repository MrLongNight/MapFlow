// Mesh Warp Shader - Perspective Correction and Mesh Warping
//
// Supports:
// - Arbitrary mesh warping (quad, triangle, custom meshes)
// - Perspective-correct texture mapping
// - Per-vertex UV interpolation
// - Horizontal/Vertical flip
// - Color correction (brightness, contrast, saturation, hue)

struct VertexInput {
    @location(0) position: vec3<f32>,  // Output position (screen space)
    @location(1) tex_coords: vec2<f32>, // Texture coordinates
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) @interpolate(perspective) perspective_tex_coords: vec2<f32>,
}

struct Uniforms {
    transform: mat4x4<f32>,  // Model-View-Projection transform
    opacity: f32,            // Layer opacity
    flip_h: f32,             // Horizontal flip (0.0 or 1.0)
    flip_v: f32,             // Vertical flip (0.0 or 1.0)
    brightness: f32,         // Brightness adjustment (-1.0 to 1.0)
    contrast: f32,           // Contrast adjustment (0.0 to 2.0, 1.0 = normal)
    saturation: f32,         // Saturation adjustment (0.0 to 2.0, 1.0 = normal)
    hue_shift: f32,          // Hue shift in degrees (-180 to 180)
    _padding: f32,           // Padding for 16-byte alignment
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var t_texture: texture_2d<f32>;

@group(1) @binding(1)
var s_sampler: sampler;

// RGB to HSV conversion
fn rgb_to_hsv(rgb: vec3<f32>) -> vec3<f32> {
    let maxc = max(rgb.r, max(rgb.g, rgb.b));
    let minc = min(rgb.r, min(rgb.g, rgb.b));
    let v = maxc;
    let s = select(0.0, (maxc - minc) / maxc, maxc > 0.0);
    var h = 0.0;
    if maxc != minc {
        let diff = maxc - minc;
        if maxc == rgb.r {
            h = (rgb.g - rgb.b) / diff;
        } else if maxc == rgb.g {
            h = 2.0 + (rgb.b - rgb.r) / diff;
        } else {
            h = 4.0 + (rgb.r - rgb.g) / diff;
        }
        h = h / 6.0;
        if h < 0.0 { h = h + 1.0; }
    }
    return vec3<f32>(h, s, v);
}

// HSV to RGB conversion
fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
    let h = hsv.x * 6.0;
    let s = hsv.y;
    let v = hsv.z;
    let i = floor(h);
    let f = h - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    let ii = i32(i) % 6;
    if ii == 0 { return vec3<f32>(v, t, p); }
    if ii == 1 { return vec3<f32>(q, v, p); }
    if ii == 2 { return vec3<f32>(p, v, t); }
    if ii == 3 { return vec3<f32>(p, q, v); }
    if ii == 4 { return vec3<f32>(t, p, v); }
    return vec3<f32>(v, p, q);
}

// Apply color correction
fn apply_color_correction(color: vec4<f32>) -> vec4<f32> {
    var rgb = color.rgb;

    // Apply brightness
    rgb = rgb + vec3<f32>(uniforms.brightness);

    // Apply contrast (around 0.5 midpoint)
    rgb = (rgb - 0.5) * uniforms.contrast + 0.5;

    // Apply saturation
    let gray = dot(rgb, vec3<f32>(0.299, 0.587, 0.114));
    rgb = mix(vec3<f32>(gray), rgb, uniforms.saturation);

    // Apply hue shift
    if abs(uniforms.hue_shift) > 0.01 {
        var hsv = rgb_to_hsv(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0)));
        hsv.x = fract(hsv.x + uniforms.hue_shift / 360.0);
        rgb = hsv_to_rgb(hsv);
    }

    return vec4<f32>(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0)), color.a);
}

// Vertex shader
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Transform vertex position
    out.clip_position = uniforms.transform * vec4<f32>(in.position, 1.0);

    // Apply flip to texture coordinates
    var uv = in.tex_coords;
    if uniforms.flip_h > 0.5 {
        uv.x = 1.0 - uv.x;
    }
    if uniforms.flip_v > 0.5 {
        uv.y = 1.0 - uv.y;
    }

    // Pass through texture coordinates
    out.tex_coords = uv;

    // For perspective-correct interpolation, multiply by w
    let w = out.clip_position.w;
    out.perspective_tex_coords = uv * w;

    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Perspective-correct texture sampling
    // Divide interpolated tex_coords by interpolated w
    let tex_coords = in.perspective_tex_coords / in.clip_position.w;

    // Sample texture
    var color = textureSample(t_texture, s_sampler, tex_coords);

    // Apply color correction
    color = apply_color_correction(color);

    // Apply opacity
    color.a *= uniforms.opacity;

    return color;
}

// Alternative: Simple (non-perspective-correct) version for flat mappings
@fragment
fn fs_main_simple(in: VertexOutput) -> @location(0) vec4<f32> {
    // Direct texture sampling (no perspective correction)
    var color = textureSample(t_texture, s_sampler, in.tex_coords);

    // Apply color correction
    color = apply_color_correction(color);

    // Apply opacity
    color.a *= uniforms.opacity;

    return color;
}
