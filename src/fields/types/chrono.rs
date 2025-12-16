use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use rorm_db::sql::value::NullType;

use crate::conditions::Value;
use crate::crud::decoder::DirectDecoder;
use crate::fields::traits::{Array, FieldColumns, FieldType};
use crate::fields::utils::field_proxy_layer::{OptionSimpleEqOrdMinMax, SimpleEqOrdMinMax};
use crate::fields::utils::{check, get_annotations, get_names};

impl FieldType for NaiveTime {
    type Columns = Array<1>;
    const NULL: FieldColumns<Self, NullType> = [NullType::ChronoNaiveTime];
    type FieldProxyLayers = SimpleEqOrdMinMax<Self>;
    type OptionFieldProxyLayers = OptionSimpleEqOrdMinMax<Self>;
    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [Value::ChronoNaiveTime(self)]
    }
    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [Value::ChronoNaiveTime(*self)]
    }
    type Decoder = DirectDecoder<Self>;
    type GetNames = get_names::single_column_name;
    type GetAnnotations = get_annotations::forward_annotations<1>;
    type Check = check::shared_linter_check<1>;
}

impl FieldType for NaiveDate {
    type Columns = Array<1>;
    const NULL: FieldColumns<Self, NullType> = [NullType::ChronoNaiveDate];
    type FieldProxyLayers = SimpleEqOrdMinMax<Self>;
    type OptionFieldProxyLayers = OptionSimpleEqOrdMinMax<Self>;
    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [Value::ChronoNaiveDate(self)]
    }
    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [Value::ChronoNaiveDate(*self)]
    }
    type Decoder = DirectDecoder<Self>;
    type GetNames = get_names::single_column_name;
    type GetAnnotations = get_annotations::forward_annotations<1>;
    type Check = check::shared_linter_check<1>;
}

impl FieldType for NaiveDateTime {
    type Columns = Array<1>;
    const NULL: FieldColumns<Self, NullType> = [NullType::ChronoNaiveDateTime];
    type FieldProxyLayers = SimpleEqOrdMinMax<Self>;
    type OptionFieldProxyLayers = OptionSimpleEqOrdMinMax<Self>;
    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [Value::ChronoNaiveDateTime(self)]
    }
    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [Value::ChronoNaiveDateTime(*self)]
    }
    type Decoder = DirectDecoder<Self>;
    type GetNames = get_names::single_column_name;
    type GetAnnotations = get_annotations::forward_annotations<1>;
    type Check = check::shared_linter_check<1>;
}

impl FieldType for DateTime<Utc> {
    type Columns = Array<1>;
    const NULL: FieldColumns<Self, NullType> = [NullType::ChronoDateTime];
    type FieldProxyLayers = SimpleEqOrdMinMax<Self>;
    type OptionFieldProxyLayers = OptionSimpleEqOrdMinMax<Self>;
    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [Value::ChronoDateTime(self)]
    }
    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [Value::ChronoDateTime(*self)]
    }
    type Decoder = DirectDecoder<Self>;
    type GetNames = get_names::single_column_name;
    type GetAnnotations = get_annotations::forward_annotations<1>;
    type Check = check::shared_linter_check<1>;
}
