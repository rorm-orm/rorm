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
impl SimpleFieldEq for NaiveTime {}
impl SimpleFieldIn for NaiveTime {}
impl SimpleFieldOrd for NaiveTime {}
impl_FieldMin_FieldMax!(NaiveTime);

impl_FieldType!(NaiveDate, ChronoNaiveDate);
impl<'a> IntoValue<'a> for NaiveDate {
    fn into_value(self) -> Value<'a> {
        Value::ChronoNaiveDate(self)
    }
}
impl SimpleFieldEq for NaiveDate {}
impl SimpleFieldIn for NaiveDate {}
impl SimpleFieldOrd for NaiveDate {}
impl_FieldMin_FieldMax!(NaiveDate);

impl_FieldType!(NaiveDateTime, ChronoNaiveDateTime);
impl<'a> IntoValue<'a> for NaiveDateTime {
    fn into_value(self) -> Value<'a> {
        Value::ChronoNaiveDateTime(self)
    }
}
impl SimpleFieldEq for NaiveDateTime {}
impl SimpleFieldIn for NaiveDateTime {}
impl SimpleFieldOrd for NaiveDateTime {}
impl_FieldMin_FieldMax!(NaiveDateTime);

impl_FieldType!(DateTime<Utc>, ChronoDateTime);
impl<'a> IntoValue<'a> for DateTime<Utc> {
    fn into_value(self) -> Value<'a> {
        Value::ChronoDateTime(self)
    }
}
impl SimpleFieldEq for DateTime<Utc> {}
impl SimpleFieldIn for DateTime<Utc> {}
impl SimpleFieldOrd for DateTime<Utc> {}
impl_FieldMin_FieldMax!(DateTime<Utc>);
