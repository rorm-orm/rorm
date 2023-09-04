use crate::conditions::Value;
use crate::impl_AsDbType;
use crate::internal::hmr::db_type;

impl_AsDbType!(uuid::Uuid, db_type::VarBinary, Value::Uuid);
