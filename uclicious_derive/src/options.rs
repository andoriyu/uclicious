use crate::block::Block;
use std::vec::IntoIter;
use darling::util::{Flag, PathList};
use darling::{self, ToTokens};
use syn::{Attribute, Generics, Ident, Visibility, Path};
use crate::builder::{Builder, BuildMethod, IntoBuilder, FromObject};
use proc_macro2::{Span, TokenStream};
use crate::initializer::Initializer;
use crate::parser::ParserMethods;
use crate::bindings;
use quote::TokenStreamExt;

#[derive(Debug, Clone, FromMeta)]
pub struct Variable {
    name: String,
    value: String
}
impl ToTokens for Variable {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let var = &self.name;
        let variable = &self.value;
        tokens.append_all(quote!(
            parser.register_variable(#var, #variable);
        ));
    }
}

#[derive(Debug, Clone, FromMeta, Default)]
pub struct FileVars {
    path: String,
    expand: Option<bool>,
}

#[derive(Debug, Clone, FromMeta, Default)]
pub struct Parser {
    #[darling(default)]
    flags: Option<Path>,
    filevars: Option<FileVars>,
}

impl ToTokens for Parser {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let parser_ty = bindings::ucl_parser();
        let parser_flags_ty = bindings::ucl_parser_flags_ty();
        if let Some(ref flags) = self.flags {
            tokens.append_all(quote!(
                    let flags: #parser_flags_ty = #flags();
                    let mut parser = #parser_ty::with_flags(flags);
                ));
        } else {
            let default_trait = bindings::default_trait();
            tokens.append_all(quote!(
                    let mut parser: #parser_ty = #default_trait::default();
                ));
        }
        if let Some(ref filevars) = self.filevars {
            let expand = filevars.expand.unwrap_or_default();
            let path = filevars.path.as_str();
            tokens.append_all(quote!(
                let _ = parser.set_filevars(#path, #expand)?;
            ));
        }
    }
}
#[derive(Debug, Clone, FromMeta)]
pub struct Include {
    path: String,
    #[darling(default)]
    priority: Option<u32>,
    #[darling(default)]
    strategy: Option<Path>,
}

impl ToTokens for Include {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let path = &self.path;
        let priority = self.priority.unwrap_or(0);
        let strategy = match self.strategy {
            Some(ref s) => s.clone(),
            None => bindings::ucl_default_strategy(),
        };
        let into_trait = bindings::into_trait();
        tokens.append_all(quote!(
            parser.add_file_full(#path, #into_trait::into(#priority), #strategy)?;
        ));
    }
}

trait FlagVisibility {
    fn public(&self) -> &Flag;
    fn private(&self) -> &Flag;

    /// Get the explicitly-expressed visibility preference from the attribute.
    /// This returns `None` if the input didn't include either keyword.
    ///
    /// # Panics
    /// This method panics if the input specifies both `public` and `private`.
    fn as_expressed_vis(&self) -> Option<Visibility> {
        match (self.public().is_some(), self.private().is_some()) {
            (true, true) => panic!("A field cannot be both public and private"),
            (true, false) => Some(syn::parse_str("pub").unwrap()),
            (false, true) => Some(Visibility::Inherited),
            (false, false) => None,
        }
    }
}

/// Contents of the `field` meta in `builder` attributes.
#[derive(Debug, Clone, Default, FromMeta)]
#[darling(default)]
pub struct FieldMeta {
    public: Flag,
    private: Flag,
}

impl FlagVisibility for FieldMeta {
    fn public(&self) -> &Flag {
        &self.public
    }

    fn private(&self) -> &Flag {
        &self.private
    }
}

#[derive(Debug, Clone, FromDeriveInput)]
#[darling(
    attributes(ucl),
    forward_attrs(doc, cfg, allow),
    supports(struct_named)
)]
pub struct Options {
    ident: Ident,
    attrs: Vec<Attribute>,
    vis: Visibility,
    generics: Generics,
    /// The name of the generated builder. Defaults to `#{ident}Builder`.
    #[darling(default)]
    name: Option<Ident>,

    #[darling(default)]
    build_fn: BuildFn,

    /// Additional traits to derive on the builder.
    #[darling(default)]
    derive: PathList,

