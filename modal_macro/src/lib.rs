extern crate proc_macro;

use crate::tags::*;
use crate::misc::RowComponent;

use proc_macro2::{Span, TokenStream};
use quote::{quote };
use syn::{ parse_macro_input, spanned::Spanned, Data, DeriveInput, Field, GenericArgument, Ident, LitStr, PathArguments, Type, TypePath};

mod misc;
mod tags;

#[proc_macro_derive(SelectionState, attributes(selection_state))]
pub fn derive_selection_state(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;
    let fields = match input.data {
        Data::Struct(d) => d.fields,
        _ => return syn::Error::new(struct_name.span(), "Not a Struct").to_compile_error().into()
    };

    let fields = fields.into_iter().filter(|f| {
        f.attrs.iter().any(|attr| {
            attr.path().is_ident("selection_state")
        })
    }).collect::<Vec<Field>>();
        
    if fields.is_empty() {
        return syn::Error::new(
            struct_name.span(),
            "No field marked as selection_state"
        ).to_compile_error().into()
    }

    let mut user_id_fields = vec![];
    let mut channel_id_fields = vec![];
    let mut role_id_fields = vec![];
    let mut generic_id_fields = vec![];
    let mut other_fields = vec![];

    for f in fields {
        let name = f.ident.as_ref().unwrap();
        let s = LitStr::new(&name.to_string(), name.span());
        let err :proc_macro::TokenStream = syn::Error::new(f.ty.span(), "Needs to be a Vec<*Selection*>").into_compile_error().into();
        let path = match &f.ty {
            Type::Path(TypePath { path, .. }) => path,
            _ => return err,
        };

        let segment = match path.segments.last() {
                    Some(segment) => segment,
                    None => return err,
        };

        if segment.ident != "Vec" {
            return err;
        }

        let inner_type = if let PathArguments::AngleBracketed(args) = &segment.arguments {
            if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                inner_type
            } else { return err }
        } else { return err };


        if let Type::Path(TypePath { path, .. }) = inner_type {
                if let Some(segment) = path.segments.last() {
                    match segment.ident.to_string().as_str() {
                        "UserId" => user_id_fields.push(quote! { #s => self.#name = values.clone(), }),
                        "ChanelId" => channel_id_fields.push(quote! { #s => self.#name = values.clone(), }),
                        "RoleId" => role_id_fields.push(quote! { #s => self.#name = values.clone(), }),
                        "GenericId" => generic_id_fields.push(quote! { #s => self.#name = values.clone(), }),
                        _ => other_fields.push(quote! { #s => self.#name = values.into_iter().map(|v| #inner_type::from_str(&v)).collect::<crate::aliases::TypedResult<Vec<#inner_type>>>()?, }),
                    }
                }
            }
    }


    let code = quote! { 
        impl #struct_name {
            pub fn set_selection_state(&mut self, interaction: &::serenity::all::ComponentInteraction) -> crate::aliases::Result {
                match interaction.data.kind {
                    ::serenity::all::ComponentInteractionDataKind::StringSelect { ref values } => {
                        match interaction.data.custom_id.as_str() {
                            #(#other_fields)*
                            _ => return Err(crate::error::BotError::Serenity(::serenity::Error::Other("Unknown custom_id!")))
                        }
                    }

                    ::serenity::all::ComponentInteractionDataKind::UserSelect { ref values } => {
                        match interaction.data.custom_id.as_str() {
                            #(#user_id_fields)*
                            _ => return Err(crate::error::BotError::Serenity(::serenity::Error::Other("Unknown custom_id!")))
                        }
                    }

                    ::serenity::all::ComponentInteractionDataKind::RoleSelect { ref values } => {
                        match interaction.data.custom_id.as_str() {
                            #(#role_id_fields)*
                            _ => return Err(crate::error::BotError::Serenity(::serenity::Error::Other("Unknown custom_id!")))
                        }
                    }

                    ::serenity::all::ComponentInteractionDataKind::ChannelSelect { ref values } => {
                        match interaction.data.custom_id.as_str() {
                            #(#channel_id_fields)*
                            _ => return Err(crate::error::BotError::Serenity(::serenity::Error::Other("Unknown custom_id!")))
                        }
                    }

                    ::serenity::all::ComponentInteractionDataKind::MentionableSelect { ref values } => {
                        match interaction.data.custom_id.as_str() {
                            #(#generic_id_fields)*
                            _ => return Err(crate::error::BotError::Serenity(::serenity::Error::Other("Unknown custom_id!")))
                        }
                    }
                    _ => return Err(crate::error::BotError::Serenity(::serenity::Error::Other("Unknown interaction type!")))
                }
                Ok(())
            }
        }
    };

    code.into()


}

#[proc_macro_derive(Selection, attributes(select_value))]
pub fn derive_selection(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = input.ident;
    let variants = match input.data {
        Data::Enum(d) => d.variants,
        _ => return syn::Error::new(enum_name.span(), "Not an Enum").to_compile_error().into(),
    };

    if variants.is_empty() {
        return syn::Error::new(enum_name.span(), "Must have at least one variant").to_compile_error().into()
    }

    if variants.len() > 25 {
        return syn::Error::new(enum_name.span(), "Must have at most 25 variants").to_compile_error().into()
    }

    let builders = variants.iter()
        .map(|v| {
        let ident = v.ident.to_string();
        let mut label = ident.to_string();

        for attr in &v.attrs {
            if !attr.path().is_ident("select_value") {
                continue;
            }

            let arg = match attr.parse_args::<LitStr>() {
                Ok(v) => v,
                Err(e) => return e.to_compile_error(),
            };

            label = arg.value();
        }

        quote! { ::serenity::all::CreateSelectMenuOption::new(
            #label,
            #ident
        ) }
    });

    let from_match = variants.iter().map(|v| {
        let ident = &v.ident;
        let string = v.ident.to_string();
        quote! { #string => Ok(#enum_name::#ident) }
    });

    let code = quote! {
        impl #enum_name {
            pub fn select_options() -> Vec<::serenity::all::CreateSelectMenuOption> {
                ::std::vec![#(#builders),*]
            }

            pub fn from_str(s: &::std::primitive::str) -> crate::aliases::TypedResult<Self> {
                match s {
                    #(#from_match,)*
                    _ => Err(crate::error::BotError::Serenity(::serenity::Error::Other("Unknown custom_id!")))
                }
            }
        }
    };

    code.into()
}

#[proc_macro]
pub fn command(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let command_tag = parse_macro_input!(input as CommandTag);
    let code = quote! { #command_tag };
    code.into()
}


#[proc_macro]
pub fn interactive_msg(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let struct_tag = parse_macro_input!(input as InteractiveTag);

    let rows = struct_tag.rows;
    let name = struct_tag.struct_name;

    let state_ident = match &struct_tag.state_ident {
        Some(i) => quote! { #i },
        None => quote! { () }
    };

    if rows.iter().any(|row| { match row.component { RowComponent::SelectMenu(_) => true, _ => false } })  && struct_tag.state_ident.is_none() {
        return syn::Error::new(name.span(), "Message has Selection but doesn't have a State").into_compile_error().into();
    }


    let embeds = struct_tag.embeds;
    let embeds_comm = embeds.clone().into_iter().map(|tag| {
        tag.embed_name
    });
    let embeds_event = embeds.into_iter().map(|tag| {
        tag.embed_name
    });

    let trait_funcs = rows.iter().map(|row| {
        match &row.component {
            RowComponent::Input(i) => { 
                let id = i.id.value();
                let ident = Ident::new(&format!("handle_{}", id), Span::call_site());
                quote! { async fn #ident(ctx: &mut crate::components::EventCtx) -> crate::aliases::Result { crate::log_warn!("Unhandled Interaction: {}", #id); Ok(()) } }
            }
            RowComponent::SelectMenu(s) => {
                let id = s.id.value();
                let ident = Ident::new(&format!("handle_{}", id), Span::call_site());
                quote! { async fn #ident(ctx: &mut crate::components::EventCtx) -> crate::aliases::Result { ctx.acknowlage().await } }
            }
            RowComponent::Buttons(b) => {
                let buttons = b.iter().map(|button| {
                    let id = button.id.value();
                    let ident = Ident::new(&format!("handle_{}", id), Span::call_site());
                    quote! { async fn #ident(ctx: &mut crate::components::EventCtx) -> crate::aliases::Result { crate::log_warn!("Unhandled Interaction: {}", #id); Ok(()) } }
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
                let ident = Ident::new(&format!("handle_{}", id), Span::call_site());
                quote! { #id => { let mut state = ctx.msg.clone_state::<Self::State>().await.unwrap(); state.set_selection_state(&ctx.interaction)?; ctx.msg.write_state::<Self::State>(state).await; Handler::#ident(ctx).await? } }
            }
        }
    });

    let handler_name = struct_tag.handler_name;
    let merged_handler_trait_ident = Ident::new(&format!("{}Trait", handler_name.to_string()), handler_name.span());
    let ephemeral = struct_tag.ephemeral;
    let content = match struct_tag.text {
        None => quote! {},
        Some(tag) => {
            let text = tag.text;
            quote! { .content(#text) }
        }
    };

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
            type State = #state_ident;
            fn into_msg() -> ::serenity::all::CreateInteractionResponseMessage {
                ::serenity::all::CreateInteractionResponseMessage::new().components(vec![#(#rows),*]).ephemeral(#ephemeral) #content
            }

            async fn with_embeds_command(ctx: &crate::components::CommandCtx, state: &crate::components::State) -> ::std::vec::Vec<::serenity::all::CreateEmbed> {
                use crate::traits::into_embed::IntoEmbedInteractive;
                ::std::vec![#(#embeds_comm::from_command(ctx, state).await)*]
            }

            async fn with_embeds_event(ctx: &crate::components::EventCtx) -> ::std::vec::Vec<::serenity::all::CreateEmbed> {
                use crate::traits::into_embed::IntoEmbedInteractive;
                ::std::vec![#(#embeds_event::from_event(ctx).await)*]
            }

            async fn handle_event(ctx: &mut crate::components::EventCtx) -> crate::aliases::Result {
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
            quote! { #field_name: String, }
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
            quote! { #field_name: #field_name.unwrap(), }
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
            async fn execute(ctx: &::serenity::all::Context, id_token: (::serenity::all::InteractionId, &::std::primitive::str)) -> crate::aliases::TypedResult<Self> where 'life0: 'ctx {
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
                let Some(modal_interaction) = modal_interaction else { return Err(::serenity::Error::Other("Didn't receive a modal interaction back!").into()) };

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

        impl crate::traits::Interactable<'_> for #struct_name<'_> {
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
