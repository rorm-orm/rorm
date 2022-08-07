//! Rorm is the rust implementation of the drorm project.

#[doc(hidden)]
pub use id::ID;
pub use linkme;
pub use rorm_macro::*;
pub use rorm_sql::imr;
use std::io::Write;

pub mod id;
pub mod model_def;

/// This trait maps rust types to database types
pub trait AsDbType {
    /// Returns the database type as defined in the Intermediate Model Representation
    fn as_db_type(annotations: &[imr::Annotation]) -> imr::DbType;

    /// Returns a list of migrator annotations which are implied by the type.
    ///
    /// For most types this would be empty. So that's its default implementation.
    /// It is called after `as_db_type` and therefore not available to it.
    fn implicit_annotations() -> Vec<imr::Annotation> {
        Vec::new()
    }

    fn is_nullable() -> bool {
        false
    }
}

macro_rules! impl_as_db_type {
    ($type:ty, $variant:ident) => {
        impl AsDbType for $type {
            fn as_db_type(_annotations: &[imr::Annotation]) -> imr::DbType {
                imr::DbType::$variant
            }
        }
    };
}
impl_as_db_type!(Vec<u8>, VarBinary);
impl_as_db_type!(i8, Int8);
impl_as_db_type!(i16, Int16);
impl_as_db_type!(i32, Int32);
impl_as_db_type!(i64, Int64);
impl_as_db_type!(isize, Int64);
impl_as_db_type!(u8, UInt8);
impl_as_db_type!(u16, UInt16);
impl_as_db_type!(u32, UInt32);
impl_as_db_type!(u64, UInt64);
impl_as_db_type!(usize, UInt64);
impl_as_db_type!(f32, Float);
impl_as_db_type!(f64, Double);
impl_as_db_type!(bool, Boolean);
impl AsDbType for String {
    fn as_db_type(annotations: &[imr::Annotation]) -> imr::DbType {
        let mut choices = false;
        for annotation in annotations.iter() {
            match annotation {
                imr::Annotation::Choices(_) => {
                    choices = true;
                }
                _ => {}
            }
        }
        if choices {
            imr::DbType::Choices
        } else {
            imr::DbType::VarChar
        }
    }
}
impl<T: AsDbType> AsDbType for Option<T> {
    fn as_db_type(annotations: &[imr::Annotation]) -> imr::DbType {
        T::as_db_type(annotations)
    }

    fn implicit_annotations() -> Vec<imr::Annotation> {
        T::implicit_annotations()
    }

    fn is_nullable() -> bool {
        true
    }
}

/// Map a rust enum, whose variant don't hold any data, into a database enum
///
/// ```rust
/// #[derive(rorm::DbEnum)]
/// pub enum Gender {
///     Male,
///     Female,
///     Other,
/// }
/// ```
pub trait DbEnum {
    fn from_str(string: &str) -> Self;
    fn to_str(&self) -> &'static str;
    fn as_choices() -> Vec<String>;
}
impl<E: DbEnum> AsDbType for E {
    fn as_db_type(_annotations: &[imr::Annotation]) -> imr::DbType {
        imr::DbType::Choices
    }

    fn implicit_annotations() -> Vec<imr::Annotation> {
        vec![imr::Annotation::Choices(E::as_choices())]
    }
}

/// Write all models in the Intermediate Model Representation to a [writer].
///
/// [writer]: std::io::Write
pub fn write_models(writer: &mut impl Write) -> Result<(), String> {
    let imf = imr::InternalModelFormat {
        models: model_def::MODELS.iter().map(|md| md.as_imr()).collect(),
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
