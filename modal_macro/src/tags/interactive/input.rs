use crate::tags::*;

use proc_macro2::{Span, TokenStream};
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

pub struct InputTag {
    pub id: LitStr,
    pub inner: LitStr,

    pub style: Ident,
    pub placeholder: Option<LitStr>,
    pub min_len: Option<LitInt>,
    pub max_len: Option<LitInt>,
    pub value: Option<LitStr>,
    pub required: Option<LitBool>,
}

impl Parse for InputTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut tag = input.parse::<Tag>()?;

        if tag.name.to_string() != "input" {
            return Err(syn::Error::new(
                tag.name.span(),
                "this should be a <input> tag",
            ));
        }
        let id = tag.id()?;
        let inner = input.parse::<LitStr>()?;
        let closing = input.parse::<ClosingTag>()?;

        if closing.name != tag.name {
            return Err(syn::Error::new(closing.name.span(), "unclosed tag"));
        }

        let style = {
            let attr = tag.required_attr::<LitStr>("style")?;
            match attr.value().as_str() {
                "short" => Ident::new("Short", Span::call_site()),
                "paragraph" => Ident::new("Paragraph", Span::call_site()),
                _ => {
                    return Err(syn::Error::new(
                        attr.span(),
                        "style can be only (short, paragraph)",
                    ))
                }
            }
        };

        let placeholder = tag.attr::<LitStr>("placeholder")?;
        let min_len = tag.attr::<LitInt>("min_len")?;
        let max_len = tag.attr::<LitInt>("max_len")?;
        let value = tag.attr::<LitStr>("value")?;
        let required = tag.attr::<LitBool>("required")?;

        return Ok(Self {
            id,
            inner,
            style,
            placeholder,
            min_len,
            max_len,
            value,
            required,
        });
    }
}

impl ToTokens for InputTag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let id = &self.id;
        let inner = &self.inner;
        let style = &self.style;

        optional_attr!(self, placeholder, .placeholder(#placeholder));
        optional_attr!(self, min_len, .min_length(#min_len));
        optional_attr!(self, max_len, .max_length(#max_len));
        optional_attr!(self, value, .value(#value));
        optional_attr!(self, required, .required(#required));

        tokens.extend(quote! {
            ::serenity::all::CreateInputText::new(
                ::serenity::all::InputTextStyle::#style,
                #inner,
                #id
            )#placeholder #min_len #max_len #value #required
        });
    }
}
