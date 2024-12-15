use self::materials::ShakeMaterial;
use crate::effect::UpdateTextEffects;
use crate::materials::{TextMaterialCache, WaveMaterial};
use crate::render::material::TextMaterial2dPlugin;
use crate::render::mesh::{GlyphMeshCache, TextMesh2dPlugin};
use crate::type_writer::TypeWriterPlugin;
use bevy::prelude::*;
use bevy::text::Update2dText;
use type_writer::TypeWriterSection;

pub mod effect;
pub mod materials;
pub mod render;
pub mod type_writer;

pub struct PrettyTextPlugin;

impl Plugin for PrettyTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TypeWriterPlugin,
            TextMesh2dPlugin,
            TextMaterial2dPlugin::<WaveMaterial>::default(),
            TextMaterial2dPlugin::<ShakeMaterial>::default(),
        ))
        .insert_resource(GlyphMeshCache::default())
        .register_required_components::<TypeWriterSection, TextMaterialCache>()
        .add_systems(
            PostUpdate,
            (
                crate::effect::compute_info,
                crate::effect::extract_effect_glyphs,
            )
                .chain()
                .in_set(UpdateTextEffects)
                .after(Update2dText),
        );
    }
}
