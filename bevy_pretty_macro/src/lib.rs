use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, Parser};
use syn::spanned::Spanned;
use syn::{parse_macro_input, DeriveInput, Error, Result};

/// Inserts a `texture` field into the struct:
///
/// ```
///  MyStruct {
///    #[texture(0)]
///    #[sampler(1)]
///    pub texture: bevy::prelude::Handle<bevy::prelude::Image>
///    ...
///  }
/// ```
#[proc_macro_attribute]
pub fn text_shader(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    insert_atlas_texture(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

fn insert_atlas_texture(mut input: DeriveInput) -> Result<proc_macro2::TokenStream> {
    match &mut input.data {
        syn::Data::Struct(ref mut data) => match &mut data.fields {
            syn::Fields::Named(fields) => {
                fields.named.push(syn::Field::parse_named.parse2(quote! {
                    #[texture(0)]
                    #[sampler(1)]
                    pub texture: bevy::prelude::Handle<bevy::prelude::Image>
                })?)
            }
            fields => return Err(syn::Error::new_spanned(fields, "fields must be named")),
        },
        _ => {
            return Err(syn::Error::new_spanned(
                input.ident,
                "type must be a struct",
            ))
        }
    }
    let name = &input.ident;

    Ok(quote! {
        #input

        impl ::text::material::SetAtlasTexture for #name {
            fn set_texture(&mut self, texture: Handle<Image>) {
                self.texture = texture;
            }
        }
    })
}

#[proc_macro]
pub fn s(input: TokenStream) -> TokenStream {
    section(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn section(input: TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let Dialogue {
        string,
        expressions,
    } = syn::parse(input)?;

    let closures = expressions
        .iter()
        .filter_map(|e| parse_closure(e).expect("invalid closure"))
        .collect::<Vec<_>>();

    let input = string.value();
    text::parse_text(&mut input.as_str(), &mut Default::default())
        .map(|t| {
            t.token_stream(&closures).ok_or_else(|| {
                syn::Error::new(
                    string.span(),
                    String::from("Wrong number of closures supplied"),
                )
            })
        })
        .map_err(|e| syn::Error::new(string.span(), e.to_string()))?
}

fn parse_closure(expr: &syn::Expr) -> syn::Result<Option<(&syn::Ident, &syn::Expr)>> {
    match expr {
        syn::Expr::Closure(closure) => {
            if closure.inputs.len() != 1 {
                return Err(syn::Error::new(
                    closure.inputs.span(),
                    "Expected a closure with exactly one input",
                ));
            }

            let name = closure.inputs.iter().next().unwrap();
            let name = match name {
                syn::Pat::Ident(ident) => &ident.ident,
                n => return Err(syn::Error::new(n.span(), "Expected a simple identifier")),
            };

            Ok(Some((name, closure.body.as_ref())))
        }
        _ => Ok(None),
    }
}

struct Dialogue {
    string: syn::LitStr,
    expressions: syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>,
}

impl Parse for Dialogue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let string = input.parse()?;

        if let Ok(_) = <syn::Token![,] as Parse>::parse(input) {
            let expressions =
                syn::punctuated::Punctuated::<_, syn::Token![,]>::parse_terminated_with(
                    input,
                    syn::Expr::parse,
                )?;

            Ok(Self {
                string,
                expressions,
            })
        } else {
            Ok(Self {
                string,
                expressions: Default::default(),
            })
        }
    }
}
