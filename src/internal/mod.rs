//! This module is not considered public api.
//!
//! But since the derive macros need to have access to its content it is all public.
//! Anyway feel free to look at and maybe even use it.

pub mod array_utils;
pub mod const_concat;
pub mod field;
pub mod hmr;
pub mod patch;
pub mod query_context;
pub mod relation_path;

pub use rorm_declaration::imr;

/// Wrap a `Fn(&mut std::fmt::Formatter<'_>) -> std::fmt::Result` to implement [`Display`](std::fmt::Display)
pub struct DisplayImpl<F: Fn(&mut std::fmt::Formatter<'_>) -> std::fmt::Result>(
    /// The wrapped closure
    pub F,
);

impl<F: Fn(&mut std::fmt::Formatter<'_>) -> std::fmt::Result> std::fmt::Display for DisplayImpl<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        (self.0)(f)
    }
}
