use crate::app::PrettyTextAppExt;
use crate::effect::TextEffectAtlas;
use crate::render::material::TextMeshMaterial2d;
use bevy::asset::load_internal_asset;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use text::material::{CacheMaterial, CacheType, TextMaterial2d};

pub mod shake;
pub mod wave;

pub struct TextShaderPlugin;

impl Plugin for TextShaderPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            wave::WAVE_SHADER_HANDLE,
            "../shaders/wave.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            shake::SHAKE_SHADER_HANDLE,
            "../shaders/shake.wgsl",
            Shader::from_wgsl
        );

        app.register_text_material::<wave::Wave>()
            .register_text_material::<shake::Shake>();
    }
}

#[macro_export]
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

pub fn cache_new_text_materials<M: TextMaterial2d>(
    mut commands: Commands,
    mut cache: ResMut<TextMaterialCache<M>>,
    query: Query<(Entity, &TextEffectAtlas, &CacheMaterial<M>, &Parent), Changed<CacheMaterial<M>>>,
    mut materials: ResMut<Assets<M>>,
) {
    for (entity, atlas, material, parent) in query.iter() {
        commands
            .entity(entity)
            .insert(TextMeshMaterial2d(cache.cache(
                parent.get(),
                &material.material,
                atlas.handle(),
                &mut materials,
            )));
    }
}

// TODO: size and scale dependent modifier on effects.
//
// TODO: instanced unique hashing
#[derive(Default, Resource)]
pub struct TextMaterialCache<M: TextMaterial2d> {
    global: Option<Handle<M>>,
    local: HashMap<Entity, Handle<M>>,
    unique: Vec<Handle<M>>,
}

impl<M: TextMaterial2d> TextMaterialCache<M> {
    pub fn cache(
        &mut self,
        root: Entity,
        material: &M,
        atlas_texture: Handle<Image>,
        materials: &mut Assets<M>,
    ) -> Handle<M> {
        let handle = match M::cache_type() {
            CacheType::Global => {
                if let Some(handle) = self.global.as_mut() {
                    *handle = materials.add(material.clone());
                    handle.clone()
                } else {
                    let handle = materials.add(material.clone());
                    self.global = Some(handle.clone());
                    handle
                }
            }
            CacheType::Local => self
                .local
                .entry(root)
                .or_insert_with(|| materials.add(material.clone()))
                .clone(),
            CacheType::Unique => {
                let handle = materials.add(material.clone());
                self.unique.push(handle.clone());
                handle
            }
        };
        self.force_font_atlas_gpu_sync(&handle, atlas_texture, materials);

        handle
    }

    fn force_font_atlas_gpu_sync(
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
