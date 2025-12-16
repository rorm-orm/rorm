use std::borrow::Cow;

use crate::conditions::Value;
use crate::crud::decoder::DirectDecoder;
use crate::db::sql::value::NullType;
use crate::fields::proxy::{EqualsProxy, LayerStackBase};
use crate::fields::traits::{Array, FieldColumns, FieldType};
use crate::fields::utils::field_proxy_layer::{
    OptionSimpleEqOrdMinMax, SimpleEq, SimpleEqOrdMinMax, SimpleSumAvg, StringLayers,
};
use crate::fields::utils::{check, get_annotations, get_names};

impl FieldType for bool {
    type Columns = Array<1>;
    const NULL: FieldColumns<Self, NullType> = [NullType::Bool];
    type FieldProxyLayers = SimpleEq<Self, LayerStackBase>;
    type OptionFieldProxyLayers = SimpleEq<Option<Self>, LayerStackBase>;
    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [Value::Bool(self)]
    }
    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [Value::Bool(*self)]
    }
    type Decoder = DirectDecoder<Self>;
    type GetNames = get_names::single_column_name;
    type GetAnnotations = get_annotations::forward_annotations<1>;
    type Check = check::shared_linter_check<1>;
}

impl FieldType for i16 {
    type Columns = Array<1>;
    const NULL: FieldColumns<Self, NullType> = [NullType::I16];
    type FieldProxyLayers = NumberLayers<Self, i64>;
    type OptionFieldProxyLayers = OptionNumberLayers<Self, i64>;
    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [Value::I16(self)]
    }
    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [Value::I16(*self)]
    }
    type Decoder = DirectDecoder<Self>;
    type GetNames = get_names::single_column_name;
    type GetAnnotations = get_annotations::forward_annotations<1>;
    type Check = check::shared_linter_check<1>;
}

impl FieldType for i32 {
    type Columns = Array<1>;
    const NULL: FieldColumns<Self, NullType> = [NullType::I16];
    type FieldProxyLayers = NumberLayers<Self, i64>;
    type OptionFieldProxyLayers = OptionNumberLayers<Self, i64>;
    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [Value::I32(self)]
    }
    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [Value::I32(*self)]
    }
    type Decoder = DirectDecoder<Self>;
    type GetNames = get_names::single_column_name;
    type GetAnnotations = get_annotations::forward_annotations<1>;
    type Check = check::shared_linter_check<1>;
}

impl FieldType for i64 {
    type Columns = Array<1>;
    const NULL: FieldColumns<Self, NullType> = [NullType::I64];
    // Summing multiple i64 can overflow an i64, so the result type is f64 instead
    // (which might lose precision)
    type FieldProxyLayers = NumberLayers<Self, f64>;
    type OptionFieldProxyLayers = OptionNumberLayers<Self, f64>;
    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [Value::I64(self)]
    }
    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [Value::I64(*self)]
    }
    type Decoder = DirectDecoder<Self>;
    type GetNames = get_names::single_column_name;
    type GetAnnotations = get_annotations::forward_annotations<1>;
    type Check = check::shared_linter_check<1>;
}

impl FieldType for f32 {
    type Columns = Array<1>;
    const NULL: FieldColumns<Self, NullType> = [NullType::F32];
    type FieldProxyLayers = NumberLayers<Self, f32>;
    type OptionFieldProxyLayers = OptionNumberLayers<Self, f32>;
    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [Value::F32(self)]
    }
    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [Value::F32(*self)]
    }
    type Decoder = DirectDecoder<Self>;
    type GetNames = get_names::single_column_name;
    type GetAnnotations = get_annotations::forward_annotations<1>;
    type Check = check::shared_linter_check<1>;
}

impl FieldType for f64 {
    type Columns = Array<1>;
    const NULL: FieldColumns<Self, NullType> = [NullType::F64];
    type FieldProxyLayers = NumberLayers<Self, f64>;
    type OptionFieldProxyLayers = OptionNumberLayers<Self, f64>;
    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [Value::F64(self)]
    }
    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [Value::F64(*self)]
    }
    type Decoder = DirectDecoder<Self>;
    type GetNames = get_names::single_column_name;
    type GetAnnotations = get_annotations::forward_annotations<1>;
    type Check = check::shared_linter_check<1>;
}

impl FieldType for String {
    type Columns = Array<1>;
    const NULL: FieldColumns<Self, NullType> = [NullType::String];
    type FieldProxyLayers = StringLayers;
    type OptionFieldProxyLayers = StringLayers;
    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [Value::String(Cow::Owned(self))]
    }
    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [Value::String(Cow::Borrowed(self))]
    }
    type Decoder = DirectDecoder<Self>;
    type GetNames = get_names::single_column_name;
    type GetAnnotations = get_annotations::forward_annotations<1>;
    type Check = check::string_check;
}

impl FieldType for Vec<u8> {
    type Columns = Array<1>;
    const NULL: FieldColumns<Self, NullType> = [NullType::Binary];
    type FieldProxyLayers = EqualsProxy<LayerStackBase>;
    type OptionFieldProxyLayers = EqualsProxy<LayerStackBase>;
    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [Value::Binary(Cow::Owned(self))]
    }
    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [Value::Binary(Cow::Borrowed(self))]
    }
    type Decoder = DirectDecoder<Self>;
    type GetNames = get_names::single_column_name;
    type GetAnnotations = get_annotations::forward_annotations<1>;
    type Check = check::shared_linter_check<1>;
}
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

fn conv_bytes<'a>(value: impl Into<Cow<'a, [u8]>>) -> Value<'a> {
    Value::Binary(value.into())
}

type NumberLayers<This, SumResult> = SimpleSumAvg<SumResult, SimpleEqOrdMinMax<This>>;
type OptionNumberLayers<This, SumResult> =
    SimpleSumAvg<SumResult, OptionSimpleEqOrdMinMax<Option<This>>>;
