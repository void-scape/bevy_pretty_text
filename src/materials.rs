use crate::render::material::TextMaterial2d;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::utils::hashbrown::HashMap;

// TODO: cache based on atlases so materials can be shared across root entities.
//
// TODO: size and scale dependent modifier on effects.
#[derive(Default, Component)]
pub struct TextMaterialCache {
    wave: HashMap<Entity, Handle<WaveMaterial>>,
    shake: HashMap<(Entity, usize), Handle<ShakeMaterial>>,
}

impl TextMaterialCache {
    pub fn wave(
        &mut self,
        root: Entity,
        atlas_texture: Handle<Image>,
        materials: &mut Assets<WaveMaterial>,
    ) -> Handle<WaveMaterial> {
        let handle = self
            .wave
            .entry(root)
            .or_insert_with(|| {
                materials.add(WaveMaterial {
                    texture: atlas_texture.clone(),
                })
            })
            .clone();
        self.force_font_atlas_gpu_sync(&handle, atlas_texture, materials);

        handle
    }

    pub fn shake(
        &mut self,
        root: Entity,
        intensity: f32,
        atlas_texture: Handle<Image>,
        materials: &mut Assets<ShakeMaterial>,
    ) -> Handle<ShakeMaterial> {
        let handle = self
            .shake
            .entry((root, (intensity.clamp(0., 1.) * 100.) as usize))
            .or_insert_with(|| {
                materials.add(ShakeMaterial {
                    texture: atlas_texture.clone(),
                    intensity: intensity.clamp(0., 1.),
                })
            })
            .clone();
        self.force_font_atlas_gpu_sync(&handle, atlas_texture, materials);

        handle
    }

    fn force_font_atlas_gpu_sync<M: TextMaterial2d>(
        &self,
        handle: &Handle<M>,
        atlas_texture: Handle<Image>,
        materials: &mut Assets<M>,
    ) {
        // If we request a material instance, it may be the case that the texture atlas has been
        // updated and we should force an update to the GPU texture.
        if let Some(mat) = materials.get_mut(handle) {
            mat.set_texture(atlas_texture.clone());
        }
    }
}

pub struct TextShaderPlugin;

impl Plugin for TextShaderPlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world_mut().resource_mut::<Assets<Shader>>();

        shaders.insert(
            &WAVE_SHADER_HANDLE,
            Shader::from_wgsl(
                WAVE_SHADER,
                "shaders/wave.wgsl",
            ),
        );
        shaders.insert(
            &SHAKE_SHADER_HANDLE,
            Shader::from_wgsl(
                SHAKE_SHADER,
                "shaders/shake.wgsl",
            ),
        );
    }
}

macro_rules! impl_text_material2d {
    ($ty:ty, $handle:expr) => {
        impl TextMaterial2d for $ty {
            fn set_texture(&mut self, texture: Handle<Image>) {
                self.texture = texture;
            }

            fn fragment_shader() -> ShaderRef {
                $handle.into()
            }

            fn vertex_shader() -> ShaderRef {
                $handle.into()
            }
        }
    };
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct WaveMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
}

pub const WAVE_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(140760193724908016942612779440926461879);
impl_text_material2d!(WaveMaterial, WAVE_SHADER_HANDLE);

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ShakeMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    #[uniform(2)]
    pub intensity: f32,
}

pub const SHAKE_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(79568255301670276214221003054688204202);
impl_text_material2d!(ShakeMaterial, SHAKE_SHADER_HANDLE);

pub const WAVE_SHADER: &'static str = r"
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
    out.position.y += sin(globals.time + out.position.x * 4.) * 0.05;

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
";


pub const SHAKE_SHADER: &'static str = r"
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

    out.position.y += sin((globals.time + uvs.max.y) * 128. * intensity) * 0.05;
    out.position.x += sin((globals.time + uvs.min.x) * 64. * intensity) * 0.05;

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
";


