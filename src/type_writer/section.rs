use crate::effect::{GlyphIndex, UpdateGlyphPosition};
use crate::materials::TextMaterialCache;
use bevy::prelude::*;
use std::borrow::Cow;
use std::ops::Range;
use text::{IndexedTextMod, TypeWriterCommand};

/// Primitive for describing how text should be rendered.
///
/// Use a [`SectionSlice`] to draw a slice of `text` to the screen.
#[derive(Debug, Default, Clone, Component)]
#[require(Text2d, TextMaterialCache)]
pub struct TypeWriterSection {
    pub text: TwText,
    pub commands: Cow<'static, [text::IndexedCommand]>,
    pub end: Option<TypeWriterCommand>,
}

impl TypeWriterSection {
    pub fn new(text: TwText) -> Self {
        Self {
            text,
            commands: Cow::Borrowed(&[]),
            end: Some(TypeWriterCommand::AwaitClear),
        }
    }

    pub fn len(&self) -> usize {
        self.text.value.len()
    }

    pub fn is_empty(&self) -> bool {
        self.text.value.is_empty()
    }

    pub fn join(&mut self, other: &Self) {
        self.end = other.end;
        let len = self.text.value.len();
        if !other.commands.is_empty() {
            self.commands
                .to_mut()
                .extend(other.commands.iter().cloned().map(|mut c| {
                    c.index += len;
                    c
                }));
        }
        if !other.text.modifiers.is_empty() {
            self.text
                .modifiers
                .to_mut()
                .extend(other.text.modifiers.iter().cloned().map(|mut m| {
                    m.start += len;
                    m.end += len;
                    m
                }));
        }
        if !other.text.value.is_empty() {
            self.text.value.to_mut().push_str(other.text.value.as_ref());
        }
    }
}

impl From<&'static str> for TypeWriterSection {
    fn from(value: &'static str) -> Self {
        Self {
            text: TwText::from(value),
            commands: Cow::Borrowed(&[]),
            end: Some(TypeWriterCommand::AwaitClear),
        }
    }
}

impl From<String> for TypeWriterSection {
    fn from(value: String) -> Self {
        Self {
            text: TwText::from(value),
            commands: Cow::Borrowed(&[]),
            end: Some(TypeWriterCommand::AwaitClear),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct TwText {
    pub value: Cow<'static, str>,
    pub modifiers: Cow<'static, [IndexedTextMod]>,
}

impl From<&'static str> for TwText {
    fn from(value: &'static str) -> Self {
        Self {
            value: Cow::Borrowed(value),
            modifiers: Cow::Borrowed(&[]),
        }
    }
}

impl From<String> for TwText {
    fn from(value: String) -> Self {
        Self {
            value: Cow::Owned(value),
            modifiers: Cow::Borrowed(&[]),
        }
    }
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

/// Sets the [`SectionSlice`] as a range from 0..`index`.
#[derive(Debug, Default, Clone, Copy, Component)]
pub struct TypeWriterIndex(pub usize);

pub fn update_section_slice_glyph_indices(
    mut commands: Commands,
    sections: Query<
        (Entity, &TypeWriterSection, &SectionSlice, Option<&Children>),
        Or<(Changed<SectionSlice>, Changed<TypeWriterSection>)>,
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
                if let Ok((entity, index)) = glyphs.get(child) {
                    if range.contains(&index.0) {
                        retained_glyphs.push(index.0);
                        commands.entity(entity).insert(UpdateGlyphPosition);
                    } else {
                        commands.entity(entity).despawn();
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
                if let Ok(entity) = spans.get(child) {
                    commands.entity(entity).despawn();
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
