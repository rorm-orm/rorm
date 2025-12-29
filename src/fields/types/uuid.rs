use uuid::Uuid;

use crate::conditions::Value;
use crate::fields::traits::into_value::IntoValue;
use crate::fields::traits::simple::{SimpleFieldEq, SimpleFieldIn};
use crate::impl_FieldType;

impl_FieldType!(Uuid, Uuid);
impl<'a> IntoValue<'a> for Uuid {
    fn into_value(self) -> Value<'a> {
        Value::Uuid(self)
    }
}
impl<'rhs> SimpleFieldEq<'rhs, Uuid> for Uuid {}
impl<'rhs> SimpleFieldIn<'rhs, Uuid> for Uuid {}
