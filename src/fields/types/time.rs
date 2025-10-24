use rorm_db::sql::value::NullType;
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};

use crate::conditions::Value;
use crate::fields::traits::simple::{SimpleFieldEq, SimpleFieldIn};
use crate::{impl_FieldMin_FieldMax, impl_FieldOrd, impl_FieldType};

impl_FieldType!(Time, TimeTime, Value::TimeTime);
impl<'rhs> SimpleFieldEq<'rhs, Time> for Time {
    fn into_value(rhs: Time) -> Value<'rhs> {
        Value::TimeTime(rhs)
    }
}
impl<'rhs> SimpleFieldIn<'rhs, Time> for Time {
    fn into_value(rhs: Time) -> Value<'rhs> {
        Value::TimeTime(rhs)
    }
}

impl_FieldOrd!(Time, Time, Value::TimeTime);
impl_FieldOrd!(Option<Time>, Option<Time>, |option: Self| option
    .map(Value::TimeTime)
    .unwrap_or(Value::Null(NullType::TimeTime)));
impl_FieldMin_FieldMax!(Time);

impl_FieldType!(Date, TimeDate, Value::TimeDate);
impl<'rhs> SimpleFieldEq<'rhs, Date> for Date {
    fn into_value(rhs: Date) -> Value<'rhs> {
        Value::TimeDate(rhs)
    }
}
impl<'rhs> SimpleFieldIn<'rhs, Date> for Date {
    fn into_value(rhs: Date) -> Value<'rhs> {
        Value::TimeDate(rhs)
    }
}

impl_FieldOrd!(Date, Date, Value::TimeDate);
impl_FieldOrd!(Option<Date>, Option<Date>, |option: Self| option
    .map(Value::TimeDate)
    .unwrap_or(Value::Null(NullType::TimeDate)));
impl_FieldMin_FieldMax!(Date);

impl_FieldType!(
    OffsetDateTime,
    TimeOffsetDateTime,
    Value::TimeOffsetDateTime
);
impl<'rhs> SimpleFieldEq<'rhs, OffsetDateTime> for OffsetDateTime {
    fn into_value(rhs: OffsetDateTime) -> Value<'rhs> {
        Value::TimeOffsetDateTime(rhs)
    }
}
impl<'rhs> SimpleFieldIn<'rhs, OffsetDateTime> for OffsetDateTime {
    fn into_value(rhs: OffsetDateTime) -> Value<'rhs> {
        Value::TimeOffsetDateTime(rhs)
    }
}

impl_FieldOrd!(OffsetDateTime, OffsetDateTime, Value::TimeOffsetDateTime);
impl_FieldOrd!(
    Option<OffsetDateTime>,
    Option<OffsetDateTime>,
    |option: Self| option
        .map(Value::TimeOffsetDateTime)
        .unwrap_or(Value::Null(NullType::TimeOffsetDateTime))
);
impl_FieldMin_FieldMax!(OffsetDateTime);

impl_FieldType!(
    PrimitiveDateTime,
    TimePrimitiveDateTime,
    Value::TimePrimitiveDateTime
);
impl<'rhs> SimpleFieldEq<'rhs, PrimitiveDateTime> for PrimitiveDateTime {
    fn into_value(rhs: PrimitiveDateTime) -> Value<'rhs> {
        Value::TimePrimitiveDateTime(rhs)
    }
}
impl<'rhs> SimpleFieldIn<'rhs, PrimitiveDateTime> for PrimitiveDateTime {
    fn into_value(rhs: PrimitiveDateTime) -> Value<'rhs> {
        Value::TimePrimitiveDateTime(rhs)
    }
}

impl_FieldOrd!(
    PrimitiveDateTime,
    PrimitiveDateTime,
    Value::TimePrimitiveDateTime
);
impl_FieldOrd!(
    Option<PrimitiveDateTime>,
    Option<PrimitiveDateTime>,
    |option: Self| option
        .map(Value::TimePrimitiveDateTime)
        .unwrap_or(Value::Null(NullType::TimePrimitiveDateTime))
);
impl_FieldMin_FieldMax!(PrimitiveDateTime);
