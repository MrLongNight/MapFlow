struct Uniforms {
    time: f32,
    intensity: f32,
    param_a: f32, // lambda
    param_b: f32, // theta
    param_c: vec2<f32>, // x: alpha, y: sigma
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

const PI: f32 = 3.141592653589793;

fn osc(mV: f32, MV: f32, i: f32, t: f32) -> f32 {
    return mV + (MV - mV) * 0.5 * (sin(2.0 * PI * t / i) + 1.0);
}

fn r2d(p: vec2<f32>, a: f32) -> vec2<f32> {
    let s: f32 = sin(a);
    let c: f32 = cos(a);
    let rm: mat2x2<f32> = mat2x2<f32>(c, -s, s, c);
    return rm * p;
}

fn cC(uv: vec2<f32>, time: f32, resolution: vec2<f32>) -> vec4<f32> {
    var p: vec3<f32> = vec3<f32>(uv, 1.0);
    let a: f32 = 0.15 * PI * time;
    let rp: vec2<f32> = r2d(p.xy, a);
    let mX: f32 = (sin(uniforms.time * 0.5) + 1.0) * uniforms.param_b; // theta -> param_b
    let mY: f32 = (cos(uniforms.time * 0.5) + 1.0) * uniforms.param_b;
    let res_y = select(1.0, resolution.y, resolution.y > 0.0);
    var hc: vec3<f32> = vec3<f32>((rp.xy * 1.5 + 0.5) * resolution.xy / res_y, mX);
    for (var i: i32 = 0; i < 45; i++) {
        let xV: f32 = osc(1.2, 1.3, 10.0, time);
        let yV: f32 = osc(uniforms.param_c.x, uniforms.param_c.y, 8.0, uniforms.time); // alpha -> c.x, sigma -> c.y
        let gamma_param = 1.0;
        let blue_param = 20.0;
        let zV: f32 = osc(gamma_param, blue_param, 8.0, uniforms.time);
        let temp: vec3<f32> = vec3<f32>(xV, yV, zV) * (abs((abs(hc) / abs(dot(hc, hc))) - vec3<f32>(1.0, 1.0, mY)));
        hc.x = temp.x;
        hc.z = temp.y;
        hc.y = temp.z;
    }
    return vec4<f32>(hc, 1.0);
}

fn bl(uv: vec2<f32>, time: f32, resolution: vec2<f32>) -> vec4<f32> {
    let bs: f32 = 0.1 / 255.0;
    var col: vec4<f32> = vec4<f32>(0.0);
    for (var x: f32 = -1.1; x <= 0.5; x += 1.0) {
        for (var y: f32 = -1.1; y <= 2.5; y += 1.0) {
            col += cC(uv + vec2<f32>(x, y) * bs, time, resolution);
        }
    }
    return col / 9.0;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let resolution: vec2<f32> = uniforms.resolution;
    // Note: FragCoord.xy is in pixels. input.uv is 0..1.
    // Original shader used FragCoord. We can convert UV back to pixel coords or adapt the formula.
    // Original: var uv: vec2<f32> = params.lambda * (1.5 * FragCoord.xy - resolution.xy) / resolution.y * 1.1;
    // We can say FragCoord.xy ~= input.uv * resolution

    let res_y = select(1.0, resolution.y, resolution.y > 0.0);
    let fragCoord = input.uv * resolution;

    var uv: vec2<f32> = uniforms.param_a * (1.5 * fragCoord - resolution.xy) / res_y * 1.1; // lambda -> param_a
    let fc: vec4<f32> = bl(uv, uniforms.time, resolution);

    // Mix with original image if intensity < 1.0 (though tunnel usually overwrites)
    let original = textureSample(input_texture, input_sampler, input.uv);
    return mix(original, fc, uniforms.intensity);
}
