use crate::misc::{AttrValue, Attribute};
use std::collections::HashMap;

use syn::{parse::Parse, Ident, Lit, LitStr, Token};

pub struct Tag {
    pub name: Ident,
    pub attributes: HashMap<String, Attribute>,
}

impl Tag {
    pub fn id(&mut self) -> syn::Result<LitStr> {
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

    pub fn required_attr(&mut self, attr_name: &str) -> syn::Result<AttrValue> {
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

    pub fn attr(&mut self, attr_name: &str) -> Option<AttrValue> {
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
