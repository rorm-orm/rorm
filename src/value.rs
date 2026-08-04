use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time};
use uuid::Uuid;

/// This enum represents a [Null](Value::Null)'s type
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NullType {
    /// String representation
    String,
    /// Choice representation
    Choice,
    /// i64 representation
    I64,
    /// i32 representation
    I32,
    /// i16 representation
    I16,
    /// Bool representation
    Bool,
    /// f64 representation
    F64,
    /// f32 representation
    F32,
    /// binary representation
    Binary,
    /// Naive Time representation
    ChronoNaiveTime,
    /// Naive Date representation
    ChronoNaiveDate,
    /// Naive DateTime representation
    ChronoNaiveDateTime,
    /// Chrono timezone aware date time representation
    ChronoDateTime,
    /// time's date representation
    TimeDate,
    /// time's time representation
    TimeTime,
    /// time's offset datetime representation
    TimeOffsetDateTime,
    /// time's primitive datetime representation
    TimePrimitiveDateTime,
    /// Uuid representation
    Uuid,
    /// Uuid in hyphenated representation
    UuidHyphenated,
    /// Uuid in simple text representation
    UuidSimple,
    /// serde_json's Value representation
    JsonValue,
    /// Mac address representation
    #[cfg(feature = "postgres-only")]
    MacAddress,
    /// IP network presentation
    #[cfg(feature = "postgres-only")]
    IpNetwork,
    /// Bit vec representation
    #[cfg(feature = "postgres-only")]
    BitVec,
}

/**
This enum represents a value
 */
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Value<'a> {
    /// null representation
    Null(NullType),
    /// Representation of an identifier, e.g. a column.
    /// This variant will not be escaped, so do not
    /// pass unchecked data to it.
    #[deprecated(note = "Is this still used?")]
    Ident(&'a str),
    /// Representation of a column name with
    /// an optional table name
    Column {
        /// Name of the table
        table_name: Option<&'a str>,
        /// Name of the column
        column_name: &'a str,
    },
    /// Representation of choices
    Choice(&'a str),
    /// String representation
    String(&'a str),
    /// i64 representation
    I64(i64),
    /// i32 representation
    I32(i32),
    /// i16 representation
    I16(i16),
    /// Bool representation
    Bool(bool),
    /// f64 representation
    F64(f64),
    /// f32 representation
    F32(f32),
    /// binary representation
    Binary(&'a [u8]),
    /// chrono's Naive Time representation
    ChronoNaiveTime(NaiveTime),
    /// chrono's Naive Date representation
    ChronoNaiveDate(NaiveDate),
    /// chrono's Naive DateTime representation
    ChronoNaiveDateTime(NaiveDateTime),
    /// chrono's Timezone aware datetime
    ChronoDateTime(DateTime<Utc>),
    /// time's date representation
    TimeDate(Date),
    /// time's time representation
    TimeTime(Time),
    /// time's offset datetime representation
    TimeOffsetDateTime(OffsetDateTime),
    /// time's primitive datetime representation
    TimePrimitiveDateTime(PrimitiveDateTime),
    /// Uuid representation
    Uuid(Uuid),
    /// Uuid in hyphenated representation
    #[deprecated(note = "Was this ever used?")]
    UuidHyphenated(Uuid),
    /// Uuid in simple text representation
    #[deprecated(note = "Was this ever used?")]
    UuidSimple(Uuid),
    /// serde_json's Value representation
    JsonValue(&'a serde_json::Value),

    /// Mac address representation
    #[cfg(feature = "postgres-only")]
    MacAddress(mac_address::MacAddress),
    /// IP network presentation
    #[cfg(feature = "postgres-only")]
    IpNetwork(ipnetwork::IpNetwork),
    /// Bit vec representation
    #[cfg(feature = "postgres-only")]
    BitVec(&'a bit_vec::BitVec),
}
