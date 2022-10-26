//! This module holds the CRUD interface.
//!
//! rorm's crud is entirely based on the builder pattern.
//! This means for every crud query ([INSERT], [SELECT], [DELETE]) there exists a builder struct
//! whose methods allow you to set the various parameters.
//!
//! To begin a builder it is recommended to use the associated macros [insert!], [query!] and [delete!].
//! The hide some of the generic details and may run some compile time checks.
//!
//! [INSERT]: insert
//! [SELECT]: query
//! [DELETE]: delete
//! [insert!]: macro@crate::insert
//! [query!]: macro@crate::query
//! [delete!]: macro@crate::delete
pub mod builder;
pub mod delete;
pub mod insert;
pub mod query;
