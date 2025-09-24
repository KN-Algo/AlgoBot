use crate::tags::*;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use quote::ToTokens;
use syn::parse::Parse;
use syn::Token;
use syn::{Ident, Lit};

#[derive(Clone)]
pub enum AttrValue {
    Lit(Lit),
    Ident(Ident),
}

impl AttrValue {
    pub fn span(&self) -> Span {
        match self {
            Self::Lit(l) => l.span(),
            Self::Ident(i) => i.span(),
        }
    }
}

impl ToTokens for AttrValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Lit(l) => tokens.extend(quote! {#l}),
            Self::Ident(i) => tokens.extend(quote! {#i}),
        }
    }
}

pub struct Attribute {
    pub name: Ident,
    pub value: AttrValue,
}

impl Parse for Attribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        let value;
        if input.peek(Lit) {
            value = AttrValue::Lit(input.parse::<Lit>()?);
        } else {
            value = AttrValue::Ident(input.parse::<Ident>()?);
        }

        return Ok(Self { name, value });
    }
}

pub enum RowComponent {
    Buttons(Vec<ButtonTag>),
    SelectMenu(SelectionTag),
    Input(InputTag),
}
