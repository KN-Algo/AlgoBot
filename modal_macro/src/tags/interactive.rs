use crate::misc::ClosingTag;
use crate::tags::*;

use syn::{parse::Parse, Ident, Token};

pub struct InteractiveTag {
    pub struct_name: Ident,
    pub rows: Vec<RowTag>,
}

impl Parse for InteractiveTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![<]>()?;
        let tag = input.parse::<Tag>()?;

        let mut rows = vec![];
        while input.peek(Token![<]) && !input.peek2(Token![/]) {
            input.parse::<Token![<]>()?;
            rows.push(input.parse::<RowTag>()?);
        }

        let closing = input.parse::<ClosingTag>()?;

        if tag.name != closing.name {
            return Err(syn::Error::new(closing.name.span(), "unclosed tag"));
        }

        Ok(Self {
            struct_name: tag.name,
            rows,
        })
    }
}
