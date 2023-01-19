//! All types valid as model fields
//!
//! # Std types
//! - [`bool`]
//! - [`i16`]
//! - [`i32`]
//! - [`i64`]
//! - [`f32`]
//! - [`f64`]
//! - [`String`]
//! - [`Vec<u8>`]
//! - [`Option<T>`] WIP (currently works only where [`T: AsDbType`](crate::internal::field::as_db_type))
//!
//! # [chrono] types
//! - [`NaiveDateTime`](chrono::NaiveDateTime)
//! - [`NaiveTime`](chrono::NaiveTime)
//! - [`NaiveDate`](chrono::NaiveDate)
//! - [`DateTime<Utc>`](chrono::DateTime)
//!
//! # Our types
//! - [`ForeignModel<M>`]
//! - [`BackRef<M>`]
//! - [`Json<T>`]
//! - [`MsgPack<T>`] (requires the "rmp-serde" crate)

mod back_ref;
pub(crate) mod foreign_model;
mod json;
#[cfg(feature = "rmp-serde")]
mod msgpack;

pub use back_ref::BackRef;
pub use foreign_model::ForeignModel;
pub use foreign_model::ForeignModelByField;
pub use json::Json;
#[cfg(feature = "rmp-serde")]
pub use msgpack::MsgPack;
#[cfg(not(feature = "rmp-serde"))]
/// Stores data by serializing it to message pack.
///
/// Requires the "rmp-serde" crate
pub enum MsgPack {}
