//! The [`Json<T>`] wrapper to store json data in the db

use std::borrow::Cow;
use std::ops::{Deref, DerefMut};

use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::conditions::Value;
use crate::internal::field::as_db_type::AsDbType;
use crate::internal::field::{kind, FieldType};
use crate::internal::hmr::db_type::VarBinary;

/// Stores data by serializing it to json.
///
/// This is just a convenience wrapper around [serde_json] and `Vec<u8>`.
///
/// ```no_run
/// # use std::collections::HashMap;
/// use rorm::Model;
/// use rorm::fields::Json;
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

impl<T: Serialize + DeserializeOwned> FieldType for Json<T> {
    type Kind = kind::AsDbType;
    type Columns<'a> = [Value<'a>; 1];

    fn into_values(self) -> Self::Columns<'static> {
        [Value::Binary(Cow::Owned(
            serde_json::to_vec(&self.0).unwrap(),
        ))] // TODO propagate error?
    }

    fn as_values(&self) -> Self::Columns<'_> {
        [Value::Binary(Cow::Owned(
            serde_json::to_vec(&self.0).unwrap(),
        ))] // TODO propagate error?
    }
}
impl<T: Serialize + DeserializeOwned> AsDbType for Json<T> {
    type Primitive = Vec<u8>;
    type DbType = VarBinary;

    fn from_primitive(primitive: Self::Primitive) -> Self {
        Self(serde_json::from_slice(&primitive).unwrap()) // TODO propagate error?
    }
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
