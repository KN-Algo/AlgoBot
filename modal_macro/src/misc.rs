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

impl TryFrom<AttrValue> for syn::Ident {
    type Error = syn::Error;
    fn try_from(value: AttrValue) -> Result<Self, Self::Error> {
        match value {
            AttrValue::Ident(i) => Ok(i),
            AttrValue::Lit(l) => Err(syn::Error::new(l.span(), "Must be an ident")),
        }
    }
}

impl TryFrom<AttrValue> for syn::LitStr {
    type Error = syn::Error;
    fn try_from(value: AttrValue) -> Result<Self, Self::Error> {
        let err = Err(syn::Error::new(value.span(), "Must be a string"));
        match value {
            AttrValue::Ident(_) => err,
            AttrValue::Lit(l) => match l {
                Lit::Str(s) => Ok(s),
                _ => err,
            },
        }
    }
}

impl TryFrom<AttrValue> for syn::LitInt {
    type Error = syn::Error;
    fn try_from(value: AttrValue) -> Result<Self, Self::Error> {
        let err = Err(syn::Error::new(value.span(), "Must be an integer"));
        match value {
            AttrValue::Ident(_) => err,
            AttrValue::Lit(l) => match l {
                Lit::Int(s) => Ok(s),
                _ => err,
            },
        }
    }
}

impl TryFrom<AttrValue> for syn::LitBool {
    type Error = syn::Error;
    fn try_from(value: AttrValue) -> Result<Self, Self::Error> {
        let err = Err(syn::Error::new(value.span(), "Must be a boolean"));
        match value {
            AttrValue::Ident(_) => err,
            AttrValue::Lit(l) => match l {
                Lit::Bool(s) => Ok(s),
                _ => err,
            },
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
