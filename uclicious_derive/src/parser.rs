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


#[derive(Debug)]
pub struct ParserMethods {
    /// Visibility of the build method, e.g. `syn::Visibility::Public`.
    pub visibility: syn::Visibility,
}

impl ToTokens for ParserMethods {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let vis = &self.visibility;
        let as_ref = bindings::as_ref_trait();
        let priority = bindings::ucilicous_priority_type();
        let dup_strategy = bindings::ucilicous_duplicate_strategy_type();
        let result = bindings::result_ty();
        let err = bindings::ucl_parser_error();
        let path = bindings::path_ty();
        tokens.append_all(quote! (
        #vis fn add_chunk_full<C: #as_ref<str>>(&mut self, chunk: C, priority: #priority, strategy: #dup_strategy) -> #result<(), #err> {
            self.__parser.add_chunk_full(chunk, priority, strategy)
        }
        #vis fn add_file_full<F: #as_ref<#path>>(&mut self, file: F, priority: #priority, strategy: #dup_strategy) -> #result<(), #err> {
            self.__parser.add_file_full(file, priority, strategy)
        }
        ))
    }
}