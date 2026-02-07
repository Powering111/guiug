struct VertexInput {
    @location(0) position: vec2f,
    @location(1) uv: vec2f,
    @location(2) instance_position: vec3i,
    @location(3) instance_scale: vec2i,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) uv: vec2f,
}

@group(0) @binding(0)
var<uniform> screen_size: vec3u;

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    let position_pixel = in.position * vec2f(in.instance_scale) + vec2f(f32(in.instance_position.x), f32(i32(screen_size.y) - in.instance_position.y));

    let screen_size_f = vec2f(screen_size.xy);

    let position_normalized = position_pixel / screen_size_f;
    let position_ndc = vec4f(2.0 * vec2f(position_normalized) - 1.0, f32(in.instance_position.z) / f32(screen_size.z), 1.0);

    out.clip_position = position_ndc;
    out.uv = in.uv;

    return out;
}

@group(1) @binding(0)
var ftexture: texture_2d<f32>;
@group(1) @binding(1)
var fsampler: sampler;

@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4f {
    return textureSample(ftexture, fsampler, in.uv);
}