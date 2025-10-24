use uuid::Uuid;

use crate::conditions::Value;
use crate::fields::traits::simple::{SimpleFieldEq, SimpleFieldIn};
use crate::impl_FieldType;

impl_FieldType!(Uuid, Uuid, Value::Uuid);
impl<'rhs> SimpleFieldEq<'rhs, Uuid> for Uuid {
    fn into_value(rhs: Uuid) -> Value<'rhs> {
        Value::Uuid(rhs)
    }
}
impl<'rhs> SimpleFieldIn<'rhs, Uuid> for Uuid {
    fn into_value(rhs: Uuid) -> Value<'rhs> {
        Value::Uuid(rhs)
    }
}
