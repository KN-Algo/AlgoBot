use crate::tags::*;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Parse, Ident, LitBool, LitInt, LitStr};

macro_rules! optional_attr {
    ($self:ident, $attr_name:ident, $($tokens:tt)*) => {
        let $attr_name = match &$self.$attr_name {
            Some($attr_name) => { quote! { $($tokens)* }},
            None => quote!{}
        };
    };
}

pub struct SelectionTag {
    pub id: LitStr,
    pub options_enum: Ident,

    pub placeholder: Option<LitStr>,
    pub min_values: Option<LitInt>,
    pub max_values: Option<LitInt>,
    pub disabled: Option<LitBool>,
}

impl Parse for SelectionTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut tag = input.parse::<Tag>()?;
        if tag.name.to_string() != "selection" {
            return Err(syn::Error::new(
                tag.name.span(),
                "this should be a <selection> tag",
            ));
        }
        let id = tag.id()?;

        let closing = input.parse::<ClosingTag>()?;

        if closing.name != tag.name {
            return Err(syn::Error::new(closing.name.span(), "unclosed tag"));
        }

        let options_enum = tag.required_attr::<Ident>("options")?;
        let placeholder = tag.attr::<LitStr>("placeholder")?;
        let min_values = tag.attr::<LitInt>("min_values")?;
        let max_values = tag.attr::<LitInt>("max_values")?;
        let disabled = tag.attr::<LitBool>("disabled")?;

        return Ok(Self {
            id,
            options_enum,
            placeholder,
            min_values,
            max_values,
            disabled,
        });
    }
}

impl ToTokens for SelectionTag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let id = &self.id;
        let options_enum = &self.options_enum;

        optional_attr!(self, placeholder, .placeholder(#placeholder));
        optional_attr!(self, min_values, .min_values(#min_values));
        optional_attr!(self, max_values, .max_values(#max_values));
        optional_attr!(self, disabled, .disabled(#disabled));

        tokens.extend(quote! {
            ::serenity::all::CreateSelectMenu::new(#id, ::serenity::all::CreateSelectMenuKind::String{
                options: #options_enum::select_options()
            }) #placeholder #min_values #max_values #disabled
        });
    }
}