    /// Struct-level value to use in place of any unfilled fields
    #[darling(default)]
    default: Option<DefaultExpression>,

    #[darling(default)]
    public: Flag,

    #[darling(default)]
    private: Flag,

    #[darling(default)]
    skip_builder: bool,

    /// The parsed body of the derived struct.
    data: darling::ast::Data<darling::util::Ignored, Field>,

    #[darling(default)]
    field: FieldMeta,

    #[darling(default, multiple)]
    include: Vec<Include>,
    #[darling(default)]
    parser: Parser,

    #[darling(default, multiple, rename = "var")]
    vars: Vec<Variable>,
}

/// Data extracted from the fields of the input struct.
#[derive(Debug, Clone, FromField)]
#[darling(attributes(ucl), forward_attrs(doc, cfg, allow))]
pub struct Field {
    ident: Option<Ident>,
    attrs: Vec<Attribute>,
    vis: syn::Visibility,
    ty: syn::Type,
    #[darling(default)]
    public: Flag,
    #[darling(default)]
    private: Flag,
    #[darling(default)]
    default: Option<DefaultExpression>,
    #[darling(default)]
    path: Option<String>,
}
impl FlagVisibility for Field {
    fn public(&self) -> &Flag {
        &self.public
    }

    fn private(&self) -> &Flag {
        &self.private
    }
}

impl Field {
    fn get_lookup_key(&self) -> String {
        match (&self.ident, &self.path) {
            (_, Some(path)) => path.clone(),
            (Some(ident), None) => ident.clone().to_string(),
            (_,_) => panic!("Can't figure out key path")
        }
    }
}

#[derive(Debug, Clone)]
pub enum DefaultExpression {
    Explicit(String),
    Trait,
}

impl DefaultExpression {
    pub fn parse_block(&self, no_std: bool) -> Block {
        let expr = match *self {
            DefaultExpression::Explicit(ref s) => {
                // We shouldn't hit this point in normal operation; the implementation
                // of `FromMeta` returns an error in this case so that the error points
                // at the empty expression rather than at the macro call-site.
                if s.is_empty() {
                    panic!(r#"Empty default expressions `default = ""` are not supported."#);
                }
                s
            }
            DefaultExpression::Trait => {
                if no_std {
                    "::core::default::Default::default()"
                } else {
                    "::std::default::Default::default()"
                }
            }
        };

        expr.parse()
            .expect(&format!("Couldn't parse default expression `{:?}`", self))
    }
}

impl darling::FromMeta for DefaultExpression {
    fn from_word() -> darling::Result<Self> {
        Ok(DefaultExpression::Trait)
    }

    fn from_string(value: &str) -> darling::Result<Self> {
        if value.is_empty() {
            Err(darling::Error::unknown_value(""))
        } else {
            Ok(DefaultExpression::Explicit(value.into()))
        }
    }
}

impl FlagVisibility for Options {
    fn public(&self) -> &Flag {
        &self.public
    }

    fn private(&self) -> &Flag {
        &self.private
    }
}

/// Options for the `build_fn` property in struct-level builder options.
/// There is no inheritance for these settings from struct-level to field-level,
/// so we don't bother using `Option` for values in this struct.
#[derive(Debug, Clone, FromMeta)]
#[darling(default)]
pub struct BuildFn {
    skip: bool,
    name: Ident,
    validate: Option<Path>,
    public: Flag,
    private: Flag,
}

impl Default for BuildFn {
    fn default() -> Self {
        BuildFn {
            skip: false,
            name: Ident::new("build", Span::call_site()),
            validate: None,
            public: Default::default(),
            private: Default::default(),
        }
    }
}

impl FlagVisibility for BuildFn {
    fn public(&self) -> &Flag {
        &self.public
    }

    fn private(&self) -> &Flag {
        &self.private
    }
}


impl Options {
    pub fn skip_builder(&self) -> bool {
        self.skip_builder
    }
    pub fn builder_ident(&self) -> Ident {
        if let Some(ref custom) = self.name {
            return custom.clone();
        }

        syn::parse_str(&format!("{}Builder", self.ident))
            .expect("Struct name with Builder suffix should be an ident")
    }

