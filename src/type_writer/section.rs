use crate::effect::UpdateGlyphPosition;
use crate::materials::TextMaterialCache;
use bevy::prelude::*;
use bevy::text::{TextBounds, TextLayoutInfo};
use std::borrow::Cow;
use std::ops::Range;
use text::IndexedTextMod;

/// Primitive for describing how text should be rendered.
///
/// Use a [`SectionSlice`] to draw a slice of `text` to the screen.
#[derive(Debug, Default, Clone, Component)]
#[require(Text2d, TextMaterialCache)]
pub struct TypeWriterSection {
    pub text: TwText,
    pub commands: &'static [text::IndexedCommand],
}

impl TypeWriterSection {
    pub fn len(&self) -> usize {
        self.text.value.len()
    }

    pub fn is_empty(&self) -> bool {
        self.text.value.is_empty()
    }
}

#[derive(Debug, Default, Clone)]
pub struct TwText {
    pub value: Cow<'static, str>,
    pub modifiers: &'static [IndexedTextMod],
}

/// Renders a slice of a [`TypeWriterSection`].
#[derive(Debug, Default, Clone, Component)]
pub enum SectionSlice {
    #[default]
    None,
    Range(Range<usize>),
    All,
}

impl SectionSlice {
    pub fn from_range(range: Range<usize>) -> Self {
        Self::Range(range)
    }

    pub fn range(&self, max: usize) -> Range<usize> {
        match self {
            Self::None => 0..0,
            Self::All => 0..max,
            Self::Range(range) => range.clone(),
        }
    }
}

pub fn update_section_slice(
    mut type_writers: Query<(&mut SectionSlice, &TypeWriterIndex), Changed<TypeWriterIndex>>,
) {
    for (mut slice, TypeWriterIndex(index)) in type_writers.iter_mut() {
        *slice = SectionSlice::Range(0..*index);
    }
}

#[derive(Debug, Default, Clone, Copy, Component)]
pub struct TypeWriterIndex(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct GlyphIndex(pub usize);

pub fn update_section_slice_glyph_indices(
    mut commands: Commands,
    sections: Query<
        (Entity, &TypeWriterSection, &SectionSlice, Option<&Children>),
        Changed<SectionSlice>,
    >,
    fonts: Query<&TextFont>,
    glyphs: Query<(Entity, &GlyphIndex)>,
    spans: Query<Entity, With<TextSpan>>,
) {
    for (section_entity, section, slice, children) in sections.iter() {
        let Ok(font) = fonts.get(section_entity) else {
            continue;
        };

        let range = slice.range(section.text.value.len());
        let mut new_glyphs = range.clone().into_iter().collect::<Vec<_>>();
        let mut retained_glyphs = Vec::with_capacity(range.len());

        if let Some(children) = children {
            for child in children.iter() {
                if let Ok((entity, index)) = glyphs.get(*child) {
                    if range.contains(&index.0) {
                        retained_glyphs.push(index.0);
                        commands.entity(entity).insert(UpdateGlyphPosition);
                    } else {
                        commands.entity(entity).despawn_recursive();
                    }
                }
            }

            retained_glyphs.sort();
            retained_glyphs.iter().rev().for_each(|i| {
                new_glyphs.remove(*i);
            });
        }

        let ranges = section
            .text
            .modifiers
            .iter()
            .filter_map(|m| m.text_mod.is_shader_effect().then(|| m.start..m.end))
            .collect::<Vec<_>>();

        let mut range = range.clone();
        range.end = range.end.min(section.len());

        new_glyphs.into_iter().map(|i| GlyphIndex(i)).for_each(|g| {
            if ranges.iter().any(|r| r.contains(&g.0)) {
                commands.entity(section_entity).with_child(g);
            }
        });

        if let Some(children) = children {
            for child in children.iter() {
                if let Ok(entity) = spans.get(*child) {
                    commands.entity(entity).despawn_recursive();
                }
            }
        }

        let mut current_entity = section_entity;
        let mut current_index = 0;
        for span in section
            .text
            .modifiers
            .iter()
            .filter_map(|m| m.text_mod.color().and_then(|c| Some((m.start, m.end, c))))
        {
            let color = TextColor(span.2.into());
            let start = span.0.min(range.end);
            let end = span.1.min(section.len()).min(range.end);

            // TODO: I have looking at this.
            if start > current_index {
                let span = commands
                    .spawn((
                        font.clone(),
                        TextSpan(section.text.value[current_index..start].to_owned()),
                    ))
                    .id();
                commands.entity(current_entity).add_child(span);
                current_entity = span;
                let span = commands
                    .spawn((
                        font.clone(),
                        color,
                        TextSpan(section.text.value[start..end].to_owned()),
                    ))
                    .id();
                commands.entity(current_entity).add_child(span);
                current_entity = span;

                current_index = end;
            } else if start == current_index {
                let span = TextSpan(section.text.value[start..end].to_owned());
                let span = commands.spawn((font.clone(), color, span)).id();
                commands.entity(current_entity).add_child(span);
                current_entity = span;

                current_index = end;
            } else {
                warn!("text color mod extends into previous span");
                let span = commands
                    .spawn((
                        font.clone(),
                        color,
                        TextSpan(section.text.value[current_index..end].to_owned()),
                    ))
                    .id();
                commands.entity(current_entity).add_child(span);
                current_entity = span;

                current_index = end;
            }
        }

        if current_index < range.end {
            commands.entity(current_entity).with_child((
                font.clone(),
                TextSpan(section.text.value[current_index..range.end].to_string()),
            ));
        }
    }
}

#[allow(unused)]
pub fn debug_sections(
    sections: Query<
        (&TextLayoutInfo, &TextBounds, &TextFont),
        (With<TypeWriterSection>, Changed<TextLayoutInfo>),
    >,
) {
    for section in sections.iter() {
        //println!("{section:#?}");
    }
}
