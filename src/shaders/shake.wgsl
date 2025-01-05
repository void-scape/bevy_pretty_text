#import bevy_pretty_text::{
    text_functions::map_and_position_vertex,
    text_types::{
        VertexInput,
        VertexOutput,
    }
}

#import bevy_render::globals::Globals
@group(0) @binding(1) var<uniform> globals: Globals;

@group(2) @binding(2) var<uniform> intensity: f32;

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out = map_and_position_vertex(in);
    out.position.y += sin((globals.time + f32(in.instance_index) * 4.) * 128. * intensity) * 0.0025;
    out.position.x += sin((globals.time + f32(in.instance_index)) * 64. * intensity) * 0.0025;
    return out;
}

