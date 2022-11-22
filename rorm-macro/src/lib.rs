//! Implementation of the Model attribute used to implement database things for structs
#![cfg_attr(feature = "unstable", feature(proc_macro_span))]
extern crate proc_macro;
use proc_macro::TokenStream;

use proc_macro2::Span;
use quote::{quote, ToTokens};

mod annotations;
mod derive;
mod rename_linkme;
mod trait_impls;
mod utils;

#[proc_macro_derive(DbEnum)]
pub fn derive_db_enum(input: TokenStream) -> TokenStream {
    match derive::db_enum(input.into()) {
        Ok(tokens) => tokens,
        Err(error) => error.write_errors(),
    }
    .into()
}

/// This attribute is used to turn a struct into a database model.
///
/// ```ignore
/// use rorm::Model;
///
/// #[derive(Model)]
/// struct User {
///     #[rorm(id)]
///     id: i32,
///     #[rorm(max_length = 255, unique)]
///     username: String,
///     #[rorm(max_length = 255)]
///     password: String,
///     #[rorm(default = false)]
///     admin: bool,
///     age: u8,
///     #[rorm(choices("m", "f", "d"))]
///     gender: String,
/// }
/// ```
#[proc_macro_derive(Model, attributes(rorm))]
pub fn derive_model(input: TokenStream) -> TokenStream {
    match derive::model(input.into()) {
        Ok(tokens) => tokens,
        Err(error) => error.write_errors(),
    }
    .into()
}

#[proc_macro_derive(Patch, attributes(rorm))]
pub fn derive_patch(input: TokenStream) -> TokenStream {
    match derive::patch(input.into()) {
        Ok(tokens) => tokens,
        Err(error) => error.write_errors(),
    }
    .into()
}

#[doc(hidden)]
#[proc_macro_attribute]
pub fn rename_linkme(_args: TokenStream, item: TokenStream) -> TokenStream {
    let mut item = syn::parse_macro_input!(item as syn::ItemStatic);
    rename_linkme::rename_expr(&mut item.expr);
    item.into_token_stream().into()
}

/// This attribute is put on your main function.
///
/// When you build with the `rorm-main` feature enabled this attribute will replace your main function.
/// The new main function will simply write all your defined models to `./.models.json`
/// to be further process by the migrator.
///
/// Make sure you have added the feature `rorm-main` to your crate i.e. put the following in your `Cargo.toml`:
/// ```toml
/// [features]
/// rorm-main = []
/// ```
///
/// If you don't like this feature name you can pass the attribute any other name to use instead:
/// ```ignore
/// use rorm::rorm_main;
///
/// #[rorm_main("other-name")]
/// fn main() {}
/// ```
#[proc_macro_attribute]
pub fn rorm_main(args: TokenStream, item: TokenStream) -> TokenStream {
    let main = syn::parse_macro_input!(item as syn::ItemFn);
    let feature = syn::parse::<syn::LitStr>(args)
        .unwrap_or_else(|_| syn::LitStr::new("rorm-main", Span::call_site()));

    (if main.sig.ident == "main" {
        quote! {
            #[cfg(feature = #feature)]
            fn main() -> Result<(), String> {
                let mut file = ::std::fs::File::create(".models.json").map_err(|err| err.to_string())?;
                ::rorm::write_models(&mut file)?;
                return Ok(());
            }
            #[cfg(not(feature = #feature))]
            #main
        }
    } else {
        quote! {
            compile_error!("only allowed on main function");
            #main
        }
    }).into()
}