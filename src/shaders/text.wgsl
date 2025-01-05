#import bevy_pretty_text::{
    text_functions::map_and_position_vertex,
    text_types::{
        VertexInput,
        VertexOutput,
    }
}

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    return map_and_position_vertex(in);
}

@group(2) @binding(0) var texture: texture_2d<f32>;
@group(2) @binding(1) var t_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, t_sampler, in.uv) * in.color;
}
