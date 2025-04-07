use std::borrow::Cow;

use crate::conditions::Value;
use crate::db::sql::value::NullType;
use crate::fields::utils::check;
use crate::{
    impl_FieldEq, impl_FieldMin_FieldMax, impl_FieldOrd, impl_FieldSum_FieldAvg, impl_FieldType,
};

impl_FieldType!(bool, Bool, Value::Bool);
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, bool> for bool { Value::Bool });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<bool>> for Option<bool> { |option: Self| option.map(Value::Bool).unwrap_or(Value::Null(NullType::Bool)) });

impl_FieldType!(i16, I16, Value::I16);
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, i16> for i16 { Value::I16 });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<i16>> for Option<i16> { |option: Self| option.map(Value::I16).unwrap_or(Value::Null(NullType::I16)) });
impl_FieldOrd!(i16, i16, Value::I16);
impl_FieldOrd!(Option<i16>, Option<i16>, |option: Self| option
    .map(Value::I16)
    .unwrap_or(Value::Null(NullType::I16)));
impl_FieldSum_FieldAvg!(i16, sum_result: i64);
impl_FieldMin_FieldMax!(i16);

impl_FieldType!(i32, I32, Value::I32);
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, i32> for i32 { Value::I32 });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<i32>> for Option<i32> { |option: Self| option.map(Value::I32).unwrap_or(Value::Null(NullType::I32)) });
impl_FieldOrd!(i32, i32, Value::I32);
impl_FieldOrd!(Option<i32>, Option<i32>, |option: Self| option
    .map(Value::I32)
    .unwrap_or(Value::Null(NullType::I32)));
impl_FieldSum_FieldAvg!(i32, sum_result: i64);
impl_FieldMin_FieldMax!(i32);

impl_FieldType!(i64, I64, Value::I64);
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, i64> for i64 { Value::I64 });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<i64>> for Option<i64> { |option: Self| option.map(Value::I64).unwrap_or(Value::Null(NullType::I64)) });
impl_FieldOrd!(i64, i64, Value::I64);
impl_FieldOrd!(Option<i64>, Option<i64>, |option: Self| option
    .map(Value::I64)
    .unwrap_or(Value::Null(NullType::I64)));
impl_FieldSum_FieldAvg!(i64, sum_result: f64);
impl_FieldMin_FieldMax!(i64);

impl_FieldType!(f32, F32, Value::F32);
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, f32> for f32 { Value::F32 });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<f32>> for Option<f32> { |option: Self| option.map(Value::F32).unwrap_or(Value::Null(NullType::F32)) });
impl_FieldOrd!(f32, f32, Value::F32);
impl_FieldOrd!(Option<f32>, Option<f32>, |option: Self| option
    .map(Value::F32)
    .unwrap_or(Value::Null(NullType::F32)));
impl_FieldSum_FieldAvg!(f32, sum_result: f32);
impl_FieldMin_FieldMax!(f32);

impl_FieldType!(f64, F64, Value::F64);
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, f64> for f64 { Value::F64 });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<f64>> for Option<f64> { |option: Self| option.map(Value::F64).unwrap_or(Value::Null(NullType::F64)) });
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
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, &'rhs str> for String { conv_string });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, &'rhs String> for String { conv_string });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, String> for String { conv_string });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Cow<'rhs, str>> for String { conv_string });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<&'rhs str>> for Option<String> { |option: Option<_>| option.map(conv_string).unwrap_or(Value::Null(NullType::String)) });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<&'rhs String>> for Option<String> { |option: Option<_>| option.map(conv_string).unwrap_or(Value::Null(NullType::String)) });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<String>> for Option<String> { |option: Option<_>| option.map(conv_string).unwrap_or(Value::Null(NullType::String)) });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<Cow<'rhs, str>>> for Option<String> { |option: Option<_>| option.map(conv_string).unwrap_or(Value::Null(NullType::String)) });
impl_FieldOrd!(String, &'rhs str, conv_string);
impl_FieldOrd!(String, &'rhs String, conv_string);
impl_FieldOrd!(String, String, conv_string);
impl_FieldOrd!(String, Cow<'rhs, str>, conv_string);
impl_FieldMin_FieldMax!(String);
fn conv_string<'a>(value: impl Into<Cow<'a, str>>) -> Value<'a> {
    Value::String(value.into())
}

impl_FieldType!(Vec<u8>, Binary, conv_bytes, conv_bytes);
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, &'rhs [u8]> for Vec<u8> { conv_bytes });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, &'rhs Vec<u8>> for Vec<u8> { conv_bytes });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Vec<u8>> for Vec<u8> { conv_bytes });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Cow<'rhs, [u8]>> for Vec<u8> { conv_bytes });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<&'rhs [u8]>> for Option<Vec<u8>> { |option: Option<_>| option.map(conv_bytes).unwrap_or(Value::Null(NullType::Binary)) });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<&'rhs Vec<u8>>> for Option<Vec<u8>> { |option: Option<_>| option.map(conv_bytes).unwrap_or(Value::Null(NullType::Binary)) });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<Vec<u8>>> for Option<Vec<u8>> { |option: Option<_>| option.map(conv_bytes).unwrap_or(Value::Null(NullType::Binary)) });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<Cow<'rhs, [u8]>>> for Option<Vec<u8>> { |option: Option<_>| option.map(conv_bytes).unwrap_or(Value::Null(NullType::Binary)) });
impl_FieldOrd!(Vec<u8>, &'rhs [u8], conv_bytes);
impl_FieldOrd!(Vec<u8>, &'rhs Vec<u8>, conv_bytes);
impl_FieldOrd!(Vec<u8>, Vec<u8>, conv_bytes);
impl_FieldOrd!(Vec<u8>, Cow<'rhs, [u8]>, conv_bytes);
fn conv_bytes<'a>(value: impl Into<Cow<'a, [u8]>>) -> Value<'a> {
    Value::Binary(value.into())
}
