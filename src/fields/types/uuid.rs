use rorm_db::sql::value::NullType;
use uuid::Uuid;

use crate::conditions::Value;
use crate::crud::decoder::DirectDecoder;
use crate::fields::proxy::LayerStackBase;
use crate::fields::traits::{Array, FieldColumns, FieldType};
use crate::fields::utils::field_proxy_layer::SimpleEq;
use crate::fields::utils::{check, get_annotations, get_names};

impl FieldType for Uuid {
    type Columns = Array<1>;
    const NULL: FieldColumns<Self, NullType> = [NullType::Uuid];
    type FieldProxyLayers = SimpleEq<Self, LayerStackBase>;
    type OptionFieldProxyLayers = SimpleEq<Option<Self>, LayerStackBase>;
    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [Value::Uuid(self)]
    }
    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [Value::Uuid(*self)]
    }
    type Decoder = DirectDecoder<Self>;
    type GetNames = get_names::single_column_name;
    type GetAnnotations = get_annotations::forward_annotations<1>;
    type Check = check::shared_linter_check<1>;
}
