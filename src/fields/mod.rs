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
//! - [`Option<T>`] where `T` is on this list
//!
//! # [chrono] types
//! - [`NaiveDateTime`](chrono::NaiveDateTime)
//! - [`NaiveTime`](chrono::NaiveTime)
//! - [`NaiveDate`](chrono::NaiveDate)
//! - [`DateTime<Utc>`](chrono::DateTime)
//!
//! # Our types
//! - [`ForeignModel<M>`]
//! - [`BackRef<M>`] (doesn't work inside an [`Option<T>`])
//! - [`Json<T>`]
//! - [`MsgPack<T>`] (requires the "rmp-serde" crate)
//!
//! ---
//!
//! ```no_run
//! use serde::{Deserialize, Serialize};
//! use rorm::{Model, field};
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
//!     also_other_model: ForeignModelByField<field!(OtherModel::F.name)>,
//!     other_model_set: BackRef<field!(OtherModel::F.some_model)>,
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
mod datetime;
mod foreign_model;
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
