//! Rorm is the rust implementation of the drorm project.
#![warn(missing_docs)]

use std::io::Write;

pub use linkme;
pub use model::{DbEnum, Model, ID};
pub use rorm_db::*;
pub use rorm_macro::*;

// Reexports to be used by macro
pub use rorm_declaration::imr;

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

macro_rules! define_query_macro {
    ($(#[doc = $doc:literal])* $name:ident) => {
        $(#[doc = $doc])*
        #[macro_export]
        macro_rules! $name {
            ($db:expr, $patch:path, $condition:expr) => {
                $db.$name(
                    <$patch as ::rorm::model::Patch>::MODEL::table_name(),
                    <$patch as ::rorm::model::Patch>::COLUMNS,
                    Some(&$condition),
                )
            };
            ($db:expr, $patch:path) => {
                $db.$name(
                    <$patch as ::rorm::model::Patch>::MODEL::table_name(),
                    <$patch as ::rorm::model::Patch>::COLUMNS,
                    None,
                )
            };
        }
    };
}

define_query_macro!(
    /// High level macro for [`Database::query_all`].
    ///
    /// It takes:
    /// - a database connection (instance of [`Database`])
    /// - a patch or model (path to your [`Patch`] struct)
    /// - an optional condition (instance of [`Condition`])
    ///
    /// and calls [`Database::query_all`] on the connection
    /// inferring its arguments from the patch and parsing the condition.
    ///
    /// [`Condition`]: conditional::Condition
    query_all
);

define_query_macro!(
    /// High level macro for [`Database::query_stream`].
    ///
    /// It takes:
    /// - a database connection (instance of [`Database`])
    /// - a patch or model (path to your [`Patch`] struct)
    /// - an optional condition (instance of [`Condition`])
    ///
    /// and calls [`Database::query_stream`] on the connection
    /// inferring its arguments from the patch and parsing the condition.
    ///
    /// [`Condition`]: conditional::Condition
    query_stream
);

define_query_macro!(
    /// High level macro for [`Database::query_one`].
    ///
    /// It takes:
    /// - a database connection (instance of [`Database`])
    /// - a patch or model (path to your [`Patch`] struct)
    /// - an optional condition (instance of [`Condition`])
    ///
    /// and calls [`Database::query_one`] on the connection
    /// inferring its arguments from the patch and parsing the condition.
    ///
    /// [`Condition`]: conditional::Condition
    query_one
);

define_query_macro!(
    /// High level macro for [`Database::query_optional`].
    ///
    /// It takes:
    /// - a database connection (instance of [`Database`])
    /// - a patch or model (path to your [`Patch`] struct)
    /// - an optional condition (instance of [`Condition`])
    ///
    /// and calls [`Database::query_optional`] on the connection
    /// inferring its arguments from the patch and parsing the condition.
    ///
    /// [`Condition`]: conditional::Condition
    query_optional
);
