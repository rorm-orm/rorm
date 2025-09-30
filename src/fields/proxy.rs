//! [`FieldProxy`] and some utility functions which are used by rorm's various macros

use std::marker::PhantomData;
use std::mem::ManuallyDrop;

use rorm_db::sql::aggregation::SelectAggregator;

use crate::conditions::{Binary, Column, In, InOperator, Unary, UnaryOperator, Value};
use crate::crud::selector::{AggregatedColumn, PathedSelector, Selector};
use crate::fields::traits::{
    FieldAvg, FieldColumns, FieldCount, FieldEq, FieldLike, FieldMax, FieldMin, FieldOrd,
    FieldRegexp, FieldSum,
};
use crate::fields::utils::column_name::ColumnName;
use crate::internal::field::{Field, SingleColumnField};
use crate::internal::relation_path::{Path, PathField};
use crate::sealed;

/// This unit struct acts as a proxy exposing a model's field (the field's declaration not its value)
/// as a value to pass around and call methods on.
///
/// It also constructs JOIN paths by following relations between models.
///
/// TODO: more docs
pub struct FieldProxy<T>(PhantomData<ManuallyDrop<T>>);

macro_rules! FieldType {
    ($I:ident) => {
        <<$I as FieldProxyImpl>::Field as Field>::Type
    };
}

impl<F, P, I> FieldProxy<I>
where
    F: Field + PathField<<F as Field>::Type>,
    P: Path<Current = <F::ParentField as Field>::Model>,
    I: FieldProxyImpl<Field = F, Path = P>,
{
    /// Query the model this field points to using `selector`
    pub fn query_as<S>(self, selector: S) -> PathedSelector<S, <I::Path as Path>::Step<I::Field>>
    where
        S: Selector<Model = <F::ChildField as Field>::Model>,
    {
        PathedSelector {
            selector,
            path: Default::default(),
        }
    }
}

impl<I: FieldProxyImpl> FieldProxy<I> {
    /// Checks if the column contains `None`
    pub fn is_none<T>(self) -> Unary<Column<I>>
    where
        // This would have to be a trait for multi-column fields
        I::Field: SingleColumnField<Type = Option<T>>,
    {
        Unary {
            operator: UnaryOperator::IsNull,
            fst_arg: Column(self),
        }
    }

    /// Checks if the column contains `Some`
    pub fn is_some<T>(self) -> Unary<Column<I>>
    where
        // This would have to be a trait for multi-column fields
        I::Field: SingleColumnField<Type = Option<T>>,
    {
        Unary {
            operator: UnaryOperator::IsNotNull,
            fst_arg: Column(self),
        }
    }

    /// Compare the field to another value using `==`
    pub fn equals<'rhs, Rhs: 'rhs, Any>(
        self,
        rhs: Rhs,
    ) -> <FieldType!(I) as FieldEq<'rhs, Rhs, Any>>::EqCond<I>
    where
        FieldType!(I): FieldEq<'rhs, Rhs, Any>,
    {
        <FieldType!(I)>::field_equals(self, rhs)
    }

    /// Compare the field to another value using `!=`
    pub fn not_equals<'rhs, Rhs: 'rhs, Any>(
        self,
        rhs: Rhs,
    ) -> <FieldType!(I) as FieldEq<'rhs, Rhs, Any>>::NeCond<I>
    where
        FieldType!(I): FieldEq<'rhs, Rhs, Any>,
    {
        <FieldType!(I)>::field_not_equals(self, rhs)
    }

