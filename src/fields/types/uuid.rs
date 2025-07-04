use rorm_db::sql::value::NullType;
use uuid::Uuid;

use crate::conditions::Value;
use crate::fields::traits::simple::SimpleFieldEq;
use crate::impl_FieldType;

impl_FieldType!(Uuid, Uuid, Value::Uuid);
impl<'rhs> SimpleFieldEq<'rhs, Uuid> for Uuid {
    fn into_value(rhs: Uuid) -> Value<'rhs> {
        Value::Uuid(rhs)
    }
}
impl<'rhs> SimpleFieldEq<'rhs, Option<Uuid>> for Option<Uuid> {
    fn into_value(rhs: Option<Uuid>) -> Value<'rhs> {
        rhs.map(Value::Uuid).unwrap_or(Value::Null(NullType::Uuid))
    }
}
