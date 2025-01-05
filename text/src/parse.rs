#![allow(unused)]

use super::TextMod;
use crate::TypeWriterCommand;
use bevy::color::LinearRgba;
use std::fmt::Debug;
use winnow::{
    ascii::float,
    combinator::{alt, delimited, opt, peek, terminated},
    token::{any, take_till, take_while},
    PResult, Parser,
};

fn parse_speed(input: &mut &str) -> PResult<f32> {
    delimited('<', float, '>').parse_next(input)
}

fn parse_pause(input: &mut &str) -> PResult<f32> {
    delimited('[', float, ']').parse_next(input)
}

fn parse_color(input: &mut &str) -> PResult<TextMod> {
    alt((
        "red".map(|_| TextMod::Color(LinearRgba::RED)),
        "green".map(|_| TextMod::Color(LinearRgba::GREEN)),
        "blue".map(|_| TextMod::Color(LinearRgba::BLUE)),
    ))
    .parse_next(input)
}

fn parse_ticks(input: &mut &str) -> PResult<Token> {
    '`'.parse_next(input)?;
    let text = take_till(0.., ['|', '`']).parse_next(input)?;
    let mut modifiers = Vec::new();

    #[allow(clippy::single_match)]
    match any.parse_next(input)? {
        '|' => {
            let color = terminated(parse_color, '`').parse_next(input)?;
            modifiers.push(color);
        }
        _ => {}
    }

    if let Some(effect) =
        opt(delimited('[', take_while(.., |c| c != ']'), ']')).parse_next(input)?
    {
        modifiers.push(TextMod::ShaderStruct(effect.to_string()));
    }

    Ok(Token::Special {
        value: text.to_owned(),
        modifiers,
    })
}

fn parse_normal<'a>(input: &mut &'a str) -> PResult<&'a str> {
    take_while(0.., |c| !['[', '<', '`', '{', '}'].contains(&c)).parse_next(input)
}

#[derive(Debug)]
pub enum Token {
    Normal(String),
    Special {
        value: String,
        modifiers: Vec<TextMod>,
    },
    Command(TypeWriterCommand),
    Section(ParsedSection),
}

impl Token {
    pub fn append_command(&mut self, command: TypeWriterCommand) {
        if let Self::Section(s) = self {
            s.tokens.push(Self::Command(command));
        }
    }
}

impl From<String> for Token {
    fn from(value: String) -> Self {
        Self::Normal(value)
    }
}

impl From<&'static str> for Token {
    fn from(value: &'static str) -> Self {
        Self::Normal(value.to_owned())
    }
}

#[derive(Debug, Default)]
pub struct ParsedSection {
    tokens: Vec<Token>,
    closure_info: Option<ClosureInfo>,
}

impl ParsedSection {
    pub fn new(tokens: Vec<Token>, closure_info: ClosureInfo) -> Self {
        if closure_info.depth == 0 {
            Self {
                tokens,
                closure_info: None,
            }
        } else {
            Self {
                tokens,
                closure_info: Some(closure_info),
            }
        }
    }
}

#[derive(Debug)]
pub struct ClosureInfo {
    closure_index: usize,
    depth: usize,
}

impl ClosureInfo {
    fn from_context(value: &ClosureContext) -> Self {
        Self {
            closure_index: value.visited,
            depth: value.depth,
        }
    }
}

#[derive(Debug, Default)]
pub struct ClosureContext {
    visited: usize,
    depth: usize,
}

