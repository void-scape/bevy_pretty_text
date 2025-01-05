use crate::{
    prelude::WrapPadding, render::mesh::GlyphMeshCache, type_writer::section::TypeWriterSection,
};
use bevy::{
    prelude::*,
    sprite::Anchor,
    text::{ComputedTextBlock, PositionedGlyph, TextLayoutInfo},
    utils::hashbrown::HashMap,
    window::PrimaryWindow,
};
use text::TextMod;

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
    /// Start of slice in [`TypeWriterSection`].
    pub start: usize,
    pub text_mod: TextMod,
    pub root: Entity,
}

#[derive(Debug, Clone, Copy, Component)]
pub struct GlyphIndex(pub usize);

/// Maps the [`TypeWriterSection`] text index to the corresponding [`TextLayoutInfo`] glyph index.
///
/// This is necessary for wrapping text, which removes the trailing space glyph on each line.
#[derive(Debug)]
struct IndexMappedGlyphs {
    glyphs: HashMap<usize, usize>,
}

impl IndexMappedGlyphs {
    pub fn new(text_layout_info: &TextLayoutInfo, font: &TextFont) -> Self {
        // It would be great to read from the cosmic buffer here, but it is not exposed by the
        // computed text block. This wastes a lot of compute for no reason.
        let mut ys = Vec::new();
        let mut glyph_hash = HashMap::<i64, Vec<i64>>::default();
        for glyph in text_layout_info.glyphs.iter() {
            if let Some(y) = ys
                .iter()
                .find(|y| (((**y - glyph.position.y) as i64).abs() as f32) < font.font_size)
            {
                let storage = glyph_hash.entry(*y as i64).or_insert_with(Vec::new);
                storage.push(glyph.position.x as i64);
            } else {
                ys.push(glyph.position.y);
                let storage = glyph_hash
                    .entry(glyph.position.y as i64)
                    .or_insert_with(Vec::new);
                storage.push(glyph.position.x as i64);
            }
        }

        let mut sorted_glyph_hash = glyph_hash.into_iter().collect::<Vec<_>>();
        sorted_glyph_hash.sort_by_key(|(y, _)| *y);
        sorted_glyph_hash.reverse();

        let mut glyphs = HashMap::default();
        if sorted_glyph_hash.len() == 1 {
            for i in 0..text_layout_info.glyphs.len() {
                glyphs.insert(i, i);
            }
        } else {
            let mut accum = 0;
            let mut current_hash = 0;
            for i in 0..text_layout_info.glyphs.len() + sorted_glyph_hash.len().saturating_sub(1) {
                let (_, hashes) = sorted_glyph_hash.get(current_hash).unwrap();
                if hashes.len() < accum {
                    accum = 0;
                    current_hash += 1;
                }

                glyphs.insert(i, i.saturating_sub(current_hash));
                accum += 1;
            }
        }

        Self { glyphs }
    }

    pub fn glyph_index(&self, type_writer_index: usize) -> Option<usize> {
        if type_writer_index == self.glyphs.len() {
            self.glyphs
                .get(&(type_writer_index - 1))
                .copied()
                .map(|i| i + 1)
        } else {
            self.glyphs.get(&type_writer_index).copied()
        }
    }
}

pub fn compute_info(
    mut commands: Commands,
    mut sections: Query<
        (
            Entity,
            &TypeWriterSection,
            &mut TextLayoutInfo,
            &WrapPadding,
            &TextFont,
        ),
        Changed<TextLayoutInfo>,
    >,
) {
    for (entity, section, mut text_layout_info, padding, font) in sections.iter_mut() {
        let Some(atlas) = text_layout_info
            .glyphs
            .iter()
            .map(|g| g.atlas_info.texture.clone())
            .next()
        else {
            continue;
        };

        let index_map = IndexMappedGlyphs::new(&text_layout_info, font);
        let mut extracted_glyphs = Vec::with_capacity(section.text.modifiers.len());
        let mut ranges = Vec::with_capacity(section.text.modifiers.len());

        for tm in section.text.modifiers.iter() {
            if !tm.text_mod.is_shader_effect() {
                continue;
            }

            match (
                index_map.glyph_index(tm.end),
                index_map.glyph_index(tm.start),
            ) {
                (Some(end), Some(start)) => {
                    let end = end.min(text_layout_info.glyphs.len() - padding.0);

                    if end > start {
                        ranges.push(start..end);
                        extracted_glyphs.push(ExtractedGlyphs {
                            glyphs: text_layout_info.glyphs[start..end].to_vec(),
                            text_mod: tm.text_mod.clone(),
                            root: entity,
                            start: tm.start,
                        });
                    }
                }
                (None, Some(start)) => {
                    let mut end = start;
                    let mut i = 1;
                    while let Some(next) = index_map.glyph_index(tm.start + i) {
                        i += 1;
                        end = next;
                    }

                    let end = end.min(text_layout_info.glyphs.len() - padding.0);

                    if end > start {
                        ranges.push(start..end);
                        extracted_glyphs.push(ExtractedGlyphs {
                            glyphs: text_layout_info.glyphs[start..end].to_vec(),
                            text_mod: tm.text_mod.clone(),
                            root: entity,
                            start: tm.start,
                        });
                    }
                }
                _ => {}
            }
        }

        let len = text_layout_info.glyphs.len();
        let padding_range = len - padding.0..len;

        let mut index = 0;
        text_layout_info.glyphs.retain(|_| {
            let keep =
                !padding_range.contains(&index) && !ranges.iter().any(|r| r.contains(&index));
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

#[derive(Component)]
pub struct TextEffectAtlas(Handle<Image>);

impl TextEffectAtlas {
    pub fn handle(&self) -> Handle<Image> {
        self.0.clone()
    }
}

#[allow(clippy::too_many_arguments)]
pub fn extract_effect_glyphs(
    mut commands: Commands,
    windows: Query<&Window, With<PrimaryWindow>>,
    text2d_query: Query<
        (
            Entity,
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
    mut meshes: ResMut<Assets<Mesh>>,
    atlases: Res<Assets<TextureAtlasLayout>>,
    mut mesh_cache: ResMut<GlyphMeshCache>,
) {
    if text2d_query
        .iter()
        .filter(|q| !q.1.extracted_glyphs.is_empty())
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

    for (section_entity, effect_info, text_layout_info, computed_block, anchor, children) in
        text2d_query.iter()
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

                    match &extracted_glyphs.text_mod {
                        TextMod::Shader(shader) => {
                            shader.insert_text_material_2d(&mut entity_commands);
                            entity_commands.insert(TextEffectAtlas(texture.clone()));
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
