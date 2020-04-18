use crate::block::Block;
use crate::initializer::Initializer;
use crate::options::{Include, Parser, Variable};
use crate::{bindings, DEFAULT_STRUCT_NAME};
use darling::ToTokens;
use proc_macro2::{Span, TokenStream};
use quote::TokenStreamExt;
use syn::punctuated::Punctuated;
use syn::Path;

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
    pub includes: Vec<Include>,
    pub parser: &'a Parser,
    pub vars: Vec<Variable>,
}

impl<'a> Builder<'a> {
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
            type_gen.clone()
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
    /// Type parameters and lifetimes attached to this builder's struct
    /// definition.
    pub generics: Option<&'a syn::Generics>,
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

impl<'a> ToTokens for FromObject<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let target_ty = &self.target_ty;
        let target_ty_generics = &self.generics;
        let initializers = &self.initializers;

        let result = bindings::result_ty();
        let error_ty = bindings::ucl_object_error();
        let try_from = bindings::from_object_trait();
        let obj_ref_ty = bindings::ucl_object_ref_ty();
        let obj_ty = bindings::ucl_object_ty();
        let borrow = bindings::borrow_trait();

        tokens.append_all(quote!(
            impl #try_from<&#obj_ref_ty> for #target_ty #target_ty_generics {
                fn try_from(root: &#obj_ref_ty) -> #result<Self, #error_ty> {
                    Ok(#target_ty {
                            #(#initializers)*
                    })
                }
            }
            impl #try_from<#obj_ref_ty> for #target_ty #target_ty_generics {
                fn try_from(source: #obj_ref_ty) -> #result<Self, #error_ty> {
                    #try_from::try_from(&source)
                }
            }
            impl #try_from<#obj_ty> for #target_ty #target_ty_generics {
                fn try_from(source: #obj_ty) -> #result<Self, #error_ty> {
                    let obj: &#obj_ref_ty = #borrow::borrow(&source);
                    #try_from::try_from(obj)
                }
            }
        ))
    }
}
impl<'a> ToTokens for BuildMethod<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let ident = self.ident;
        let vis = &self.visibility;
        let target_ty = &self.target_ty;
        let target_ty_generics = &self.target_ty_generics;
        let default_struct = self.default_struct.as_ref().map(|default_expr| {
            let ident = syn::Ident::new(DEFAULT_STRUCT_NAME, Span::call_site());
            quote!(let #ident: #target_ty #target_ty_generics = #default_expr;)
        });
        let result = bindings::result_ty();
        let boxed_error = bindings::boxed_error();
        let ucl_error_ty = bindings::ucl_parser_error();
        let ucl_obj_error_ty = bindings::ucl_object_error();
        let from_obj = bindings::from_object_trait();
        tokens.append_all(quote!(
            #[doc = "Build target struct or return first encountered error."]
            #vis fn #ident(mut self) -> #result<#target_ty #target_ty_generics, #boxed_error> {
                #default_struct
                let root = self.__parser.get_object().map_err(|e: #ucl_error_ty| e.boxed() as #boxed_error)?;
                #from_obj::try_from(root).map_err(|e: #ucl_obj_error_ty| e.boxed() as #boxed_error)
            }
        ))
    }
}
impl<'a> ToTokens for Builder<'a> {
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
            let traits: Punctuated<&Path, Token![,]> = Default::default();
            quote!(#traits)
        };
        let includes: Vec<TokenStream> =
            self.includes.iter().map(|e| e.to_token_stream()).collect();
        let vars: Vec<TokenStream> = self.vars.iter().map(ToTokens::to_token_stream).collect();
        let builder_doc_comment = &self.doc_comment;
        let result_ty = bindings::result_ty();
        let ucl_error_ty = bindings::ucl_parser_error();
        let parser = self.parser;
        tokens.append_all(quote!(
                #[derive(#derived_traits)]
                #builder_doc_comment
                #builder_vis struct #builder_ident #struct_generics #where_clause {
                    #(#builder_fields)*
                }

                #[allow(dead_code)]
                impl #impl_generics #builder_ident #ty_generics #where_clause {
                    #(#functions)*
                    /// Create a new builder.
                    #builder_vis fn new() -> #result_ty<Self #ty_generics #where_clause, #ucl_error_ty> {
                        #parser
                        #(#vars)*
                        #(#includes)*
                        Ok(
                            Self {
                                __parser: parser
                            }
                        )
                    }
                }
            ));
    }
}

impl<'a> ToTokens for IntoBuilder<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let builder_vis = &self.visibility;
        let builder_ident = &self.ident;
        let target = &self.target_ty;
        let result_ty = bindings::result_ty();
        let ucl_error_ty = bindings::ucl_parser_error();
        let (_struct_generics, ty_generics, where_clause) = self
            .generics
            .map(syn::Generics::split_for_impl)
            .map(|(i, t, w)| (Some(i), Some(t), Some(w)))
            .unwrap_or((None, None, None));
        tokens.append_all(quote!(
            impl #target {
                /// Creates a builder struct that can be used to create this struct.
                #builder_vis fn builder() -> #result_ty<#builder_ident #ty_generics #where_clause, #ucl_error_ty> {
                    #builder_ident::new()
                }
            }
        ));
    }
}
