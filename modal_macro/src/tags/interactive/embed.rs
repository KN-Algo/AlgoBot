use syn::{parse::Parse, Ident};

use crate::tags::{ClosingTag, Tag};

#[derive(Clone)]
pub struct EmbedTag {
    pub embed_name: Ident,
}

impl Parse for EmbedTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let tag = input.parse::<Tag>()?;
        let embed_name = input.parse::<Ident>()?;
        let closing = input.parse::<ClosingTag>()?;

        if tag.name != closing.name {
            return Err(syn::Error::new(closing.name.span(), "unclosed tag"));
        }

        Ok(Self { embed_name })
    }
}
