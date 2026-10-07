//! All types valid as model fields and traits to make them valid.
//!
//! # Supported types
//!
//! ## `std`
//! - [`bool`]
//! - [`i16`]
//! - [`i32`]
//! - [`i64`]
//! - [`f32`]
//! - [`f64`]
//! - [`String`]
//! - [`Vec<u8>`]
//! - [`Option<T>`] where `T` is on this list
//! - [`PhantomData<T>`](std::marker::PhantomData)
//!
//! *The following required the "postgres-only" feature:*
//!
//! - [`IpAddr`](std::net::IpAddr)
//! - [`Ipv4Addr`](std::net::Ipv4Addr)
//! - [`Ipv6Addr`](std::net::Ipv6Addr)
//!
//! ## `rorm`
//! - [`ForeignModel<M>`](types::ForeignModel)
//! - [`CascadingForeignModel<M>`](types::CascadingForeignModel)
//! - [`BackRef<M>`](types::BackRef) (doesn't work inside an [`Option<T>`])
//! - [`Json<T>`](types::Json)
//! - [`MsgPack<T>`](types::MsgPack) (requires the "msgpack" feature)
//! - [`MaxStr<MAX_LEN>`](types::MaxStr)
//!
//! ## Bring your own
//! - [`#[derive(DbEnum)]`](crate::DbEnum)
//! - [`#[derive(FieldType)]`](crate::FieldType)
//!
//! ## `chrono` (requires the "chrono" feature)
//! - [`NaiveDateTime`](chrono::NaiveDateTime)
//! - [`NaiveTime`](chrono::NaiveTime)
//! - [`NaiveDate`](chrono::NaiveDate)
//! - [`DateTime<Utc>`](chrono::DateTime)
//!
//! ## `time` (requires the "time" feature)
//! - [`PrimitiveDateTime`](time::PrimitiveDateTime)
//! - [`Time`](time::Time)
//! - [`Date`](time::Date)
//! - [`OffsetDateTime<Utc>`](time::OffsetDateTime)
//!
//! ## `uuid` (requires the "uuid" feature)
//! - [`Uuid`](uuid::Uuid)
//!
//! ## `url` (requires the "url" feature)
//! - [`Url`](url::Url)
//!
//! ## `ipnetwork` (requires the "postgres-only" feature)
//! - [`IpNetwork`](ipnetwork::IpNetwork)
//! - [`Ipv4Network`](ipnetwork::Ipv4Network)
//! - [`Ipv6Network`](ipnetwork::Ipv6Network)
//!
//! # Field vs Column
//!
//! `rorm` bridges two domains: rust and SQL
//!
//! On the rust side there are structs (deriving `Model`) and fields.
//! On the SQL side there are tables and columns.
//!
//! Models and tables map to each other one-to-one. (Ignoring generic models)
//!
//! Fields and columns don't.
//! A field might correspond to multiple columns (or none).
//!
//! This allows writing abstractions which store more complex rust types
//! in the database by splitting it into multiple columns.
//!
//! The feature is not fully developed yet. But the distinction is already good to keep in mind.
//!
//! # Example
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
//!     also_other_model: ForeignModelByField<field!(OtherModel.name)>,
//!     other_model_set: BackRef<field!(OtherModel.some_model)>,
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

pub mod proxy;
pub mod traits;
pub mod types;
pub mod utils;
