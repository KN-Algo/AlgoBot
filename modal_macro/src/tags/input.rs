use crate::misc::{AttrValue, ClosingTag};
use crate::tags::Tag;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse::Parse, Ident, Lit, LitStr};

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
    pub placeholder: Option<AttrValue>,
    pub min_len: Option<AttrValue>,
    pub max_len: Option<AttrValue>,
    pub value: Option<AttrValue>,
    pub required: Option<AttrValue>,
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
            match tag.required_attr("style")? {
                AttrValue::Ident(i) => i,
                AttrValue::Lit(l) => match l {
                    Lit::Str(s) => match s.value().as_str() {
                        "short" => Ident::new("Short", Span::call_site()),
                        "paragraph" => Ident::new("Paragraph", Span::call_site()),
                        _ => {
                            return Err(syn::Error::new(
                                s.span(),
                                "style can be only (short, paragraph)",
                            ))
                        }
                    },
                    _ => {
                        return Err(syn::Error::new(
                            l.span(),
                            "style can be only (short, paragraph)",
                        ))
                    }
                },
            }
        };

        let placeholder = tag.attr("placeholder");
        let min_len = tag.attr("min_len");
        let max_len = tag.attr("max_len");
        let value = tag.attr("value");
        let required = tag.attr("required");

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
