//! The [`MsgPack<T>`] wrapper to store message pack data in the db

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

/// Stores data by serializing it to message pack.
///
/// This is just a convenience wrapper around [rmp_serde] and `Vec<u8>`.
///
/// ```no_run
/// # use std::collections::HashMap;
/// use rorm::Model;
/// use rorm::fields::types::MsgPack;
///
/// #[derive(Model)]
/// pub struct Session {
///     #[rorm(id)]
///     pub id: i64,
///
///     pub data: MsgPack<HashMap<String, String>>,
/// }
/// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MsgPack<T: Serialize + DeserializeOwned>(pub T);

impl<T: Serialize + DeserializeOwned> MsgPack<T> {
    /// Unwrap into inner T value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

new_converting_decoder!(
    pub MsgPackDecoder<T: Serialize + DeserializeOwned>,
    |value: Vec<u8>| -> MsgPack<T> {
        rmp_serde::from_slice(&value)
            .map(MsgPack)
            .map_err(|err| format!("Couldn't decode msg pack: {err}"))
    }
);
impl<T: Serialize + DeserializeOwned + 'static> FieldType for MsgPack<T> {
    type Columns = Array<1>;

    const NULL: FieldColumns<Self, NullType> = [NullType::Binary];

    fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
        [Value::Binary(Cow::Owned(
            rmp_serde::to_vec(&self.0).unwrap(), // TODO propagate error?
        ))]
    }

    fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
        [Value::Binary(Cow::Owned(
            rmp_serde::to_vec(&self.0).unwrap(), // TODO propagate error?
        ))]
    }

    type Decoder = MsgPackDecoder<T>;

    type GetAnnotations = forward_annotations<1>;
    type Check = shared_linter_check<1>;
    type GetNames = single_column_name;
    type FieldProxyLayers = LayerStackBase;
    type OptionFieldProxyLayers = LayerStackBase;
}

// From
impl<T: Serialize + DeserializeOwned> From<T> for MsgPack<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

// Deref
impl<T: Serialize + DeserializeOwned> Deref for MsgPack<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: Serialize + DeserializeOwned> DerefMut for MsgPack<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// AsRef
impl<T: Serialize + DeserializeOwned> AsRef<T> for MsgPack<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}
impl<T: Serialize + DeserializeOwned> AsMut<T> for MsgPack<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
