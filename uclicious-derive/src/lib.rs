#![recursion_limit = "128"]
extern crate proc_macro;
extern crate proc_macro2;

#[macro_use]
extern crate darling;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use crate::proc_macro::TokenStream;
use darling::FromDeriveInput;
use syn::{parse_macro_input, DeriveInput};

mod options;
use options::Options;
use quote::TokenStreamExt;

mod bindings;
mod block;
mod builder;
mod utils;
mod parser;
mod initializer;

const DEFAULT_STRUCT_NAME: &str = "__default";

#[proc_macro_derive(Uclicious, attributes(ucl))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    derive_for_struct(ast).into()
}

fn derive_for_struct(ast: syn::DeriveInput) -> proc_macro2::TokenStream {
    let opts: Options = match Options::from_derive_input(&ast) {
        Ok(val) => val,
        Err(err) => {
            return err.write_errors();
        }
    };
    let mut builder = opts.as_builder();
    let mut build_fn = opts.as_build_method();

    builder.push_field(&parser::ParserField::default());
    builder.push_build_fn(&opts.as_parser_methods());
    for field in opts.fields() {
        build_fn.push_initializer(field.as_initializer());
    }
    builder.push_build_fn(&build_fn);


    let into_builder = opts.as_into_builder();
    let tokens = quote!(
        #into_builder
        #builder
    );
    //panic!(tokens.to_string());
    tokens
}
