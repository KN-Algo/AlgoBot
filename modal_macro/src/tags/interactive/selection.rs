use crate::misc::AttrValue;
use crate::tags::*;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Parse, LitStr, Token};

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
    pub options: Vec<OptionTag>,

    pub placeholder: Option<AttrValue>,
    pub min_values: Option<AttrValue>,
    pub max_values: Option<AttrValue>,
    pub disabled: Option<AttrValue>,
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
        let mut options = vec![];
        while input.peek(Token![<]) && !input.peek2(Token![/]) {
            input.parse::<Token![<]>()?;
            options.push(input.parse::<OptionTag>()?);
        }

        if options.len() < 1 {
            return Err(syn::Error::new(
                tag.name.span(),
                "select menus must have at least one option",
            ));
        }

        if options.len() > 25 {
            return Err(syn::Error::new(
                tag.name.span(),
                "select menus can have up to 25 options",
            ));
        }

        let closing = input.parse::<ClosingTag>()?;

        if closing.name != tag.name {
            return Err(syn::Error::new(closing.name.span(), "unclosed tag"));
        }

        let placeholder = tag.attr("placeholder");
        let min_values = tag.attr("min_values");
        let max_values = tag.attr("max_values");
        let disabled = tag.attr("disabled");

        return Ok(Self {
            id,
            options,
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
        let options = &self.options;

        optional_attr!(self, placeholder, .placeholder(#placeholder));
        optional_attr!(self, min_values, .min_values(#min_values));
        optional_attr!(self, max_values, .max_values(#max_values));
        optional_attr!(self, disabled, .disabled(#disabled));

        tokens.extend(quote! {
            ::serenity::all::CreateSelectMenu::new(#id, ::serenity::all::CreateSelectMenuKind::String{
                options: vec![#(#options),*]
            }) #placeholder #min_values #max_values #disabled
        });
    }
}
