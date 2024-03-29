use crate::block::Block;
use crate::{bindings, DEFAULT_STRUCT_NAME};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, TokenStreamExt};
use syn::Path;

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
    pub validation: Option<Path>,
    pub from: Option<Path>,
    pub try_from: Option<Path>,
    pub map: Option<Path>,
    pub from_str: bool,
}

impl<'a> ToTokens for Initializer<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let struct_field = &self.field_ident;
        let lookup_path = &self.lookup_path;
        let match_none = self.match_none();
        let match_some = self.match_some();
        tokens.append_all(quote!(
            #struct_field: match root.lookup_path(#lookup_path) {
                Some(obj) => {
                    let lookup_path = #lookup_path;
                    #match_some
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
    fn match_some(&'a self) -> MatchSome {
        match (
            &self.validation,
            &self.from,
            &self.try_from,
            &self.map,
            &self.from_str,
        ) {
            (None, None, None, None, false) => MatchSome::Simple,
            (Some(validation), None, None, None, false) => MatchSome::Validation(validation),
            (None, Some(src_type), None, None, false) => MatchSome::From(src_type),
            (None, None, Some(src_type), None, false) => MatchSome::TryFrom(src_type),
            (Some(validation), Some(from), None, None, false) => {
                MatchSome::FromValidation(from, validation)
            }
            (Some(validation), None, Some(from), None, false) => {
                MatchSome::TryFromValidation(from, validation)
            }
            (None, None, None, Some(map_func), false) => MatchSome::Map(map_func),
            (Some(validation), None, None, Some(map_func), false) => {
                MatchSome::MapValidation(map_func, validation)
            }
            (None, None, None, None, true) => MatchSome::FromStr,
            (Some(validation), None, None, None, true) => MatchSome::FromStrValidation(validation),
            _ => panic!(
                "field {}: map, from and try_from are mutually exclusive",
                self.field_ident
            ),
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

enum MatchSome<'a> {
    Simple,
    Validation(&'a Path),
    From(&'a Path),
    FromValidation(&'a Path, &'a Path),
    TryFrom(&'a Path),
    TryFromValidation(&'a Path, &'a Path),
    Map(&'a Path),
    MapValidation(&'a Path, &'a Path),
    FromStr,
    FromStrValidation(&'a Path),
}

impl<'a> ToTokens for MatchNone<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let obj_error_ty = bindings::ucl_object_error();
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

impl<'a> ToTokens for MatchSome<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let from_object = bindings::from_object_trait();
        let into_trait = bindings::into_trait();
        let try_into_trait = bindings::try_into_trait();
        let object_error_ty = bindings::ucl_object_error();
        let string_ty = bindings::string_ty();
        let from_str_trait = bindings::from_str_trait();
        let quote = match self {
            MatchSome::Simple => quote!(#from_object::try_from(obj)?),
            MatchSome::Validation(path) => quote!(
                let v = #from_object::try_from(obj)?;
                #path(&lookup_path, &v).map(|_| v)?
            ),
            MatchSome::From(src_type) => quote!(
                let v: #src_type = #from_object::try_from(obj)?;
                #into_trait::into(v)
            ),
            MatchSome::TryFrom(src_type) => quote!(
                let v: #src_type = #from_object::try_from(obj)
                        .map_err(|e| #object_error_ty::other(e))?;
                #try_into_trait::try_into(v)?
            ),
            MatchSome::FromValidation(src_type, validation) => quote!(
                 let v: #src_type = #from_object::try_from(obj)?;
                 let v = #into_trait::into(v);
                 #validation(&lookup_path, &v).map(|_| v)?
            ),
            MatchSome::TryFromValidation(src_type, validation) => quote!(
                let v: #src_type = #from_object::try_from(obj)?;
                let v = #try_into_trait::try_into(v)
                        .map_err(|e| #object_error_ty::other(e))?;
                #validation(&lookup_path, &v).map(|_| v)?
            ),
            MatchSome::Map(map_func) => quote!(
                #map_func(obj)?
            ),
            MatchSome::MapValidation(map_func, validation) => quote!(
                let v = #map_func(obj)?;
                #validation(&lookup_path, &v).map(|_| v)?
            ),
            MatchSome::FromStr => quote!(
                let v: #string_ty = #from_object::try_from(obj)?;
                #from_str_trait::from_str(&v)
                        .map_err(|e| #object_error_ty::other(e))?
            ),
            MatchSome::FromStrValidation(validation) => quote!(
                let v: #string_ty = #from_object::try_from(obj)?;
                let v = #from_str_trait::from_str(&v)
                        .map_err(|e| #object_error_ty::other(e))?;
                #validation(&lookup_path, &v).map(|_| v)?
            ),
        };
        tokens.append_all(quote);
    }
}
