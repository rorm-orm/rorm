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

/// Exposes a `NEW` constant, which acts like [`Default::default`] but constant.
///
/// It's a workaround for not having const methods in traits
pub trait ConstNew: 'static {
    /// A new or default instance
    const NEW: Self;
}

/// Exposes a `REF` constant, which is a reference to the value of [`ConstNew::NEW`]
///
/// This trait is another workaround for not being able to create a constant / static
/// reference to the value of [`ConstNew::NEW`] in a generic context:
///
/// ```no_compile
/// fn using_statics<T: ConstNew>() -> &'static T {
///     static INSTANCE: T = T::NEW; // <- Can't use generics in statics
///     &INSTANCE
/// }
/// fn using_consts<T: ConstNew>() -> &'static T {
///     const INSTANCE: T = T::NEW; // <- Can't store potentially interior mutable data in consts
///     &INSTANCE
/// }
/// ```
///
/// However, if the type `T` is not generic but known, then both functions would work without a problem.
/// This trait can't be implemented generically ^1 because this trait's implementor has to write one of the functions above.
/// Since this trait needs to be usable in a const context,
/// the reference must be assigned to a constant and can't be returned from a function.
///
/// This trait hopefully becomes obsolete once `Freeze` is stabilized.
pub trait ConstRef: ConstNew {
    /// A static reference to [`ConstNew::NEW`]
    const REF: &'static Self;
}
