use crate::errors::Errors;
use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

pub fn to_db_name(name: String) -> String {
    let mut name = name;
    name.make_ascii_lowercase();
    name
}

/// Create the expression for creating a Option<Source> instance from a span
#[cfg(feature = "unstable")]
pub fn get_source<T: Spanned>(spanned: &T) -> TokenStream {
    let span = spanned.span().unwrap();
    let file = proc_macro2::Literal::string(&span.source_file().path().display().to_string());
    let line = proc_macro2::Literal::usize_unsuffixed(span.start().line);
    let column = proc_macro2::Literal::usize_unsuffixed(span.start().column);
    quote! {
        Some(::rorm::model::Source {
            file: #file,
            line: #line,
            column: #column,
        })
    }
}
#[cfg(not(feature = "unstable"))]
pub fn get_source<T: Spanned>(_spanned: &T) -> TokenStream {
    quote! {None}
}

/// Iterate over all "arguments" inside any #[rorm(..)] attribute
///
/// It enforces the rorm attributes to look like function calls (see [syn::Meta::List])
/// as well as excluding literals as their direct arguments (see [syn::NestedMeta::lit])
pub fn iter_rorm_attributes<'a>(
    attrs: &'a Vec<syn::Attribute>,
    errors: &'a Errors,
) -> impl Iterator<Item = syn::Meta> + 'a {
    attrs
        .iter()
        .filter(|attr| attr.path.is_ident("rorm"))
        .map(syn::Attribute::parse_meta)
        .map(Result::ok)
        .flatten()
        .map(|meta| match meta {
            syn::Meta::List(syn::MetaList { nested, .. }) => Some(nested.into_iter()),
            _ => {
                errors.push_new(meta.span(), "Attribute should be of shape: `rorm(..)`");
                None
            }
        })
        .flatten()
        .flatten()
        .map(|nested_meta| match nested_meta {
            syn::NestedMeta::Meta(meta) => Some(meta),
            syn::NestedMeta::Lit(_) => {
                errors.push_new(
                    nested_meta.span(),
                    "`rorm(..)` doesn't take literals directly",
                );
                None
            }
        })
        .flatten()
}
