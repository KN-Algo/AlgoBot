use crate::misc::RowComponent;
use crate::tags::*;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Parse, Token};

pub struct ModalRowTag {
    pub component: RowComponent,
}

impl Parse for ModalRowTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let tag = input.parse::<Tag>()?;
        if tag.name.to_owned() != "row" {
            return Err(syn::Error::new(
                tag.name.span(),
                "this should be a <row> tag",
            ));
        }
        input.parse::<Token![<]>()?;
        let fork = input.fork();
        let next_tag = fork.parse::<Tag>()?;

        match next_tag.name.to_string().as_str() {
            "input" => {
                let inputtag = input.parse::<InputTag>()?;
                input.parse::<ClosingTag>()?;
                return Ok(Self {
                    component: RowComponent::Input(inputtag),
                });
            }

            _ => {
                return Err(syn::Error::new(
                    next_tag.name.span(),
                    "modals accept only <input> tags",
                ))
            }
        }
    }
}

impl ToTokens for ModalRowTag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use RowComponent::*;
        let t = match &self.component {
            Buttons(buttons) => {
                quote! { ::serenity::all::CreateActionRow::Buttons(vec![#(#buttons),*]) }
            }
            Input(input) => {
                quote! { ::serenity::all::CreateActionRow::InputText(#input) }
            }
            SelectMenu(menu) => {
                quote! { ::serenity::all::CreateActionRow::SelectMenu(#menu) }
            }
        };

        tokens.extend(t);
    }
}
