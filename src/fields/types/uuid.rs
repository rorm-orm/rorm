use std::borrow::Cow;

use uuid::Uuid;

use crate::conditions::Value;
use crate::fields::traits::FieldType;
use crate::internal::field::as_db_type::AsDbType;
use crate::internal::field::kind;
use crate::internal::hmr::db_type;
use crate::Error::DecodeError;
use crate::{impl_AsDbType, new_converting_decoder};

new_converting_decoder!(UuidDecoder, |value: Vec<u8>| -> Uuid {
    Uuid::from_slice(&value).map_err(|err| DecodeError(format!("Couldn't decode uuid: {err}")))
});
impl FieldType for Uuid {
    type Kind = kind::AsDbType;
    type Columns<T> = [T; 1];

    fn into_values(self) -> Self::Columns<Value<'static>> {
        [Value::Binary(Cow::Owned(self.into_bytes().to_vec()))]
    }

    fn as_values(&self) -> Self::Columns<Value<'_>> {
        [Value::Binary(Cow::Borrowed(self.as_bytes().as_slice()))]
    }

    type Decoder = UuidDecoder;
}

impl AsDbType for Uuid {
    type Primitive = Vec<u8>;
    type DbType = db_type::VarBinary;

    fn from_primitive(primitive: Self::Primitive) -> Self {
        Uuid::from_slice(&primitive).expect("Malformed database data") // TODO propagate error?
    }
}

new_converting_decoder!(
    OptionUuidDecoder,
    |value: Option<Vec<u8>>| -> Option<Uuid> {
        value
            .map(|value| {
                Uuid::from_slice(&value)
                    .map_err(|err| DecodeError(format!("Couldn't decode uuid: {err}")))
            })
            .transpose()
    }
);
impl_AsDbType!(Option<Uuid>, OptionUuidDecoder);
