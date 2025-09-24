use quote::{quote, ToTokens};
use syn::{parse::Parse, spanned::Spanned, Ident, LitStr, Token};

use crate::tags::*;

macro_rules! optional_attr {
    ($self:ident, $attr_name:ident, $($tokens:tt)*) => {
        let $attr_name = match &$self.$attr_name {
            Some($attr_name) => { quote! { $($tokens)* }},
            None => quote!{}
        };
    };
}

pub struct CommandTag {
    pub struct_name: Ident,
    pub description: Option<LitStr>,
    pub options: Vec<CommandOptionTag>,
}

impl Parse for CommandTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![<]>()?;
        let tag = input.parse::<Tag>()?;

        let mut desc: Option<LitStr> = None;
        let mut options: Vec<CommandOptionTag> = vec![];

        while input.peek(Token![<]) && !input.peek2(Token![/]) {
            input.parse::<Token![<]>()?;
            let fork = input.fork();
            let next_tag = fork.parse::<Tag>()?;

            match next_tag.name.to_string().as_str() {
                "description" => {
                    desc = Some(input.parse::<DescriptionTag>()?.desc);
                }
                "option" => options.push(input.parse::<CommandOptionTag>()?),

                _ => return Err(syn::Error::new(next_tag.name.span(), "unknown tag")),
            }
        }

        let closing = input.parse::<ClosingTag>()?;

        if tag.name != closing.name {
            return Err(syn::Error::new(closing.name.span(), "unclosed tag"));
        }

        if options.len() > 25 {
            return Err(syn::Error::new(
                tag.name.span(),
                "Commands can have up to 25 options",
            ));
        }

        Ok(Self {
            struct_name: tag.name,
            description: desc,
            options,
        })
    }
}

impl ToTokens for CommandTag {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_name = &self.struct_name;
        let option_struct_name = Ident::new(
            &format!("{}Options", struct_name.to_string()),
            struct_name.span(),
        );
        let options = self.options.iter();
        optional_attr!(self, description, .description(#description));

        let option_fields = options.clone().map(|option| {
            let i = Ident::new(&option.name.value().to_lowercase(), option.span());
            let ttype = match option.ttype.to_string().as_str() {
                "User" => quote! { ::serenity::all::UserId },
                "Role" => quote! { ::serenity::all::RoleId },
                "Channel" => quote! { ::serenity::all::ChannelId },
                "Number" => quote! { ::std::primitive::f64 },
                "Integer" => quote! { ::std::primitive::i64 },
                "Boolean" => quote! { ::std::primitive::bool },
                _ => quote! { &'o ::std::primitive::str },
            };

            let ttype = match &option.required {
                None => quote! { ::std::option::Option<#ttype> },
                Some(r) => match r.value {
                    true => ttype,
                    false => quote! { ::std::option::Option<#ttype> },
                },
            };

            quote! { pub #i: #ttype, }
        });

        let option_match = options.clone().map(|option| {
            let name = &option.name;
            let i = Ident::new(&option.name.value().to_lowercase(), option.span());
            let eq = match option.ttype.to_string().as_str() {
                "User" => quote! { .as_user_id() },
                "Role" => quote! { .as_role_id() },
                "Channel" => quote! { .as_channel_id() },
                "Number" => quote! { .as_f64() },
                "Integer" => quote! { .as_i64() },
                "Boolean" => quote! { .as_bool() },
                _ => quote! { .as_str() },
            };

            let eq = match &option.required {
                None => eq,
                Some(r) => match r.value {
                    true => quote! { #eq.unwrap() },
                    false => eq,
                },
            };

            quote! { #name => s.#i = option.value #eq, }
        });

        tokens.extend(quote! {
            struct #struct_name;
            impl #struct_name {
                fn new(command: ::serenity::all::CreateCommand) -> ::serenity::all::CreateCommand {
                    command #description #(.add_option(#options))*
                }

                fn options<'o>(ctx: &crate::components::CommandCtx<'o>) -> #option_struct_name<'o> {
                    let mut s = #option_struct_name::default();
                    for option in &ctx.interaction.data.options {
                        match option.name.as_str() {
                            #(#option_match)*
                            _ => unreachable!()
                        }
                    }

                    return s;
                }
            }

            #[derive(::std::default::Default)]
            struct #option_struct_name<'o> {
                #(#option_fields)*
            }
        });
    }
}
