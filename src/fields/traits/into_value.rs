//! Trait converting types into SQL [`Value`]s

use std::borrow::Cow;

use crate::conditions::Value;

/// A type which can be converted into an SQL [`Value`]
pub trait IntoValue<'a> {
    /// Converts `self` into an SQL [`Value`]
    fn into_value(self) -> Value<'a>;
}

impl<'a> IntoValue<'a> for bool {
    fn into_value(self) -> Value<'a> {
        Value::Bool(self)
    }
}
impl<'a> IntoValue<'a> for i16 {
    fn into_value(self) -> Value<'a> {
        Value::I16(self)
    }
}
impl<'a> IntoValue<'a> for i32 {
    fn into_value(self) -> Value<'a> {
        Value::I32(self)
    }
}
impl<'a> IntoValue<'a> for i64 {
    fn into_value(self) -> Value<'a> {
        Value::I64(self)
    }
}
impl<'a> IntoValue<'a> for f32 {
    fn into_value(self) -> Value<'a> {
        Value::F32(self)
    }
}
impl<'a> IntoValue<'a> for f64 {
    fn into_value(self) -> Value<'a> {
        Value::F64(self)
    }
}
impl<'a> IntoValue<'a> for &'a str {
    fn into_value(self) -> Value<'a> {
        Value::String(Cow::Borrowed(self))
    }
}
impl<'a> IntoValue<'a> for &'a String {
    fn into_value(self) -> Value<'a> {
        Value::String(Cow::Borrowed(self))
    }
}
impl<'a> IntoValue<'a> for String {
    fn into_value(self) -> Value<'a> {
        Value::String(Cow::Owned(self))
    }
}
impl<'a> IntoValue<'a> for Cow<'a, str> {
    fn into_value(self) -> Value<'a> {
        Value::String(self)
    }
}
impl<'a> IntoValue<'a> for &'a [u8] {
    fn into_value(self) -> Value<'a> {
        Value::Binary(Cow::Borrowed(self))
    }
}
impl<'a> IntoValue<'a> for &'a Vec<u8> {
    fn into_value(self) -> Value<'a> {
        Value::Binary(Cow::Borrowed(self))
    }
}
impl<'a> IntoValue<'a> for Vec<u8> {
    fn into_value(self) -> Value<'a> {
        Value::Binary(Cow::Owned(self))
    }
}
impl<'a> IntoValue<'a> for Cow<'a, [u8]> {
    fn into_value(self) -> Value<'a> {
        Value::Binary(self)
    }
}
