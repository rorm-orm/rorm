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

#[doc(hidden)]
pub use linkme;

pub use internal::field::back_ref::BackRef;
pub use internal::field::foreign_model::ForeignModel;
pub use model::{Model, Patch};
pub use rorm_db::*;

use std::io::Write;

// Reexports to be used by macro
pub use rorm_declaration::imr;

// Reexported for use in parser structs of user
pub use rorm_declaration::config;

pub mod aggregate;
pub mod conditions;
pub mod crud;
pub mod internal;
/// This module holds traits and structs for working with models
pub mod model;

/// This slice is populated by the [`Model`] macro with all models.
///
/// [`Model`]: rorm_macro::Model
#[allow(non_camel_case_types)]
#[linkme::distributed_slice]
#[doc(hidden)]
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
pub fn print_models() -> Result<(), String> {
    write_models(&mut std::io::stdout())
}

#[doc(hidden)]
pub(crate) mod private {
    pub trait Private {}
}
/// Put this macro inside a trait to seal it i.e. prevent extern implementations.
#[macro_export]
macro_rules! sealed {
    () => {
        /// This method prohibits implementation of this trait out side of its defining crate.
        fn _not_implementable<P: $crate::private::Private>() {}
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! get_field {
    ($patch:ty, $field:ident) => {
        <<$patch as ::rorm::model::Patch>::Model as ::rorm::model::FieldByIndex<
            { <Self as ::rorm::model::Patch>::Model::FIELDS.$field.index() },
        >>::Field
    };
}

#[doc(hidden)]
pub use rorm_macro::rename_linkme;

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
/// ```
/// use rorm::rorm_main;
///
/// #[rorm_main("other-name")]
/// fn main() {}
/// ```
pub use rorm_macro::rorm_main;

/// ```no_run
/// use rorm::DbEnum;
///
/// #[derive(DbEnum)]
/// pub enum Gender {
///     Male,
///     Female,
///     Other,
/// }
/// ```
pub use rorm_macro::DbEnum;

/// ```no_run
/// use rorm::Model;
///
/// #[derive(Model)]
/// struct User {
///
///     #[rorm(id)]
///     id: i32,
///
///     #[rorm(max_length = 255, unique)]
///     username: String,
///
///     #[rorm(max_length = 255)]
///     password: String,
///
///     #[rorm(default = false)]
///     admin: bool,
///
///     age: i16,
///
///     #[rorm(choices("m", "f", "d"))]
///     gender: String,
/// }
/// ```
pub use rorm_macro::Model;

/// ```no_run
/// use rorm::{Model, Patch};
///
/// #[derive(Model)]
/// struct User {
///
///     #[rorm(id)]
///     id: i32,
///
///     #[rorm(max_length = 255, unique)]
///     username: String,
///
///     #[rorm(max_length = 255)]
///     password: String,
///
///     #[rorm(default = false)]
///     admin: bool,
///
///     age: i16,
/// }
///
/// #[derive(Patch)]
/// #[rorm(model = "User")]
/// struct InsertNormalUser {
///     // id set by database
///
///     username: String,
///
///     password: String,
///
///     // admin defaults to false
///
///     age: i16,
/// }
/// ```
pub use rorm_macro::Patch;
