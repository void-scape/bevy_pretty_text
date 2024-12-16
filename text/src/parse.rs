use super::TextMod;
use crate::{IndexedCommand, IndexedTextMod, Text, TextSection, TypeWriterCommand};
use bevy::color::LinearRgba;
use std::borrow::Cow;
use winnow::{
    ascii::float,
    combinator::{alt, delimited, opt, peek, terminated},
    stream::Location,
    token::{any, take_till, take_while},
    Located, PResult, Parser,
};

// TODO: recursive effects, e.g. "``Hello|green`, World`[wave]"
pub fn parse_section(input: &str) -> PResult<TextSection> {
    let sections = parse_text(&mut Located::new(input), &mut 0)?;
    let mut section = TextSection::from_sections(sections);
    process_section(&mut section);

    Ok(section)
}

impl From<String> for TextSection {
    fn from(value: String) -> Self {
        Self {
            text: Text::from(value),
            commands: Vec::new(),
        }
    }
}

impl From<Text> for TextSection {
    fn from(value: Text) -> Self {
        Self {
            text: value,
            commands: Vec::new(),
        }
    }
}

fn parse_speed(input: &mut Located<&str>) -> PResult<f32> {
    delimited('<', float, '>').parse_next(input)
}

fn parse_pause(input: &mut Located<&str>) -> PResult<f32> {
    delimited('[', float, ']').parse_next(input)
}

fn parse_effect(input: &mut Located<&str>) -> PResult<TextMod> {
    alt((
        "wave".map(|_| TextMod::Wave),
        "shake".map(|_| TextMod::Shake(0.5)),
    ))
    .parse_next(input)
}

fn parse_color(input: &mut Located<&str>) -> PResult<TextMod> {
    alt((
        "red".map(|_| TextMod::Color(LinearRgba::RED)),
        "green".map(|_| TextMod::Color(LinearRgba::GREEN)),
        "blue".map(|_| TextMod::Color(LinearRgba::BLUE)),
    ))
    .parse_next(input)
}

fn parse_ticks(input: &mut Located<&str>) -> PResult<Text> {
    '`'.parse_next(input)?;
    let text = take_till(0.., ['|', '`']).parse_next(input)?;

    let color = match any.parse_next(input)? {
        '|' => {
            let color = terminated(parse_color, '`').parse_next(input)?;
            Some(color)
        }
        _ => None,
    };

    let effect = opt(delimited('[', parse_effect, ']')).parse_next(input)?;

    let mut modifiers = Vec::new();

    if let Some(color) = color {
        modifiers.push(IndexedTextMod {
            start: 0,
            end: text.len(),
            text_mod: color,
        })
    }

    if let Some(effect) = effect {
        modifiers.push(IndexedTextMod {
            start: 0,
            end: text.len(),
            text_mod: effect,
        })
    }

    Ok(Text {
        value: Cow::Owned(text.into()),
        modifiers,
    })
}

fn parse_text(input: &mut Located<&str>, accumulator: &mut usize) -> PResult<Vec<TextSection>> {
    let mut result = Vec::new();

    while let Ok(text) = parse_normal(input) {
        if !text.is_empty() {
            result.push(TextSection::from(text.to_owned()));
        }

        if let Some(t) = peek(any::<_, ()>).parse_next(input).ok() {
            match t {
                '<' => {
                    let index = input.location() - *accumulator;
                    let speed = parse_speed(input)?;
                    let indexed_command = IndexedCommand {
                        index,
                        command: TypeWriterCommand::Speed(speed),
                    };
                    *accumulator += input.location() - *accumulator - index;

                    if let Some(first) = result.first_mut() {
                        first.commands.push(indexed_command);
                    } else {
                        result.push(TextSection {
                            text: Default::default(),
                            commands: vec![indexed_command],
                        })
                    }
                }
                '[' => {
                    let index = input.location() - *accumulator;
                    let duration = parse_pause(input)?;
                    let first = result.first_mut().unwrap();
                    first.commands.push(IndexedCommand {
                        index,
                        command: TypeWriterCommand::Pause(duration),
                    });
                    *accumulator += input.location() - *accumulator - index;
                }
                '`' => {
                    let index = input.location() - *accumulator;
                    let section = TextSection::from(parse_ticks.parse_next(input)?);
                    *accumulator +=
                        input.location() - *accumulator - index - section.text.value.len() + 1;
                    result.push(section);
                }
                '{' => {
                    unimplemented!();
                    //any.parse_next(input)?;
                    //result.extend(parse_text(input, accumulator)?);
                }
                _ => {
                    any.parse_next(input)?;
                    break;
                }
            }
        } else {
            break;
        }
    }

    Ok(result)
}

