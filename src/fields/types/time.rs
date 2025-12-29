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
impl SimpleFieldEq<Time> for Time {}
impl SimpleFieldIn<Time> for Time {}
impl SimpleFieldOrd<Time> for Time {}
impl_FieldMin_FieldMax!(Time);

impl_FieldType!(Date, TimeDate);
impl<'a> IntoValue<'a> for Date {
    fn into_value(self) -> Value<'a> {
        Value::TimeDate(self)
    }
}
impl SimpleFieldEq<Date> for Date {}
impl SimpleFieldIn<Date> for Date {}
impl SimpleFieldOrd<Date> for Date {}
impl_FieldMin_FieldMax!(Date);

impl_FieldType!(OffsetDateTime, TimeOffsetDateTime);
impl<'a> IntoValue<'a> for OffsetDateTime {
    fn into_value(self) -> Value<'a> {
        Value::TimeOffsetDateTime(self)
    }
}
impl SimpleFieldEq<OffsetDateTime> for OffsetDateTime {}
impl SimpleFieldIn<OffsetDateTime> for OffsetDateTime {}
impl SimpleFieldOrd<OffsetDateTime> for OffsetDateTime {}
impl_FieldMin_FieldMax!(OffsetDateTime);

impl_FieldType!(PrimitiveDateTime, TimePrimitiveDateTime);
impl<'a> IntoValue<'a> for PrimitiveDateTime {
    fn into_value(self) -> Value<'a> {
        Value::TimePrimitiveDateTime(self)
    }
}
impl SimpleFieldEq<PrimitiveDateTime> for PrimitiveDateTime {}
impl SimpleFieldIn<PrimitiveDateTime> for PrimitiveDateTime {}
impl SimpleFieldOrd<PrimitiveDateTime> for PrimitiveDateTime {}
impl_FieldMin_FieldMax!(PrimitiveDateTime);
