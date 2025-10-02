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
    pub style: Ident,
    pub options_enum: Option<Ident>,

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

        let style = tag.required_attr::<Ident>("style")?;
        let options_enum = tag.attr::<Ident>("options")?;
        let placeholder = tag.attr::<LitStr>("placeholder")?;
        let min_values = tag.attr::<LitInt>("min_values")?;
        let max_values = tag.attr::<LitInt>("max_values")?;
        let disabled = tag.attr::<LitBool>("disabled")?;

        match style.to_string().as_str() {
            "String" => {
                if options_enum.is_none() {
                    return Err(syn::Error::new(
                        style.span(),
                        "string select without options",
                    ));
                }
            }
            "User" | "Channel" | "Role" | "Mentionable" => (),
            _ => {
                return Err(syn::Error::new(
                    style.span(),
                    "unknown style; allowed (String, User, Channle, Role, Mentionable)",
                ))
            }
        }

        return Ok(Self {
            id,
            style,
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
        let style = &self.style;

        optional_attr!(self, placeholder, .placeholder(#placeholder));
        optional_attr!(self, min_values, .min_values(#min_values));
        optional_attr!(self, max_values, .max_values(#max_values));
        optional_attr!(self, disabled, .disabled(#disabled));

        let inner = match style.to_string().as_str() {
            "String" => quote! { options: #options_enum::select_options() },
            "User" => quote! { default_users: None },
            "Role" => quote! { default_roles: None },
            "Mentionable" => quote! { default_users: None, default_roles: None },
            "Channel" => quote! { channel_types: None, default_channels: None },
            _ => unreachable!(),
        };

        tokens.extend(quote! {
            ::serenity::all::CreateSelectMenu::new(#id, ::serenity::all::CreateSelectMenuKind::#style{
                #inner
            }) #placeholder #min_values #max_values #disabled
        });
    }
}
