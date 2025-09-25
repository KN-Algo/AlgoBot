use syn::{parse::Parse, LitStr};

use crate::tags::{ClosingTag, Tag};

#[derive(Clone)]
pub struct TextTag {
    pub text: LitStr,
}

impl Parse for TextTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let tag = input.parse::<Tag>()?;
        let text = input.parse::<LitStr>()?;
        let closing = input.parse::<ClosingTag>()?;

        if tag.name != closing.name {
            return Err(syn::Error::new(closing.name.span(), "unclosed tag"));
        }

        Ok(Self { text })
    }
}
