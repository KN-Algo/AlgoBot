use quote::{quote, ToTokens};
use syn::{parse::Parse, Ident, LitBool, LitInt, LitStr, Token};

use crate::{
    command::{description::DescriptionTag, name::NameTag, r#type::TypeTag},
    misc::{AttrValue, ClosingTag},
    Tag,
};

macro_rules! optional_attr {
    ($self:ident, $attr_name:ident, $($tokens:tt)*) => {
        let $attr_name = match &$self.$attr_name {
            Some($attr_name) => { quote! { $($tokens)* }},
            None => quote!{}
        };
    };
}

pub struct OptionTag {
    pub name: LitStr,
    desc: LitStr,
    pub ttype: Ident,
    pub required: Option<LitBool>,
    min_len: Option<LitInt>,
    max_len: Option<LitInt>,
}

impl Parse for OptionTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut tag = input.parse::<Tag>()?;
        let required = match tag.attr("required") {
            None => None,
            Some(attr) => match attr {
                AttrValue::Ident(i) => return Err(syn::Error::new(i.span(), "Should be a bool")),
                AttrValue::Lit(l) => match l {
                    syn::Lit::Bool(b) => Some(b),
                    _ => return Err(syn::Error::new(l.span(), "Should be a bool")),
                },
            },
        };

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

        let min_len = match tag.attr("min_len") {
            Some(v) => match v {
                AttrValue::Lit(l) => match l {
                    syn::Lit::Int(i) => Some(i),
                    _ => return Err(syn::Error::new(l.span(), "Should be an int")),
                },
                AttrValue::Ident(i) => return Err(syn::Error::new(i.span(), "Should be an int")),
            },
            None => None,
        };

        let max_len = match tag.attr("max_len") {
            Some(v) => match v {
                AttrValue::Lit(l) => match l {
                    syn::Lit::Int(i) => Some(i),
                    _ => return Err(syn::Error::new(l.span(), "Should be an int")),
                },
                AttrValue::Ident(i) => return Err(syn::Error::new(i.span(), "Should be an int")),
            },
            None => None,
        };

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

impl ToTokens for OptionTag {
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
