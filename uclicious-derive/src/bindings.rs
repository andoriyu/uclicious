use syn::{Path, Type};

pub fn string_ty() -> Path {
    syn::parse_str("::std::string::String").unwrap()
}

/// Result type.
pub fn result_ty() -> Path {
    syn::parse_str("::std::result::Result").unwrap()
}

/// Option type.
pub fn option_ty() -> Path {
    syn::parse_str("::std::option::Option").unwrap()
}

/// PhantomData type.
pub fn phantom_data_ty() -> Path {
    syn::parse_str("::std::marker::PhantomData").unwrap()
}

/// Default trait.
pub fn default_trait() -> Path {
    syn::parse_str("::std::default::Default").unwrap()
}

/// Clone trait.
pub fn clone_trait() -> Path {
    syn::parse_str("::std::clone::Clone").unwrap()
}

/// Into trait.
#[allow(clippy::wrong_self_convention)]
pub fn into_trait() -> Path {
    syn::parse_str("::std::convert::Into").unwrap()
}

/// TryInto trait.
pub fn try_into_trait() -> Path {
    syn::parse_str("::std::convert::TryInto").unwrap()
}

/// Boxed error type
pub fn boxed_error() -> Type {
    syn::parse_str("::std::boxed::Box<dyn ::std::error::Error>").unwrap()
}

/// UCL Parser
pub fn ucl_parser() -> Path {
    syn::parse_str("::uclicious::Parser").unwrap()
}
/// UCL Parser Error
pub fn ucl_parser_error() -> Path {
    syn::parse_str("::uclicious::UclError").unwrap()
}

/// UCL Object Error
pub fn ucl_object_error() -> Path {
    syn::parse_str("::uclicious::ObjectError").unwrap()
}


pub fn as_ref_trait() -> Path {
    syn::parse_str("::std::convert::AsRef").unwrap()
}

pub fn ucilicous_priority_type() -> Path {
    syn::parse_str("::uclicious::Priority").unwrap()
}
pub fn ucilicous_duplicate_strategy_type() -> Path {
    syn::parse_str("::uclicious::DuplicateStrategy").unwrap()
}

pub fn path_ty() -> Path {
    syn::parse_str("::std::path::Path").unwrap()
}
