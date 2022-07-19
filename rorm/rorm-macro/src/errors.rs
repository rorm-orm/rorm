//! Collect errors and return them all at once to the user

use std::cell::RefCell;
use std::fmt::Display;

use proc_macro2::Span;
use proc_macro2::TokenStream;

/// List of errors
///
/// Use this in any macro to collect occurring errors and put all of them into the resulting TokenStream in a single bulk.
///
/// To avoid worrying about mutability this type uses a RefCell and all its methods take immutable references.
/// Since errors are only ever pushed, order doesn't matter and macros are evaluated single threaded, this is fine.
///
/// ```ignore
/// fn some_macro(input: TokenStream) -> TokenStream {
///     let errors = Errors::new();
///     
///     // Fancy processing
///     ..
///
///     // Oh not found an error
///     errors.push_new(Span::call_site(), "Something went wrong");
///
///     // Futher processing
///     ..
///
///     // Report the errors inside the TokenStream
///     let errors = errors.into_compile_errors();
///     quote!{
///         // Fancy expansion
///         ..
///
///         #(#errors);*
///     }
/// }
pub struct Errors(RefCell<Vec<syn::Error>>);

impl Errors {
    pub fn new() -> Errors {
        Errors(RefCell::new(Vec::new()))
    }

    pub fn push(&self, value: syn::Error) {
        self.0.borrow_mut().push(value);
    }

    pub fn push_new<T: Display>(&self, span: Span, msg: T) {
        self.push(syn::Error::new(span, msg));
    }

    pub fn is_empty(&self) -> bool {
        self.0.borrow().is_empty()
    }

    pub fn into_compile_errors(self) -> impl Iterator<Item = TokenStream> {
        self.into_iter().map(syn::Error::into_compile_error)
    }
}

impl IntoIterator for Errors {
    type Item = syn::Error;
    type IntoIter = <Vec<syn::Error> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_inner().into_iter()
    }
}
