#import bevy_sprite::mesh2d_functions

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(16) uv_rect: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    let model = mesh2d_functions::get_world_from_local(vertex.instance_index);
    out.clip_position = mesh2d_functions::mesh2d_position_local_to_clip(model, vec4<f32>(vertex.position, 1.0));
    out.color = vertex.uv_rect;

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4(1., 1., 1., 1.);
}
