#define_import_path bevy_pretty_text::text_functions

#import bevy_sprite::{
    mesh2d_functions as mesh_functions,
    mesh2d_view_bindings::view,
}

#import bevy_pretty_text::text_types::{
    VertexOutput,
    VertexInput,
    UvRect,
}

fn map_and_position_vertex(vertex: VertexInput) -> VertexOutput {
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
    out.color = vertex.color;

    return out;
}
