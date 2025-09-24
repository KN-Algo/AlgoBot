use crate::misc::AttrValue;
use crate::tags::{ClosingTag, Tag};

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Parse, LitStr};

macro_rules! optional_attr {
    ($self:ident, $attr_name:ident, $($tokens:tt)*) => {
        let $attr_name = match &$self.$attr_name {
            Some($attr_name) => { quote! { $($tokens)* }},
            None => quote!{}
        };
    };
}

pub struct OptionTag {
    pub id: LitStr,
    pub inner: LitStr,

    pub description: Option<AttrValue>,
    pub default: Option<AttrValue>,
}

impl Parse for OptionTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut tag = input.parse::<Tag>()?;
        if tag.name.to_string() != "option" {
            return Err(syn::Error::new(
                tag.name.span(),
                "this should be a <option> tag",
            ));
        }

        let id = tag.id()?;
        let inner = input.parse::<LitStr>()?;
        let closing = input.parse::<ClosingTag>()?;

        if closing.name != tag.name {
            return Err(syn::Error::new(closing.name.span(), "unclosed tag"));
        }

        let description = tag.attr("description");
        let default = tag.attr("default");

        return Ok(Self {
            id,
            inner,
            description,
            default,
        });
    }
}

impl ToTokens for OptionTag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let id = &self.id;
        let inner = &self.inner;

        optional_attr!(self, description, .description(#description));
        optional_attr!(self, default, .default_selection(#default));

        tokens.extend(quote! {
            ::serenity::all::CreateSelectMenuOption::new(#inner, #id) #description #default
        });
    }
}
