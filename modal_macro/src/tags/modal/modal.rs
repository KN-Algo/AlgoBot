use crate::misc::RowComponent;
use crate::tags::*;

use proc_macro2::Span;
use syn::LitInt;
use syn::LitStr;
use syn::{parse::Parse, Ident, Token};

pub struct ModalField {
    pub field_name: Ident,
}

pub struct ModalTag {
    pub struct_name: Ident,
    pub title: LitStr,
    pub duration: LitInt,
    pub rows: Vec<ModalRowTag>,
}

impl ModalTag {
    pub fn fields(&self) -> Vec<ModalField> {
        let mut fields = vec![];

        for row in &self.rows {
            if let RowComponent::Input(input) = &row.component {
                fields.push(ModalField {
                    field_name: Ident::new(&input.id.value(), Span::call_site()),
                });
            }
        }
        fields
    }
}

impl Parse for ModalTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![<]>()?;
        let mut tag = input.parse::<Tag>()?;

        let title = tag.required_attr::<LitStr>("title")?;
        let duration = tag.required_attr::<LitInt>("duration")?;

        let mut rows = vec![];
        while input.peek(Token![<]) && !input.peek2(Token![/]) {
            input.parse::<Token![<]>()?;
            rows.push(input.parse::<ModalRowTag>()?);
        }

        let closing = input.parse::<ClosingTag>()?;

        if tag.name != closing.name {
            return Err(syn::Error::new(closing.name.span(), "unclosed tag"));
        }

        Ok(Self {
            struct_name: tag.name,
            rows,
            title,
            duration,
        })
    }
}
