use quote::{quote, ToTokens};
use syn::{parse::Parse, Ident, LitBool, LitInt, LitStr, Token};

use crate::tags::*;

macro_rules! optional_attr {
    ($self:ident, $attr_name:ident, $($tokens:tt)*) => {
        let $attr_name = match &$self.$attr_name {
            Some($attr_name) => { quote! { $($tokens)* }},
            None => quote!{}
        };
    };
}

pub struct CommandOptionTag {
    pub name: LitStr,
    desc: LitStr,
    pub ttype: Ident,
    pub required: Option<LitBool>,
    min_len: Option<LitInt>,
    max_len: Option<LitInt>,
}

impl Parse for CommandOptionTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut tag = input.parse::<Tag>()?;
        let required = tag.attr("required")?;

        let mut name: Option<LitStr> = None;
        let mut desc: Option<LitStr> = None;
        let mut ttype: Option<Ident> = None;

        let mut i = 0;
        while i != 3 {
            input.parse::<Token![<]>()?;
            let fork = input.fork();
            let next_tag = fork.parse::<Tag>()?;

            match next_tag.name.to_string().as_str() {
                "option_type" => {
                    ttype = Some(input.parse::<TypeTag>()?.ttype);
                }
                "name" => {
                    name = Some(input.parse::<NameTag>()?.name);
                }
                "description" => {
                    desc = Some(input.parse::<DescriptionTag>()?.desc);
                }

                _ => return Err(syn::Error::new(next_tag.name.span(), "unknown tag")),
            }

            i += 1;
        }

        let closing = input.parse::<ClosingTag>()?;

        let min_len = tag.attr("min_len")?;
        let max_len = tag.attr("max_len")?;

        if tag.name != closing.name {
            return Err(syn::Error::new(closing.name.span(), "unclosed tag"));
        }

        if let None = name {
            return Err(syn::Error::new(tag.name.span(), "Missing name tag"));
        }

        if let None = desc {
            return Err(syn::Error::new(tag.name.span(), "Missing description tag"));
        }

        if let None = ttype {
            return Err(syn::Error::new(tag.name.span(), "Missing type tag"));
        }

        Ok(Self {
            name: name.unwrap(),
            desc: desc.unwrap(),
            ttype: ttype.unwrap(),
            required,
            min_len,
            max_len,
        })
    }
}

impl ToTokens for CommandOptionTag {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let desc = &self.desc;
        let ttype = &self.ttype;

        optional_attr!(self, required, .required(#required));
        optional_attr!(self, min_len, .min_length(#min_len));
        optional_attr!(self, max_len, .min_length(#max_len));

        tokens.extend(quote! {
            ::serenity::all::CreateCommandOption::new(::serenity::all::CommandOptionType::#ttype, #name, #desc)#required #min_len #max_len
        });
    }
}
