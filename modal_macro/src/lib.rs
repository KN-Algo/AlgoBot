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
                quote! { async fn #ident(_ctx: &::serenity::all::Context, _interaction: &::serenity::all::ComponentInteraction, _msg: &mut crate::components::InteractiveMessage, db: &::sqlx::SqlitePool) -> Result<(), ::serenity::Error> { Ok(()) } }
            }
            RowComponent::SelectMenu(s) => {
                let options = s.options.iter().map(|option| {
                    let ident = Ident::new(&format!("handle_{}", option.id.value()), Span::call_site());
                    quote! { async fn #ident(_ctx: &::serenity::all::Context, _interaction: &::serenity::all::ComponentInteraction, _msg: &mut crate::components::InteractiveMessage, db: &::sqlx::SqlitePool) -> Result<(), ::serenity::Error> { Ok(()) } }
                });

                quote! { #(#options)* }
            }
            RowComponent::Buttons(b) => {
                let buttons = b.iter().map(|button| {
                    let ident = Ident::new(&format!("handle_{}", button.id.value()), Span::call_site());
                    quote! { async fn #ident(_ctx: &::serenity::all::Context, _interaction: &::serenity::all::ComponentInteraction, _msg: &mut crate::components::InteractiveMessage, db: &::sqlx::SqlitePool) -> Result<(), ::serenity::Error> { Ok(()) } }
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
                quote! { #id => Handler::#ident(ctx, interaction, msg, db).await?, }
            }
            RowComponent::Buttons(b) => {
                let buttons = b.iter().map(|button| {
                    let id = button.id.value();
                    let ident = Ident::new(&format!("handle_{}", id), Span::call_site());

                    quote! { #id => { Handler::#ident(ctx, interaction, msg, db).await? } }
                });

                quote! { #(#buttons),* }
            }
            RowComponent::SelectMenu(s) => {
                let id = s.id.value();
                let options = s.options.iter().map(|option| {
                    let id = option.id.value();
                    let ident = Ident::new(&format!("handle_{}", id), Span::call_site());
                    quote! { #id => Handler::#ident(ctx, interaction, msg, db).await?, }
                });

                quote! { #id => {
                    if let ::serenity::all::ComponentInteractionDataKind::StringSelect { ref values } = interaction.data.kind {
                        match values[0].as_str() {
                            #(#options)*
                            _ => crate::log_error!("Unknown custom_id: {} for interaction: ", interaction.data.custom_id)
                        }
                    }
                } }
            }
        }
    });

    let merged_handler_trait_ident = Ident::new(&format!("{}HandlerTrait", name.to_string()), Span::call_site());

    let code = quote! {
        #[::serenity::async_trait]
        trait #merged_handler_trait_ident {
            #(#trait_funcs)*
        }

        struct #name<Handler: #merged_handler_trait_ident> {
            p: ::std::marker::PhantomData<Handler>
        }

        #[::serenity::async_trait]
        impl<Handler: #merged_handler_trait_ident> crate::traits::InteractiveMessageTrait for #name<Handler> {
            fn into_msg() -> ::serenity::all::CreateInteractionResponseMessage {
                ::serenity::all::CreateInteractionResponseMessage::new().components(vec![#(#rows),*])
            }

            async fn handle_event(ctx: &::serenity::all::Context, interaction: &::serenity::all::ComponentInteraction, msg: &mut crate::components::InteractiveMessage, db: &::sqlx::SqlitePool) -> Result<(), ::serenity::Error> {
                //crate::log!("Running {}", stringify!(#name));
                match interaction.data.custom_id.as_str() {
                    #(#handle_func)*
                    _ => crate::log_error!("Unknown custom_id: {} for interaction: ", interaction.data.custom_id)
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
        struct #struct_name {
            #declare_fields
        }

        #[::serenity::async_trait]
        impl crate::traits::modal::Modal for #struct_name {
            async fn execute(ctx: &::serenity::all::Context, interaction: &::serenity::all::CommandInteraction) -> Result<Self, ::serenity::Error> {
                use ::serenity::builder::Builder;
                let custom_id = interaction.id.to_string();
                let modal = ::serenity::builder::CreateModal::new(&custom_id, #title).components(
                    vec![#(#components),*]
                );

                let builder = ::serenity::builder::CreateInteractionResponse::Modal(modal);
                builder
                    .execute(ctx, (interaction.id, &interaction.token))
                    .await?;

                let collector = ::serenity::collector::ModalInteractionCollector::new(&ctx.shard)
                    .custom_ids(vec![custom_id])
                    .timeout(std::time::Duration::from_secs(#duration));

                let modal_interaction = collector.next().await;
                let Some(modal_interaction) = modal_interaction else { return Err(::serenity::Error::Other("SUS")) };
                modal_interaction.create_response(ctx, ::serenity::all::CreateInteractionResponse::Message(
                    ::serenity::all::CreateInteractionResponseMessage::default().ephemeral(true).content("Done!")
                )).await?;

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

                Ok(Self { #fields_punct })
            }
        }
    };

    code.into()
}
