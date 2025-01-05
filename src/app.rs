use crate::effect;
use crate::materials::{TextMaterialCache, TextShaderPlugin};
use crate::render::material::TextMaterial2dPlugin;
use crate::render::mesh::{GlyphMeshCache, TextMesh2dPlugin};
use crate::type_writer::TypeWriterPlugin;
use bevy::prelude::*;
use bevy::text::Update2dText;
use std::hash::Hash;
use text::material::TextMaterial2d;

pub struct PrettyTextPlugin;

impl Plugin for PrettyTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((TypeWriterPlugin, TextMesh2dPlugin, TextShaderPlugin))
            .insert_resource(GlyphMeshCache::default())
            .add_systems(
                PostUpdate,
                (
                    effect::compute_info,
                    effect::extract_effect_glyphs.after(effect::compute_info),
                )
                    .chain()
                    .in_set(UpdateTextEffects),
            )
            .configure_sets(PostUpdate, UpdateTextEffects.after(Update2dText));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct UpdateTextEffects;

pub trait PrettyTextAppExt {
    fn register_text_material<M>(&mut self) -> &mut Self
    where
        M: TextMaterial2d + Default,
        M::Data: PartialEq + Eq + Hash + Clone;
}

impl PrettyTextAppExt for App {
    fn register_text_material<M>(&mut self) -> &mut Self
    where
        M: TextMaterial2d + Default,
        M::Data: PartialEq + Eq + Hash + Clone,
    {
        self.add_plugins(TextMaterial2dPlugin::<M>::default())
            .insert_resource(TextMaterialCache::<M>::default())
            .add_systems(
                PostUpdate,
                crate::materials::cache_new_text_materials::<M>.after(Update2dText),
            );
        self
    }
}
