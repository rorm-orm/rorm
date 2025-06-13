//! This module is not considered public api.
//!
//! But since the derive macros need to have access to its content it is all public.
//! Anyway feel free to look at and maybe even use it.

pub mod const_concat;
mod djb2;
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

/// Provides a `'static` reference to a / "the" value of `Self`
///
/// This trait is a double workaround:
/// 1. The is no const equivalent for [`Default`] because const functions are not stable
/// 2. If 1 were fixed, one could not create a constant / static
/// reference to this default value in a generic context:
///
/// ```no_compile
/// // Workaround for 1
/// trait ConstDefault {
///     const DEFAULT: Self;
/// }
///
/// fn using_statics<T: ConstDefault>() -> &'static T {
///     static INSTANCE: T = T::DEFAULT; // <- Can't use generics in statics
///     &INSTANCE
/// }
/// fn using_consts<T: ConstDefault>() -> &'static T {
///     const INSTANCE: T = T::DEFAULT; // <- Can't store potentially interior mutable data in consts
///     &INSTANCE
/// }
/// ```
///
/// However, if the type `T` is not generic but known, then both functions would work without a problem.
/// This trait can't be implemented generically because this trait's implementor has to write one of the functions above.
/// Since this trait needs to be usable in a const context,
/// the reference must be assigned to a constant and can't be returned from a function.
///
/// This trait hopefully becomes obsolete once `Freeze` is stabilized.
pub trait ConstRef: 'static {
    /// A static reference to `const Default::default`
    const REF: &'static Self;
}
