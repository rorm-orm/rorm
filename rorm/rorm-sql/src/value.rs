/**
This enum represents a value
 */
#[derive(Copy, Clone)]
pub enum Value<'a> {
    /// String representation
    String(&'a str),
    /// i64 representation
    I64(i64),
    /// i32 representation
    I32(i32),
    /// i16 representation
    I16(i16),
    /// Bool representation
    Bool(bool),
    /// f64 representation
    F64(f64),
    /// f32 representation
    F32(f32),
    /// null representation
    Null,
}

impl<'a> From<&'a str> for Value<'a> {
    fn from(value: &'a str) -> Self {
        Value::String(value)
    }
}

macro_rules! impl_from {
    ($variant:ident, $T:path) => {
        impl From<$T> for Value<'static> {
            fn from(value: $T) -> Self {
                Value::$variant(value)
            }
        }
    };
}
impl_from!(I64, i64);
impl_from!(I32, i32);
impl_from!(I16, i16);
impl_from!(Bool, bool);
impl_from!(F64, f64);
impl_from!(F32, f32);

impl<'a, T> From<&'a T> for Value<'static>
where
    Self: From<T>,
    T: Copy,
{
    fn from(reference: &'a T) -> Self {
        Self::from(*reference)
    }
}
