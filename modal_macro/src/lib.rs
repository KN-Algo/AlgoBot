extern crate proc_macro;

use std::collections::HashMap;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse::Parse, parse_macro_input, Ident, Lit, LitStr, Token};

macro_rules! optional_attr {
    ($self:ident, $attr_name:ident, $($tokens:tt)*) => {
        let $attr_name = match &$self.$attr_name {
            Some($attr_name) => { quote! { $($tokens)* }},
            None => quote!{}
        };
    };
}

#[derive(Clone)]
enum AttrValue {
    Lit(Lit),
    Ident(Ident),
}

impl AttrValue {
    fn span(&self) -> Span {
        match self {
            Self::Lit(l) => l.span(),
            Self::Ident(i) => i.span(),
        }
    }
}

impl ToTokens for AttrValue {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Lit(l) => tokens.extend(quote! {#l}),
            Self::Ident(i) => tokens.extend(quote! {#i}),
        }
    }
}

//trait TryFromLit<Type: Spanned>: Sized {
//    fn try_trom_lit(lit: AttrValue<Type>) -> syn::Result<Self>;
//}
//
//impl TryFromLit<Self> for LitStr {
//    fn try_trom_lit(lit: AttrValue<Self>) -> syn::Result<Self> {
//        if let AttrValue::Lit(s) = lit {
//            return Ok(s);
//        }
//
//        Err(syn::Error::new(lit.span(), "expected string literal"))
//    }
//}
//
//impl TryFromLit<Self> for LitInt {
//    fn try_trom_lit(lit: AttrValue<Self>) -> syn::Result<Self> {
//        if let AttrValue::Lit(s) = lit {
//            return Ok(s);
//        }
//
//        Err(syn::Error::new(lit.span(), "expected int literal"))
//    }
//}
//
//impl TryFromLit<Self> for LitBool {
//    fn try_trom_lit(lit: AttrValue<Self>) -> syn::Result<Self> {
//        if let AttrValue::Lit(s) = lit {
//            return Ok(s);
//        }
//
//        Err(syn::Error::new(lit.span(), "expected bool literal"))
//    }
//}
//
//impl TryFromLit for String {
//    fn try_trom_lit(lit: AttrValue) -> syn::Result<Self> {
//        let lit = LitStr::try_trom_lit(lit)?;
//        Ok(lit.value())
//    }
//}
//
//impl TryFromLit for Ident {
//    fn try_trom_lit(lit: AttrValue) -> syn::Result<Self> {
//        if let AttrValue::Ident(ident) = lit {
//            return Ok(ident);
//        }
//
//        Err(syn::Error::new(lit.span(), "expected ident literal"))
//    }
//}

struct ClosingTag {
    name: Ident,
}

impl Parse for ClosingTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![<]>()?;
        input.parse::<Token![/]>()?;
        let name = input.parse::<Ident>()?;
        input.parse::<Token![>]>()?;

        Ok(Self { name })
    }
}

struct Attribute {
    name: Ident,
    value: AttrValue,
}

impl Parse for Attribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        let value;
        if input.peek(Lit) {
            value = AttrValue::Lit(input.parse::<Lit>()?);
        } else {
            value = AttrValue::Ident(input.parse::<Ident>()?);
        }

        return Ok(Self { name, value });
    }
}

struct Tag {
    name: Ident,
    attributes: HashMap<String, Attribute>,
}

impl Tag {
    fn id(&mut self) -> syn::Result<LitStr> {
        let attr = match self.attributes.remove("id") {
            Some(a) => a,
            None => {
                return Err(syn::Error::new(
                    self.name.span(),
                    "component should have an id",
                ))
            }
        };

        let id;
        if let AttrValue::Lit(Lit::Str(s)) = attr.value {
            id = s;
        } else {
            return Err(syn::Error::new(
                attr.value.span(),
                "must be a literal string",
            ));
        }

        if id.value().contains(" ") {
            return Err(syn::Error::new(id.span(), "ids can't have spaces"));
        }

        return Ok(id);
    }

    fn required_attr(&mut self, attr_name: &str) -> syn::Result<AttrValue> {
        match self.attributes.remove(attr_name) {
            Some(attr) => {
                return Ok(attr.value);
            }
            None => {
                return Err(syn::Error::new(
                    self.name.span(),
                    format!("component should have a {attr_name}"),
                ))
            }
        }
    }

