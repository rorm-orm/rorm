use std::borrow::Cow;

use uuid::Uuid;

use crate::conditions::Value;
use crate::internal::field::as_db_type::AsDbType;
use crate::internal::field::{kind, FieldType};
use crate::internal::hmr::db_type;

impl FieldType for Uuid {
    type Kind = kind::AsDbType;
    type Columns<'a> = [Value<'a>; 1];

    fn into_values(self) -> Self::Columns<'static> {
        [Value::Binary(Cow::Owned(self.into_bytes().to_vec()))]
    }

    fn as_values(&self) -> Self::Columns<'_> {
        [Value::Binary(Cow::Borrowed(self.as_bytes().as_slice()))]
    }
}

impl AsDbType for Uuid {
    type Primitive = Vec<u8>;
    type DbType = db_type::VarBinary;

    fn from_primitive(primitive: Self::Primitive) -> Self {
        Uuid::from_slice(&primitive).expect("Malformed database data") // TODO propagate error?
    }
}
