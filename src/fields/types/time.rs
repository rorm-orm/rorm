use rorm_db::sql::value::NullType;
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};

use crate::conditions::Value;
use crate::crud::decoder::DirectDecoder;
use crate::fields::traits::{Array, FieldColumns, FieldType};
use crate::fields::utils::field_proxy_layer::{OptionSimpleEqOrdMinMax, SimpleEqOrdMinMax};
use crate::fields::utils::{check, get_annotations, get_names};

impl FieldType for Time {
    type Columns = Array<1>;
    const NULL: FieldColumns<Self, NullType> = [NullType::TimeTime];
    type FieldProxyLayers = SimpleEqOrdMinMax<Self>;
    type OptionFieldProxyLayers = OptionSimpleEqOrdMinMax<Self>;
    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [Value::TimeTime(self)]
    }
    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [Value::TimeTime(*self)]
    }
    type Decoder = DirectDecoder<Self>;
    type GetNames = get_names::single_column_name;
    type GetAnnotations = get_annotations::forward_annotations<1>;
    type Check = check::shared_linter_check<1>;
}

impl FieldType for Date {
    type Columns = Array<1>;
    const NULL: FieldColumns<Self, NullType> = [NullType::TimeDate];
    type FieldProxyLayers = SimpleEqOrdMinMax<Self>;
    type OptionFieldProxyLayers = OptionSimpleEqOrdMinMax<Self>;
    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [Value::TimeDate(self)]
    }
    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [Value::TimeDate(*self)]
    }
    type Decoder = DirectDecoder<Self>;
    type GetNames = get_names::single_column_name;
    type GetAnnotations = get_annotations::forward_annotations<1>;
    type Check = check::shared_linter_check<1>;
}

impl FieldType for OffsetDateTime {
    type Columns = Array<1>;
    const NULL: FieldColumns<Self, NullType> = [NullType::TimeOffsetDateTime];
    type FieldProxyLayers = SimpleEqOrdMinMax<Self>;
    type OptionFieldProxyLayers = OptionSimpleEqOrdMinMax<Self>;
    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [Value::TimeOffsetDateTime(self)]
    }
    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [Value::TimeOffsetDateTime(*self)]
    }
    type Decoder = DirectDecoder<Self>;
    type GetNames = get_names::single_column_name;
    type GetAnnotations = get_annotations::forward_annotations<1>;
    type Check = check::shared_linter_check<1>;
}

impl FieldType for PrimitiveDateTime {
    type Columns = Array<1>;
    const NULL: FieldColumns<Self, NullType> = [NullType::TimePrimitiveDateTime];
    type FieldProxyLayers = SimpleEqOrdMinMax<Self>;
    type OptionFieldProxyLayers = OptionSimpleEqOrdMinMax<Self>;
    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [Value::TimePrimitiveDateTime(self)]
    }
    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [Value::TimePrimitiveDateTime(*self)]
    }
    type Decoder = DirectDecoder<Self>;
    type GetNames = get_names::single_column_name;
    type GetAnnotations = get_annotations::forward_annotations<1>;
    type Check = check::shared_linter_check<1>;
}
