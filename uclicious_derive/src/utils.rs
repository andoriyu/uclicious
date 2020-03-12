use syn::Attribute;

pub fn doc_comment_from(s: String) -> Attribute {
    parse_quote!(#[doc=#s])
}
