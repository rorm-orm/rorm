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
