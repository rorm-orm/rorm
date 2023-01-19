//! A collection of convenience features based on [serde]

mod json;
pub use json::Json;

#[cfg(feature = "rmp-serde")]
mod msgpack;
#[cfg(feature = "rmp-serde")]
pub use msgpack::MsgPack;
