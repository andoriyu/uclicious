use crate::block::Block;
use std::vec::IntoIter;
use darling::util::{Flag, PathList};
use darling::{self};
use syn::{Attribute, Generics, Ident, Visibility, Path};
use crate::builder::{Builder, BuildMethod, BuilderField};
use proc_macro2::Span;

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

    /// The parsed body of the derived struct.
    data: darling::ast::Data<darling::util::Ignored, Field>,

    #[darling(default)]
    field: FieldMeta,
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
}
impl FlagVisibility for Field {
    fn public(&self) -> &Flag {
        &self.public
    }

    fn private(&self) -> &Flag {
        &self.private
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

    pub fn as_builder(&self) -> Builder {
        Builder {
            ident: self.builder_ident(),
            generics: Some(&self.generics),
            visibility: self.builder_vis(),
            fields: Vec::with_capacity(self.field_count()),
            functions: Vec::with_capacity(self.field_count()),
            doc_comment: None,
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
    pub fn field_enabled(&self) -> bool {
        true
    }
    /// Get the ident of the input field. This is also used as the ident of the
    /// emitted field.
    pub fn field_ident(&self) -> &syn::Ident {
        self.field
            .ident
            .as_ref()
            .expect("Tuple structs are not supported")
    }

    pub fn field_vis(&self) -> Visibility {
        self.field
            .as_expressed_vis()
            .or_else(|| self.parent.field.as_expressed_vis())
            .unwrap_or(Visibility::Inherited)
    }
    pub fn use_parent_default(&self) -> bool {
        self.field.default.is_none() && self.parent.default.is_some()
    }
    pub fn as_builder_field(&'a self) -> BuilderField<'a> {
        BuilderField {
            field_ident: self.field_ident(),
            field_type: &self.field.ty,
            field_visibility: self.field_vis(),
            attrs: &self.field.attrs,
        }
    }
}