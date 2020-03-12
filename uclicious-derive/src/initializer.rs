use crate::block::Block;
use quote::{ToTokens, TokenStreamExt};
use proc_macro2::{TokenStream, Span};
use crate::{bindings, DEFAULT_STRUCT_NAME};

#[derive(Debug, Clone)]
pub struct Initializer<'a> {
    /// Name of the target field.
    pub field_ident: &'a syn::Ident,
    /// Default value for the target field.
    ///
    /// This takes precedence over a default struct identifier.
    pub default_value: Option<Block>,
    /// Whether the build_method defines a default struct.
    pub use_default_struct: bool,
    /// path that will be passed down to lookup method.
    pub lookup_path: String,
}

///
/// Something like:
/// ```rust,no_run
///   health: match self.health {
//                     Some(ref value) => ::std::clone::Clone::clone(value),
//                     None => {
//                         return ::std::result::Result::Err(::std::string::String::from(
//                             "`health` must be initialized",
//                         ))
//                     }
//                 },
/// ```
impl<'a> ToTokens for Initializer<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let struct_field = &self.field_ident;
        let lookup_path = &self.lookup_path;
        let obj_error_ty = bindings::ucl_object_error();
        let from_object = bindings::from_object_trait();
        let match_none = self.match_none();
        tokens.append_all(quote!(
                #struct_field: match root.lookup_path(#lookup_path) {
                    Some(obj) => {
                        #from_object::try_from(obj)?
                    },
                    #match_none
                },
            ));
    }
}

impl<'a> Initializer<'a> {
    /// To be used inside of `#struct_field: match self.#builder_field { ... }`
    fn match_none(&'a self) -> MatchNone<'a> {
        match self.default_value {
            Some(ref expr) => MatchNone::DefaultTo(expr),
            None => {
                if self.use_default_struct {
                    MatchNone::UseDefaultStructField(self.field_ident)
                } else {
                    MatchNone::ReturnError(self.lookup_path.clone())
                }
            }
        }
    }

}

// To be used inside of `#struct_field: match self.#builder_field { ... }`
enum MatchNone<'a> {
    /// Inner value must be a valid Rust expression
    DefaultTo(&'a Block),
    /// Inner value must be the field identifier
    ///
    /// The default struct must be in scope in the build_method.
    UseDefaultStructField(&'a syn::Ident),
    /// Inner value must be the field name
    ReturnError(String),
}

impl<'a> ToTokens for MatchNone<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let obj_error_ty = bindings::ucl_object_error();
        let boxed_error = bindings::boxed_error();
        match *self {
            MatchNone::DefaultTo(expr) => tokens.append_all(quote!(
                None => #expr
            )),
            MatchNone::UseDefaultStructField(field_ident) => {
                let struct_ident = syn::Ident::new(DEFAULT_STRUCT_NAME, Span::call_site());
                tokens.append_all(quote!(
                    None => #struct_ident.#field_ident
                ))
            }
            MatchNone::ReturnError(ref err) => tokens.append_all(quote!(
                None => return ::std::result::Result::Err(#obj_error_ty::KeyNotFound(#err.to_string()))
            )),
        }
    }
}