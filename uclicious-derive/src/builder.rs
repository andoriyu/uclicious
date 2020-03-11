use crate::{bindings, DEFAULT_STRUCT_NAME};
use crate::block::Block;
use crate::utils::doc_comment_from;
use darling::ToTokens;
use proc_macro2::{TokenStream, Span};
use quote::TokenStreamExt;
use syn::punctuated::Punctuated;
use syn::{Path};
use crate::initializer::Initializer;

pub struct Builder<'a> {
    /// Name of this builder struct.
    pub ident: syn::Ident,
    /// Type parameters and lifetimes attached to this builder's struct
    /// definition.
    pub generics: Option<&'a syn::Generics>,
    /// Visibility of the builder struct, e.g. `syn::Visibility::Public`.
    pub visibility: syn::Visibility,
    /// Fields of the builder struct, e.g. `foo: u32,`
    ///
    /// Expects each entry to be terminated by a comma.
    pub fields: Vec<TokenStream>,
    /// Functions of the builder struct, e.g. `fn bar() -> { unimplemented!() }`
    pub functions: Vec<TokenStream>,
    /// Doc-comment of the builder struct.
    pub doc_comment: Option<syn::Attribute>,
}

impl<'a> Builder<'a> {
    /// Set a doc-comment for this item.
    pub fn doc_comment(&mut self, s: String) -> &mut Self {
        self.doc_comment = Some(doc_comment_from(s));
        self
    }

