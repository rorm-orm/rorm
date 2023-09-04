//! Experimental trait to hide a [`FieldProxy`]s two generics behind a single one.

use crate::fields::traits::{FieldEq, FieldLike, FieldOrd, FieldRegexp};
use crate::internal::field::{FieldProxy, RawField};
use crate::internal::relation_path::Path;

#[allow(non_snake_case)] // the macro produces a datatype which are named using CamelCase
macro_rules! FieldType {
    () => {
        <Self::Field as RawField>::Type
    };
}

/// Trait only implemented by [`FieldProxy`] to reduce the amount of generics when using them.
///
/// ## Why
/// ```no-run
/// // function using FieldProxy
/// fn do_something<F, P>(proxy: FieldProxy<F, P>) {...}
///
/// // but in order to do useful things with the proxy, you will need bounds:
/// fn do_useful<F: RawField, P: Path>(proxy: FieldProxy<F, P>) {...}
///
/// // function using FieldAccess
/// fn do_something<A: FieldAccess>(proxy: A) {...}
///
/// // the above already covers the useful part, but depending on your usage you could also use the `impl` sugar:
/// fn do_sugared(proxy: FieldAccess) {...}
/// ```
///
/// ## Comparison operations
/// This trait also adds methods for comparing fields which just wrap their underlying [comparison traits](crate::fields::traits::cmp).
/// ```no-run
/// use rorm::Model;
/// use rorm::internal::field::FieldAccess;
///
/// #[derive(Model)]
/// struct User {
///     #[rorm(id)]
///     id: i64,
///
///     #[rorm(max_length = 255)]
///     name: String,
/// }
///
/// // Uses the `FieldEq` impl of `String`
/// let condition = User::F.name.eq("Bob".to_string());
/// ```
pub trait FieldAccess: Sized + Send + 'static {
    /// Field which is accessed
    ///
    /// Corresponds to the proxy's `F` parameter
    type Field: RawField;

    /// Path the field is accessed through
    ///
    /// Corresponds to the proxy's `P` parameter
    type Path: Path;

    /// Compare the field to another value using `==`
    fn eq<'rhs, Rhs: 'rhs>(self, rhs: Rhs) -> <FieldType!() as FieldEq<'rhs, Rhs>>::EqCond<Self>
    where
        FieldType!(): FieldEq<'rhs, Rhs>,
    {
        <FieldType!()>::field_equals(self, rhs)
    }

    /// Compare the field to another value using `!=`
    fn ne<'rhs, Rhs: 'rhs>(self, rhs: Rhs) -> <FieldType!() as FieldEq<'rhs, Rhs>>::NeCond<Self>
    where
        FieldType!(): FieldEq<'rhs, Rhs>,
    {
        <FieldType!()>::field_not_equals(self, rhs)
    }

    /// Compare the field to another value using `<`
    fn lt<'rhs, Rhs: 'rhs>(self, rhs: Rhs) -> <FieldType!() as FieldOrd<'rhs, Rhs>>::LtCond<Self>
    where
        FieldType!(): FieldOrd<'rhs, Rhs>,
    {
        <FieldType!()>::field_less_than(self, rhs)
    }

    /// Compare the field to another value using `<=`
    fn le<'rhs, Rhs: 'rhs>(self, rhs: Rhs) -> <FieldType!() as FieldOrd<'rhs, Rhs>>::LeCond<Self>
    where
        FieldType!(): FieldOrd<'rhs, Rhs>,
    {
        <FieldType!()>::field_less_equals(self, rhs)
    }

    /// Compare the field to another value using `<`
    fn gt<'rhs, Rhs: 'rhs>(self, rhs: Rhs) -> <FieldType!() as FieldOrd<'rhs, Rhs>>::GtCond<Self>
    where
        FieldType!(): FieldOrd<'rhs, Rhs>,
    {
        <FieldType!()>::field_greater_than(self, rhs)
    }

    /// Compare the field to another value using `>=`
    fn ge<'rhs, Rhs: 'rhs>(self, rhs: Rhs) -> <FieldType!() as FieldOrd<'rhs, Rhs>>::GeCond<Self>
    where
        FieldType!(): FieldOrd<'rhs, Rhs>,
    {
        <FieldType!()>::field_greater_equals(self, rhs)
    }

    /// Compare the field to another value using `LIKE`
    fn like<'rhs, Rhs: 'rhs>(self, rhs: Rhs) -> <FieldType!() as FieldLike<'rhs, Rhs>>::LiCond<Self>
    where
        FieldType!(): FieldLike<'rhs, Rhs>,
    {
        <FieldType!()>::field_like(self, rhs)
    }

    /// Compare the field to another value using `NOT LIKE`
    fn not_like<'rhs, Rhs: 'rhs>(
        self,
        rhs: Rhs,
    ) -> <FieldType!() as FieldLike<'rhs, Rhs>>::NlCond<Self>
    where
        FieldType!(): FieldLike<'rhs, Rhs>,
    {
        <FieldType!()>::field_not_like(self, rhs)
    }

    /// Compare the field to another value using `>=`
    fn regexp<'rhs, Rhs: 'rhs>(
        self,
        rhs: Rhs,
    ) -> <FieldType!() as FieldRegexp<'rhs, Rhs>>::ReCond<Self>
    where
        FieldType!(): FieldRegexp<'rhs, Rhs>,
    {
        <FieldType!()>::field_regexp(self, rhs)
    }

    /// Compare the field to another value using `>=`
    fn not_regexp<'rhs, Rhs: 'rhs>(
        self,
        rhs: Rhs,
    ) -> <FieldType!() as FieldRegexp<'rhs, Rhs>>::NrCond<Self>
    where
        FieldType!(): FieldRegexp<'rhs, Rhs>,
    {
        <FieldType!()>::field_not_regexp(self, rhs)
    }
}

impl<F: RawField, P: Path> FieldAccess for FieldProxy<F, P> {
    type Field = F;
    type Path = P;
}
