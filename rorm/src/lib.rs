//! Rorm is the rust implementation of the drorm project.
#![warn(missing_docs)]

pub use linkme;
pub use rorm_db::*;
pub use rorm_declaration::model::{DbEnum, Model, ID};
pub use rorm_macro::*;

// Reexports to be used by macro
pub use rorm_declaration::imr;
pub use rorm_declaration::model;

use rorm_declaration::model::GetModelDefinition;
use std::io::Write;

/// This slice is populated by the [`Model`] macro with all models.
///
/// [`Model`]: rorm_macro::Model
#[allow(non_camel_case_types)]
#[linkme::distributed_slice]
pub static MODELS: [&'static dyn GetModelDefinition] = [..];

/// Write all models in the Intermediate Model Representation to a [writer].
///
/// [writer]: std::io::Write
pub fn write_models(writer: &mut impl Write) -> Result<(), String> {
    let imf = imr::InternalModelFormat {
        models: MODELS.iter().map(|md| md.as_imr()).collect(),
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

/// High level macro for [`Database::query_all`].
///
/// It takes a database connection and patch or model and infers the table and columns for it.
#[macro_export]
macro_rules! query_all {
    ($db:expr, $patch:path) => {{
        let table = <$patch as ::rorm::model::Patch>::MODEL::table_name();
        let columns = <$patch as ::rorm::model::Patch>::COLUMNS;
        $db.query_all(table, columns, None)
    }};
}

/// High level macro for [`Database::query_one`].
///
/// It takes a database connection and patch or model and infers the table and columns for it.
#[macro_export]
macro_rules! query_one {
    ($db:expr, $patch:path) => {{
        let table = <$patch as ::rorm::model::Patch>::MODEL::table_name();
        let columns = <$patch as ::rorm::model::Patch>::COLUMNS;
        $db.query_one(table, columns, None)
    }};
}

/// High level macro for [`Database::query_stream`].
///
/// It takes a database connection and patch or model and infers the table and columns for it.
#[macro_export]
macro_rules! query_stream {
    ($db:expr, $patch:path) => {{
        let table = <$patch as ::rorm::model::Patch>::MODEL::table_name();
        let columns = <$patch as ::rorm::model::Patch>::COLUMNS;
        $db.query_stream(table, columns, None)
    }};
}

/// High level macro for [`Database::query_optional`].
///
/// It takes a database connection and patch or model and infers the table and columns for it.
#[macro_export]
macro_rules! query_optional {
    ($db:expr, $patch:path) => {{
        let table = <$patch as ::rorm::model::Patch>::MODEL::table_name();
        let columns = <$patch as ::rorm::model::Patch>::COLUMNS;
        $db.query_optional(table, columns, None)
    }};
}
