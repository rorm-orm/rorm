use std::borrow::Cow;

use crate::conditions::Value;
use crate::db::sql::value::NullType;
#[cfg(feature = "postgres-only")]
use crate::fields::traits::simple::SimpleFieldILike;
use crate::fields::traits::simple::{SimpleFieldEq, SimpleFieldIn, SimpleFieldLike};
use crate::fields::utils::check;
use crate::{impl_FieldMin_FieldMax, impl_FieldOrd, impl_FieldSum_FieldAvg, impl_FieldType};

impl_FieldType!(bool, Bool, Value::Bool);
impl<'rhs> SimpleFieldEq<'rhs, bool> for bool {
    fn into_value(rhs: bool) -> Value<'rhs> {
        Value::Bool(rhs)
    }
}
impl<'rhs> SimpleFieldIn<'rhs, bool> for bool {
    fn into_value(rhs: bool) -> Value<'rhs> {
        Value::Bool(rhs)
    }
}

impl_FieldType!(i16, I16, Value::I16);
impl<'rhs> SimpleFieldEq<'rhs, i16> for i16 {
    fn into_value(rhs: i16) -> Value<'rhs> {
        Value::I16(rhs)
    }
}
impl<'rhs> SimpleFieldIn<'rhs, i16> for i16 {
    fn into_value(rhs: i16) -> Value<'rhs> {
        Value::I16(rhs)
    }
}

impl_FieldOrd!(i16, i16, Value::I16);
impl_FieldOrd!(Option<i16>, Option<i16>, |option: Self| option
    .map(Value::I16)
    .unwrap_or(Value::Null(NullType::I16)));
impl_FieldSum_FieldAvg!(i16, sum_result: i64);
impl_FieldMin_FieldMax!(i16);

impl_FieldType!(i32, I32, Value::I32);
impl<'rhs> SimpleFieldEq<'rhs, i32> for i32 {
    fn into_value(rhs: i32) -> Value<'rhs> {
        Value::I32(rhs)
    }
}
impl<'rhs> SimpleFieldIn<'rhs, i32> for i32 {
    fn into_value(rhs: i32) -> Value<'rhs> {
        Value::I32(rhs)
    }
}

impl_FieldOrd!(i32, i32, Value::I32);
impl_FieldOrd!(Option<i32>, Option<i32>, |option: Self| option
    .map(Value::I32)
    .unwrap_or(Value::Null(NullType::I32)));
impl_FieldSum_FieldAvg!(i32, sum_result: i64);
impl_FieldMin_FieldMax!(i32);

impl_FieldType!(i64, I64, Value::I64);
impl<'rhs> SimpleFieldEq<'rhs, i64> for i64 {
    fn into_value(rhs: i64) -> Value<'rhs> {
        Value::I64(rhs)
    }
}
impl<'rhs> SimpleFieldIn<'rhs, i64> for i64 {
    fn into_value(rhs: i64) -> Value<'rhs> {
        Value::I64(rhs)
    }
}

impl_FieldOrd!(i64, i64, Value::I64);
impl_FieldOrd!(Option<i64>, Option<i64>, |option: Self| option
    .map(Value::I64)
    .unwrap_or(Value::Null(NullType::I64)));
impl_FieldSum_FieldAvg!(i64, sum_result: f64);
impl_FieldMin_FieldMax!(i64);

impl_FieldType!(f32, F32, Value::F32);
impl<'rhs> SimpleFieldEq<'rhs, f32> for f32 {
    fn into_value(rhs: f32) -> Value<'rhs> {
        Value::F32(rhs)
    }
}
impl<'rhs> SimpleFieldIn<'rhs, f32> for f32 {
    fn into_value(rhs: f32) -> Value<'rhs> {
        Value::F32(rhs)
    }
}

impl_FieldOrd!(f32, f32, Value::F32);
impl_FieldOrd!(Option<f32>, Option<f32>, |option: Self| option
    .map(Value::F32)
    .unwrap_or(Value::Null(NullType::F32)));
impl_FieldSum_FieldAvg!(f32, sum_result: f32);
impl_FieldMin_FieldMax!(f32);

impl_FieldType!(f64, F64, Value::F64);
impl<'rhs> SimpleFieldEq<'rhs, f64> for f64 {
    fn into_value(rhs: f64) -> Value<'rhs> {
        Value::F64(rhs)
    }
}
impl<'rhs> SimpleFieldIn<'rhs, f64> for f64 {
    fn into_value(rhs: f64) -> Value<'rhs> {
        Value::F64(rhs)
    }
}

impl_FieldOrd!(f64, f64, Value::F64);
impl_FieldOrd!(Option<f64>, Option<f64>, |option: Self| option
    .map(Value::F64)
    .unwrap_or(Value::Null(NullType::F64)));
impl_FieldSum_FieldAvg!(f64, sum_result: f64);
impl_FieldMin_FieldMax!(f64);

