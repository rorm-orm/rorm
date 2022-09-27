use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

/**
This enum represents a value
 */
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Value<'a> {
    /// null representation
    Null,
    /// Representation of an identifier, e.g. a column.
    /// This variant will not be escaped, so do not
    /// pass unchecked data to it.
    Ident(&'a str),
    /// String representation
    String(&'a str),
    /// i64 representation
    I64(i64),
    /// i32 representation
    I32(i32),
    /// i16 representation
    I16(i16),
    /// Bool representation
    Bool(bool),
    /// f64 representation
    F64(f64),
    /// f32 representation
    F32(f32),
    /// binary representation
    Binary(&'a [u8]),
    /// Naive Time representation
    NaiveTime(NaiveTime),
    /// Naive Date representation
    NaiveDate(NaiveDate),
    /// Naive DateTime representation
    NaiveDateTime(NaiveDateTime),
}

macro_rules! impl_from_with_lft {
    ($variant:ident, $T:ty) => {
        impl<'a> From<&'a $T> for Value<'a> {
            fn from(value: &'a $T) -> Self {
                Value::$variant(value)
            }
        }
    };
}

impl_from_with_lft!(Binary, [u8]);
impl_from_with_lft!(String, str);

macro_rules! impl_from {
    ($variant:ident, $T:ty) => {
        impl From<$T> for Value<'static> {
            fn from(value: $T) -> Self {
                Value::$variant(value)
            }
        }
    };
}
impl_from!(I64, i64);
impl_from!(I32, i32);
impl_from!(I16, i16);
impl_from!(Bool, bool);
impl_from!(F64, f64);
impl_from!(F32, f32);
impl_from!(NaiveDate, chrono::NaiveDate);
impl_from!(NaiveTime, chrono::NaiveTime);
impl_from!(NaiveDateTime, chrono::NaiveDateTime);

impl<'a, T> From<&'a T> for Value<'static>
where
    Self: From<T>,
    T: Copy,
{
    fn from(reference: &'a T) -> Self {
        Self::from(*reference)
    }
}
