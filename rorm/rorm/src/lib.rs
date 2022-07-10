pub use rorm_common::imr;
pub use rorm_macro::*;

#[allow(non_camel_case_types)]
#[::linkme::distributed_slice]
pub static MODELS: [&'static dyn ModelDefinition] = [..];

// sync and send is required in order to store it as a static
pub trait ModelDefinition: Sync + Send {
    fn as_imr(&self) -> imr::Model;
}

pub trait AsDbType {
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

pub fn print_models() -> Result<(), String> {
    serde_json::to_writer(
        std::io::stdout(),
        &Vec::from_iter(MODELS.iter().map(|md| md.as_imr())),
    )
    .map_err(|err| err.to_string())
}