fn process_section(section: &mut TextSection) {
    let mut prev_was_space = false;
    let mut remove_indicies = Vec::with_capacity(16);

    for (i, char) in section.text.value.chars().enumerate() {
        let is_space = char == ' ';
        if prev_was_space && is_space {
            remove_indicies.push(i);
        }
        prev_was_space = is_space;
    }

    if section.text.value.chars().next().is_some_and(|c| c == ' ') && !remove_indicies.contains(&0)
    {
        remove_indicies.insert(0, 0);
    }

    for index in remove_indicies.iter().rev() {
        section.text.value.to_mut().remove(*index);

        for modifier in section.text.modifiers.iter_mut() {
            if modifier.start >= *index {
                modifier.start = modifier.start.saturating_sub(1);
                modifier.end = modifier.end.saturating_sub(1);
            }
        }

        for command in section.commands.iter_mut() {
            if command.index >= *index {
                command.index = command.index.saturating_sub(1);
            }
        }
    }
}

fn parse_normal<'a>(input: &mut Located<&'a str>) -> PResult<&'a str> {
    take_while(0.., |c| !['[', '<', '`', '{', '}'].contains(&c)).parse_next(input)
}

#[cfg(feature = "proc-macro")]
use quote::{quote, TokenStreamExt};

#[cfg(feature = "proc-macro")]
impl quote::ToTokens for &'_ TextMod {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_all(quote! { bevy_pretty_text::text::TextMod:: });
        tokens.append_all(match self {
            TextMod::Wave => quote! { Wave },
            TextMod::Shake(s) => quote! { Shake(#s) },
            TextMod::Color(c) => {
                let r = c.red;
                let g = c.green;
                let b = c.blue;
                let a = c.alpha;

                quote! {
                    Color(bevy::color::LinearRgba {
                        red: #r,
                        green: #g,
                        blue: #b,
                        alpha: #a,
                    })
                }
            }
        });
    }
}

#[cfg(feature = "proc-macro")]
impl quote::ToTokens for IndexedTextMod {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let start = self.start;
        let end = self.end;
        let text_mod = &self.text_mod;

        tokens.append_all(quote! {
            bevy_pretty_text::text::IndexedTextMod {
                start: #start,
                end: #end,
                text_mod: #text_mod,
            }
        });
    }
}

#[cfg(feature = "proc-macro")]
impl quote::ToTokens for TypeWriterCommand {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_all(quote! { bevy_pretty_text::text::TypeWriterCommand:: });
        tokens.append_all(match self {
            TypeWriterCommand::Clear => quote! { Clear },
            TypeWriterCommand::AwaitClear => quote! { AwaitClear },
            TypeWriterCommand::ClearAfter(d) => quote! { ClearAfter(#d) },
            TypeWriterCommand::Speed(s) => quote! { Speed(#s) },
            TypeWriterCommand::Pause(d) => quote! { Pause(#d) },
            TypeWriterCommand::Delete(n) => quote! { Delete(#n) },
        });
    }
}

#[cfg(feature = "proc-macro")]
impl quote::ToTokens for IndexedCommand {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let index = self.index;
        let command = &self.command;

        tokens.append_all(quote! {
            bevy_pretty_text::text::IndexedCommand {
                index: #index,
                command: #command,
            }
        });
    }
}

/// The [`TextSection`] will get implicitly converted into the TypeWriterSection from the root
/// crate. This will convert all of the section's lifetimes into `static.
#[cfg(feature = "proc-macro")]
impl quote::ToTokens for TextSection {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let text = self.text.value.as_ref();
        let mods = self.text.modifiers.iter().map(|m| {
            quote! {
                #m
            }
        });
        let commands = self.commands.iter().map(|c| {
            quote! {
                #c
            }
        });

        tokens.append_all(quote! {
            bevy_pretty_text::type_writer::section::TypeWriterSection {
                text: bevy_pretty_text::type_writer::section::TwText {
                    value: std::borrow::Cow::Borrowed(#text),
                    modifiers: &[#(#mods),*],
                },
                commands: &[#(#commands),*],
            }
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_simple() {
        let input = "Hello, world!";
        let output = parse_section(input).unwrap();

        assert!(output.text.modifiers.is_empty());
        assert!(output.commands.is_empty());
        assert_eq!(input.len(), output.text.value.len());
    }

    #[test]
    fn test_effects_and_commands() {
        let input = "<0.3> Hello, <0.5> `world|red`[wave]!";
        let output = parse_section(input).unwrap();

        assert_eq!("Hello, world!", &output.text.value);
        assert!(matches!(
            output.text.modifiers.as_slice(),
            &[
                IndexedTextMod {
                    start: 7,
                    end: 12,
                    text_mod: TextMod::Color(LinearRgba::RED),
                },
                IndexedTextMod {
                    start: 7,
                    end: 12,
                    text_mod: TextMod::Wave,
                },
            ]
        ));
        assert!(matches!(
            output.commands.as_slice(),
            &[
                IndexedCommand {
                    index: 0,
                    command: TypeWriterCommand::Speed(0.3),
                },
                IndexedCommand {
                    index: 6,
                    command: TypeWriterCommand::Speed(0.5),
                }
            ]
        ));
    }
}
