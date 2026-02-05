struct VertexInput {
    @location(0) position: vec3f,
    @location(1) uv: vec2f,
    @location(2) instance_position: vec3u,
    @location(3) instance_scale: vec2u,
    @location(4) instance_color: vec4f,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) color: vec4f,
}

@group(0) @binding(0)
var<uniform> screen_size: vec2u;


@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    let position_pixel = in.position * vec3f(vec2f(in.instance_scale), 1.0) + vec3f(in.instance_position);

    let screen_size_f = vec2f(screen_size);
    let position_ndc = vec4f(2.0 * (position_pixel.xy / screen_size_f) - 1.0, position_pixel.z / 10000.0, 1.0);

    out.clip_position = position_ndc;
    out.color = in.instance_color;

    return out;
}

@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4f {
    return in.color;
}