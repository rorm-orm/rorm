use uuid::Uuid;

use crate::conditions::Value;
use crate::fields::traits::into_value::IntoValue;
use crate::fields::traits::simple::{SimpleFieldEq, SimpleFieldIn, SimpleFieldOrd};
use crate::impl_FieldType;

impl_FieldType!(Uuid, Uuid);
impl<'a> IntoValue<'a> for Uuid {
    fn into_value(self) -> Value<'a> {
        Value::Uuid(self)
    }
}
impl SimpleFieldEq<Uuid> for Uuid {}
impl SimpleFieldIn<Uuid> for Uuid {}
impl SimpleFieldOrd<Uuid> for Uuid {}
