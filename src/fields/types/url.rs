use std::borrow::Cow;

use crate::conditions::Value;
use crate::fields::traits::into_value::IntoValue;
use crate::fields::traits::simple::{SimpleFieldEq, SimpleFieldIn};
use crate::fields::traits::{Array, FieldColumns, FieldType};
use crate::fields::utils::get_annotations::forward_annotations;
use crate::fields::utils::get_names::single_column_name;
use rorm_db::row::{RowError, RowIndex};
use rorm_db::sql::value::NullType;
use rorm_db::Row;
use url::Url;

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

    fn decode(row: &Row, index: usize) -> Result<Self, RowError<'static>> {
        let string = String::decode(row, index)?;
        Url::parse(&string).map_err(|err| RowError::Decode {
            index: RowIndex::Position(index),
            source: format!("Couldn't parse url: {err}").into(),
        })
    }

    type GetNames = single_column_name;

    type GetAnnotations = forward_annotations<1>;
}
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
impl SimpleFieldEq<&'_ Url> for Url {}
impl SimpleFieldIn<&'_ Url> for Url {}
impl SimpleFieldEq<Url> for Url {}
impl SimpleFieldIn<Url> for Url {}
