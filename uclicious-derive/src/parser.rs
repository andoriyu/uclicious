use darling::ToTokens;
use proc_macro2::{TokenStream, Ident};
use quote::TokenStreamExt;
use crate::bindings;

pub struct ParserField {}

impl ToTokens for ParserField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident: Ident = syn::parse_str("__parser").unwrap();
        let ty = bindings::ucl_parser();
        tokens.append_all(quote!(
            #ident: #ty,
        ))
    }
}

impl Default for ParserField {
    fn default() -> Self {
        ParserField {}
    }
}