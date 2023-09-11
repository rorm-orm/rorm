use std::borrow::Cow;

use crate::conditions::Value;
use crate::db::sql::value::NullType;
use crate::internal::hmr::db_type;
use crate::{impl_AsDbType, impl_FieldEq, impl_FieldOrd};

impl_AsDbType!(bool, db_type::Boolean, Value::Bool);
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, bool> for bool { Value::Bool });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<bool>> for Option<bool> { |option: Self| option.map(Value::Bool).unwrap_or(Value::Null(NullType::Bool)) });

impl_AsDbType!(i16, db_type::Int16, Value::I16);
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, i16> for i16 { Value::I16 });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<i16>> for Option<i16> { |option: Self| option.map(Value::I16).unwrap_or(Value::Null(NullType::I16)) });
impl_FieldOrd!(i16, i16, Value::I16);
impl_FieldOrd!(Option<i16>, Option<i16>, |option: Self| option
    .map(Value::I16)
    .unwrap_or(Value::Null(NullType::I16)));

impl_AsDbType!(i32, db_type::Int32, Value::I32);
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, i32> for i32 { Value::I32 });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<i32>> for Option<i32> { |option: Self| option.map(Value::I32).unwrap_or(Value::Null(NullType::I32)) });
impl_FieldOrd!(i32, i32, Value::I32);
impl_FieldOrd!(Option<i32>, Option<i32>, |option: Self| option
    .map(Value::I32)
    .unwrap_or(Value::Null(NullType::I32)));

impl_AsDbType!(i64, db_type::Int64, Value::I64);
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, i64> for i64 { Value::I64 });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<i64>> for Option<i64> { |option: Self| option.map(Value::I64).unwrap_or(Value::Null(NullType::I64)) });
impl_FieldOrd!(i64, i64, Value::I64);
impl_FieldOrd!(Option<i64>, Option<i64>, |option: Self| option
    .map(Value::I64)
    .unwrap_or(Value::Null(NullType::I64)));

impl_AsDbType!(f32, db_type::Float, Value::F32);
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, f32> for f32 { Value::F32 });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<f32>> for Option<f32> { |option: Self| option.map(Value::F32).unwrap_or(Value::Null(NullType::F32)) });
impl_FieldOrd!(f32, f32, Value::F32);
impl_FieldOrd!(Option<f32>, Option<f32>, |option: Self| option
    .map(Value::F32)
    .unwrap_or(Value::Null(NullType::F32)));

impl_AsDbType!(f64, db_type::Double, Value::F64);
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, f64> for f64 { Value::F64 });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<f64>> for Option<f64> { |option: Self| option.map(Value::F64).unwrap_or(Value::Null(NullType::F64)) });
impl_FieldOrd!(f64, f64, Value::F64);
impl_FieldOrd!(Option<f64>, Option<f64>, |option: Self| option
    .map(Value::F64)
    .unwrap_or(Value::Null(NullType::F64)));

impl_AsDbType!(String, db_type::VarChar, conv_string, conv_string);
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
fn conv_string<'a>(value: impl Into<Cow<'a, str>>) -> Value<'a> {
    Value::String(value.into())
}

impl_AsDbType!(Vec<u8>, db_type::Binary, conv_bytes, conv_bytes);
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
