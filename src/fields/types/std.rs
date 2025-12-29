use std::borrow::Cow;

#[cfg(feature = "postgres-only")]
use crate::fields::traits::simple::SimpleFieldILike;
use crate::fields::traits::simple::{
    SimpleFieldEq, SimpleFieldIn, SimpleFieldLike, SimpleFieldOrd,
};
use crate::fields::utils::check;
use crate::{impl_FieldMin_FieldMax, impl_FieldSum_FieldAvg, impl_FieldType};

impl_FieldType!(bool, Bool);
impl SimpleFieldEq for bool {}
impl SimpleFieldIn for bool {}

impl_FieldType!(i16, I16);
impl SimpleFieldEq for i16 {}
impl SimpleFieldIn for i16 {}
impl SimpleFieldOrd for i16 {}

impl_FieldSum_FieldAvg!(i16, sum_result: i64);
impl_FieldMin_FieldMax!(i16);

impl_FieldType!(i32, I32);
impl SimpleFieldEq for i32 {}
impl SimpleFieldIn for i32 {}
impl SimpleFieldOrd for i32 {}

impl_FieldSum_FieldAvg!(i32, sum_result: i64);
impl_FieldMin_FieldMax!(i32);

impl_FieldType!(i64, I64);
impl SimpleFieldEq for i64 {}
impl SimpleFieldIn for i64 {}
impl SimpleFieldOrd for i64 {}

impl_FieldSum_FieldAvg!(i64, sum_result: f64);
impl_FieldMin_FieldMax!(i64);

impl_FieldType!(f32, F32);
impl SimpleFieldEq for f32 {}
impl SimpleFieldIn for f32 {}
impl SimpleFieldOrd for f32 {}

impl_FieldSum_FieldAvg!(f32, sum_result: f32);
impl_FieldMin_FieldMax!(f32);

impl_FieldType!(f64, F64);
impl SimpleFieldEq for f64 {}
impl SimpleFieldIn for f64 {}
impl SimpleFieldOrd for f64 {}

impl_FieldSum_FieldAvg!(f64, sum_result: f64);
impl_FieldMin_FieldMax!(f64);

impl_FieldType!(String, String, check::string_check);

impl SimpleFieldEq for String {}
impl<'rhs> SimpleFieldEq<&'rhs str> for String {}
impl<'rhs> SimpleFieldEq<&'rhs String> for String {}
impl<'rhs> SimpleFieldEq<Cow<'rhs, str>> for String {}

impl SimpleFieldIn for String {}
impl<'rhs> SimpleFieldIn<&'rhs str> for String {}
impl<'rhs> SimpleFieldIn<&'rhs String> for String {}
impl<'rhs> SimpleFieldIn<Cow<'rhs, str>> for String {}

impl SimpleFieldOrd for String {}
impl<'rhs> SimpleFieldOrd<&'rhs str> for String {}
impl<'rhs> SimpleFieldOrd<&'rhs String> for String {}
impl<'rhs> SimpleFieldOrd<Cow<'rhs, str>> for String {}

impl SimpleFieldLike for String {}
impl<'rhs> SimpleFieldLike<&'rhs str> for String {}
impl<'rhs> SimpleFieldLike<&'rhs String> for String {}
impl<'rhs> SimpleFieldLike<Cow<'rhs, str>> for String {}

#[cfg(feature = "postgres-only")]
impl SimpleFieldILike for String {}
#[cfg(feature = "postgres-only")]
impl<'rhs> SimpleFieldILike<&'rhs str> for String {}
#[cfg(feature = "postgres-only")]
impl<'rhs> SimpleFieldILike<&'rhs String> for String {}
#[cfg(feature = "postgres-only")]
impl<'rhs> SimpleFieldILike<Cow<'rhs, str>> for String {}

impl_FieldMin_FieldMax!(String);

impl_FieldType!(Vec<u8>, Binary);

impl SimpleFieldEq for Vec<u8> {}
impl<'rhs> SimpleFieldEq<&'rhs [u8]> for Vec<u8> {}
impl<'rhs> SimpleFieldEq<&'rhs Vec<u8>> for Vec<u8> {}
impl<'rhs> SimpleFieldEq<Cow<'rhs, [u8]>> for Vec<u8> {}

impl SimpleFieldOrd for Vec<u8> {}
impl<'rhs> SimpleFieldOrd<&'rhs [u8]> for Vec<u8> {}
impl<'rhs> SimpleFieldOrd<&'rhs Vec<u8>> for Vec<u8> {}
impl<'rhs> SimpleFieldOrd<Cow<'rhs, [u8]>> for Vec<u8> {}
