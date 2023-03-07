//! Aggregation functions

use std::marker::PhantomData;

use rorm_db::row::DecodeOwned;
use rorm_db::sql::aggregation::SelectAggregator;

use crate::internal::field::{Field, FieldProxy};
use crate::internal::relation_path::Path;

/// A function which can be used in aggregation.
///
/// Since an aggregation's "return type" depends on the used function, it has to be implemented using a trait.
pub trait AggregationFunc {
    /// The function's "return type" depending on the column type it is used over;
    type Result<Input: DecodeOwned>: DecodeOwned;

    /// A name to generate an join alias.
    const NAME: &'static str;

    /// The rorm-sql representation
    const SQL: SelectAggregator;
}

/// Returns the average value of all non-null values.
/// The result of avg is a floating point value, except all input values are null, than the
/// result will also be null.
pub struct Avg;
impl AggregationFunc for Avg {
    type Result<Input: DecodeOwned> = Option<f64>;
    const NAME: &'static str = "avg";
    const SQL: SelectAggregator = SelectAggregator::Avg;
}

/// Returns the count of the number of times that the column is not null.
pub struct Count;
impl AggregationFunc for Count {
    type Result<Input: DecodeOwned> = i64;
    const NAME: &'static str = "count";
    const SQL: SelectAggregator = SelectAggregator::Count;
}

/// Returns the summary off all non-null values in the group.
/// If there are only null values in the group, this function will return null.
pub struct Sum;
impl AggregationFunc for Sum {
    type Result<Input: DecodeOwned> = Option<Input>;
    const NAME: &'static str = "sum";
    const SQL: SelectAggregator = SelectAggregator::Sum;
}

/// Returns the maximum value of all values in the group.
/// If there are only null values in the group, this function will return null.
pub struct Max;
impl AggregationFunc for Max {
    type Result<Input: DecodeOwned> = Option<Input>;
    const NAME: &'static str = "max";
    const SQL: SelectAggregator = SelectAggregator::Max;
}

/// Returns the minimum value of all values in the group.
/// If there are only null values in the group, this function will return null.
pub struct Min;
impl AggregationFunc for Min {
    type Result<Input: DecodeOwned> = Option<Input>;
    const NAME: &'static str = "min";
    const SQL: SelectAggregator = SelectAggregator::Min;
}

impl<F: Field, P: Path> FieldProxy<F, P> {
    const fn new_aggr<A: AggregationFunc>() -> AggregatedColumn<A, F, P> {
        AggregatedColumn {
            function: PhantomData,
            field: PhantomData,
            path: PhantomData,
        }
    }

    /// Get the column's average
    pub fn avg(&self) -> AggregatedColumn<Avg, F, P> {
        Self::new_aggr()
    }

    /// Get the column's count
    pub fn count(&self) -> AggregatedColumn<Count, F, P> {
        Self::new_aggr()
    }

    /// Get the column's sum
    pub fn sum(&self) -> AggregatedColumn<Sum, F, P> {
        Self::new_aggr()
    }

    /// Get the column's min
    pub fn min(&self) -> AggregatedColumn<Min, F, P> {
        Self::new_aggr()
    }

    /// Get the column's max
    pub fn max(&self) -> AggregatedColumn<Max, F, P> {
        Self::new_aggr()
    }
}

/// A column to select and call a aggregation function on
#[derive(Copy, Clone)]
pub struct AggregatedColumn<A, F, P> {
    function: PhantomData<A>,
    field: PhantomData<F>,
    path: PhantomData<P>,
}
