struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

struct UniformState {
    camera: CameraUniform,
    is_srgb: f32,
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@group(1) @binding(0)
var<uniform> u_state: UniformState;

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.uv = model.uv;
    out.clip_position = u_state.camera.view_proj * vec4<f32>(model.position, 1.0);

    return out;
}

fn less_than(a: vec4<f32>, b: vec4<f32>) -> vec4<bool> {
    return vec4<bool>(
        a.x < b.x,
        a.y < b.y,
        a.z < b.z,
        a.w < b.w,
    );
}
fn boolean_mix(a: vec4<f32>, b: vec4<f32>, c: vec4<bool>) -> vec4<f32> {
    let floats = vec4<f32>(c);
    return mix(a, b, floats);
}
fn from_linear(linearRGB: vec4<f32>) -> vec4<f32> {
    let cutoff: vec4<bool> = less_than(linearRGB, vec4<f32>(0.0031308));
    let higher: vec4<f32> = vec4<f32>(1.055) * pow(linearRGB, vec4<f32>(1. / 2.4)) - vec4<f32>(0.055);
    let lower: vec4<f32> = linearRGB * vec4<f32>(12.92);
    return boolean_mix(higher, lower, cutoff);
}
fn to_linear(sRGB: vec4<f32>) -> vec4<f32> {
    let cutoff: vec4<bool> = less_than(sRGB, vec4<f32>(0.04045));
    let higher: vec4<f32> = pow((sRGB + vec4<f32>(0.055)) / vec4<f32>(1.055), vec4<f32>(2.4));
    let lower: vec4<f32> = sRGB / vec4<f32>(12.92);
    return boolean_mix(higher, lower, cutoff);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(t_diffuse, s_diffuse, in.uv);

    if (u_state.is_srgb < 0.5) {
        color = from_linear(color);
    }

    return color;
}
