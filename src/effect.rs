use crate::{
    materials::{ShakeMaterial, TextMaterialCache, WaveMaterial},
    render::{material::TextMeshMaterial2d, mesh::GlyphMeshCache},
    type_writer::section::{GlyphIndex, TypeWriterSection},
};
use bevy::{
    prelude::*,
    sprite::Anchor,
    text::{ComputedTextBlock, PositionedGlyph, TextLayoutInfo},
    window::PrimaryWindow,
};
use text::TextMod;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct UpdateTextEffects;

/// Generated for a an entity that contains a [`TypeWriterSection`] and a non empty [`TextLayoutInfo`].
///
/// Generation occurs after every change to the [`TextLayoutInfo`].
#[derive(Debug, Component)]
pub struct TextEffectInfo {
    pub atlas: Handle<Image>,
    pub extracted_glyphs: Vec<ExtractedGlyphs>,
}

/// Collection of ['PositionedGlyph'](bevy::text::PositionedGlyph), extracted from [`TextLayoutInfo`],
/// that require a [`TextMeshMaterial2d`].
#[derive(Debug)]
pub struct ExtractedGlyphs {
    pub glyphs: Vec<PositionedGlyph>,
    pub start: usize,
    pub text_mod: TextMod,
    pub root: Entity,
}

pub fn compute_info(
    mut commands: Commands,
    mut sections: Query<(Entity, &TypeWriterSection, &mut TextLayoutInfo), Changed<TextLayoutInfo>>,
) {
    for (entity, section, mut text_layout_info) in sections.iter_mut() {
        let Some(atlas) = text_layout_info
            .glyphs
            .iter()
            .map(|g| g.atlas_info.texture.clone())
            .next()
        else {
            continue;
        };

        let mut extracted_glyphs = Vec::with_capacity(section.text.modifiers.len());
        let mut ranges = Vec::with_capacity(section.text.modifiers.len());

        for tm in section.text.modifiers.iter() {
            if !tm.text_mod.is_shader_effect() {
                continue;
            }

            let start = tm.start;
            let end = tm.end.min(text_layout_info.glyphs.len());
            ranges.push(start..end);

            if text_layout_info.glyphs.len() > tm.start {
                extracted_glyphs.push(ExtractedGlyphs {
                    glyphs: text_layout_info.glyphs[start..end].to_vec(),
                    text_mod: tm.text_mod,
                    root: entity,
                    start,
                });
            }
        }

        let mut index = 0;
        text_layout_info.glyphs.retain(|_| {
            let keep = !ranges.iter().any(|r| r.contains(&index));
            index += 1;
            keep
        });

        commands.entity(entity).insert(TextEffectInfo {
            atlas,
            extracted_glyphs,
        });
    }
}

#[derive(Component)]
pub struct UpdateGlyphPosition;

pub fn extract_effect_glyphs(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut text2d_query: Query<
        (
            Entity,
            &mut TextMaterialCache,
            &TextEffectInfo,
            &TextLayoutInfo,
            &ComputedTextBlock,
            &Anchor,
            &Children,
        ),
        With<TypeWriterSection>,
    >,
    text_styles: Query<(&TextFont, &TextColor)>,
    glyphs_to_render: Query<(Entity, &GlyphIndex), Without<Mesh2d>>,
    mut glyphs_to_update: Query<
        (Entity, &GlyphIndex, &mut Transform),
        (With<Mesh2d>, With<UpdateGlyphPosition>),
    >,
    mut wave_materials: ResMut<Assets<WaveMaterial>>,
    mut shake_materials: ResMut<Assets<ShakeMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    atlases: Res<Assets<TextureAtlasLayout>>,
    mut mesh_cache: ResMut<GlyphMeshCache>,
) {
    if text2d_query
        .iter()
        .filter(|q| !q.2.extracted_glyphs.is_empty())
        .count()
        == 0
    {
        return;
    }

    let scale_factor = windows
        .get_single()
        .map(|window| window.resolution.scale_factor())
        .unwrap_or(1.0);
    let scaling = Transform::from_scale(Vec2::splat(scale_factor.recip()).extend(1.));

    for (
        section_entity,
        mut material_cache,
        effect_info,
        text_layout_info,
        computed_block,
        anchor,
        children,
    ) in text2d_query.iter_mut()
    {
        if effect_info.extracted_glyphs.is_empty() {
            continue;
        }

        let texture = effect_info.atlas.clone();
        let text_anchor = -(anchor.as_vec() + 0.5);
        let alignment_translation = text_layout_info.size * text_anchor;
        let transform = Transform::from_translation(alignment_translation.extend(0.)) * scaling;

        let mut current_span = usize::MAX;
        let mut color = LinearRgba::WHITE;

        for child in children.iter() {
            if let Ok((entity, index)) = glyphs_to_render.get(*child) {
                if let Some(extracted_glyphs) = effect_info.extracted_glyphs.iter().find(|g| {
                    g.root == section_entity
                        && (g.start..g.start + g.glyphs.len()).contains(&index.0)
                }) {
                    let glyph = extracted_glyphs
                        .glyphs
                        .get(index.0 - extracted_glyphs.start)
                        .unwrap();

                    if glyph.span_index != current_span {
                        color = text_styles
                            .get(
                                computed_block
                                    .entities()
                                    .get(glyph.span_index)
                                    .map(|t| t.entity)
                                    .unwrap_or(Entity::PLACEHOLDER),
                            )
                            .map(|(_, text_color)| LinearRgba::from(text_color.0))
                            .unwrap_or_default();
                        current_span = glyph.span_index;
                    }

                    let mut entity_commands = commands.entity(entity);
                    entity_commands.insert((
                        Mesh2d(mesh_cache.create_or_retrieve_mesh(
                            glyph,
                            &color,
                            &mut meshes,
                            &atlases,
                        )),
                        transform * Transform::from_translation(glyph.position.extend(0.)),
                    ));

                    match extracted_glyphs.text_mod {
                        TextMod::Wave => {
                            entity_commands.insert(TextMeshMaterial2d(material_cache.wave(
                                section_entity,
                                texture.clone(),
                                &mut wave_materials,
                            )));
                        }
                        TextMod::Shake(intensity) => {
                            entity_commands.insert(TextMeshMaterial2d(material_cache.shake(
                                section_entity,
                                intensity,
                                texture.clone(),
                                &mut shake_materials,
                            )));
                        }
                        _ => unimplemented!(),
                    }
                }
            } else if let Ok((entity, index, mut t)) = glyphs_to_update.get_mut(*child) {
                if let Some(extracted_glyphs) = effect_info.extracted_glyphs.iter().find(|g| {
                    g.root == section_entity
                        && (g.start..g.start + g.glyphs.len()).contains(&index.0)
                }) {
                    let glyph = extracted_glyphs
                        .glyphs
                        .get(index.0 - extracted_glyphs.start)
                        .unwrap();

                    *t = transform * Transform::from_translation(glyph.position.extend(0.));
                    commands.entity(entity).remove::<UpdateGlyphPosition>();
                }
            }
        }
    }
}
