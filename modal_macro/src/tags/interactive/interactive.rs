use crate::misc::AttrValue;
use crate::tags::*;

use proc_macro2::Span;
use syn::LitBool;
use syn::{parse::Parse, Ident, Token};

pub struct InteractiveTag {
    pub struct_name: Ident,
    pub rows: Vec<RowTag>,
    pub embeds: Vec<EmbedTag>,
    pub handler_name: Ident,
    pub ephemeral: LitBool,
}

impl Parse for InteractiveTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![<]>()?;
        let mut tag = input.parse::<Tag>()?;

        let mut rows = vec![];
        let mut embeds = vec![];

        while input.peek(Token![<]) && !input.peek2(Token![/]) {
            input.parse::<Token![<]>()?;
            let fork = input.fork();
            let next_tag = fork.parse::<Tag>()?;

            match next_tag.name.to_string().as_str() {
                "row" => {
                    rows.push(input.parse::<RowTag>()?);
                }

                "embed" => {
                    embeds.push(input.parse::<EmbedTag>()?);
                }

                _ => {
                    return Err(syn::Error::new(next_tag.name.span(), "unknown tag"));
                }
            }
        }

        let closing = input.parse::<ClosingTag>()?;

        if tag.name != closing.name {
            return Err(syn::Error::new(closing.name.span(), "unclosed tag"));
        }

        let handler_name = match tag.required_attr("handler")? {
            AttrValue::Lit(l) => return Err(syn::Error::new(l.span(), "Should be an Ident")),
            AttrValue::Ident(i) => i,
        };

        let ephemeral = match tag.attr("ephemeral") {
            Some(v) => match v {
                AttrValue::Lit(l) => match l {
                    syn::Lit::Bool(b) => b,
                    _ => return Err(syn::Error::new(l.span(), "Should be a bool")),
                },
                AttrValue::Ident(i) => return Err(syn::Error::new(i.span(), "Should be a bool")),
            },
            None => LitBool::new(false, Span::call_site()),
        };

        Ok(Self {
            struct_name: tag.name,
            rows,
            handler_name,
            embeds,
            ephemeral,
        })
    }
}
