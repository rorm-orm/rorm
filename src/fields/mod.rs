//! All types valid as model fields and traits to make them valid.
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
//! # Our types
//! - [`ForeignModel<M>`](types::ForeignModel)
//! - [`BackRef<M>`](types::BackRef) (doesn't work inside an [`Option<T>`])
//! - [`Json<T>`](types::Json)
//! - [`MsgPack<T>`](types::MsgPack) (requires the "msgpack" feature)
//!
//! # chrono types (requires the "chrono" feature)
//! - [`NaiveDateTime`](chrono::NaiveDateTime)
//! - [`NaiveTime`](chrono::NaiveTime)
//! - [`NaiveDate`](chrono::NaiveDate)
//! - [`DateTime<Utc>`](chrono::DateTime)
//! - [`DateTime<FixedOffset>`](chrono::DateTime) **WIP** (todo annotations)
//!
//! # time types (requires the "time" feature)
//! - [`PrimitiveDateTime`](time::PrimitiveDateTime)
//! - [`Time`](time::Time)
//! - [`Date`](time::Date)
//! - [`OffsetDateTime<Utc>`](time::OffsetDateTime)
//!
//! # uuid types (requires the "uuid" feature)
//! - [`Uuid`](uuid::Uuid)
//!
//! ---
//!
//! ```no_run
//! use serde::{Deserialize, Serialize};
//! use rorm::{Model, field};
//! use rorm::fields::types::*;
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

pub mod traits;
pub mod types;
