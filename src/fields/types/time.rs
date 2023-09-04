use crate::conditions::Value;
use crate::impl_AsDbType;
use crate::internal::hmr::db_type;

impl_AsDbType!(time::Time, db_type::Time, Value::TimeTime);
impl_AsDbType!(time::Date, db_type::Date, Value::TimeDate);
impl_AsDbType!(
    time::OffsetDateTime,
    db_type::DateTime,
    Value::TimeOffsetDateTime
);
impl_AsDbType!(
    time::PrimitiveDateTime,
    db_type::DateTime,
    Value::TimePrimitiveDateTime
);
