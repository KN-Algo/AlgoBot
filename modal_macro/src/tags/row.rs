use crate::misc::ClosingTag;
use crate::tags::*;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse::Parse, Token};

pub enum RowComponent {
    Buttons(Vec<ButtonTag>),
    SelectMenu(SelectionTag),
    Input(InputTag),
}

pub struct RowTag {
    pub component: RowComponent,
}

impl Parse for RowTag {
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
            "button" => {
                let b = input.parse::<ButtonTag>()?;
                let mut buttons = vec![b];
                while input.peek(Token![<]) && !input.peek2(Token![/]) {
                    input.parse::<Token![<]>()?;
                    buttons.push(input.parse::<ButtonTag>()?);
                }
                input.parse::<ClosingTag>()?;
                return Ok(Self {
                    component: RowComponent::Buttons(buttons),
                });
            }
            "selection" => {
                let selecttag = input.parse::<SelectionTag>()?;
                input.parse::<ClosingTag>()?;
                return Ok(Self {
                    component: RowComponent::SelectMenu(selecttag),
                });
            }

            _ => {
                return Err(syn::Error::new(
                    next_tag.name.span(),
                    "rows accept only <button> and <selection> tags",
                ))
            }
        }
    }
}

impl ToTokens for RowTag {
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
