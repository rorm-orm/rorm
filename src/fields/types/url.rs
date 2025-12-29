use std::borrow::Cow;

use rorm_db::sql::value::NullType;
use url::Url;

use crate::conditions::Value;
use crate::fields::traits::into_value::IntoValue;
use crate::fields::traits::simple::{SimpleFieldEq, SimpleFieldIn};
use crate::fields::traits::{Array, FieldColumns, FieldType};
use crate::fields::utils::check::string_check;
use crate::fields::utils::get_annotations::forward_annotations;
use crate::fields::utils::get_names::single_column_name;
use crate::new_converting_decoder;

impl FieldType for Url {
    type Columns = Array<1>;

    const NULL: FieldColumns<Self, NullType> = [NullType::String];

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
impl<'a> IntoValue<'a> for Url {
    fn into_value(self) -> Value<'a> {
        Value::String(Cow::Owned(self.into()))
    }
}
impl<'a> IntoValue<'a> for &'a Url {
    fn into_value(self) -> Value<'a> {
        Value::String(Cow::Borrowed(self.as_str()))
    }
}
impl<'rhs> SimpleFieldEq<&'rhs Url> for Url {}
impl<'rhs> SimpleFieldIn<&'rhs Url> for Url {}
impl SimpleFieldEq<Url> for Url {}
impl SimpleFieldIn<Url> for Url {}
