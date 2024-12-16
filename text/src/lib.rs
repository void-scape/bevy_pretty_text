use bevy::color::LinearRgba;
use bevy::prelude::Component;
use std::borrow::Cow;

mod parse;
pub use parse::parse_section;

#[derive(Debug, Default, Clone, Component)]
pub struct TextSection {
    pub text: Text,
    pub commands: Vec<IndexedCommand>,
}

impl TextSection {
    pub fn from_sections(sections: Vec<Self>) -> Self {
        let mut slf = Self::default();
        let mut index = 0;
        for section in sections.into_iter() {
            let section_len = section.text.value.len();
            slf.text.append(section.text);
            slf.commands
                .extend(section.commands.into_iter().map(|mut c| {
                    c.index += index;
                    c
                }));

            index += section_len;
        }
        slf
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IndexedCommand {
    pub index: usize,
    pub command: TypeWriterCommand,
}

#[derive(Debug, Clone, Copy)]
pub enum TypeWriterCommand {
    Clear,
    AwaitClear,
    ClearAfter(f32),
    /// Relative speed
    Speed(f32),
    Pause(f32),
    Delete(usize),
}

/// String with a collection of modifiers.
#[derive(Debug, Default, Clone)]
pub struct Text {
    pub value: Cow<'static, str>,
    pub modifiers: Vec<IndexedTextMod>,
}

impl Text {
    pub fn from_value(value: String) -> Self {
        Self {
            value: Cow::Owned(value),
            modifiers: Vec::new(),
        }
    }

    pub fn append(&mut self, other: Self) {
        let len = self.value.len();
        self.value.to_mut().push_str(&other.value);
        self.modifiers
            .extend(other.modifiers.into_iter().map(|mut m| {
                m.start += len;
                m.end += len;
                m
            }));
    }
}

impl From<String> for Text {
    fn from(value: String) -> Self {
        Self {
            value: Cow::Owned(value),
            modifiers: Vec::new(),
        }
    }
}

impl From<&'static str> for Text {
    fn from(value: &'static str) -> Self {
        Self {
            value: Cow::Borrowed(value),
            modifiers: Vec::new(),
        }
    }
}

/// Text modifier that applies to a [`Text`] section.
#[derive(Debug, Clone)]
pub struct IndexedTextMod {
    pub start: usize,
    /// Non inclusive
    pub end: usize,
    pub text_mod: TextMod,
}

/// Modifies visual qualities of [`Text`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextMod {
    Color(LinearRgba),
    Wave,
    /// With intensity 0.0..1.0
    Shake(f32),
}

impl TextMod {
    pub fn is_shader_effect(&self) -> bool {
        match self {
            Self::Wave => true,
            Self::Shake(_) => true,
            Self::Color(_) => false,
        }
    }

    pub fn color(&self) -> Option<LinearRgba> {
        match self {
            Self::Color(color) => Some(*color),
            _ => None,
        }
    }
}
