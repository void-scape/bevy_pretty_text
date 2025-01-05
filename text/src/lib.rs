use bevy::color::LinearRgba;
use bevy::prelude::Component;
use material::InsertTextMaterial2d;
use std::borrow::Cow;
use std::fmt::Debug;

pub mod material;
mod parse;

pub use parse::*;

#[derive(Debug, Default, Component)]
pub struct TextSection {
    pub text: Text,
    pub commands: Vec<IndexedCommand>,
    pub end: Option<TypeWriterCommand>,
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

    pub fn deduplicate_spaces(&mut self) {
        let mut indices = Vec::new();
        let mut prev_space = false;

        for (i, char) in self.text.value.chars().enumerate() {
            let is_space = char == ' ';
            if is_space && prev_space {
                indices.push(i);
            }
            prev_space = is_space;
        }

        for index in indices.iter().rev() {
            self.text.value.to_mut().remove(*index);
            for text_mod in self.text.modifiers.iter_mut() {
                if text_mod.start >= *index {
                    text_mod.start = text_mod.start.saturating_sub(1);
                }

                if text_mod.end >= *index {
                    text_mod.end = text_mod.end.saturating_sub(1);
                }
            }

            for command in self.commands.iter_mut() {
                if command.index >= *index {
                    command.index = command.index.saturating_sub(1);
                }
            }
        }
    }
}

impl From<String> for TextSection {
    fn from(value: String) -> Self {
        Self {
            text: Text::from(value),
            commands: Vec::new(),
            end: None,
        }
    }
}

impl From<Text> for TextSection {
    fn from(value: Text) -> Self {
        Self {
            text: value,
            commands: Vec::new(),
            end: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IndexedCommand {
    pub index: usize,
    pub command: TypeWriterCommand,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TypeWriterCommand {
    //Clear,
    AwaitClear,
    //ClearAfter(f32),
    /// Relative speed
    Speed(f32),
    Pause(f32),
    Delete(usize),
}

/// String with a collection of modifiers.
#[derive(Debug, Default)]
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
pub enum TextMod {
    Color(LinearRgba),
    Shader(Box<dyn InsertTextMaterial2d + Send + Sync>),
    ShaderStruct(String),
}

impl Debug for TextMod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Color(color) => f.write_str(&format!("TextMod::Color({color:?})")),
            Self::Shader(_) => f.write_str("TextMod::Shader( .. )"),
            Self::ShaderStruct(s) => f.write_str(&format!("TextMod::ShaderStruct({s})")),
        }
    }
}

impl Clone for TextMod {
    fn clone(&self) -> Self {
        match self {
            Self::Color(color) => Self::Color(*color),
            Self::ShaderStruct(s) => Self::ShaderStruct(s.clone()),
            Self::Shader(s) => s.clone_mod(),
        }
    }
}

impl TextMod {
    pub fn is_shader_effect(&self) -> bool {
        match self {
            Self::Shader(_) => true,
            Self::ShaderStruct(_) => false,
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
