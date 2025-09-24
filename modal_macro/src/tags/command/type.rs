use syn::{parse::Parse, Ident};

use crate::tags::{ClosingTag, Tag};

pub struct TypeTag {
    pub ttype: Ident,
}

impl Parse for TypeTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let tag = input.parse::<Tag>()?;
        let ttype = input.parse::<Ident>()?;
        let closing = input.parse::<ClosingTag>()?;

        if tag.name != closing.name {
            return Err(syn::Error::new(closing.name.span(), "unclosed tag"));
        }

        match ttype.to_string().as_str() {
            "User" | "String" | "Number" | "Boolean" | "Integer" | "SubCommand" | "Channel"
            | "Role" | "Mentionable" => (),
            _ => return Err(syn::Error::new(ttype.span(), "invalid type")),
        };

        Ok(Self { ttype })
    }
}
