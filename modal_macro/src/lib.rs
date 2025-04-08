extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Group, Punct};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::Ident;
use syn::{parse::Parse, parse_macro_input, DataStruct, DeriveInput, LitInt, LitStr, Token};

#[derive(Debug)]
struct ModalField {
    field_name: Ident,
    field_type: Ident,
    name: String,
}

impl ToTokens for ModalField {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append(proc_macro2::TokenTree::Punct(Punct::new(
            '.',
            proc_macro2::Spacing::Alone,
        )));
        tokens.append(self.field_type.clone());
        tokens.append(proc_macro2::Group::new(
            Delimiter::Parenthesis,
            self.name.to_token_stream(),
        ));
    }
}

struct ModalInput {
    title: String,
    duration: u64,
}

impl Parse for ModalInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let title = input.parse::<LitStr>()?.value();
        input.parse::<Token![,]>()?;
        let duration = input.parse::<LitInt>()?.base10_parse()?;

        Ok(Self { title, duration })
    }
}

fn fields_to_stream(fields: &Vec<ModalField>) -> proc_macro2::TokenStream {
    let mut stream = proc_macro2::TokenStream::new();

    for (i, field) in fields.iter().enumerate() {
        stream.append(field.field_name.clone());
        stream.append(Group::new(
            Delimiter::None,
            quote! {: response.inputs[#i].clone(),},
        ));
    }

    stream
}

#[proc_macro_derive(Modal, attributes(modal, short_field, paragraph_field))]
pub fn modal_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let args = ast.attrs[0].parse_args::<ModalInput>().unwrap();

    let struct_: DataStruct = match ast.data {
        syn::Data::Struct(ref data) => data.clone(),
        _ => panic!("This is not a struct type"),
    };

    let fields: Vec<ModalField> = struct_
        .fields
        .iter()
        .filter_map(|field| {
            for attr in field.attrs.iter() {
                for segment in attr.meta.path().segments.iter() {
                    let field_type = segment.ident.clone();
                    let name = attr.parse_args::<LitStr>().expect("literal string").value();
                    let field_name = field.ident.clone().unwrap();
                    return Some(ModalField {
                        field_type,
                        name,
                        field_name,
                    });
                }
            }

            None
        })
        .collect();

    let title: String = args.title;
    let timeout = args.duration;

    let other = fields_to_stream(&fields);
    let add_fields = fields.iter();

    let code = quote! {
        #[async_trait]
        impl Modal for #name {
            async fn execute(ctx: &::serenity::all::Context, id: ::serenity::all::InteractionId, token: &::std::primitive::str) -> serenity::Result<(Self, ModalResponse)> {

                let mut modal = serenity::all::CreateQuickModal::new(#title)
                    .timeout(::std::time::Duration::from_secs(#timeout))
                    #(#add_fields)*;

                let response = match modal.execute(ctx, id, token).await? {
                    Some(r) => r,
                    None => return Err(serenity::Error::Other("sus")),
                };

                Ok((Self { #other }, ModalResponse { id: response.interaction.id, token: response.interaction.token }))
            }
        }
    };

    code.into()
}
