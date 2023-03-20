#![cfg_attr(feature = "unstable", feature(proc_macro_span))]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
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

#[proc_macro_attribute]
pub fn rename_linkme(_args: TokenStream, item: TokenStream) -> TokenStream {
    let mut item = syn::parse_macro_input!(item as syn::ItemStatic);
    rename_linkme::rename_expr(&mut item.expr);
    item.into_token_stream().into()
}

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
