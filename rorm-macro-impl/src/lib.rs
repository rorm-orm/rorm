//! This crate tries to follow the base layout proposed by a [ferrous-systems.com](https://ferrous-systems.com/blog/testing-proc-macros/#the-pipeline) blog post.

use proc_macro2::TokenStream;
use quote::quote;

use crate::analyze::field_type::analyze_field_type;
use crate::analyze::model::analyze_model;
use crate::generate::db_enum::generate_db_enum;
use crate::generate::field_type::generate_field_type;
use crate::generate::model::generate_model;
use crate::generate::patch::generate_patch;
use crate::parse::db_enum::parse_db_enum;
use crate::parse::field_type::parse_field_type;
use crate::parse::model::parse_model;
use crate::parse::patch::parse_patch;

mod analyze;
mod generate;
mod parse;
mod utils;

/// Implementation of `rorm`'s `#[derive(DbEnum)]` macro
pub fn derive_db_enum(input: TokenStream, config: MacroConfig) -> TokenStream {
    match parse_db_enum(input) {
        Ok(model) => generate_db_enum(&model, &config),
        Err(error) => error.write_errors(),
    }
}

/// Implementation of `rorm`'s `#[derive(Model)]` macro
pub fn derive_model(input: TokenStream, config: MacroConfig) -> TokenStream {
    match parse_model(input).and_then(analyze_model) {
        Ok(model) => generate_model(&model, &config),
        Err(error) => error.write_errors(),
    }
}

/// Implementation of `rorm`'s `#[derive(Patch)]` macro
pub fn derive_patch(input: TokenStream, config: MacroConfig) -> TokenStream {
    match parse_patch(input) {
        Ok(patch) => generate_patch(&patch, &config),
        Err(error) => error.write_errors(),
    }
}

/// Implementation of `rorm`'s `#[derive(FieldType)]` macro
pub fn derive_field_type(input: TokenStream, config: MacroConfig) -> TokenStream {
    match parse_field_type(input).and_then(analyze_field_type) {
        Ok(patch) => generate_field_type(&patch, &config),
        Err(error) => error.write_errors(),
    }
}

/// Configuration for `rorm`'s macros
///
/// This struct can be useful for other crates wrapping `rorm`'s macros to tweak their behaviour.
#[cfg_attr(doc, non_exhaustive)]
pub struct MacroConfig {
    /// Path to the `rorm` crate
    ///
    /// This path can be overwritten by another library which wraps and re-exports `rorm`.
    ///
    /// Defaults to `::rorm` which requires `rorm` to be a direct dependency
    /// of any crate using `rorm`'s macros.
    pub rorm_path: TokenStream,

    #[cfg(not(doc))]
    pub non_exhaustive: private::NonExhaustive,
}

mod private {
    pub struct NonExhaustive;
}

impl Default for MacroConfig {
    fn default() -> Self {
        Self {
            rorm_path: quote! { ::rorm },
            non_exhaustive: private::NonExhaustive,
        }
    }
}
