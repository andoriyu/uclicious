use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Block(Vec<syn::Stmt>);

impl Default for Block {
    fn default() -> Self {
        "".parse().unwrap()
    }
}

impl ToTokens for Block {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let inner = &self.0;
        tokens.append_all(quote!(
            { #( #inner )* }
        ));
    }
}

impl FromStr for Block {
    type Err = String;

    /// Parses a string `s` to return a `Block`.
    ///
    /// # Errors
    ///
    /// When `expr` cannot be parsed as `Vec<syn::TokenTree>`. E.g. unbalanced
    /// opening/closing delimiters like `{`, `(` and `[` will be _rejected_ as
    /// parsing error.
    fn from_str(expr: &str) -> Result<Self, Self::Err> {
        let b: syn::Block =
            syn::parse_str(&format!("{{{}}}", expr)).map_err(|e| format!("{}", e))?;
        Ok(Self(b.stmts))
    }
}
