use rorm_db::sql::value::NullType;
use uuid::Uuid;

use crate::conditions::Value;
use crate::internal::hmr::db_type;
use crate::{impl_AsDbType, impl_FieldEq};

impl_AsDbType!(uuid::Uuid, db_type::Uuid, Value::Uuid);
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Uuid> for Uuid { Value::Uuid });
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, Option<Uuid>> for Option<Uuid> { |option: Option<_>| option.map(Value::Uuid).unwrap_or(Value::Null(NullType::Uuid)) });
