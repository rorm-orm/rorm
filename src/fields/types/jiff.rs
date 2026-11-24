use jiff::{civil, Timestamp};

use crate::conditions::Value;
use crate::fields::traits::into_value::IntoValue;
use crate::fields::traits::simple::{SimpleFieldEq, SimpleFieldIn, SimpleFieldOrd};
use crate::{impl_FieldMin_FieldMax, impl_FieldType};

impl_FieldType!(civil::Time, JiffCivilTime);
impl<'a> IntoValue<'a> for civil::Time {
    fn into_value(self) -> Value<'a> {
        Value::JiffCivilTime(self)
    }
}
impl SimpleFieldEq for civil::Time {}
impl SimpleFieldIn for civil::Time {}
impl SimpleFieldOrd for civil::Time {}
impl_FieldMin_FieldMax!(civil::Time);

impl_FieldType!(civil::Date, JiffCivilDate);
impl<'a> IntoValue<'a> for civil::Date {
    fn into_value(self) -> Value<'a> {
        Value::JiffCivilDate(self)
    }
}
impl SimpleFieldEq for civil::Date {}
impl SimpleFieldIn for civil::Date {}
impl SimpleFieldOrd for civil::Date {}
impl_FieldMin_FieldMax!(civil::Date);

impl_FieldType!(civil::DateTime, JiffCivilDateTime);
impl<'a> IntoValue<'a> for civil::DateTime {
    fn into_value(self) -> Value<'a> {
        Value::JiffCivilDateTime(self)
    }
}
impl SimpleFieldEq for civil::DateTime {}
impl SimpleFieldIn for civil::DateTime {}
impl SimpleFieldOrd for civil::DateTime {}
impl_FieldMin_FieldMax!(civil::DateTime);

impl_FieldType!(Timestamp, JiffTimestamp);
impl<'a> IntoValue<'a> for Timestamp {
    fn into_value(self) -> Value<'a> {
        Value::JiffTimestamp(self)
    }
}
impl SimpleFieldEq for Timestamp {}
impl SimpleFieldIn for Timestamp {}
impl SimpleFieldOrd for Timestamp {}
impl_FieldMin_FieldMax!(Timestamp);
