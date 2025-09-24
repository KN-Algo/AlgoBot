use syn::parse::Parse;
use syn::Ident;
use syn::Token;

pub struct ClosingTag {
    pub name: Ident,
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
