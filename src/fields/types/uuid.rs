use crate::conditions::Value;
use crate::internal::hmr::db_type;
use crate::{impl_AsDbType, impl_FieldEq};

impl_AsDbType!(uuid::Uuid, db_type::VarBinary, Value::Uuid);
impl_FieldEq!(impl<'rhs> FieldEq<'rhs, uuid::Uuid> for uuid::Uuid { Value::Uuid });
