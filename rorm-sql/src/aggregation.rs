/**
Representation of an aggregator function
*/
#[derive(Copy, Clone, Debug)]
pub enum SelectAggregator {
    /// Returns the average value of all non-null values.
    /// The result of avg is a floating point value, except all input values are null, than the
    /// result will also be null.
    Avg,
    /// Returns the count of the number of times that the column is not null.
    Count,
    /// Returns the summary off all non-null values in the group.
    /// If there are only null values in the group, this function will return null.
    Sum,
    /// Returns the maximum value of all values in the group.
    /// If there are only null values in the group, this function will return null.
    Max,
    /// Returns the minimum value of all values in the group.
    /// If there are only null values in the group, this function will return null.
    Min,
}
