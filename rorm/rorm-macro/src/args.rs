//! Structs for parsing function-like macro's arguments.
//!
//! - `Macro[N]Args` is used to parse exactly `N` arguments of different AST node types:
//! ```ignore
//! /// Usage: `assign!(foo, 1 + 2);`
//! /// Expansion: `let foo = 1 + 2;`
//! #[proc_macro]
//! pub fn assign(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
//!     let Macro2Args (var, expr): Macro2Args<syn::Ident, syn::Expr>
//!         = syn::parse_macro_input!(input);
//!
//!     (quote::quote!{
//!         let #var = #expr;
//!     }).into()
//! }
//! ```
//!
//! - `ModelFields` is a draft for specifying a list of a model's fields:
//!   `some::path::to::Model` or `some::path::to::Model::{some_field, another_field}`
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
};

type Comma = syn::Token!(,);
macro_rules! impl_n_args {
    ($name:ident<$generic:ident $(, $generics:ident)*>) => {
        pub struct $name<$generic $(, $generics)*>(pub $generic $(, pub $generics)*);
        impl<$generic: Parse $(, $generics: Parse)*> Parse for $name<$generic $(, $generics)*> {
            #[allow(non_snake_case)]
            fn parse(input: ParseStream) -> syn::Result<Self> {
                let $generic = $generic::parse(input)?;
                $(
                    Comma::parse(input)?;
                    let $generics = $generics::parse(input)?;
                )*
                Ok($name($generic $(, $generics)*))
            }
        }
    };
}

impl_n_args!(Macro2Args<A, B>);
impl_n_args!(Macro3Args<A, B, C>);
impl_n_args!(Macro4Args<A, B, C, D>);
impl_n_args!(Macro5Args<A, B, C, D, E>);

pub struct ModelFields {
    pub model: syn::Path,
    pub fields: Vec<syn::Ident>,
}
impl Parse for ModelFields {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        syn::UseTree::parse(input)?.try_into()
    }
}
impl TryFrom<syn::UseTree> for ModelFields {
    type Error = syn::Error;

    fn try_from(mut value: syn::UseTree) -> Result<Self, Self::Error> {
        use syn::UseTree::*;
        let mut remaining_tree = &mut value;
        let mut model_fields = ModelFields {
            model: syn::Path {
                leading_colon: None,
                segments: syn::punctuated::Punctuated::new(),
            },
            fields: Vec::new(),
        };
        let path = &mut model_fields.model;
        match remaining_tree {
            Path(syn::UsePath { ident, tree, .. }) => {
                path.segments.push(ident.clone().into());
                remaining_tree = tree;
            }
            Name(syn::UseName { ident }) => {
                path.segments.push(ident.clone().into());
                return Ok(model_fields);
            }
            invalid => return Err(syn::Error::new(invalid.span(), "")),
        }
        loop {
            match remaining_tree {
                Path(syn::UsePath { ident, tree, .. }) => {
                    path.segments.push(ident.clone().into());
                    remaining_tree = tree;
                }
                Name(syn::UseName { ident }) => {
                    path.segments.push(ident.clone().into());
                    return Ok(model_fields);
                }
                Group(syn::UseGroup { items, .. }) => {
                    for tree in items.into_iter() {
                        match tree {
                            Name(syn::UseName { ident }) => {
                                model_fields.fields.push(ident.clone());
                            }
                            invalid => return Err(syn::Error::new(invalid.span(), "")),
                        }
                    }
                    return Ok(model_fields);
                }
                invalid => return Err(syn::Error::new(invalid.span(), "")),
            }
        }
    }
}
