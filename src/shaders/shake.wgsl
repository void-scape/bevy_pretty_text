#import bevy_sprite::{
    mesh2d_functions as mesh_functions,
    mesh2d_view_bindings::view,
}

#import bevy_render::globals::Globals
@group(0) @binding(1) var<uniform> globals: Globals;

struct Vertex {
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    // @location(2) uv: vec2<f32>,
    @location(16) atlas_uv: vec4<f32>,
    @location(17) color: vec4<f32>,
};

struct UvRect {
    min: vec2<f32>,
    max: vec2<f32>,
}
@group(2) @binding(2) var<uniform> intensity: f32;

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    var uvs: UvRect;
    uvs.min = vec2(vertex.atlas_uv.x, vertex.atlas_uv.y);
    uvs.max = vec2(vertex.atlas_uv.z, vertex.atlas_uv.w);

    switch vertex.vertex_index % 4u {
        case 0u: { // Top Right
            out.uv = vec2(uvs.max.x, uvs.max.y);
            break;
        }
        case 1u: { // Top Left
            out.uv = vec2(uvs.min.x, uvs.max.y);
            break;
        }
        case 2u: { // Bottom Left
            out.uv = vec2(uvs.min.x, uvs.min.y);
            break;
        }
        case 3u: { // Bottom Right
            out.uv = vec2(uvs.max.x, uvs.min.y);
            break;
        }
        default: {
            break;
        }
    }

    var world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
    let world_position = mesh_functions::mesh2d_position_local_to_world(
        world_from_local,
        vec4<f32>(vertex.position, 1.0)
    );
    out.position = mesh_functions::mesh2d_position_world_to_clip(world_position);

    out.position.y += sin((globals.time + uvs.max.y) * 128. * intensity) * 0.0025;
    out.position.x += sin((globals.time + uvs.min.x) * 64. * intensity) * 0.0025;

    out.color = vertex.color;

    return out;
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>
}

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var t_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, t_sampler, in.uv) * in.color;
}
