use std::borrow::Cow;
use std::marker::PhantomData;

use rorm_db::sql::value::NullType;

use crate::conditions::Value;
use crate::crud::decoder::NoopDecoder;
#[cfg(feature = "postgres-only")]
use crate::fields::traits::simple::SimpleFieldILike;
use crate::fields::traits::simple::{
    SimpleFieldEq, SimpleFieldIn, SimpleFieldLike, SimpleFieldOrd,
};
use crate::fields::traits::{Array, FieldColumns, FieldType};
use crate::fields::utils::check;
use crate::fields::utils::check::disallow_annotations_check;
use crate::fields::utils::get_annotations::forward_annotations;
use crate::fields::utils::get_names::no_columns_names;
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
impl SimpleFieldEq<&'_ str> for String {}
impl SimpleFieldEq<&'_ String> for String {}
impl SimpleFieldEq<Cow<'_, str>> for String {}

impl SimpleFieldIn for String {}
impl SimpleFieldIn<&'_ str> for String {}
impl SimpleFieldIn<&'_ String> for String {}
impl SimpleFieldIn<Cow<'_, str>> for String {}

impl SimpleFieldOrd for String {}
impl SimpleFieldOrd<&'_ str> for String {}
impl SimpleFieldOrd<&'_ String> for String {}
impl SimpleFieldOrd<Cow<'_, str>> for String {}

impl SimpleFieldLike for String {}
impl SimpleFieldLike<&'_ str> for String {}
impl SimpleFieldLike<&'_ String> for String {}
impl SimpleFieldLike<Cow<'_, str>> for String {}

#[cfg(feature = "postgres-only")]
impl SimpleFieldILike for String {}
#[cfg(feature = "postgres-only")]
impl SimpleFieldILike<&'_ str> for String {}
#[cfg(feature = "postgres-only")]
impl SimpleFieldILike<&'_ String> for String {}
#[cfg(feature = "postgres-only")]
impl SimpleFieldILike<Cow<'_, str>> for String {}

impl_FieldMin_FieldMax!(String);

impl_FieldType!(Vec<u8>, Binary);

impl SimpleFieldEq for Vec<u8> {}
impl SimpleFieldEq<&'_ [u8]> for Vec<u8> {}
impl SimpleFieldEq<&'_ Vec<u8>> for Vec<u8> {}
impl SimpleFieldEq<Cow<'_, [u8]>> for Vec<u8> {}

impl SimpleFieldOrd for Vec<u8> {}
impl SimpleFieldOrd<&'_ [u8]> for Vec<u8> {}
impl SimpleFieldOrd<&'_ Vec<u8>> for Vec<u8> {}
impl SimpleFieldOrd<Cow<'_, [u8]>> for Vec<u8> {}

impl<T: 'static> FieldType for PhantomData<T> {
    type Columns = Array<0>;
    const NULL: FieldColumns<Self, NullType> = [];

    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        []
    }

    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        []
    }

    type Decoder = NoopDecoder<Self>;

    type GetNames = no_columns_names;

    type GetAnnotations = forward_annotations<0>;

    type Check = disallow_annotations_check<0>;
}