    /// Check if the field's value is in a given list of values
    pub fn r#in<'rhs, Rhs: 'rhs, Any>(
        self,
        rhs: impl IntoIterator<Item = Rhs>,
    ) -> In<Column<I>, Value<'rhs>>
    where
        FieldType!(I): FieldEq<'rhs, Rhs, Any, EqCond<I> = Binary<Column<I>, Value<'rhs>>>,
    {
        let values = rhs
            .into_iter()
            .map(|rhs| self.equals(rhs).snd_arg)
            .collect();
        In {
            operator: InOperator::In,
            fst_arg: Column(self),
            snd_arg: values,
        }
    }

    /// Check if the field's value is not in a given list of values
    pub fn not_in<'rhs, Rhs: 'rhs, Any>(
        self,
        rhs: impl IntoIterator<Item = Rhs>,
    ) -> In<Column<I>, Value<'rhs>>
    where
        FieldType!(I): FieldEq<'rhs, Rhs, Any, EqCond<I> = Binary<Column<I>, Value<'rhs>>>,
    {
        let values = rhs
            .into_iter()
            .map(|rhs| self.equals(rhs).snd_arg)
            .collect();
        In {
            operator: InOperator::NotIn,
            fst_arg: Column(self),
            snd_arg: values,
        }
    }

    /// Compare the field to another value using `<`
    pub fn less_than<'rhs, Rhs: 'rhs, Any>(
        self,
        rhs: Rhs,
    ) -> <FieldType!(I) as FieldOrd<'rhs, Rhs, Any>>::LtCond<I>
    where
        FieldType!(I): FieldOrd<'rhs, Rhs, Any>,
    {
        <FieldType!(I)>::field_less_than(self, rhs)
    }

    /// Compare the field to another value using `<=`
    pub fn less_equals<'rhs, Rhs: 'rhs, Any>(
        self,
        rhs: Rhs,
    ) -> <FieldType!(I) as FieldOrd<'rhs, Rhs, Any>>::LeCond<I>
    where
        FieldType!(I): FieldOrd<'rhs, Rhs, Any>,
    {
        <FieldType!(I)>::field_less_equals(self, rhs)
    }

    /// Compare the field to another value using `<`
    pub fn greater_than<'rhs, Rhs: 'rhs, Any>(
        self,
        rhs: Rhs,
    ) -> <FieldType!(I) as FieldOrd<'rhs, Rhs, Any>>::GtCond<I>
    where
        FieldType!(I): FieldOrd<'rhs, Rhs, Any>,
    {
        <FieldType!(I)>::field_greater_than(self, rhs)
    }

    /// Compare the field to another value using `>=`
    pub fn greater_equals<'rhs, Rhs: 'rhs, Any>(
        self,
        rhs: Rhs,
    ) -> <FieldType!(I) as FieldOrd<'rhs, Rhs, Any>>::GeCond<I>
    where
        FieldType!(I): FieldOrd<'rhs, Rhs, Any>,
    {
        <FieldType!(I)>::field_greater_equals(self, rhs)
    }

    /// Compare the field to another value using `LIKE`
    pub fn like<'rhs, Rhs: 'rhs, Any>(
        self,
        rhs: Rhs,
    ) -> <FieldType!(I) as FieldLike<'rhs, Rhs, Any>>::LiCond<I>
    where
        FieldType!(I): FieldLike<'rhs, Rhs, Any>,
    {
        <FieldType!(I)>::field_like(self, rhs)
    }

    /// Compare the field to another value using `NOT LIKE`
    pub fn not_like<'rhs, Rhs: 'rhs, Any>(
        self,
        rhs: Rhs,
    ) -> <FieldType!(I) as FieldLike<'rhs, Rhs, Any>>::NlCond<I>
    where
        FieldType!(I): FieldLike<'rhs, Rhs, Any>,
    {
        <FieldType!(I)>::field_not_like(self, rhs)
    }

    /// Uses `LIKE` to check whether the field contains the string `rhs`
    pub fn contains<'rhs, Any>(
        self,
        rhs: &str,
    ) -> <FieldType!(I) as FieldLike<'rhs, String, Any>>::LiCond<I>
    where
        FieldType!(I): FieldLike<'rhs, String, Any>,
    {
        self.like(format!("%{}%", escape_like(&rhs.to_string())))
    }

    /// Uses `LIKE` to check whether the field starts with the string `rhs`
    pub fn starts_with<'rhs, Any>(
        self,
        rhs: &str,
    ) -> <FieldType!(I) as FieldLike<'rhs, String, Any>>::LiCond<I>
    where
        FieldType!(I): FieldLike<'rhs, String, Any>,
    {
        self.like(format!("{}%", escape_like(&rhs.to_string())))
    }

    /// Uses `LIKE` to check whether the field ends with the string `rhs`
    pub fn ends_with<'rhs, Any>(
        self,
        rhs: &str,
    ) -> <FieldType!(I) as FieldLike<'rhs, String, Any>>::LiCond<I>
    where
        FieldType!(I): FieldLike<'rhs, String, Any>,
    {
        self.like(format!("%{}", escape_like(&rhs.to_string())))
    }

    /// Compare the field to another value using `>=`
    pub fn regexp<'rhs, Rhs: 'rhs, Any>(
        self,
        rhs: Rhs,
    ) -> <FieldType!(I) as FieldRegexp<'rhs, Rhs, Any>>::ReCond<I>
    where
        FieldType!(I): FieldRegexp<'rhs, Rhs, Any>,
    {
        <FieldType!(I)>::field_regexp(self, rhs)
    }

    /// Compare the field to another value using `>=`
    pub fn not_regexp<'rhs, Rhs: 'rhs, Any>(
        self,
        rhs: Rhs,
    ) -> <FieldType!(I) as FieldRegexp<'rhs, Rhs, Any>>::NrCond<I>
    where
        FieldType!(I): FieldRegexp<'rhs, Rhs, Any>,
    {
        <FieldType!(I)>::field_not_regexp(self, rhs)
    }

    /// Returns the count of the number of times that the column is not null.
    pub fn count(self) -> AggregatedColumn<I, i64>
    where
        FieldType!(I): FieldCount,
    {
        AggregatedColumn {
            sql: SelectAggregator::Count,
            alias: "count",
            field: self,
            result: PhantomData,
        }
    }

    /// Returns the summary off all non-null values in the group.
    /// If there are only null values in the group, this function will return null.
    pub fn sum(self) -> AggregatedColumn<I, <FieldType!(I) as FieldSum>::Result>
    where
        FieldType!(I): FieldSum,
    {
        AggregatedColumn {
            sql: SelectAggregator::Sum,
            alias: "sum",
            field: self,
            result: PhantomData,
        }
    }

    /// Returns the average value of all non-null values.
    /// The result of avg is a floating point value, except all input values are null, than the
    /// result will also be null.
    pub fn avg(self) -> AggregatedColumn<I, Option<f64>>
    where
        FieldType!(I): FieldAvg,
    {
        AggregatedColumn {
            sql: SelectAggregator::Avg,
            alias: "avg",
            field: self,
            result: PhantomData,
        }
    }

    /// Returns the maximum value of all values in the group.
    /// If there are only null values in the group, this function will return null.
    pub fn max(self) -> AggregatedColumn<I, <FieldType!(I) as FieldMax>::Result>
    where
        FieldType!(I): FieldMax,
    {
        AggregatedColumn {
            sql: SelectAggregator::Max,
            alias: "max",
            field: self,
            result: PhantomData,
        }
    }

    /// Returns the minimum value of all values in the group.
    /// If there are only null values in the group, this function will return null.
    pub fn min(self) -> AggregatedColumn<I, <FieldType!(I) as FieldMin>::Result>
    where
        FieldType!(I): FieldMin,
    {
        AggregatedColumn {
            sql: SelectAggregator::Min,
            alias: "min",
            field: self,
            result: PhantomData,
        }
    }
}

