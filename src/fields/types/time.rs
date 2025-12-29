use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};

use crate::conditions::Value;
use crate::fields::traits::into_value::IntoValue;
use crate::fields::traits::simple::{SimpleFieldEq, SimpleFieldIn, SimpleFieldOrd};
use crate::{impl_FieldMin_FieldMax, impl_FieldType};

impl_FieldType!(Time, TimeTime);
impl<'a> IntoValue<'a> for Time {
    fn into_value(self) -> Value<'a> {
        Value::TimeTime(self)
    }
}
impl SimpleFieldEq for Time {}
impl SimpleFieldIn for Time {}
impl SimpleFieldOrd for Time {}
impl_FieldMin_FieldMax!(Time);

impl_FieldType!(Date, TimeDate);
impl<'a> IntoValue<'a> for Date {
    fn into_value(self) -> Value<'a> {
        Value::TimeDate(self)
    }
}
impl SimpleFieldEq for Date {}
impl SimpleFieldIn for Date {}
impl SimpleFieldOrd for Date {}
impl_FieldMin_FieldMax!(Date);

impl_FieldType!(OffsetDateTime, TimeOffsetDateTime);
impl<'a> IntoValue<'a> for OffsetDateTime {
    fn into_value(self) -> Value<'a> {
        Value::TimeOffsetDateTime(self)
    }
}
impl SimpleFieldEq for OffsetDateTime {}
impl SimpleFieldIn for OffsetDateTime {}
impl SimpleFieldOrd for OffsetDateTime {}
impl_FieldMin_FieldMax!(OffsetDateTime);

impl_FieldType!(PrimitiveDateTime, TimePrimitiveDateTime);
impl<'a> IntoValue<'a> for PrimitiveDateTime {
    fn into_value(self) -> Value<'a> {
        Value::TimePrimitiveDateTime(self)
    }
}
impl SimpleFieldEq for PrimitiveDateTime {}
impl SimpleFieldIn for PrimitiveDateTime {}
impl SimpleFieldOrd for PrimitiveDateTime {}
impl_FieldMin_FieldMax!(PrimitiveDateTime);
