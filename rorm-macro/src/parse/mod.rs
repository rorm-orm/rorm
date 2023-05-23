use syn::{Fields, FieldsNamed, Generics};

pub mod annotations;
pub mod db_enum;
pub mod model;
pub mod patch;

/// Get the [`Fields::Named(..)`](Fields::Named) variant's data or produce an error
pub fn get_fields_named(fields: Fields) -> darling::Result<FieldsNamed> {
    match fields {
        Fields::Named(fields) => Ok(fields),
        Fields::Unnamed(_) => Err(darling::Error::unsupported_shape_with_expected(
            "named tuple",
            &"struct with named fields",
        )
        .with_span(&fields)),
        Fields::Unit => Err(darling::Error::unsupported_shape_with_expected(
            "unit struct",
            &"struct with named fields",
        )
        .with_span(&fields)),
    }
}

/// Check a struct or enum to don't have [`Generics`]
pub fn check_non_generic(generics: Generics) -> darling::Result<()> {
    if generics.lt_token.is_none() {
        Ok(())
    } else {
        Err(darling::Error::unsupported_shape_with_expected(
            "generic struct",
            &"struct without generics",
        )
        .with_span(&generics))
    }
}
