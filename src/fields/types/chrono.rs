use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};

use crate::conditions::Value;
use crate::fields::traits::into_value::IntoValue;
use crate::fields::traits::simple::{SimpleFieldEq, SimpleFieldIn, SimpleFieldOrd};
use crate::{impl_FieldMin_FieldMax, impl_FieldType};

impl_FieldType!(NaiveTime, ChronoNaiveTime);
impl<'a> IntoValue<'a> for NaiveTime {
    fn into_value(self) -> Value<'a> {
        Value::ChronoNaiveTime(self)
    }
}
impl SimpleFieldEq<NaiveTime> for NaiveTime {}
impl SimpleFieldIn<NaiveTime> for NaiveTime {}
impl SimpleFieldOrd<NaiveTime> for NaiveTime {}
impl_FieldMin_FieldMax!(NaiveTime);

impl_FieldType!(NaiveDate, ChronoNaiveDate);
impl<'a> IntoValue<'a> for NaiveDate {
    fn into_value(self) -> Value<'a> {
        Value::ChronoNaiveDate(self)
    }
}
impl SimpleFieldEq<NaiveDate> for NaiveDate {}
impl SimpleFieldIn<NaiveDate> for NaiveDate {}
impl SimpleFieldOrd<NaiveDate> for NaiveDate {}
impl_FieldMin_FieldMax!(NaiveDate);

impl_FieldType!(NaiveDateTime, ChronoNaiveDateTime);
impl<'a> IntoValue<'a> for NaiveDateTime {
    fn into_value(self) -> Value<'a> {
        Value::ChronoNaiveDateTime(self)
    }
}
impl SimpleFieldEq<NaiveDateTime> for NaiveDateTime {}
impl SimpleFieldIn<NaiveDateTime> for NaiveDateTime {}
impl SimpleFieldOrd<NaiveDateTime> for NaiveDateTime {}
impl_FieldMin_FieldMax!(NaiveDateTime);

impl_FieldType!(DateTime<Utc>, ChronoDateTime);
impl<'a> IntoValue<'a> for DateTime<Utc> {
    fn into_value(self) -> Value<'a> {
        Value::ChronoDateTime(self)
    }
}
impl SimpleFieldEq<DateTime<Utc>> for DateTime<Utc> {}
impl SimpleFieldIn<DateTime<Utc>> for DateTime<Utc> {}
impl SimpleFieldOrd<DateTime<Utc>> for DateTime<Utc> {}
impl_FieldMin_FieldMax!(DateTime<Utc>);