    /// Add a field to the builder
    pub fn push_field<T: ToTokens>(&mut self, f: &T) -> &mut Self {
        self.fields.push(quote!(#f));
        self
    }
    /// Add final build function to the builder
    pub fn push_method<T: ToTokens>(&mut self, f: &T) -> &mut Self {
        self.functions.push(quote!(#f));
        self
    }

    /// Add `Clone` trait bound to generic types for non-owned builders.
    /// This enables target types to declare generics without requiring a
    /// `Clone` impl. This is the same as how the built-in derives for
    /// `Clone`, `Default`, `PartialEq`, and other traits work.
    fn compute_impl_bounds(&self) -> syn::Generics {
        if let Some(type_gen) = self.generics {
            let generics = type_gen.clone();
            return generics;
        } else {
            Default::default()
        }
    }
}

#[derive(Debug)]
pub struct BuildMethod<'a> {
    /// Name of this build fn.
    pub ident: &'a syn::Ident,
    /// Visibility of the build method, e.g. `syn::Visibility::Public`.
    pub visibility: syn::Visibility,
    /// Type of the target field.
    ///
    /// The corresonding builder field will be `Option<field_type>`.
    pub target_ty: &'a syn::Ident,
    /// Type parameters and lifetimes attached to this builder struct.
    pub target_ty_generics: Option<syn::TypeGenerics<'a>>,
    /// Field initializers for the target type.
    pub initializers: Vec<TokenStream>,
    /// Doc-comment of the builder struct.
    pub doc_comment: Option<syn::Attribute>,
    /// Default value for the whole struct.
    ///
    /// This will be in scope for all initializers as `__default`.
    pub default_struct: Option<Block>,
    /// Validation function with signature `&FooBuilder -> Result<(), String>`
    /// to call before the macro-provided struct buildout.
    pub validate_fn: Option<&'a syn::Path>,
}

impl<'a> FromObject<'a> {
    pub fn push_initializer(&mut self, init: Initializer) -> &mut Self {
        self.initializers.push(quote!(#init));
        self
    }
}

#[derive(Debug, Clone)]
pub struct IntoBuilder<'a> {
    /// Name of this build fn.
    pub ident: syn::Ident,
    /// Visibility of the build method, e.g. `syn::Visibility::Public`.
    pub visibility: syn::Visibility,
    /// Type of the target field.
    ///
    /// The corresonding builder field will be `Option<field_type>`.
    pub target_ty: &'a syn::Ident,
}

pub struct FromObject<'a> {
    /// Type of the target
    pub target_ty: syn::Ident,
    /// Type parameters and lifetimes attached to target type
    pub generics: Option<&'a syn::Generics>,
    /// Field initializers for the target type.
    pub initializers: Vec<TokenStream>,
    /// Default value for the whole struct.
    ///
    /// This will be in scope for all initializers as `__default`.
    pub default_struct: Option<Block>,
}

impl< 'a > ToTokens for FromObject < 'a > {
    fn to_tokens( & self, tokens: & mut TokenStream) {
        let target_ty = &self.target_ty;
        let target_ty_generics = &self.generics;
        let initializers = &self.initializers;
        let default_struct = self.default_struct.as_ref().map(|default_expr| {
            let ident = syn::Ident::new(DEFAULT_STRUCT_NAME, Span::call_site());
            quote!(let #ident: #target_ty #target_ty_generics = #default_expr;)
        });

        let result = bindings::result_ty();
        let error_ty = bindings::ucl_object_error();
        let try_from = bindings::try_from_trait();
        let try_into = bindings::try_into_trait();
        let obj_ref_ty = bindings::ucl_object_ref_ty();
        let obj_ty = bindings::ucl_object_ty();
        let borrow = bindings::borrow_trait();
        let as_ref = bindings::as_ref_trait();

        let deref = bindings::deref_trait();
        tokens.append_all(quote!(
            impl #try_from<&#obj_ref_ty> for #target_ty #target_ty_generics {
                type Error = #error_ty;
                fn try_from(root: &#obj_ref_ty) -> #result<Self, Self::Error> {
                    Ok(#target_ty {
                            #(#initializers)*
                    })
                }
            }
            impl #try_from<#obj_ref_ty> for #target_ty #target_ty_generics {
                type Error = #error_ty;
                fn try_from(source: #obj_ref_ty) -> #result<Self, Self::Error> {
                    #try_into::try_into(&source)
                }
            }
            impl #try_from<#obj_ty> for #target_ty #target_ty_generics {
                type Error = #error_ty;
                fn try_from(source: #obj_ty) -> #result<Self, Self::Error> {
                    let obj: &#obj_ref_ty = #borrow::borrow(&source);
                    #try_into::try_into(obj)
                }
            }
        ))
    }
}
impl< 'a > ToTokens for BuildMethod < 'a > {
    fn to_tokens( & self, tokens: & mut TokenStream) {
        let ident = self.ident;
        let vis = &self.visibility;
        let target_ty = &self.target_ty;
        let target_ty_generics = &self.target_ty_generics;
        let initializers = &self.initializers;
        let doc_comment = &self.doc_comment;
        let default_struct = self.default_struct.as_ref().map(|default_expr| {
            let ident = syn::Ident::new(DEFAULT_STRUCT_NAME, Span::call_site());
            quote!(let #ident: #target_ty #target_ty_generics = #default_expr;)
        });
        let result = bindings::result_ty();
        let boxed_error = bindings::boxed_error();
        let ucl_error_ty = bindings::ucl_parser_error();
        let ucl_obj_error_ty = bindings::ucl_object_error();
        let try_into = bindings::try_into_trait();
        let into = bindings::into_trait();
        tokens.append_all(quote!(
            #doc_comment
            #vis fn #ident(mut self) -> #result<#target_ty #target_ty_generics, #boxed_error> {
                #default_struct
                let root = self.__parser.get_object().map_err(|e: #ucl_error_ty| e.boxed() as #boxed_error)?;
                #try_into::try_into(root).map_err(|e: #ucl_obj_error_ty| e.boxed() as #boxed_error)
            }
        ))
    }
}
impl< 'a > ToTokens for Builder < 'a > {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let builder_vis = &self.visibility;
        let builder_ident = &self.ident;
        let bounded_generics = self.compute_impl_bounds();
        let (impl_generics, _, _) = bounded_generics.split_for_impl();
        let (struct_generics, ty_generics, where_clause) = self
            .generics
            .map(syn::Generics::split_for_impl)
            .map(|(i, t, w)| (Some(i), Some(t), Some(w)))
            .unwrap_or((None, None, None));
        let builder_fields = &self.fields;
        let functions = &self.functions;
        let derived_traits = {
            let default_trait: Path = parse_quote!(Default);

            let mut traits: Punctuated<&Path, Token![,]> = Default::default();
            traits.push(&default_trait);
            quote!(#traits)
        };
        let builder_doc_comment = &self.doc_comment;
        tokens.append_all(quote!(
                #[derive(#derived_traits)]
                #builder_doc_comment
                #builder_vis struct #builder_ident #struct_generics #where_clause {
                    #(#builder_fields)*
                }

                #[allow(dead_code)]
                impl #impl_generics #builder_ident #ty_generics #where_clause {
                    #(#functions)*
                }
            ));
    }
}


impl<'a> ToTokens for IntoBuilder<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let builder_vis = &self.visibility;
        let builder_ident = &self.ident;
        let target = &self.target_ty;
        let parser = bindings::ucl_parser();
        tokens.append_all(quote!(
        impl #target {
            #builder_vis fn builder() -> #builder_ident {
                #builder_ident::default()
            }

            #builder_vis fn builder_with_parser(parser: #parser) -> #builder_ident {
                #builder_ident {
                    __parser: parser,
                }
            }
        }
        ));

    }
}
