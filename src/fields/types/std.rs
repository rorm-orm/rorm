use std::borrow::Cow;

use crate::conditions::Value;
use crate::db::sql::value::NullType;
#[cfg(feature = "postgres-only")]
use crate::fields::traits::simple::SimpleFieldILike;
use crate::fields::traits::simple::{SimpleFieldEq, SimpleFieldIn, SimpleFieldLike};
use crate::fields::utils::check;
use crate::{impl_FieldMin_FieldMax, impl_FieldOrd, impl_FieldSum_FieldAvg, impl_FieldType};

impl_FieldType!(bool, Bool);
impl<'rhs> SimpleFieldEq<bool> for bool {}
impl<'rhs> SimpleFieldIn<bool> for bool {}

impl_FieldType!(i16, I16);
impl<'rhs> SimpleFieldEq<i16> for i16 {}
impl<'rhs> SimpleFieldIn<i16> for i16 {}

impl_FieldOrd!(i16, i16, Value::I16);
impl_FieldOrd!(Option<i16>, Option<i16>, |option: Self| option
    .map(Value::I16)
    .unwrap_or(Value::Null(NullType::I16)));
impl_FieldSum_FieldAvg!(i16, sum_result: i64);
impl_FieldMin_FieldMax!(i16);

impl_FieldType!(i32, I32);
impl<'rhs> SimpleFieldEq<i32> for i32 {}
impl<'rhs> SimpleFieldIn<i32> for i32 {}

impl_FieldOrd!(i32, i32, Value::I32);
impl_FieldOrd!(Option<i32>, Option<i32>, |option: Self| option
    .map(Value::I32)
    .unwrap_or(Value::Null(NullType::I32)));
impl_FieldSum_FieldAvg!(i32, sum_result: i64);
impl_FieldMin_FieldMax!(i32);

impl_FieldType!(i64, I64);
impl<'rhs> SimpleFieldEq<i64> for i64 {}
impl<'rhs> SimpleFieldIn<i64> for i64 {}

impl_FieldOrd!(i64, i64, Value::I64);
impl_FieldOrd!(Option<i64>, Option<i64>, |option: Self| option
    .map(Value::I64)
    .unwrap_or(Value::Null(NullType::I64)));
impl_FieldSum_FieldAvg!(i64, sum_result: f64);
impl_FieldMin_FieldMax!(i64);

impl_FieldType!(f32, F32);
impl<'rhs> SimpleFieldEq<f32> for f32 {}
impl<'rhs> SimpleFieldIn<f32> for f32 {}

impl_FieldOrd!(f32, f32, Value::F32);
impl_FieldOrd!(Option<f32>, Option<f32>, |option: Self| option
    .map(Value::F32)
    .unwrap_or(Value::Null(NullType::F32)));
impl_FieldSum_FieldAvg!(f32, sum_result: f32);
impl_FieldMin_FieldMax!(f32);

impl_FieldType!(f64, F64);
impl<'rhs> SimpleFieldEq<f64> for f64 {}
impl<'rhs> SimpleFieldIn<f64> for f64 {}

impl_FieldOrd!(f64, f64, Value::F64);
impl_FieldOrd!(Option<f64>, Option<f64>, |option: Self| option
    .map(Value::F64)
    .unwrap_or(Value::Null(NullType::F64)));
impl_FieldSum_FieldAvg!(f64, sum_result: f64);
impl_FieldMin_FieldMax!(f64);

impl_FieldType!(String, String, check::string_check);
impl<'rhs> SimpleFieldEq<&'rhs str> for String {}
impl<'rhs> SimpleFieldIn<&'rhs str> for String {}
impl<'rhs> SimpleFieldEq<&'rhs String> for String {}
impl<'rhs> SimpleFieldIn<&'rhs String> for String {}
impl<'rhs> SimpleFieldEq<String> for String {}
impl<'rhs> SimpleFieldIn<String> for String {}
impl<'rhs> SimpleFieldEq<Cow<'rhs, str>> for String {}
impl<'rhs> SimpleFieldIn<Cow<'rhs, str>> for String {}
impl<'rhs> SimpleFieldLike<&'rhs str> for String {}
impl<'rhs> SimpleFieldLike<&'rhs String> for String {}
impl<'rhs> SimpleFieldLike<String> for String {}
impl<'rhs> SimpleFieldLike<Cow<'rhs, str>> for String {}
#[cfg(feature = "postgres-only")]
impl<'rhs> SimpleFieldILike<&'rhs str> for String {}
#[cfg(feature = "postgres-only")]
impl<'rhs> SimpleFieldILike<&'rhs String> for String {}
#[cfg(feature = "postgres-only")]
impl SimpleFieldILike<String> for String {}
#[cfg(feature = "postgres-only")]
impl<'rhs> SimpleFieldILike<Cow<'rhs, str>> for String {}

impl_FieldOrd!(String, &'rhs str, conv_string);
impl_FieldOrd!(String, &'rhs String, conv_string);
impl_FieldOrd!(String, String, conv_string);
impl_FieldOrd!(String, Cow<'rhs, str>, conv_string);
impl_FieldMin_FieldMax!(String);
fn conv_string<'a>(value: impl Into<Cow<'a, str>>) -> Value<'a> {
    Value::String(value.into())
}

impl_FieldType!(Vec<u8>, Binary);
impl<'rhs> SimpleFieldEq<&'rhs [u8]> for Vec<u8> {}
impl<'rhs> SimpleFieldEq<&'rhs Vec<u8>> for Vec<u8> {}
impl<'rhs> SimpleFieldEq<Vec<u8>> for Vec<u8> {}
impl<'rhs> SimpleFieldEq<Cow<'rhs, [u8]>> for Vec<u8> {}

impl_FieldOrd!(Vec<u8>, &'rhs [u8], conv_bytes);
impl_FieldOrd!(Vec<u8>, &'rhs Vec<u8>, conv_bytes);
impl_FieldOrd!(Vec<u8>, Vec<u8>, conv_bytes);
impl_FieldOrd!(Vec<u8>, Cow<'rhs, [u8]>, conv_bytes);
fn conv_bytes<'a>(value: impl Into<Cow<'a, [u8]>>) -> Value<'a> {
    Value::Binary(value.into())
}
