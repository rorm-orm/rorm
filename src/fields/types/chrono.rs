use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use rorm_db::sql::value::NullType;

use crate::conditions::Value;
use crate::fields::traits::simple::{SimpleFieldEq, SimpleFieldIn};
use crate::{impl_FieldMin_FieldMax, impl_FieldOrd, impl_FieldType};

impl_FieldType!(NaiveTime, ChronoNaiveTime, Value::ChronoNaiveTime);
impl<'rhs> SimpleFieldEq<'rhs, NaiveTime> for NaiveTime {
    fn into_value(rhs: NaiveTime) -> Value<'rhs> {
        Value::ChronoNaiveTime(rhs)
    }
}
impl<'rhs> SimpleFieldIn<'rhs, NaiveTime> for NaiveTime {
    fn into_value(rhs: NaiveTime) -> Value<'rhs> {
        Value::ChronoNaiveTime(rhs)
    }
}
impl_FieldOrd!(NaiveTime, NaiveTime, Value::ChronoNaiveTime);
impl_FieldOrd!(Option<NaiveTime>, Option<NaiveTime>, |option: Self| option
    .map(Value::ChronoNaiveTime)
    .unwrap_or(Value::Null(NullType::ChronoNaiveTime)));
impl_FieldMin_FieldMax!(NaiveTime);

impl_FieldType!(NaiveDate, ChronoNaiveDate, Value::ChronoNaiveDate);
impl<'rhs> SimpleFieldEq<'rhs, NaiveDate> for NaiveDate {
    fn into_value(rhs: NaiveDate) -> Value<'rhs> {
        Value::ChronoNaiveDate(rhs)
    }
}
impl<'rhs> SimpleFieldIn<'rhs, NaiveDate> for NaiveDate {
    fn into_value(rhs: NaiveDate) -> Value<'rhs> {
        Value::ChronoNaiveDate(rhs)
    }
}
impl_FieldOrd!(NaiveDate, NaiveDate, Value::ChronoNaiveDate);
impl_FieldOrd!(Option<NaiveDate>, Option<NaiveDate>, |option: Self| option
    .map(Value::ChronoNaiveDate)
    .unwrap_or(Value::Null(NullType::ChronoNaiveDate)));
impl_FieldMin_FieldMax!(NaiveDate);

impl_FieldType!(
    NaiveDateTime,
    ChronoNaiveDateTime,
    Value::ChronoNaiveDateTime
);
impl<'rhs> SimpleFieldEq<'rhs, NaiveDateTime> for NaiveDateTime {
    fn into_value(rhs: NaiveDateTime) -> Value<'rhs> {
        Value::ChronoNaiveDateTime(rhs)
    }
}
impl<'rhs> SimpleFieldIn<'rhs, NaiveDateTime> for NaiveDateTime {
    fn into_value(rhs: NaiveDateTime) -> Value<'rhs> {
        Value::ChronoNaiveDateTime(rhs)
    }
}
impl_FieldOrd!(NaiveDateTime, NaiveDateTime, Value::ChronoNaiveDateTime);
impl_FieldOrd!(
    Option<NaiveDateTime>,
    Option<NaiveDateTime>,
    |option: Self| option
        .map(Value::ChronoNaiveDateTime)
        .unwrap_or(Value::Null(NullType::ChronoNaiveDateTime))
);
impl_FieldMin_FieldMax!(NaiveDateTime);

impl_FieldType!(DateTime<Utc>, ChronoDateTime, Value::ChronoDateTime);
impl<'rhs> SimpleFieldEq<'rhs, DateTime<Utc>> for DateTime<Utc> {
    fn into_value(rhs: DateTime<Utc>) -> Value<'rhs> {
        Value::ChronoDateTime(rhs)
    }
}
impl<'rhs> SimpleFieldIn<'rhs, DateTime<Utc>> for DateTime<Utc> {
    fn into_value(rhs: DateTime<Utc>) -> Value<'rhs> {
        Value::ChronoDateTime(rhs)
    }
}
impl_FieldOrd!(DateTime<Utc>, DateTime<Utc>, Value::ChronoDateTime);
impl_FieldOrd!(
    Option<DateTime<Utc>>,
    Option<DateTime<Utc>>,
    |option: Self| option
        .map(Value::ChronoDateTime)
        .unwrap_or(Value::Null(NullType::ChronoDateTime))
);
impl_FieldMin_FieldMax!(DateTime<Utc>);
