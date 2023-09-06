use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use rorm_db::sql::value::NullType;

use crate::conditions::Value;
use crate::internal::hmr::db_type;
use crate::{impl_AsDbType, impl_FieldEq, impl_FieldOrd};

impl_AsDbType!(NaiveTime, db_type::Time, Value::ChronoNaiveTime);
impl_AsDbType!(NaiveDate, db_type::Date, Value::ChronoNaiveDate);
impl_AsDbType!(NaiveDateTime, db_type::DateTime, Value::ChronoNaiveDateTime);
impl_AsDbType!(DateTime<Utc>, db_type::DateTime, Value::ChronoDateTime);
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, NaiveTime> for NaiveTime         { Value::ChronoNaiveTime });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, NaiveDate> for NaiveDate         { Value::ChronoNaiveDate });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, NaiveDateTime> for NaiveDateTime { Value::ChronoNaiveDateTime });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, DateTime<Utc>> for DateTime<Utc> { Value::ChronoDateTime });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<NaiveTime>> for Option<NaiveTime>         { |option: Self| option.map(Value::ChronoNaiveTime).unwrap_or(Value::Null(NullType::ChronoNaiveTime)) });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<NaiveDate>> for Option<NaiveDate>         { |option: Self| option.map(Value::ChronoNaiveDate).unwrap_or(Value::Null(NullType::ChronoNaiveDate)) });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<NaiveDateTime>> for Option<NaiveDateTime> { |option: Self| option.map(Value::ChronoNaiveDateTime).unwrap_or(Value::Null(NullType::ChronoNaiveDateTime)) });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<DateTime<Utc>>> for Option<DateTime<Utc>> { |option: Self| option.map(Value::ChronoDateTime).unwrap_or(Value::Null(NullType::ChronoDateTime)) });
impl_FieldOrd!(NaiveTime, NaiveTime, Value::ChronoNaiveTime);
impl_FieldOrd!(NaiveDate, NaiveDate, Value::ChronoNaiveDate);
impl_FieldOrd!(NaiveDateTime, NaiveDateTime, Value::ChronoNaiveDateTime);
impl_FieldOrd!(DateTime<Utc>, DateTime<Utc>, Value::ChronoDateTime);
#[rustfmt::skip]
impl_FieldOrd!(Option<NaiveTime>, Option<NaiveTime>, |option: Self| option.map(Value::ChronoNaiveTime).unwrap_or(Value::Null(NullType::ChronoNaiveTime)));
#[rustfmt::skip]
impl_FieldOrd!(Option<NaiveDate>, Option<NaiveDate>, |option: Self| option.map(Value::ChronoNaiveDate).unwrap_or(Value::Null(NullType::ChronoNaiveDate)));
#[rustfmt::skip]
impl_FieldOrd!(Option<NaiveDateTime>, Option<NaiveDateTime>, |option: Self| option.map(Value::ChronoNaiveDateTime).unwrap_or(Value::Null(NullType::ChronoNaiveDateTime)));
#[rustfmt::skip]
impl_FieldOrd!(Option<DateTime<Utc>>, Option<DateTime<Utc>>, |option: Self| option.map(Value::ChronoDateTime).unwrap_or(Value::Null(NullType::ChronoDateTime)));
