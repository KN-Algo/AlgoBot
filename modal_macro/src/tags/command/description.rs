use syn::{parse::Parse, LitStr};

use crate::{misc::ClosingTag, Tag};

pub struct DescriptionTag {
    pub desc: LitStr,
}

impl Parse for DescriptionTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let tag = input.parse::<Tag>()?;
        let desc = input.parse::<LitStr>()?;
        let closing = input.parse::<ClosingTag>()?;

        if tag.name != closing.name {
            return Err(syn::Error::new(closing.name.span(), "unclosed tag"));
        }

        if desc.value().len() > 100 {
            return Err(syn::Error::new(
                desc.span(),
                "description cant be longer than 100 characters",
            ));
        }

        Ok(Self { desc })
    }
}
