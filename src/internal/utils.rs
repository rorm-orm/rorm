//! Utility functions

use rorm_sql::value::{NullType, Value};
use sqlx::types::Json;

use super::any::{AnyEncode, AnyQuery, AnyType};

/// This helper method is used to bind ConditionValues to the query.
pub fn bind_param<'post_query, 'query>(query: &mut AnyQuery<'query>, param: Value<'post_query>)
where
    'post_query: 'query,
{
    match param {
        Value::String(x) => query.bind(x),
        Value::I64(x) => query.bind(x),
        Value::I32(x) => query.bind(x),
        Value::I16(x) => query.bind(x),
        Value::Bool(x) => query.bind(x),
        Value::F32(x) => query.bind(x),
        Value::F64(x) => query.bind(x),
        Value::Binary(x) => query.bind(x),
        Value::Ident(_) => {}
        Value::Column { .. } => {}
        Value::Choice(_) => {}

        Value::ChronoNaiveDate(x) => query.bind(x),
        Value::ChronoNaiveTime(x) => query.bind(x),
        Value::ChronoNaiveDateTime(x) => query.bind(x),
        Value::ChronoDateTime(x) => query.bind(x),

        Value::TimeDate(x) => query.bind(x),
        Value::TimeTime(x) => query.bind(x),
        Value::TimeOffsetDateTime(x) => query.bind(x),
        Value::TimePrimitiveDateTime(x) => query.bind(x),

        Value::Uuid(x) => query.bind(x),
        Value::UuidHyphenated(x) => query.bind(x.hyphenated().to_string()),
        Value::UuidSimple(x) => query.bind(x.simple().to_string()),

        Value::JsonValue(x) => query.bind(Json(x)),

        #[cfg(feature = "postgres-only")]
        Value::MacAddress(x) => query.bind(x),
        #[cfg(feature = "postgres-only")]
        Value::IpNetwork(x) => query.bind(x),
        #[cfg(feature = "postgres-only")]
        Value::BitVec(x) => query.bind(x),

        Value::Null(null_type) => match null_type {
            NullType::String => query.bind(None::<&str>),
            NullType::I64 => query.bind(None::<i64>),
            NullType::I32 => query.bind(None::<i32>),
            NullType::I16 => query.bind(None::<i16>),
            NullType::Bool => query.bind(None::<bool>),
            NullType::F64 => query.bind(None::<f64>),
            NullType::F32 => query.bind(None::<f32>),
            NullType::Binary => query.bind(None::<&[u8]>),
            NullType::Choice => {}

            NullType::ChronoNaiveTime => query.bind(none(Value::ChronoNaiveTime)),
            NullType::ChronoNaiveDate => query.bind(none(Value::ChronoNaiveDate)),
            NullType::ChronoNaiveDateTime => query.bind(none(Value::ChronoNaiveDateTime)),
            NullType::ChronoDateTime => query.bind(none(Value::ChronoDateTime)),

            NullType::TimeDate => query.bind(none(Value::TimeDate)),
            NullType::TimeTime => query.bind(none(Value::TimeTime)),
            NullType::TimeOffsetDateTime => query.bind(none(Value::TimeOffsetDateTime)),
            NullType::TimePrimitiveDateTime => query.bind(none(Value::TimePrimitiveDateTime)),

            NullType::Uuid => query.bind(none(Value::Uuid)),
            NullType::UuidHyphenated => query.bind(None::<String>),
            NullType::UuidSimple => query.bind(None::<String>),

            NullType::JsonValue => query.bind(none(Value::JsonValue)),

            #[cfg(feature = "postgres-only")]
            NullType::MacAddress => query.bind(none(Value::MacAddress)),
            #[cfg(feature = "postgres-only")]
            NullType::IpNetwork => query.bind(none(Value::IpNetwork)),
            #[cfg(feature = "postgres-only")]
            NullType::BitVec => query.bind(none(Value::BitVec)),
        },
    }
}

/// Little helper hack to avoid using naming the types
fn none<'a, T, F>(_value_variant: F) -> Option<T>
where
    F: Fn(T) -> Value<'a>,
    Option<T>: 'a + Send + AnyEncode<'a> + AnyType,
{
    None
}
