use crate::tags::*;

use proc_macro2::Span;
use syn::LitBool;
use syn::{parse::Parse, Ident, Token};

pub struct InteractiveTag {
    pub struct_name: Ident,
    pub rows: Vec<RowTag>,
    pub embeds: Vec<EmbedTag>,
    pub handler_name: Ident,
    pub state_ident: Option<Ident>,
    pub ephemeral: LitBool,
    pub text: Option<TextTag>,
}

impl Parse for InteractiveTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![<]>()?;
        let mut tag = input.parse::<Tag>()?;

        let mut rows = vec![];
        let mut embeds = vec![];
        let mut text = None;

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

                "text" => text = Some(input.parse::<TextTag>()?),

                _ => {
                    return Err(syn::Error::new(next_tag.name.span(), "unknown tag"));
                }
            }
        }

        let closing = input.parse::<ClosingTag>()?;

        if tag.name != closing.name {
            return Err(syn::Error::new(closing.name.span(), "unclosed tag"));
        }

        let handler_name = tag.required_attr::<Ident>("handler")?;
        let state_ident = tag.attr::<Ident>("state")?;

        let ephemeral = match tag.attr::<LitBool>("ephemeral")? {
            Some(v) => v,
            None => LitBool::new(false, Span::call_site()),
        };

        Ok(Self {
            struct_name: tag.name,
            rows,
            handler_name,
            embeds,
            ephemeral,
            text,
            state_ident,
        })
    }
}