// SAFETY:
// struct contains no data
unsafe impl<T> Send for FieldProxy<T> {}
unsafe impl<T> Sync for FieldProxy<T> {}

impl<T> Clone for FieldProxy<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for FieldProxy<T> {}

/// Implementation detail of [`FieldProxy`], `FieldProxy`'s generic must implement this trait.
///
/// This trait is not relevant for the average rorm user.
pub trait FieldProxyImpl: 'static {
    sealed!(trait);

    /// Field which is proxied
    type Field: Field;

    /// Path the field is accessed through
    type Path: Path;

    /// "Type level function" which swap's the `Path` for a new one
    type Through<NewPath: Path>: FieldProxyImpl<Field = Self::Field, Path = NewPath>;
}

impl<F, P> FieldProxyImpl for (F, P)
where
    F: Field,
    P: Path,
{
    sealed!(impl);

    type Field = F;
    type Path = P;
    type Through<NewPath: Path> = (F, NewPath);
}

/// Construct a new `FieldProxy`
///
/// *Not relevant for the average rorm user*
///
/// This function is used by the `#[derive(Model)]` macro to populate the Fields struct.
pub const fn new<I: FieldProxyImpl>() -> FieldProxy<I> {
    FieldProxy(PhantomData)
}

/// Get a [`Field`]'s `INDEX` from a `FieldProxy`
///
/// *Not relevant for the average rorm user*
///
/// This function is used by the [`get_field`](crate::get_field) and [`field`](crate::field) macros.
pub const fn index<I: FieldProxyImpl>(_: fn() -> FieldProxy<I>) -> usize {
    <I::Field as Field>::INDEX
}

/// Change a `FieldProxy`'s path
///
/// *Not relevant for the average rorm user*
///
/// This function is used by the `#[derive(Patch)]` to construct a `Selector` with a custom path.
/// This is subject to change.
pub const fn through<I: FieldProxyImpl, NewPath: Path>(
    _: fn() -> FieldProxy<I>,
) -> FieldProxy<I::Through<NewPath>> {
    new()
}

/// Get the names of the columns which store the field
///
/// *Not relevant for the average rorm user*
///
/// This function is used by the `#[derive(Patch)]` macro to gather a list of all columns.
pub const fn columns<T: FieldProxyImpl>(
    _: fn() -> FieldProxy<T>,
) -> FieldColumns<<T::Field as Field>::Type, ColumnName> {
    <T::Field as Field>::EFFECTIVE_NAMES
}

/// Escape the special character from an argument to `LIKE`
fn escape_like(string: &str) -> String {
    string
        .replace('\\', r"\\")
        .replace('%', r"\%")
        .replace('_', r"\_")
}
