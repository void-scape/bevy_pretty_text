#define_import_path bevy_pretty_text::text_types

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(16) atlas_uv: vec4<f32>,
    @location(17) color: vec4<f32>,
};

struct UvRect {
    min: vec2<f32>,
    max: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>
}
