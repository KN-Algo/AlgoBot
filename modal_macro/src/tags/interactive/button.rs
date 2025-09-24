use crate::misc::AttrValue;
use crate::tags::*;

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

pub struct ButtonTag {
    pub id: LitStr,
    pub inner: LitStr,

    pub link: Option<AttrValue>,
    pub style: Option<Ident>,
    pub disabled: Option<AttrValue>,
}

impl Parse for ButtonTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut tag = input.parse::<Tag>()?;
        if tag.name.to_string() != "button" {
            return Err(syn::Error::new(
                tag.name.span(),
                "this should be a <button> tag",
            ));
        }
        if tag.name.to_owned() != "button" {}
        let id = tag.id()?;
        let inner = input.parse::<LitStr>()?;
        let closing = input.parse::<ClosingTag>()?;

        if closing.name != tag.name {
            return Err(syn::Error::new(closing.name.span(), "unclosed tag"));
        }

        let link = tag.attr("link");
        let disabled = tag.attr("disabled");
        let s = tag.attr("style");
        let style = if let Some(attr) = s {
            match attr {
                AttrValue::Ident(i) => Some(i),
                AttrValue::Lit(l) => match l {
                    Lit::Str(s) => match s.value().as_str() {
                        "primary" => Some(Ident::new("Primary", Span::call_site())),
                        "secondary" => Some(Ident::new("Secondary", Span::call_site())),
                        "success" => Some(Ident::new("Success", Span::call_site())),
                        "danger" => Some(Ident::new("Danger", Span::call_site())),
                        _ => {
                            return Err(syn::Error::new(
                                s.span(),
                                "style can be only (primary, secondary, success, danger)",
                            ))
                        }
                    },
                    _ => {
                        return Err(syn::Error::new(
                            l.span(),
                            "style can be only (primary, secondary, success, danger)",
                        ))
                    }
                },
            }
        } else {
            None
        };

        return Ok(Self {
            id,
            inner,
            disabled,
            link,
            style,
        });
    }
}

impl ToTokens for ButtonTag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let id = &self.id;
        let btype = match &self.link {
            Some(l) => quote! { new_link(#l) },
            None => quote! { new(#id) },
        };

        let inner = &self.inner;

        optional_attr!(self, disabled, .disabled(#disabled));
        optional_attr!(self, style, .style(::serenity::all::ButtonStyle::#style));

        tokens.extend(quote! {
            ::serenity::all::CreateButton::#btype.label(#inner) #disabled #style
        });
    }
}
