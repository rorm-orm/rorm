//! The [`MsgPack<T>`] wrapper to store message pack data in the db

use std::borrow::Cow;
use std::ops::{Deref, DerefMut};

use rorm_db::Error::DecodeError;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::conditions::Value;
use crate::fields::traits::FieldType;
use crate::internal::field::as_db_type::AsDbType;
use crate::internal::field::kind;
use crate::internal::hmr::db_type::{DbType, VarBinary};
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
    MsgPackDecoder<T: Serialize + DeserializeOwned>,
    |value: Vec<u8>| -> MsgPack<T> {
        rmp_serde::from_slice(&value)
            .map(MsgPack)
            .map_err(|err| DecodeError(format!("Couldn't decode msg pack: {err}")))
    }
);
impl<T: Serialize + DeserializeOwned + 'static> FieldType for MsgPack<T> {
    type Kind = kind::AsDbType;
    type Columns<C> = [C; 1];

    fn into_values(self) -> Self::Columns<Value<'static>> {
        [Value::Binary(Cow::Owned(
            rmp_serde::to_vec(&self.0).unwrap(), // TODO propagate error?
        ))]
    }

    fn as_values(&self) -> Self::Columns<Value<'_>> {
        [Value::Binary(Cow::Owned(
            rmp_serde::to_vec(&self.0).unwrap(), // TODO propagate error?
        ))]
    }

    type Decoder = MsgPackDecoder<T>;
}
impl<T: Serialize + DeserializeOwned + 'static> AsDbType for MsgPack<T> {
    type Primitive = Vec<u8>;
    type DbType = VarBinary;

    fn from_primitive(primitive: Self::Primitive) -> Self {
        Self(rmp_serde::from_slice(&primitive).unwrap()) // TODO propagate error?
    }
}

new_converting_decoder!(
    OptionMsgPackDecoder<T: Serialize + DeserializeOwned>,
    |value: Option<Vec<u8>>| -> Option<MsgPack<T>> {
        value
            .map(|value| {
                rmp_serde::from_slice(&value)
                    .map(MsgPack)
                    .map_err(|err| DecodeError(format!("Couldn't decode msg pack: {err}")))
            })
            .transpose()
    }
);
impl<T: Serialize + DeserializeOwned + 'static> FieldType for Option<MsgPack<T>> {
    type Kind = kind::AsDbType;
    type Columns<C> = [C; 1];

    fn into_values(self) -> Self::Columns<Value<'static>> {
        self.map(MsgPack::into_values)
            .unwrap_or([Value::Null(VarBinary::NULL_TYPE)])
    }

    fn as_values(&self) -> Self::Columns<Value<'_>> {
        self.as_ref()
            .map(MsgPack::as_values)
            .unwrap_or([Value::Null(VarBinary::NULL_TYPE)])
    }

    type Decoder = OptionMsgPackDecoder<T>;
}
impl<T: Serialize + DeserializeOwned + 'static> AsDbType for Option<MsgPack<T>> {
    type Primitive = Option<Vec<u8>>;
    type DbType = VarBinary;

    fn from_primitive(primitive: Self::Primitive) -> Self {
        primitive.map(MsgPack::<T>::from_primitive)
    }
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
