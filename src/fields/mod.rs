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
//!
//! ---
//!
//! ```no_run
//! use serde::{Deserialize, Serialize};
//! use rorm::Model;
//! use rorm::fields::*;
//!
//! #[derive(Model)]
//! pub struct SomeModel {
//!     #[rorm(id)]
//!     id: i64,
//!
//!     // std
//!     boolean: bool,
//!     integer: i32,
//!     float: f64,
//!     #[rorm(max_length = 255)]
//!     string: String,
//!     binary: Vec<u8>,
//!
//!     // times
//!     time: chrono::NaiveTime,
//!     date: chrono::NaiveDate,
//!     datetime: chrono::DateTime<chrono::Utc>,
//!
//!     // relations
//!     other_model: ForeignModel<OtherModel>,
//!     #[rorm(field = "OtherModel::F.name")]
//!     also_other_model: ForeignModel<OtherModel, String>,
//!     #[rorm(field = "OtherModel::F.some_model")]
//!     other_model_set: BackRef<OtherModel>,
//!
//!     // serde
//!     data: Json<Data>,
//! }
//!
//! #[derive(Model)]
//! pub struct OtherModel {
//!     #[rorm(id)]
//!     id: i64,
//!
//!     #[rorm(max_length = 255)]
//!     name: String,
//!
//!     some_model: ForeignModel<SomeModel>,
//! }
//!
//! #[derive(Serialize, Deserialize)]
//! pub struct Data {
//!     stuff: String,
//! }
//! ```

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