    fn attr(&mut self, attr_name: &str) -> Option<AttrValue> {
        self.attributes.remove(attr_name).map(|a| a.value)
    }
}

impl Parse for Tag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        let mut attributes = HashMap::new();
        while !input.peek(Token![>]) {
            let attr = input.parse::<Attribute>()?;
            attributes.insert(attr.name.to_string(), attr);
        }

        input.parse::<Token![>]>()?;

        Ok(Self { name, attributes })
    }
}

struct OptionTag {
    id: LitStr,
    inner: LitStr,

    description: Option<AttrValue>,
    default: Option<AttrValue>,
}

impl Parse for OptionTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut tag = input.parse::<Tag>()?;
        if tag.name.to_string() != "option" {
            return Err(syn::Error::new(
                tag.name.span(),
                "this should be a <option> tag",
            ));
        }

        let id = tag.id()?;
        let inner = input.parse::<LitStr>()?;
        let closing = input.parse::<ClosingTag>()?;

        if closing.name != tag.name {
            return Err(syn::Error::new(closing.name.span(), "unclosed tag"));
        }

        let description = tag.attr("description");
        let default = tag.attr("default");

        return Ok(Self {
            id,
            inner,
            description,
            default,
        });
    }
}

impl ToTokens for OptionTag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let id = &self.id;
        let inner = &self.inner;

        optional_attr!(self, description, .description(#description));
        optional_attr!(self, default, .default_selection(#default));

        tokens.extend(quote! {
            ::serenity::all::CreateSelectMenuOption::new(#inner, #id) #description #default
        });
    }
}

struct InputTag {
    id: LitStr,
    inner: LitStr,

    style: Ident,
    placeholder: Option<AttrValue>,
    min_len: Option<AttrValue>,
    max_len: Option<AttrValue>,
    value: Option<AttrValue>,
    required: Option<AttrValue>,
}

impl Parse for InputTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut tag = input.parse::<Tag>()?;

        if tag.name.to_string() != "input" {
            return Err(syn::Error::new(
                tag.name.span(),
                "this should be a <input> tag",
            ));
        }
        let id = tag.id()?;
        let inner = input.parse::<LitStr>()?;
        let closing = input.parse::<ClosingTag>()?;

        if closing.name != tag.name {
            return Err(syn::Error::new(closing.name.span(), "unclosed tag"));
        }

        let style = {
            match tag.required_attr("style")? {
                AttrValue::Ident(i) => i,
                AttrValue::Lit(l) => match l {
                    Lit::Str(s) => match s.value().as_str() {
                        "short" => Ident::new("Short", Span::call_site()),
                        "paragraph" => Ident::new("Paragraph", Span::call_site()),
                        _ => {
                            return Err(syn::Error::new(
                                s.span(),
                                "style can be only (short, paragraph)",
                            ))
                        }
                    },
                    _ => {
                        return Err(syn::Error::new(
                            l.span(),
                            "style can be only (short, paragraph)",
                        ))
                    }
                },
            }
        };

        let placeholder = tag.attr("placeholder");
        let min_len = tag.attr("min_len");
        let max_len = tag.attr("max_len");
        let value = tag.attr("value");
        let required = tag.attr("required");

        return Ok(Self {
            id,
            inner,
            style,
            placeholder,
            min_len,
            max_len,
            value,
            required,
        });
    }
}

impl ToTokens for InputTag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let id = &self.id;
        let inner = &self.inner;
        let style = &self.style;

        optional_attr!(self, placeholder, .placeholder(#placeholder));
        optional_attr!(self, min_len, .min_length(#min_len));
        optional_attr!(self, max_len, .max_length(#max_len));
        optional_attr!(self, value, .value(#value));
        optional_attr!(self, required, .required(#required));

        tokens.extend(quote! {
            ::serenity::all::CreateInputText::new(
                ::serenity::all::InputTextStyle::#style,
                #inner,
                #id
            )#placeholder #min_len #max_len #value #required
        });
    }
}

struct ButtonTag {
    id: LitStr,
    inner: LitStr,

    link: Option<AttrValue>,
    style: Option<Ident>,
    disabled: Option<AttrValue>,
}

impl Parse for ButtonTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut tag = input.parse::<Tag>()?;
        if tag.name.to_string() != "button" {
            return Err(syn::Error::new(
                tag.name.span(),
                "this should be a <button> tag",
            ));
        }
        if tag.name.to_owned() != "button" {}
        let id = tag.id()?;
        let inner = input.parse::<LitStr>()?;
        let closing = input.parse::<ClosingTag>()?;

        if closing.name != tag.name {
            return Err(syn::Error::new(closing.name.span(), "unclosed tag"));
        }

        let link = tag.attr("link");
        let disabled = tag.attr("disabled");
        let s = tag.attr("style");
        let style = if let Some(attr) = s {
            match attr {
                AttrValue::Ident(i) => Some(i),
                AttrValue::Lit(l) => match l {
                    Lit::Str(s) => match s.value().as_str() {
                        "primary" => Some(Ident::new("Primary", Span::call_site())),
                        "secondary" => Some(Ident::new("Secondary", Span::call_site())),
                        "success" => Some(Ident::new("Success", Span::call_site())),
                        "danger" => Some(Ident::new("Danger", Span::call_site())),
                        _ => {
                            return Err(syn::Error::new(
                                s.span(),
                                "style can be only (primary, secondary, success, danger)",
                            ))
                        }
                    },
                    _ => {
                        return Err(syn::Error::new(
                            l.span(),
                            "style can be only (primary, secondary, success, danger)",
                        ))
                    }
                },
            }
        } else {
            None
        };

        return Ok(Self {
            id,
            inner,
            disabled,
            link,
            style,
        });
    }
}

impl ToTokens for ButtonTag {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let id = &self.id;
        let btype = match &self.link {
            Some(l) => quote! { new_link(#l) },
            None => quote! { new(#id) },
        };

        let inner = &self.inner;

        optional_attr!(self, disabled, .disabled(#disabled));
        optional_attr!(self, style, .style(::serenity::all::ButtonStyle::#style));

        tokens.extend(quote! {
            ::serenity::all::CreateButton::#btype.label(#inner) #disabled #style
        });
    }
}

//struct TextTag {
//    text: String,
//}
//
//impl Parse for TextTag {
//    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
//        let tag = input.parse::<Tag>()?;
//        if tag.name.to_string() != "text" {
//            return Err(syn::Error::new(
//                tag.name.span(),
//                "this should be a <text> tag",
//            ));
//        }
//        let text = input.parse::<LitStr>()?.value();
//        let closing = input.parse::<ClosingTag>()?;
//
//        if closing.name != tag.name {
//            return Err(syn::Error::new(closing.name.span(), "unclosed tag"));
//        }
//
//        return Ok(Self { text });
//    }
//}

struct SelectionTag {
    id: LitStr,
    options: Vec<OptionTag>,

    placeholder: Option<AttrValue>,
    min_values: Option<AttrValue>,
    max_values: Option<AttrValue>,
    disabled: Option<AttrValue>,
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
        let mut options = vec![];
        while input.peek(Token![<]) && !input.peek2(Token![/]) {
            input.parse::<Token![<]>()?;
            options.push(input.parse::<OptionTag>()?);
        }

        if options.len() < 1 {
            return Err(syn::Error::new(
                tag.name.span(),
                "select menus must have at least one option",
            ));
        }

        if options.len() > 25 {
            return Err(syn::Error::new(
                tag.name.span(),
                "select menus can have up to 25 options",
            ));
        }

        let closing = input.parse::<ClosingTag>()?;

        if closing.name != tag.name {
            return Err(syn::Error::new(closing.name.span(), "unclosed tag"));
        }

        let placeholder = tag.attr("placeholder");
        let min_values = tag.attr("min_values");
        let max_values = tag.attr("max_values");
        let disabled = tag.attr("disabled");

        return Ok(Self {
            id,
            options,
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
        let options = &self.options;

        optional_attr!(self, placeholder, .placeholder(#placeholder));
        optional_attr!(self, min_values, .min_values(#min_values));
        optional_attr!(self, max_values, .max_values(#max_values));
        optional_attr!(self, disabled, .disabled(#disabled));

        tokens.extend(quote! {
            ::serenity::all::CreateSelectMenu::new(#id, ::serenity::all::CreateSelectMenuKind::String{
                options: vec![#(#options),*]
            }) #placeholder #min_values #max_values #disabled
        });
    }
}

enum RowComponent {
    Buttons(Vec<ButtonTag>),
    SelectMenu(SelectionTag),
    Input(InputTag),
}

struct RowTag {
    component: RowComponent,
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

struct ModalRowTag {
    component: RowComponent,
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

struct InteractiveTag {
    struct_name: Ident,
    rows: Vec<RowTag>,
}

impl Parse for InteractiveTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![<]>()?;
        let tag = input.parse::<Tag>()?;

        let mut rows = vec![];
        while input.peek(Token![<]) && !input.peek2(Token![/]) {
            input.parse::<Token![<]>()?;
            rows.push(input.parse::<RowTag>()?);
        }

        let closing = input.parse::<ClosingTag>()?;

        if tag.name != closing.name {
            return Err(syn::Error::new(closing.name.span(), "unclosed tag"));
        }

        Ok(Self {
            struct_name: tag.name,
            rows,
        })
    }
}

struct ModalField {
    field_name: Ident,
}

struct ModalTag {
    struct_name: Ident,
    title: AttrValue,
    duration: AttrValue,
    rows: Vec<ModalRowTag>,
}

impl ModalTag {
    fn fields(&self) -> Vec<ModalField> {
        let mut fields = vec![];

        for row in &self.rows {
            if let RowComponent::Input(input) = &row.component {
                fields.push(ModalField {
                    field_name: Ident::new(&input.id.value(), Span::call_site()),
                });
            }
        }
        fields
    }
}

impl Parse for ModalTag {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<Token![<]>()?;
        let mut tag = input.parse::<Tag>()?;

        let title = tag.required_attr("title")?;
        let duration = tag.required_attr("duration")?;

        let mut rows = vec![];
        while input.peek(Token![<]) && !input.peek2(Token![/]) {
            input.parse::<Token![<]>()?;
            rows.push(input.parse::<ModalRowTag>()?);
        }

        let closing = input.parse::<ClosingTag>()?;

        if tag.name != closing.name {
            return Err(syn::Error::new(closing.name.span(), "unclosed tag"));
        }

        Ok(Self {
            struct_name: tag.name,
            rows,
            title,
            duration,
        })
    }
}

#[proc_macro]
pub fn interactive_msg(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let struct_tag = parse_macro_input!(input as InteractiveTag);

    let rows = struct_tag.rows;
    let name = struct_tag.struct_name;

    let trait_funcs = rows.iter().map(|row| {
        match &row.component {
            RowComponent::Input(i) => { 
                let ident = Ident::new(&format!("handle_{}", i.id.value()), Span::call_site());
                quote! { async fn #ident(_ctx: &::serenity::all::Context, _interaction: &::serenity::all::ComponentInteraction, _msg: &mut crate::components::InteractiveMessage) -> Result<(), ::serenity::Error> { Ok(()) } }
            }
            RowComponent::SelectMenu(s) => {
                let options = s.options.iter().map(|option| {
                    let ident = Ident::new(&format!("handle_{}", option.id.value()), Span::call_site());
                    quote! { async fn #ident(_ctx: &::serenity::all::Context, _interaction: &::serenity::all::ComponentInteraction, _msg: &mut crate::components::InteractiveMessage) -> Result<(), ::serenity::Error> { Ok(()) } }
                });

                quote! { #(#options)* }
            }
            RowComponent::Buttons(b) => {
                let buttons = b.iter().map(|button| {
                    let ident = Ident::new(&format!("handle_{}", button.id.value()), Span::call_site());
                    quote! { async fn #ident(_ctx: &::serenity::all::Context, _interaction: &::serenity::all::ComponentInteraction, _msg: &mut crate::components::InteractiveMessage) -> Result<(), ::serenity::Error> { Ok(()) } }
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
                quote! { #id => Handler::#ident(ctx, interaction, msg).await?, }
            }
            RowComponent::Buttons(b) => {
                let buttons = b.iter().map(|button| {
                    let id = button.id.value();
                    let ident = Ident::new(&format!("handle_{}", id), Span::call_site());

                    quote! { #id => { Handler::#ident(ctx, interaction, msg).await? } }
                });

                quote! { #(#buttons),* }
            }
            RowComponent::SelectMenu(s) => {
                let id = s.id.value();
                let options = s.options.iter().map(|option| {
                    let id = option.id.value();
                    let ident = Ident::new(&format!("handle_{}", id), Span::call_site());
                    quote! { #id => Handler::#ident(ctx, interaction, msg).await?, }
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

            async fn handle_event(ctx: &::serenity::all::Context, interaction: &::serenity::all::ComponentInteraction, msg: &mut crate::components::InteractiveMessage) -> Result<(), ::serenity::Error> {
                crate::log!("Running {}", stringify!(#name));
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
