use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use rorm_db::sql::value::NullType;

use crate::conditions::Value;
use crate::fields::traits::into_value::IntoValue;
use crate::fields::traits::simple::{SimpleFieldEq, SimpleFieldIn};
use crate::{impl_FieldMin_FieldMax, impl_FieldOrd, impl_FieldType};

impl_FieldType!(NaiveTime, ChronoNaiveTime);
impl<'a> IntoValue<'a> for NaiveTime {
    fn into_value(self) -> Value<'a> {
        Value::ChronoNaiveTime(self)
    }
}
impl SimpleFieldEq<NaiveTime> for NaiveTime {}
impl SimpleFieldIn<NaiveTime> for NaiveTime {}
impl_FieldOrd!(NaiveTime, NaiveTime, Value::ChronoNaiveTime);
impl_FieldOrd!(Option<NaiveTime>, Option<NaiveTime>, |option: Self| option
    .map(Value::ChronoNaiveTime)
    .unwrap_or(Value::Null(NullType::ChronoNaiveTime)));
impl_FieldMin_FieldMax!(NaiveTime);

impl_FieldType!(NaiveDate, ChronoNaiveDate);
impl<'a> IntoValue<'a> for NaiveDate {
    fn into_value(self) -> Value<'a> {
        Value::ChronoNaiveDate(self)
    }
}
impl SimpleFieldEq<NaiveDate> for NaiveDate {}
impl SimpleFieldIn<NaiveDate> for NaiveDate {}
impl_FieldOrd!(NaiveDate, NaiveDate, Value::ChronoNaiveDate);
impl_FieldOrd!(Option<NaiveDate>, Option<NaiveDate>, |option: Self| option
    .map(Value::ChronoNaiveDate)
    .unwrap_or(Value::Null(NullType::ChronoNaiveDate)));
impl_FieldMin_FieldMax!(NaiveDate);

impl_FieldType!(NaiveDateTime, ChronoNaiveDateTime);
impl<'a> IntoValue<'a> for NaiveDateTime {
    fn into_value(self) -> Value<'a> {
        Value::ChronoNaiveDateTime(self)
    }
}
impl SimpleFieldEq<NaiveDateTime> for NaiveDateTime {}
impl SimpleFieldIn<NaiveDateTime> for NaiveDateTime {}
impl_FieldOrd!(NaiveDateTime, NaiveDateTime, Value::ChronoNaiveDateTime);
impl_FieldOrd!(
    Option<NaiveDateTime>,
    Option<NaiveDateTime>,
    |option: Self| option
        .map(Value::ChronoNaiveDateTime)
        .unwrap_or(Value::Null(NullType::ChronoNaiveDateTime))
);
impl_FieldMin_FieldMax!(NaiveDateTime);

impl_FieldType!(DateTime<Utc>, ChronoDateTime);
impl<'a> IntoValue<'a> for DateTime<Utc> {
    fn into_value(self) -> Value<'a> {
        Value::ChronoDateTime(self)
    }
}
impl SimpleFieldEq<DateTime<Utc>> for DateTime<Utc> {}
impl SimpleFieldIn<DateTime<Utc>> for DateTime<Utc> {}
impl_FieldOrd!(DateTime<Utc>, DateTime<Utc>, Value::ChronoDateTime);
impl_FieldOrd!(
    Option<DateTime<Utc>>,
    Option<DateTime<Utc>>,
    |option: Self| option
        .map(Value::ChronoDateTime)
        .unwrap_or(Value::Null(NullType::ChronoDateTime))
);
impl_FieldMin_FieldMax!(DateTime<Utc>);