pub fn parse_text(input: &mut &str, closure_context: &mut ClosureContext) -> PResult<Token> {
    let mut tokens = Vec::new();
    let info = ClosureInfo::from_context(closure_context);

    while let Ok(text) = parse_normal(input) {
        if !text.is_empty() {
            tokens.push(Token::Normal(text.to_owned()));
        }

        if let Ok(t) = peek(any::<_, ()>).parse_next(input) {
            match t {
                '<' => {
                    let speed = parse_speed(input)?;
                    tokens.push(Token::Command(TypeWriterCommand::Speed(speed)));
                }
                '[' => {
                    let duration = parse_pause(input)?;
                    tokens.push(Token::Command(TypeWriterCommand::Pause(duration)));
                }
                '`' => {
                    tokens.push(parse_ticks(input)?);
                }
                '{' => {
                    any.parse_next(input)?;
                    closure_context.depth += 1;
                    closure_context.visited += 1;
                    tokens.push(parse_text(input, closure_context)?);
                    closure_context.depth -= 1;
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

    Ok(Token::Section(ParsedSection::new(tokens, info)))
}

#[cfg(feature = "proc-macro")]
pub mod proc_macro {
    use super::TextMod;
    use super::*;
    use crate::TypeWriterCommand;
    use crate::{IndexedCommand, IndexedTextMod, Text, TextSection};
    use quote::{format_ident, quote, TokenStreamExt};
    use std::borrow::Cow;
    use winnow::{
        combinator::{delimited, opt},
        token::take_while,
        Parser,
    };

    impl Token {
        pub fn token_stream(
            &self,
            closures: &[(&syn::Ident, &syn::Expr)],
        ) -> Option<proc_macro2::TokenStream> {
            token_to_tokens(self, &mut 0, &mut Vec::new(), closures)
        }
    }

    fn token_to_tokens(
        token: &Token,
        index: &mut usize,
        sub_tokens: &mut Vec<proc_macro2::TokenStream>,
        closures: &[(&syn::Ident, &syn::Expr)],
    ) -> Option<proc_macro2::TokenStream> {
        match token {
            Token::Section(section) => {
                let mut stream = proc_macro2::TokenStream::new();
                if section.tokens.is_empty() {
                    return Some(stream);
                }

                let mut sections = vec![TextSection::default()];
                for token in section.tokens.iter() {
                    match token {
                        Token::Section(_) => {
                            if !sections.is_empty() {
                                let mut new_section =
                                    TextSection::from_sections(std::mem::take(&mut sections));
                                new_section.deduplicate_spaces();

                                if let Some(closure_info) = &section.closure_info {
                                    match closures.get(&closure_info.closure_index - 1) {
                                        Some((ident, body)) => {
                                            sub_tokens.push(
                                                quote! { { let #ident = #new_section; #body } },
                                            );
                                        }
                                        None => return None,
                                    }
                                } else {
                                    sub_tokens.push(quote! { #new_section });
                                }
                            }

                            token_to_tokens(token, index, sub_tokens, closures)?;
                        }
                        Token::Normal(str) => {
                            *index += str.len();
                            sections.push(TextSection::from(str.clone()));
                        }
                        Token::Command(command) => {
                            let last = sections.last_mut().unwrap();
                            last.commands.push(IndexedCommand {
                                index: last.text.value.len(),
                                command: *command,
                            })
                        }
                        Token::Special { value, modifiers } => {
                            sections.push(TextSection::from(Text {
                                value: Cow::Owned(value.clone()),
                                modifiers: modifiers
                                    .iter()
                                    .map(|m| IndexedTextMod {
                                        start: 0,
                                        end: value.len(),
                                        text_mod: match m {
                                            TextMod::Color(color) => TextMod::Color(*color),
                                            TextMod::ShaderStruct(s) => {
                                                TextMod::ShaderStruct(s.clone())
                                            }
                                            _ => panic!(),
                                        },
                                    })
                                    .collect(),
                            }))
                        }
                    }
                }

                if !sections.is_empty() {
                    let mut new_section = TextSection::from_sections(sections);

                    // last section
                    if section.closure_info.is_none() {
                        new_section.end = Some(TypeWriterCommand::AwaitClear);
                    }

                    new_section.deduplicate_spaces();

                    if let Some(closure_info) = &section.closure_info {
                        match closures.get(&closure_info.closure_index - 1) {
                            Some((ident, body)) => {
                                sub_tokens.push(quote! { { let #ident = #new_section; #body } });
                            }
                            None => return None,
                        }
                    } else {
                        sub_tokens.push(quote! { #new_section });
                    }
                }

                stream.append_all(quote! { (#(#sub_tokens),*) });
                Some(stream)
            }
            _ => unreachable!(),
        }
    }

    impl quote::ToTokens for &'_ TextMod {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            tokens.append_all(quote! { bevy_pretty_text::text::TextMod:: });
            tokens.append_all(match self {
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
                TextMod::ShaderStruct(s) => {
                    parse_shader_mod(s)

                    // let mut input = s.as_str();
                    // let ident = take_while::<fn(char) -> bool, &str, ContextError>(.., |c| c != '(')
                    //     .parse_next(&mut input)
                    //     .unwrap();
                    // let ident = format_ident!("{ident}");
                    //
                    // if input.peek_token().is_some() {
                    //     quote! {
                    //         Shader(Box::new(#ident::default()))
                    //     }
                    // } else {
                    //     quote! {
                    //         Shader(Box::new(#ident::default()))
                    //     }
                    // }
                }
                _ => {
                    panic!("cannot parse a text mod shader");
                }
            });
        }
    }

    fn parse_shader_mod(s: &str) -> proc_macro2::TokenStream {
        use syn::{parse_str, Expr};
        use winnow::error::ContextError;

        let mut input = s;

        let ident: &str = take_while::<fn(char) -> bool, &str, ContextError>(.., |c: char| {
            c != '(' && !c.is_whitespace()
        })
        .parse_next(&mut input)
        .expect("Expected struct identifier");

        let ident = format_ident!("{ident}");

        let args_opt = opt::<&str, &str, ContextError, _>(delimited(
            '(',
            // winnow::combinator::separated(.., take_while(.., |c: char| c != ',' && c != ')'), ','),
            take_while(.., |c: char| c != ')'),
            ')',
        ))
        .parse_next(&mut input)
        .unwrap();

        if let Some(args_str) = args_opt {
            let expr: Expr = parse_str(args_str).unwrap();
            quote! {
                Shader(Box::new(#ident::new(#expr)))
            }
        } else {
            quote! {
                Shader(Box::new(#ident::default()))
            }
        }
    }

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

    impl quote::ToTokens for TypeWriterCommand {
        fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
            tokens.append_all(quote! { bevy_pretty_text::text::TypeWriterCommand:: });
            tokens.append_all(match self {
                //TypeWriterCommand::Clear => quote! { Clear },
                TypeWriterCommand::AwaitClear => quote! { AwaitClear },
                //TypeWriterCommand::ClearAfter(d) => quote! { ClearAfter(#d) },
                TypeWriterCommand::Speed(s) => quote! { Speed(#s) },
                TypeWriterCommand::Pause(d) => quote! { Pause(#d) },
                TypeWriterCommand::Delete(n) => quote! { Delete(#n) },
            });
        }
    }

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
            let end = match self.end {
                Some(end) => quote! { Some(#end) },
                None => quote! { None },
            };

            tokens.append_all(quote! {
                bevy_pretty_text::type_writer::section::TypeWriterSection {
                    text: bevy_pretty_text::type_writer::section::TwText {
                        value: std::borrow::Cow::Borrowed(#text),
                        modifiers: std::borrow::Cow::Owned(vec![#(#mods),*]),
                    },
                    commands: std::borrow::Cow::Borrowed(&[#(#commands),*]),
                    end: #end
                }
            });
        }
    }
}
