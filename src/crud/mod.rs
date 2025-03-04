//! This module holds the CRUD interface.
//!
//! rorm's crud is entirely based on the builder pattern.
//! This means for every crud query ([INSERT](insert), [SELECT](query), [UPDATE](update), [DELETE](delete)) there exists a builder struct
//! whose methods allow you to set the various parameters.
//!
//! To begin a builder use their associated functions [`insert`], [`query`], [`update`] and [`delete`].
pub mod builder;
pub mod decoder;
pub mod delete;
pub mod insert;
pub mod query;
pub mod selector;
pub mod update;
