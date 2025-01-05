use crate::app::PrettyTextAppExt;
use crate::effect::TextEffectAtlas;
use crate::render::material::TextMeshMaterial2d;
use bevy::asset::load_internal_asset;
use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use text::material::{CacheMaterial, InstanceType, TextMaterial2d, DEFAULT_TEXT_SHADER_HANDLE};

pub mod shake;
pub mod wave;

pub const TEXT_TYPES_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(289359829217483435493729152664771819836);
pub const TEXT_FUNCTIONS_SHADER_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(251997945793016399612357380483481955329);

pub struct TextShaderPlugin;

impl Plugin for TextShaderPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            TEXT_TYPES_SHADER_HANDLE,
            "../shaders/text_types.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            TEXT_FUNCTIONS_SHADER_HANDLE,
            "../shaders/text_functions.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            DEFAULT_TEXT_SHADER_HANDLE,
            "../shaders/text.wgsl",
            Shader::from_wgsl
        );

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
        let handle = match M::instance_type() {
            InstanceType::Global => {
                if let Some(handle) = self.global.as_mut() {
                    *handle = materials.add(material.clone());
                    handle.clone()
                } else {
                    let handle = materials.add(material.clone());
                    self.global = Some(handle.clone());
                    handle
                }
            }
            InstanceType::Local => self
                .local
                .entry(root)
                .or_insert_with(|| materials.add(material.clone()))
                .clone(),
            InstanceType::Unique => {
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
