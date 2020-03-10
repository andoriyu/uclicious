use syn::Path;

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


/// UCL Parser
pub fn ucl_parser() -> Path {
    syn::parse_str("::ucilicious::raw::Parser").unwrap()
}