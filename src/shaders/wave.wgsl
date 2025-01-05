#import bevy_pretty_text::{
    text_functions::map_and_position_vertex,
    text_types::{
        VertexInput,
        VertexOutput,
    }
}

#import bevy_render::globals::Globals
@group(0) @binding(1) var<uniform> globals: Globals;

@group(2) @binding(2) var<uniform> speed: f32;

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out = map_and_position_vertex(in);
    out.position.y += sin((globals.time + out.position.x) * speed) * 0.025;
    return out;
}

