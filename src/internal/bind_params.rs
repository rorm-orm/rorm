use rorm_sql::value::{NullType, Value};
use sqlx::types::Json;

use super::any::{AnyEncode, AnyQuery, AnyType};

/// This helper method is used to bind condition [`Value`]s to the query.
pub fn bind_param<'exe, 'val>(query: &mut AnyQuery<'exe>, param: Value<'val>) {
    match param {
        Value::String(x) => query.bind(x),
        Value::I64(x) => query.bind(x),
        Value::I32(x) => query.bind(x),
        Value::I16(x) => query.bind(x),
        Value::Bool(x) => query.bind(x),
        Value::F32(x) => query.bind(x),
        Value::F64(x) => query.bind(x),
        Value::Binary(x) => query.bind(x),
        #[allow(deprecated)]
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
        #[allow(deprecated)]
        Value::UuidHyphenated(x) => query.bind(x.hyphenated().to_string()),
        #[allow(deprecated)]
        Value::UuidSimple(x) => query.bind(x.simple().to_string()),

        Value::JsonValue(x) => query.bind(Json(x)),

        #[cfg(feature = "postgres-only")]
        Value::MacAddress(x) => query.bind(x),
        #[cfg(feature = "postgres-only")]
        Value::IpNetwork(x) => query.bind(x),
        #[cfg(feature = "postgres-only")]
        Value::BitVec(x) => query.bind(x),

        #[cfg(feature = "postgres-only")]
        Value::ArrayString(x) => query.bind(x),
        #[cfg(feature = "postgres-only")]
        Value::ArrayI64(x) => query.bind(x),
        #[cfg(feature = "postgres-only")]
        Value::ArrayI32(x) => query.bind(x),
        #[cfg(feature = "postgres-only")]
        Value::ArrayI16(x) => query.bind(x),
        #[cfg(feature = "postgres-only")]
        Value::ArrayBool(x) => query.bind(x),
        #[cfg(feature = "postgres-only")]
        Value::ArrayF32(x) => query.bind(x),
        #[cfg(feature = "postgres-only")]
        Value::ArrayF64(x) => query.bind(x),
        #[cfg(feature = "postgres-only")]
        Value::ArrayBinary(x) => query.bind(x),
        #[cfg(feature = "postgres-only")]
        Value::ArrayChronoNaiveDate(x) => query.bind(x),
        #[cfg(feature = "postgres-only")]
        Value::ArrayChronoNaiveTime(x) => query.bind(x),
        #[cfg(feature = "postgres-only")]
        Value::ArrayChronoNaiveDateTime(x) => query.bind(x),
        #[cfg(feature = "postgres-only")]
        Value::ArrayChronoDateTime(x) => query.bind(x),

        #[cfg(feature = "postgres-only")]
        Value::ArrayTimeDate(x) => query.bind(x),
        #[cfg(feature = "postgres-only")]
        Value::ArrayTimeTime(x) => query.bind(x),
        #[cfg(feature = "postgres-only")]
        Value::ArrayTimeOffsetDateTime(x) => query.bind(x),
        #[cfg(feature = "postgres-only")]
        Value::ArrayTimePrimitiveDateTime(x) => query.bind(x),

        #[cfg(feature = "postgres-only")]
        Value::ArrayUuid(x) => query.bind(x),

        #[cfg(feature = "postgres-only")]
        Value::ArrayJsonValue(x) => query.bind(Json(x)),

        #[cfg(feature = "postgres-only")]
        Value::ArrayMacAddress(x) => query.bind(x),
        #[cfg(feature = "postgres-only")]
        Value::ArrayIpNetwork(x) => query.bind(x),
        #[cfg(feature = "postgres-only")]
        Value::ArrayBitVec(x) => query.bind(x),

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
        #[cfg(feature = "postgres-only")]
        Value::ArrayNull(null_type) => match null_type {
            NullType::String => query.bind(None::<&str>),
            NullType::I64 => query.bind(None::<i64>.map(vec)),
            NullType::I32 => query.bind(None::<i32>.map(vec)),
            NullType::I16 => query.bind(None::<i16>.map(vec)),
            NullType::Bool => query.bind(None::<bool>.map(vec)),
            NullType::F64 => query.bind(None::<f64>.map(vec)),
            NullType::F32 => query.bind(None::<f32>.map(vec)),
            NullType::Binary => query.bind(None::<&[u8]>.map(vec)),
            NullType::Choice => {}

            NullType::ChronoNaiveTime => query.bind(none(Value::ChronoNaiveTime).map(vec)),
            NullType::ChronoNaiveDate => query.bind(none(Value::ChronoNaiveDate).map(vec)),
            NullType::ChronoNaiveDateTime => query.bind(none(Value::ChronoNaiveDateTime).map(vec)),
            NullType::ChronoDateTime => query.bind(none(Value::ChronoDateTime).map(vec)),

            NullType::TimeDate => query.bind(none(Value::TimeDate).map(vec)),
            NullType::TimeTime => query.bind(none(Value::TimeTime).map(vec)),
            NullType::TimeOffsetDateTime => query.bind(none(Value::TimeOffsetDateTime).map(vec)),
            NullType::TimePrimitiveDateTime => {
                query.bind(none(Value::TimePrimitiveDateTime).map(vec))
            }

            NullType::Uuid => query.bind(none(Value::Uuid).map(vec)),
            NullType::UuidHyphenated => query.bind(None::<String>.map(vec)),
            NullType::UuidSimple => query.bind(None::<String>.map(vec)),

            NullType::JsonValue => query.bind(none(Value::JsonValue).map(vec)),

            NullType::MacAddress => query.bind(none(Value::MacAddress).map(vec)),
            NullType::IpNetwork => query.bind(none(Value::IpNetwork).map(vec)),
            NullType::BitVec => query.bind(none(Value::BitVec).map(vec)),
        },
    }
}

/// Hack to avoid naming the types which would require direct dependencies
fn none<'a, T, F>(_value_variant: F) -> Option<T>
where
    F: Fn(T) -> Value<'a>,
    Option<T>: 'a + Send + AnyEncode<'a> + AnyType,
{
    None
}

/// Hack to avoid naming the types which would require direct dependencies
///
/// Used to convert an `Option<T>` which is **always** `None` with `.map()`
#[cfg(feature = "postgres-only")]
fn vec<T>(_never: T) -> Vec<T> {
    unreachable!()
}
