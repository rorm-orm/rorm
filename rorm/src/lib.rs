//! Rorm is the rust implementation of the drorm project.
#![warn(missing_docs)]

#[cfg(any(
    all(
        feature = "actix-rustls",
        any(
            feature = "actix-native-tls",
            feature = "tokio-native-tls",
            feature = "tokio-rustls",
            feature = "async-std-native-tls",
            feature = "async-std-rustls"
        )
    ),
    all(
        feature = "actix-native-tls",
        any(
            feature = "tokio-native-tls",
            feature = "tokio-rustls",
            feature = "async-std-native-tls",
            feature = "async-std-rustls"
        )
    ),
    all(
        feature = "tokio-rustls",
        any(
            feature = "tokio-native-tls",
            feature = "async-std-native-tls",
            feature = "async-std-rustls"
        )
    ),
    all(
        feature = "tokio-native-tls",
        any(feature = "async-std-native-tls", feature = "async-std-rustls")
    ),
    all(feature = "async-std-native-tls", feature = "async-std-rustls")
))]
compile_error!("Using multiple runtime / tls configurations at the same time is not allowed");

#[cfg(not(any(
    feature = "async-std-native-tls",
    feature = "async-std-rustls",
    feature = "tokio-native-tls",
    feature = "tokio-rustls",
    feature = "actix-native-tls",
    feature = "actix-rustls"
)))]
compile_error!(
    r#"One of async-std-native-tls, async-std-rustls, tokio-native-tls, tokio-rustls, 
    actix-native-tls, actix-rustls is required"#
);

use std::io::Write;

pub use crud::query::QueryBuilder;
pub use linkme;
pub use model::{DbEnum, Model, Patch};
pub use rorm_db::*;
pub use rorm_macro::*;

// Reexports to be used by macro
pub use rorm_declaration::hmr;
pub use rorm_declaration::imr;

/// This module implements a struct to build and store annotations
pub mod annotation_builder;
/// Module implementing methods to [`Condition`] based on [`Field<T>`]
///
/// [`Condition`]: rorm_db::conditional::Condition
/// [`Field<T>`]: model::Field
mod conditions;
pub mod crud;
/// This module holds traits and structs for working with models
pub mod model;

/// This slice is populated by the [`Model`] macro with all models.
///
/// [`Model`]: rorm_macro::Model
#[allow(non_camel_case_types)]
#[linkme::distributed_slice]
pub static MODELS: [fn() -> imr::Model] = [..];

/// Write all models in the Intermediate Model Representation to a [writer].
///
/// [writer]: std::io::Write
pub fn write_models(writer: &mut impl Write) -> Result<(), String> {
    let imf = imr::InternalModelFormat {
        models: MODELS.iter().map(|func| func()).collect(),
    };
    serde_json::to_writer(writer, &imf).map_err(|err| err.to_string())
}

/// Prints all models in the Intermediate Model Representation to stdout.
/// This should be used as a main function to produce the file for the migrator.
///
/// See also [`rorm_main`]
///
/// [`rorm_main`]: rorm_macro::rorm_main
pub fn print_models() -> Result<(), String> {
    write_models(&mut std::io::stdout())
}
