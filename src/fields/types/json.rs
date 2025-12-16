//! The [`Json<T>`] wrapper to store json data in the db

use std::borrow::Cow;
use std::ops::{Deref, DerefMut};

use rorm_db::sql::value::NullType;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::conditions::Value;
use crate::fields::proxy::LayerStackBase;
use crate::fields::traits::{Array, FieldColumns, FieldType};
use crate::fields::utils::check::shared_linter_check;
use crate::fields::utils::get_annotations::forward_annotations;
use crate::fields::utils::get_names::single_column_name;
use crate::new_converting_decoder;

/// Stores data by serializing it to json.
///
/// This is just a convenience wrapper around [serde_json] and `Vec<u8>`.
///
/// ```no_run
/// # use std::collections::HashMap;
/// use rorm::Model;
/// use rorm::fields::types::Json;
///
/// #[derive(Model)]
/// pub struct Session {
///     #[rorm(id)]
///     pub id: i64,
///
///     pub data: Json<HashMap<String, String>>,
/// }
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Json<T: Serialize + DeserializeOwned>(pub T);

impl<T: Serialize + DeserializeOwned> Json<T> {
    /// Unwrap into inner T value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

new_converting_decoder!(
    pub JsonDecoder<T: Serialize + DeserializeOwned>,
    |value: Vec<u8>| -> Json<T> {
        serde_json::from_slice(&value)
            .map(Json)
            .map_err(|err| format!("Couldn't decoder json: {err}"))
    }
);
impl<T: Serialize + DeserializeOwned + 'static> FieldType for Json<T> {
    type Columns = Array<1>;

    const NULL: FieldColumns<Self, NullType> = [NullType::Binary];

    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [Value::Binary(Cow::Owned(
            serde_json::to_vec(&self.0).unwrap(),
        ))] // TODO propagate error?
    }

    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [Value::Binary(Cow::Owned(
            serde_json::to_vec(&self.0).unwrap(),
        ))] // TODO propagate error?
    }

    type Decoder = JsonDecoder<T>;

    type GetAnnotations = forward_annotations<1>;

    type Check = shared_linter_check<1>;

    type GetNames = single_column_name;
    type FieldProxyLayers = LayerStackBase;
    type OptionFieldProxyLayers = LayerStackBase;
}

// From
impl<T: Serialize + DeserializeOwned> From<T> for Json<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

// Deref
impl<T: Serialize + DeserializeOwned> Deref for Json<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: Serialize + DeserializeOwned> DerefMut for Json<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// AsRef
impl<T: Serialize + DeserializeOwned> AsRef<T> for Json<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}
impl<T: Serialize + DeserializeOwned> AsMut<T> for Json<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