    /// The visibility of the builder struct.
    /// If a visibility was declared in attributes, that will be used;
    /// otherwise the struct's own visibility will be used.
    pub fn builder_vis(&self) -> Visibility {
        self.as_expressed_vis().unwrap_or_else(|| self.vis.clone())
    }

    /// Get the visibility of the emitted `build` method.
    /// This defaults to the visibility of the parent builder, but can be overridden.
    pub fn build_method_vis(&self) -> Visibility {
        self.build_fn
            .as_expressed_vis()
            .unwrap_or_else(|| self.builder_vis())
    }

    pub fn raw_fields(&self) -> Vec<&Field> {
        self.data
            .as_ref()
            .take_struct()
            .expect("Only structs supported")
            .fields
    }
    pub fn field_count(&self) -> usize {
        self.raw_fields().len()
    }
    /// Get an iterator over the input struct's fields which pulls fallback
/// values from struct-level settings.
    pub fn fields(&self) -> FieldIter {
        FieldIter(self, self.raw_fields().into_iter())
    }

    pub fn as_from_object(&self) -> FromObject {
        FromObject {
            target_ty: self.ident.clone(),
            generics: Some(&self.generics),
            initializers: Vec::with_capacity(self.field_count()),
            default_struct: self
                .default
                .as_ref()
                .map(|x| x.parse_block(false)),
        }
    }
    pub fn as_builder(&self) -> Builder {
        Builder {
            ident: self.builder_ident(),
            generics: Some(&self.generics),
            visibility: self.builder_vis(),
            fields: Vec::with_capacity(self.field_count()),
            functions: Vec::with_capacity(self.field_count()),
            doc_comment: None,
            includes: self.include.clone(),
            parser: &self.parser,
            vars: self.vars.clone(),
        }
    }
    pub fn as_build_method(&self) -> BuildMethod {
        let (_, ty_generics, _) = self.generics.split_for_impl();
        BuildMethod {
            ident: &self.build_fn.name,
            visibility: self.build_method_vis(),
            target_ty: &self.ident,
            target_ty_generics: Some(ty_generics),
            initializers: Vec::with_capacity(self.field_count()),
            doc_comment: None,
            default_struct: self
                .default
                .as_ref()
                .map(|x| x.parse_block(false)),
            validate_fn: self.build_fn.validate.as_ref(),
        }
    }
    pub fn as_parser_methods(&self) -> ParserMethods {
        ParserMethods {
            visibility: self.build_method_vis()
        }
    }

    pub fn as_into_builder(&self) -> IntoBuilder {
        IntoBuilder {
            ident: self.builder_ident(),
            visibility: self.build_method_vis(),
            target_ty: &self.ident,
            generics: Some(&self.generics),
        }
    }
}

pub struct FieldIter<'a>(&'a Options, IntoIter<&'a Field>);

impl<'a> Iterator for FieldIter<'a> {
    type Item = FieldWithDefaults<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.1.next().map(|field| FieldWithDefaults {
            parent: self.0,
            field,
        })
    }
}


/// Accessor for field data which can pull through options from the parent
/// struct.
pub struct FieldWithDefaults<'a> {
    parent: &'a Options,
    field: &'a Field,
}
impl<'a> FieldWithDefaults<'a> {
    /// Get the ident of the input field. This is also used as the ident of the
    /// emitted field.
    pub fn field_ident(&self) -> &syn::Ident {
        self.field
            .ident
            .as_ref()
            .expect("Tuple structs are not supported")
    }

    #[allow(unused)]
    pub fn field_vis(&self) -> Visibility {
        self.field
            .as_expressed_vis()
            .or_else(|| self.parent.field.as_expressed_vis())
            .unwrap_or(Visibility::Inherited)
    }
    pub fn use_parent_default(&self) -> bool {
        self.field.default.is_none() && self.parent.default.is_some()
    }
   /// Returns an `Initializer` according to the options.
   ///
   /// # Panics
   ///
   /// if `default_expression` can not be parsed as `Block`.
    pub fn as_initializer(&'a self) -> Initializer<'a> {
        Initializer {
            field_ident: self.field_ident(),
            default_value: self
                .field
                .default
                .as_ref()
                .map(|x| x.parse_block(false)),
            use_default_struct: self.use_parent_default(),
            lookup_path: self.field.get_lookup_key(),
        }
    }
}