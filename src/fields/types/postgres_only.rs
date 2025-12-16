use bit_vec::BitVec;
use ipnetwork::IpNetwork;
use mac_address::MacAddress;
use rorm_db::sql::value::NullType;

use crate::conditions::Value;
use crate::crud::decoder::DirectDecoder;
use crate::fields::proxy::LayerStackBase;
use crate::fields::traits::{Array, FieldColumns, FieldType};
use crate::fields::utils::field_proxy_layer::SimpleEq;
use crate::fields::utils::{check, get_annotations, get_names};

impl FieldType for MacAddress {
    type Columns = Array<1>;
    const NULL: FieldColumns<Self, NullType> = [NullType::MacAddress];
    type FieldProxyLayers = SimpleEq<Self, LayerStackBase>;
    type OptionFieldProxyLayers = SimpleEq<Option<Self>, LayerStackBase>;
    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [Value::MacAddress(self)]
    }
    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [Value::MacAddress(*self)]
    }
    type Decoder = DirectDecoder<Self>;
    type GetNames = get_names::single_column_name;
    type GetAnnotations = get_annotations::forward_annotations<1>;
    type Check = check::shared_linter_check<1>;
}

impl FieldType for IpNetwork {
    type Columns = Array<1>;
    const NULL: FieldColumns<Self, NullType> = [NullType::IpNetwork];
    type FieldProxyLayers = SimpleEq<Self, LayerStackBase>;
    type OptionFieldProxyLayers = SimpleEq<Option<Self>, LayerStackBase>;
    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [Value::IpNetwork(self)]
    }
    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [Value::IpNetwork(*self)]
    }
    type Decoder = DirectDecoder<Self>;
    type GetNames = get_names::single_column_name;
    type GetAnnotations = get_annotations::forward_annotations<1>;
    type Check = check::shared_linter_check<1>;
}

impl FieldType for BitVec {
    type Columns = Array<1>;
    const NULL: FieldColumns<Self, NullType> = [NullType::BitVec];
    type FieldProxyLayers = LayerStackBase;
    type OptionFieldProxyLayers = LayerStackBase;
    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [Value::BitVec(BitCow::Owned(self))]
    }
    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [Value::BitVec(BitCow::Borrowed(self))]
    }
    type Decoder = DirectDecoder<Self>;
    type GetNames = get_names::single_column_name;
    type GetAnnotations = get_annotations::forward_annotations<1>;
    type Check = check::shared_linter_check<1>;
}
impl<'rhs> SimpleFieldEq<'rhs, &'rhs BitVec> for BitVec {
    fn into_value(rhs: &'rhs BitVec) -> Value<'rhs> {
        Value::BitVec(BitCow::Borrowed(rhs))
    }
}
impl<'rhs> SimpleFieldIn<'rhs, &'rhs BitVec> for BitVec {
    fn into_value(rhs: &'rhs BitVec) -> Value<'rhs> {
        Value::BitVec(BitCow::Borrowed(rhs))
    }
}
impl<'rhs> SimpleFieldEq<'rhs, BitVec> for BitVec {
    fn into_value(rhs: BitVec) -> Value<'rhs> {
        Value::BitVec(BitCow::Owned(rhs))
    }
}
impl<'rhs> SimpleFieldIn<'rhs, BitVec> for BitVec {
    fn into_value(rhs: BitVec) -> Value<'rhs> {
        Value::BitVec(BitCow::Owned(rhs))
    }
}

#[derive(Clone, Debug)]
pub enum BitCow<'a> {
    Borrowed(&'a BitVec),
    Owned(BitVec),
}

impl AsRef<BitVec> for BitCow<'_> {
    fn as_ref(&self) -> &BitVec {
        match self {
            BitCow::Borrowed(bit_vec) => bit_vec,
            BitCow::Owned(bit_vec) => bit_vec,
        }
    }
}
