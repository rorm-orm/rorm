//! Rorm is the rust implementation of the drorm project.
pub use rorm_common::imr;
pub use rorm_macro::*;

/// This slice is populated by the [Model] attribute with all models.
#[allow(non_camel_case_types)]
#[::linkme::distributed_slice]
pub static MODELS: [&'static dyn ModelDefinition] = [..];

/// A ModelDefinition provides methods to do something similar to reflection on model structs.
///
/// This trait is only implemented on empty types and used as dyn objects i.e. it is a highler
/// level representation for a function table.
/// It is automatically implemented for you by the [Model] attribute.
// sync and send is required in order to store it as a static
pub trait ModelDefinition: Sync + Send {
    /// Build the Intermediate Model Representation
    fn as_imr(&self) -> imr::Model;
}

/// This trait maps rust types to database types
pub trait AsDbType {
    /// Returns the database type as defined in the Intermediate Model Representation
    fn as_db_type() -> imr::DbType;
}

macro_rules! impl_as_db_type {
    ($type:ty, $variant:ident) => {
        impl AsDbType for $type {
            fn as_db_type() -> imr::DbType {
                imr::DbType::$variant
            }
        }
    };
}
impl_as_db_type!(String, VarChar);
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

/// Prints all models in the Intermediate Model Representation to stdout.
/// This should be used as a main function to produce the file for the migrator.
///
/// WIP: A tool to automate this is planned
pub fn print_models() -> Result<(), String> {
    serde_json::to_writer(
        std::io::stdout(),
        &Vec::from_iter(MODELS.iter().map(|md| md.as_imr())),
    )
    .map_err(|err| err.to_string())
}
