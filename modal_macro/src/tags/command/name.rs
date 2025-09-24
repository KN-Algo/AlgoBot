use syn::{parse::Parse, LitStr};

use crate::tags::{ClosingTag, Tag};

pub struct NameTag {
    pub name: LitStr,
}

impl Parse for NameTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let tag = input.parse::<Tag>()?;
        let name = input.parse::<LitStr>()?;
        let closing = input.parse::<ClosingTag>()?;

        if tag.name != closing.name {
            return Err(syn::Error::new(closing.name.span(), "unclosed tag"));
        }

        if !name
            .value()
            .chars()
            .all(|c| c.is_lowercase() || c.is_numeric() || c == '-')
        {
            return Err(syn::Error::new(
                name.span(),
                "invalid name: must be lowercase and [a-z][0-9] or -",
            ));
        }

        if name.value().len() > 32 {
            return Err(syn::Error::new(
                name.span(),
                "name cant be longer than 32 characters",
            ));
        }

        Ok(Self { name })
    }
}
