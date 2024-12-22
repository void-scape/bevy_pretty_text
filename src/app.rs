use crate::materials::{ShakeMaterial, TextShaderPlugin};
use crate::type_writer::TypeWriterPlugin;
use crate::effect;
use crate::materials::WaveMaterial;
use crate::render::material::TextMaterial2dPlugin;
use crate::render::mesh::{GlyphMeshCache, TextMesh2dPlugin};
use bevy::prelude::*;
use bevy::text::Update2dText;

pub struct PrettyTextPlugin;

impl Plugin for PrettyTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TypeWriterPlugin,
            TextMesh2dPlugin,
            TextMaterial2dPlugin::<WaveMaterial>::default(),
            TextMaterial2dPlugin::<ShakeMaterial>::default(),
            TextShaderPlugin,
        ))
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
