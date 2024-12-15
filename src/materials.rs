use crate::render::material::TextMaterial2d;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::utils::hashbrown::HashMap;

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

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct WaveMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
}

impl TextMaterial2d for WaveMaterial {
    fn set_texture(&mut self, texture: Handle<Image>) {
        self.texture = texture;
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/wave.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/wave.wgsl".into()
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ShakeMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    #[uniform(2)]
    pub intensity: f32,
}

impl TextMaterial2d for ShakeMaterial {
    fn set_texture(&mut self, texture: Handle<Image>) {
        self.texture = texture;
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/shake.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/shake.wgsl".into()
    }
}
