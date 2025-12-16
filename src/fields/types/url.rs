use std::borrow::Cow;

use rorm_db::sql::value::NullType;
use url::Url;

use crate::conditions::Value;
use crate::fields::traits::{Array, FieldColumns, FieldType};
use crate::fields::utils::check::string_check;
use crate::fields::utils::field_proxy_layer::StringLayers;
use crate::fields::utils::get_annotations::forward_annotations;
use crate::fields::utils::get_names::single_column_name;
use crate::new_converting_decoder;

impl FieldType for Url {
    type Columns = Array<1>;

    const NULL: FieldColumns<Self, NullType> = [NullType::String];
    type FieldProxyLayers = StringLayers;
    type OptionFieldProxyLayers = StringLayers;

    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [Value::String(Cow::Owned(self.into()))]
    }

    #[inline(always)]
    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [Value::String(Cow::Borrowed(self.as_str()))]
    }

    type Decoder = UrlDecoder;

    type GetNames = single_column_name;

    type GetAnnotations = forward_annotations<1>;

    type Check = string_check;
}
new_converting_decoder!(
    pub UrlDecoder,
    |value: String| -> Url {
        Url::parse(&value).map_err(|err| format!("Couldn't parse url: {err}"))
    }
);
