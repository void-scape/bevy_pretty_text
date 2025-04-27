use crate::render::material::TextMaterial2d;
use bevy::asset::{load_internal_asset, weak_handle};
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

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
        load_internal_asset!(
            app,
            WAVE_SHADER_HANDLE,
            "shaders/wave.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            SHAKE_SHADER_HANDLE,
            "shaders/shake.wgsl",
            Shader::from_wgsl
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

pub const WAVE_SHADER_HANDLE: Handle<Shader> = weak_handle!("590c0854-6e78-4fb7-a700-f31e826185bc");
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
    weak_handle!("391909dd-7f0b-4c3c-8a16-5021e32b2a48");
impl_text_material2d!(ShakeMaterial, SHAKE_SHADER_HANDLE);