impl_FieldType!(
    String,
    String,
    conv_string,
    conv_string,
    check::string_check
);
impl<'rhs> SimpleFieldEq<'rhs, &'rhs str> for String {
    fn into_value(rhs: &'rhs str) -> Value<'rhs> {
        conv_string(rhs)
    }
}
impl<'rhs> SimpleFieldIn<'rhs, &'rhs str> for String {
    fn into_value(rhs: &'rhs str) -> Value<'rhs> {
        conv_string(rhs)
    }
}
impl<'rhs> SimpleFieldEq<'rhs, &'rhs String> for String {
    fn into_value(rhs: &'rhs String) -> Value<'rhs> {
        conv_string(rhs)
    }
}
impl<'rhs> SimpleFieldIn<'rhs, &'rhs String> for String {
    fn into_value(rhs: &'rhs String) -> Value<'rhs> {
        conv_string(rhs)
    }
}
impl<'rhs> SimpleFieldEq<'rhs, String> for String {
    fn into_value(rhs: String) -> Value<'rhs> {
        conv_string(rhs)
    }
}
impl<'rhs> SimpleFieldIn<'rhs, String> for String {
    fn into_value(rhs: String) -> Value<'rhs> {
        conv_string(rhs)
    }
}
impl<'rhs> SimpleFieldEq<'rhs, Cow<'rhs, str>> for String {
    fn into_value(rhs: Cow<'rhs, str>) -> Value<'rhs> {
        conv_string(rhs)
    }
}
impl<'rhs> SimpleFieldIn<'rhs, Cow<'rhs, str>> for String {
    fn into_value(rhs: Cow<'rhs, str>) -> Value<'rhs> {
        conv_string(rhs)
    }
}
impl<'rhs> SimpleFieldLike<'rhs, &'rhs str> for String {
    fn into_value(rhs: &'rhs str) -> Value<'rhs> {
        conv_string(rhs)
    }
}
impl<'rhs> SimpleFieldLike<'rhs, &'rhs String> for String {
    fn into_value(rhs: &'rhs String) -> Value<'rhs> {
        conv_string(rhs)
    }
}
impl<'rhs> SimpleFieldLike<'rhs, String> for String {
    fn into_value(rhs: String) -> Value<'rhs> {
        conv_string(rhs)
    }
}
impl<'rhs> SimpleFieldLike<'rhs, Cow<'rhs, str>> for String {
    fn into_value(rhs: Cow<'rhs, str>) -> Value<'rhs> {
        conv_string(rhs)
    }
}
#[cfg(feature = "postgres-only")]
impl<'rhs> SimpleFieldILike<'rhs, &'rhs str> for String {
    fn into_value(rhs: &'rhs str) -> Value<'rhs> {
        conv_string(rhs)
    }
}
#[cfg(feature = "postgres-only")]
impl<'rhs> SimpleFieldILike<'rhs, &'rhs String> for String {
    fn into_value(rhs: &'rhs String) -> Value<'rhs> {
        conv_string(rhs)
    }
}
#[cfg(feature = "postgres-only")]
impl<'rhs> SimpleFieldILike<'rhs, String> for String {
    fn into_value(rhs: String) -> Value<'rhs> {
        conv_string(rhs)
    }
}
#[cfg(feature = "postgres-only")]
impl<'rhs> SimpleFieldILike<'rhs, Cow<'rhs, str>> for String {
    fn into_value(rhs: Cow<'rhs, str>) -> Value<'rhs> {
        conv_string(rhs)
    }
}

impl_FieldOrd!(String, &'rhs str, conv_string);
impl_FieldOrd!(String, &'rhs String, conv_string);
impl_FieldOrd!(String, String, conv_string);
impl_FieldOrd!(String, Cow<'rhs, str>, conv_string);
impl_FieldMin_FieldMax!(String);
fn conv_string<'a>(value: impl Into<Cow<'a, str>>) -> Value<'a> {
    Value::String(value.into())
}

impl_FieldType!(Vec<u8>, Binary, conv_bytes, conv_bytes);
impl<'rhs> SimpleFieldEq<'rhs, &'rhs [u8]> for Vec<u8> {
    fn into_value(rhs: &'rhs [u8]) -> Value<'rhs> {
        conv_bytes(rhs)
    }
}
impl<'rhs> SimpleFieldEq<'rhs, &'rhs Vec<u8>> for Vec<u8> {
    fn into_value(rhs: &'rhs Vec<u8>) -> Value<'rhs> {
        conv_bytes(rhs)
    }
}
impl<'rhs> SimpleFieldEq<'rhs, Vec<u8>> for Vec<u8> {
    fn into_value(rhs: Vec<u8>) -> Value<'rhs> {
        conv_bytes(rhs)
    }
}
impl<'rhs> SimpleFieldEq<'rhs, Cow<'rhs, [u8]>> for Vec<u8> {
    fn into_value(rhs: Cow<'rhs, [u8]>) -> Value<'rhs> {
        conv_bytes(rhs)
    }
}

impl_FieldOrd!(Vec<u8>, &'rhs [u8], conv_bytes);
impl_FieldOrd!(Vec<u8>, &'rhs Vec<u8>, conv_bytes);
impl_FieldOrd!(Vec<u8>, Vec<u8>, conv_bytes);
impl_FieldOrd!(Vec<u8>, Cow<'rhs, [u8]>, conv_bytes);
fn conv_bytes<'a>(value: impl Into<Cow<'a, [u8]>>) -> Value<'a> {
    Value::Binary(value.into())
}
