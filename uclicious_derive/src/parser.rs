use crate::bindings;
use darling::ToTokens;
use proc_macro2::{Ident, TokenStream};
use quote::TokenStreamExt;

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
        /// Add a chunk of text to the parser. String must:
        /// - not have `\0` character;
        /// - must be valid UCL object;
        #vis fn add_chunk_full<C: #as_ref<str>>(&mut self, chunk: C, priority: #priority, strategy: #dup_strategy) -> #result<(), #err> {
            self.__parser.add_chunk_full(chunk, priority, strategy)
        }
        /// Add a file by a file path to the parser. This function uses mmap call to load file, therefore, it should not be shrunk during parsing.
        #vis fn add_file_full<F: #as_ref<#path>>(&mut self, file: F, priority: #priority, strategy: #dup_strategy) -> #result<(), #err> {
            self.__parser.add_file_full(file, priority, strategy)
        }
        /// Register new variable `$var` that should be replaced by the parser to the `value` string.
        /// Variables need to be registered _before_ they are referenced.
        ///
        /// #### Panics
        /// This function panics if either `var` or `value` has `\0`.
        #vis fn register_variable<K: #as_ref<str>, V: #as_ref<str>>(
            &mut self,
            var: K,
            value: V,
        ) -> &mut Self {
            self.__parser.register_variable(var, value);
            self
        }
        /// Add the standard file variables to the `parser` based on the `filename` specified:
        ///
        /// - `$FILENAME`- a filename of ucl input
        /// - `$CURDIR` - a current directory of the input
        ///
        /// For example, if a filename param is `../something.conf` then the variables will have the following values:
        ///
        /// - `$FILENAME` - `../something.conf`
        /// - `$CURDIR` - `..`
        ///
        /// if need_expand parameter is true then all relative paths are expanded using realpath call. In this example if .. is /etc/dir then variables will have these values:
        ///
        /// - `$FILENAME` - `/etc/something.conf`
        /// - `$CURDIR` - `/etc`
        #vis fn set_filevars<F: #as_ref<#path>>(
            &mut self,
            filename: F,
            need_expand: bool,
        ) -> #result<(), #err> {
            self.__parser.set_filevars(filename, need_expand)
        }
        ))
    }
}
