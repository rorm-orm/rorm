use std::borrow::Cow;

#[cfg(feature = "postgres-only")]
use crate::fields::traits::simple::SimpleFieldILike;
use crate::fields::traits::simple::{
    SimpleFieldEq, SimpleFieldIn, SimpleFieldLike, SimpleFieldOrd,
};
use crate::fields::utils::check;
use crate::{impl_FieldMin_FieldMax, impl_FieldSum_FieldAvg, impl_FieldType};

impl_FieldType!(bool, Bool);
impl SimpleFieldEq<bool> for bool {}
impl SimpleFieldIn<bool> for bool {}

impl_FieldType!(i16, I16);
impl SimpleFieldEq<i16> for i16 {}
impl SimpleFieldIn<i16> for i16 {}
impl SimpleFieldOrd<i16> for i16 {}

impl_FieldSum_FieldAvg!(i16, sum_result: i64);
impl_FieldMin_FieldMax!(i16);

impl_FieldType!(i32, I32);
impl SimpleFieldEq<i32> for i32 {}
impl SimpleFieldIn<i32> for i32 {}
impl SimpleFieldOrd<i32> for i32 {}

impl_FieldSum_FieldAvg!(i32, sum_result: i64);
impl_FieldMin_FieldMax!(i32);

impl_FieldType!(i64, I64);
impl SimpleFieldEq<i64> for i64 {}
impl SimpleFieldIn<i64> for i64 {}
impl SimpleFieldOrd<i64> for i64 {}

impl_FieldSum_FieldAvg!(i64, sum_result: f64);
impl_FieldMin_FieldMax!(i64);

impl_FieldType!(f32, F32);
impl SimpleFieldEq<f32> for f32 {}
impl SimpleFieldIn<f32> for f32 {}
impl SimpleFieldOrd<f32> for f32 {}

impl_FieldSum_FieldAvg!(f32, sum_result: f32);
impl_FieldMin_FieldMax!(f32);

impl_FieldType!(f64, F64);
impl SimpleFieldEq<f64> for f64 {}
impl SimpleFieldIn<f64> for f64 {}
impl SimpleFieldOrd<f64> for f64 {}

impl_FieldSum_FieldAvg!(f64, sum_result: f64);
impl_FieldMin_FieldMax!(f64);

impl_FieldType!(String, String, check::string_check);

impl<'rhs> SimpleFieldEq<&'rhs str> for String {}
impl<'rhs> SimpleFieldEq<&'rhs String> for String {}
impl SimpleFieldEq<String> for String {}
impl<'rhs> SimpleFieldEq<Cow<'rhs, str>> for String {}

impl<'rhs> SimpleFieldIn<&'rhs str> for String {}
impl<'rhs> SimpleFieldIn<&'rhs String> for String {}
impl SimpleFieldIn<String> for String {}
impl<'rhs> SimpleFieldIn<Cow<'rhs, str>> for String {}

impl<'rhs> SimpleFieldOrd<&'rhs str> for String {}
impl<'rhs> SimpleFieldOrd<&'rhs String> for String {}
impl SimpleFieldOrd<String> for String {}
impl<'rhs> SimpleFieldOrd<Cow<'rhs, str>> for String {}

impl<'rhs> SimpleFieldLike<&'rhs str> for String {}
impl<'rhs> SimpleFieldLike<&'rhs String> for String {}
impl SimpleFieldLike<String> for String {}
impl<'rhs> SimpleFieldLike<Cow<'rhs, str>> for String {}

#[cfg(feature = "postgres-only")]
impl<'rhs> SimpleFieldILike<&'rhs str> for String {}
#[cfg(feature = "postgres-only")]
impl<'rhs> SimpleFieldILike<&'rhs String> for String {}
#[cfg(feature = "postgres-only")]
impl SimpleFieldILike<String> for String {}
#[cfg(feature = "postgres-only")]
impl<'rhs> SimpleFieldILike<Cow<'rhs, str>> for String {}

impl_FieldMin_FieldMax!(String);

impl_FieldType!(Vec<u8>, Binary);

impl<'rhs> SimpleFieldEq<&'rhs [u8]> for Vec<u8> {}
impl<'rhs> SimpleFieldEq<&'rhs Vec<u8>> for Vec<u8> {}
impl SimpleFieldEq<Vec<u8>> for Vec<u8> {}
impl<'rhs> SimpleFieldEq<Cow<'rhs, [u8]>> for Vec<u8> {}

impl<'rhs> SimpleFieldOrd<&'rhs [u8]> for Vec<u8> {}
impl<'rhs> SimpleFieldOrd<&'rhs Vec<u8>> for Vec<u8> {}
impl SimpleFieldOrd<Vec<u8>> for Vec<u8> {}
impl<'rhs> SimpleFieldOrd<Cow<'rhs, [u8]>> for Vec<u8> {}
