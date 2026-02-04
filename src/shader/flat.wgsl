struct VertexInput {
    @location(0) position: vec3f,
    @location(1) uv: vec2f,
    @location(2) instance_position: vec3f,
    @location(3) instance_scale: vec3f,
    @location(4) instance_color: vec4f,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) color: vec4f,
}

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    let position = in.position * in.instance_scale + in.instance_position;

    out.clip_position = vec4f(position, 1.0);
    out.color = in.instance_color;

    return out;
}

@fragment
fn fs_main(
    in: VertexOutput,
) -> @location(0) vec4f {
    return in.color;
}