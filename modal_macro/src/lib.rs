extern crate proc_macro;

use crate::tags::*;
use crate::row::RowComponent;

use proc_macro2::{Span, TokenStream};
use quote::{quote };
use syn::{ parse_macro_input, Ident};

mod misc;
mod tags;



#[proc_macro]
pub fn interactive_msg(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let struct_tag = parse_macro_input!(input as InteractiveTag);

    let rows = struct_tag.rows;
    let name = struct_tag.struct_name;

    let trait_funcs = rows.iter().map(|row| {
        match &row.component {
            RowComponent::Input(i) => { 
                let ident = Ident::new(&format!("handle_{}", i.id.value()), Span::call_site());
                quote! { async fn #ident(ctx: &mut crate::components::EventCtx) -> Result<(), ::serenity::Error> { Ok(()) } }
            }
            RowComponent::SelectMenu(s) => {
                let options = s.options.iter().map(|option| {
                    let ident = Ident::new(&format!("handle_{}", option.id.value()), Span::call_site());
                    quote! { async fn #ident(ctx: &mut crate::components::EventCtx) -> Result<(), ::serenity::Error> { Ok(()) } }
                });

                quote! { #(#options)* }
            }
            RowComponent::Buttons(b) => {
                let buttons = b.iter().map(|button| {
                    let ident = Ident::new(&format!("handle_{}", button.id.value()), Span::call_site());
                    quote! { async fn #ident(ctx: &mut crate::components::EventCtx) -> Result<(), ::serenity::Error> { Ok(()) } }
                });

                quote! { #(#buttons)* }
            }
        }
    });

    let handle_func = rows.iter().map(|row| {
        match &row.component {
            RowComponent::Input(i) => {
                let id = i.id.value();
                let ident = Ident::new(&format!("handle_{}", id), Span::call_site());
                quote! { #id => Handler::#ident(ctx).await?, }
            }
            RowComponent::Buttons(b) => {
                let buttons = b.iter().map(|button| {
                    let id = button.id.value();
                    let ident = Ident::new(&format!("handle_{}", id), Span::call_site());

                    quote! { #id => { Handler::#ident(ctx).await? } }
                });

                quote! { #(#buttons),* }
            }
            RowComponent::SelectMenu(s) => {
                let id = s.id.value();
                let options = s.options.iter().map(|option| {
                    let id = option.id.value();
                    let ident = Ident::new(&format!("handle_{}", id), Span::call_site());
                    quote! { #id => Handler::#ident(ctx).await?, }
                });

                quote! { #id => {
                    if let ::serenity::all::ComponentInteractionDataKind::StringSelect { ref values } = ctx.interaction.data.kind {
                        match values[0].as_str() {
                            #(#options)*
                            _ => crate::log_error!("Unknown custom_id: {} for interaction: ", ctx.interaction.data.custom_id)
                        }
                    }
                } }
            }
        }
    });

    let handler_name = struct_tag.handler_name;
    let merged_handler_trait_ident = Ident::new(&format!("{}Trait", handler_name.to_string()), handler_name.span());

    let code = quote! {
        #[::serenity::async_trait]
        trait #merged_handler_trait_ident {
            #(#trait_funcs)*
        }

        struct #name<Handler: #merged_handler_trait_ident> {
            p: ::std::marker::PhantomData<Handler>
        }

        struct #handler_name;

        #[::serenity::async_trait]
        impl<Handler: #merged_handler_trait_ident> crate::traits::InteractiveMessageTrait for #name<Handler> {
            fn into_msg() -> ::serenity::all::CreateInteractionResponseMessage {
                ::serenity::all::CreateInteractionResponseMessage::new().components(vec![#(#rows),*])
            }

            async fn handle_event(ctx: &mut crate::components::EventCtx) -> Result<(), ::serenity::Error> {
                //crate::log!("Running {}", stringify!(#name));
                match ctx.interaction.data.custom_id.as_str() {
                    #(#handle_func)*
                    _ => crate::log_error!("Unknown custom_id: {} for interaction: ", ctx.interaction.data.custom_id)
                }

                Ok(())
            }
        }


    };

    code.into()
}

#[proc_macro]
pub fn modal(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let modal = parse_macro_input!(input as ModalTag);

    let struct_name = &modal.struct_name;
    let title = &modal.title;
    let duration = &modal.duration;
    let components = &modal.rows;

    let fields = modal.fields();

    let declare_fields = fields
        .iter()
        .map(|field| {
            let field_name = &field.field_name;
            quote! { #field_name: Option<String>, }
        })
        .collect::<TokenStream>();

    let let_components = fields
        .iter()
        .map(|field| {
            let field_name = &field.field_name;
            quote! {
                let mut #field_name = None;
            }
        })
        .collect::<TokenStream>();

    let from_components = fields
        .iter()
        .map(|field| {
            let name = field.field_name.to_string();
            let field_name = &field.field_name;

            quote! {
                if &text.custom_id == #name { #field_name = text.value.clone(); }
            }
        })
        .collect::<TokenStream>();

    let fields_punct = fields
        .iter()
        .map(|field| {
            let field_name = &field.field_name;
            quote! { #field_name, }
        })
        .collect::<TokenStream>();

    let code = quote! {
        struct #struct_name<'ctx> {
            interaction: ::serenity::all::ModalInteraction,
            discord_ctx: &'ctx ::serenity::all::Context,
            #declare_fields
        }

        #[::serenity::async_trait]
        impl<'ctx> crate::traits::modal::ModalTrait<'ctx> for #struct_name<'ctx> {
            async fn execute(ctx: &::serenity::all::Context, id_token: (::serenity::all::InteractionId, &::std::primitive::str)) -> Result<Self, ::serenity::Error> where 'life0: 'ctx {
                use ::serenity::builder::Builder;
                let custom_id = id_token.0.to_string();
                let modal = ::serenity::builder::CreateModal::new(&custom_id, #title).components(
                    vec![#(#components),*]
                );

                let builder = ::serenity::builder::CreateInteractionResponse::Modal(modal);
                builder
                    .execute(ctx, id_token)
                    .await?;

                let collector = ::serenity::collector::ModalInteractionCollector::new(&ctx.shard)
                    .custom_ids(vec![custom_id])
                    .timeout(std::time::Duration::from_secs(#duration));

                let modal_interaction = collector.next().await;
                let Some(modal_interaction) = modal_interaction else { return Err(::serenity::Error::Other("Didn't receive a modal interaction back!")) };

                let inputs = modal_interaction.data.components.iter();
                #let_components
                for input in inputs {
                    match input.components.first() {
                        Some(::serenity::all::ActionRowComponent::InputText(text)) => {
                            #from_components
                        },

                        _ => (),
                    }
                }

                Ok(Self { interaction: modal_interaction, discord_ctx: ctx, #fields_punct })
            }

        }

        impl<'ctx> crate::traits::Interactable<'ctx> for #struct_name<'ctx> {
            fn discord_ctx(&self) -> &::serenity::all::Context {
                self.discord_ctx
            }

            fn id_token(&self) -> (::serenity::all::InteractionId, &::std::primitive::str) {
                (self.interaction.id, &self.interaction.token)
            }
        }
    };

    code.into()
}
