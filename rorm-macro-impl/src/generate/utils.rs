use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::Generics;

use crate::MacroConfig;

/// Creates a ZST which captures all generics
///
/// I.e. `PhantomData<(&'a (), T)>`
pub fn phantom_data(generics: &Generics) -> TokenStream {
    let mut tokens = TokenStream::new();
    tokens.extend(generics.lifetimes().map(|lifetime| {
        let lifetime = &lifetime.lifetime;
        quote! { & #lifetime (), }
    }));
    tokens.extend(generics.type_params().map(|parameter| {
        let parameter = &parameter.ident;
        quote! { #parameter, }
    }));
    quote! {
        ::std::marker::PhantomData<( #tokens )>
    }
}

/// Creates an expression for a `Source` instance from a span
pub fn get_source(span: Span, config: &MacroConfig) -> TokenStream {
    let MacroConfig { rorm_path } = config;
    quote_spanned! {span=>
        #rorm_path::internal::hmr::Source {
            file: ::std::file!(),
            line: ::std::line!() as usize,
            column: ::std::column!() as usize,
        }
    }
}
