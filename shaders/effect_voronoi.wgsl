struct Uniforms {
    time: f32,
    intensity: f32,
    param_a: f32, // Scale (lambda)
    param_b: f32, // Offset (theta)
    param_c: vec2<f32>, // x: Cell (alpha), y: Distortion (sigma)
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

const PI: f32 = 3.14;

fn hash3(p: vec3<f32>) -> vec3<f32> {
    let q: vec3<f32> = vec3<f32>(
        dot(p, vec3<f32>(127.1, 311.7, 189.2)),
        dot(p, vec3<f32>(269.5, 183.3, 324.7)),
        dot(p, vec3<f32>(419.2, 371.9, 128.5))
    );
    return fract(sin(q) * 43758.5453);
}

fn osc(minValue: f32, maxValue: f32, interval: f32, currentTime: f32) -> f32 {
    return minValue + (maxValue - minValue) * 0.5 * (sin(2.0 * PI * currentTime / interval) + 1.0);
}

fn noise(x: vec3<f32>, v: f32) -> f32 {
    let p: vec3<f32> = floor(x);
    let f: vec3<f32> = fract(x);
    let s: f32 = 1.0 + uniforms.param_a * v; // lambda -> param_a
    var va: f32 = 0.0;
    var wt: f32 = 0.0;
    var k: i32 = -2;
    while (k <= 1) {
        var j: i32 = -2;
        while (j <= 1) {
            var i: i32 = -2;
            while (i <= 1) {
                let g: vec3<f32> = vec3<f32>(f32(i), f32(j), f32(k));
                let o: vec3<f32> = hash3(p + g);
                let r: vec3<f32> = g - f + o + 0.5;
                let d: f32 = dot(r, r);
                let w: f32 = pow(1.0 - smoothstep(0.0, 1.414, sqrt(d)), s);
                va += o.z * w;
                wt += w;
                i += 1;
            }
            j += 1;
        }
        k += 1;
    }
    return va / wt;
}

fn fBm(p: vec3<f32>, v: f32) -> f32 {
    var sum: f32 = 0.0;
    let blue_param = 20.0;
    let scramb: f32 = osc(0.0, blue_param, 20.0, uniforms.time);
    var amp: f32 = scramb;
    var mutable_p = p;
    var i: i32 = 0;
    while (i < 4) {
        sum += amp * noise(mutable_p, v);
        amp *= 0.3;
        mutable_p *= 2.0;
        i += 1;
    }
    return sum;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let resolution: vec2<f32> = uniforms.resolution;
    let uv: vec2<f32> = input.uv; // Use input UV directly

    // Original shader logic
    // let p: vec2<f32> = uv;
    // let rd: vec3<f32> = normalize(vec3<f32>(p.x, p.y, 1.0));
    // Reconstruct rd from UV approx
    let p = uv * 2.0 - 1.0;
    let rd: vec3<f32> = normalize(vec3<f32>(p.x, p.y * (resolution.y/resolution.x), 1.0));

    let gamma_param = 1.0;
    let pos: vec3<f32> = vec3<f32>(0.0, 0.0, 1.0) * uniforms.time + rd * gamma_param;

    let center: vec2<f32> = vec2<f32>(uniforms.param_b, uniforms.param_b); // theta -> param_b
    let toCenter: vec2<f32> = center - uv;
    let distanceFromCenter: f32 = length(toCenter);
    let adjustedDistance: f32 = distanceFromCenter * uniforms.param_c.x - uniforms.param_c.x; // alpha -> param_c.x

    let sigma_param = uniforms.param_c.y; // sigma -> param_c.y
    let distortionStrength: f32 = fBm(pos, sigma_param) * sigma_param;
    let distortionDirection: vec2<f32> = normalize(toCenter) * adjustedDistance;
    let distortedUV: vec2<f32> = uv + distortionDirection * distortionStrength;

    // Mix with original based on intensity
    let texColor: vec4<f32> = textureSample(input_texture, input_sampler, distortedUV);
    let original: vec4<f32> = textureSample(input_texture, input_sampler, input.uv);

    return mix(original, texColor, uniforms.intensity);
}
