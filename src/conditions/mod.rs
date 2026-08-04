//! A high-level generic condition tree
//!
//! It is basically a generic version of the [`rorm_sql::Condition`](rorm_db::sql::conditional::Condition) tree.

use std::borrow::Cow;
use std::sync::Arc;

pub use rorm_db::sql::conditional::BinaryOperator;
pub use rorm_db::sql::conditional::TernaryOperator;
pub use rorm_db::sql::conditional::UnaryOperator;
use rorm_db::sql::value;

pub mod collections;
mod r#in;

pub use collections::{DynamicCollection, StaticCollection};
pub use r#in::{In, InOperator};

use crate::fields::proxy::{FieldProxy, FieldProxyImpl};
use crate::internal::field::Field;
use crate::internal::query_context::flat_conditions::FlatCondition;
use crate::internal::query_context::ConditionBuilder;

/// Node in a condition tree
pub trait Condition<'a>: Send + Sync {
    /// Adds this condition to a query context's internal representation
    ///
    /// If you're not implementing `Condition`,you'll probably want [`QueryContext::add_condition`].
    ///
    /// If you are implementing `Condition` for a custom type,
    /// please convert your type into one from [`rorm::conditions`](crate::conditions) first
    /// and then simply forward `build`.
    ///
    /// [`QueryContext::add_condition`]: crate::internal::query_context::QueryContext::add_condition
    fn build(&self, builder: ConditionBuilder<'_, 'a>);

    /// Convert the condition into a boxed trait object to erase its concrete type
    fn boxed<'this>(self) -> Box<dyn Condition<'a> + 'this>
    where
        Self: Sized + 'this,
    {
        Box::new(self)
    }

    /// Convert the condition into an arced trait object to erase its concrete type while remaining cloneable
    fn arc<'this>(self) -> Arc<dyn Condition<'a> + 'this>
    where
        Self: Sized + 'this,
    {
        Arc::new(self)
    }
}

impl<'a> Condition<'a> for Box<dyn Condition<'a> + '_> {
    fn build(&self, builder: ConditionBuilder<'_, 'a>) {
        self.as_ref().build(builder);
    }

    fn boxed<'this>(self) -> Box<dyn Condition<'a> + 'this>
    where
        Self: Sized + 'this,
    {
        self
    }

    fn arc<'this>(self) -> Arc<dyn Condition<'a> + 'this>
    where
        Self: Sized + 'this,
    {
        Arc::from(self)
    }
}
impl<'a> Condition<'a> for Arc<dyn Condition<'a> + '_> {
    fn build(&self, builder: ConditionBuilder<'_, 'a>) {
        self.as_ref().build(builder);
    }

    fn boxed<'this>(self) -> Box<dyn Condition<'a> + 'this>
    where
        Self: Sized + 'this,
    {
        Box::from(self)
    }

    fn arc<'this>(self) -> Arc<dyn Condition<'a> + 'this>
    where
        Self: Sized + 'this,
    {
        self
    }
}
impl<'a, C: Condition<'a> + ?Sized> Condition<'a> for &'_ C {
    fn build(&self, builder: ConditionBuilder<'_, 'a>) {
        <C as Condition<'a>>::build(*self, builder);
    }
}

/// A value
///
/// However unlike rorm-sql's Value, this does not include an ident.
// TODO: fix weird lifetime issue with arrays of non-Copy types
#[derive(Clone, Debug)]
pub enum Value<'a> {
    /// null representation
    Null(value::NullType),
    /// String representation
    String(Cow<'a, str>),
    /// Representation of choices
    Choice(Cow<'a, str>),
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
    Binary(Cow<'a, [u8]>),
    /// Naive Time representation
    #[cfg(feature = "chrono")]
    ChronoNaiveTime(chrono::NaiveTime),
    /// Naive Date representation
    #[cfg(feature = "chrono")]
    ChronoNaiveDate(chrono::NaiveDate),
    /// Naive DateTime representation
    #[cfg(feature = "chrono")]
    ChronoNaiveDateTime(chrono::NaiveDateTime),
    /// DateTime representation
    #[cfg(feature = "chrono")]
    ChronoDateTime(chrono::DateTime<chrono::Utc>),
    /// time's date representation
    #[cfg(feature = "time")]
    TimeDate(time::Date),
    /// time's time representation
    #[cfg(feature = "time")]
    TimeTime(time::Time),
    /// time's offset datetime representation
    #[cfg(feature = "time")]
    TimeOffsetDateTime(time::OffsetDateTime),
    /// time's primitive datetime representation
    #[cfg(feature = "time")]
    TimePrimitiveDateTime(time::PrimitiveDateTime),
    /// Uuid representation
    #[cfg(feature = "uuid")]
    Uuid(uuid::Uuid),
    /// Mac address representation
    #[cfg(feature = "postgres-only")]
    MacAddress(mac_address::MacAddress),
    /// IP network presentation
    #[cfg(feature = "postgres-only")]
    IpNetwork(ipnetwork::IpNetwork),
    /// Bit vec representation
    #[cfg(feature = "postgres-only")]
    BitVec(Cow<'a, bit_vec::BitVec>),

    /// null representation
    #[cfg(feature = "postgres-only")]
    ArrayNull(value::NullType),
    /// String representation
    #[cfg(feature = "postgres-only")]
    ArrayString(Vec<&'a str>), // TODO: why no Cow?
    /// i64 representation
    #[cfg(feature = "postgres-only")]
    ArrayI64(Cow<'a, [i64]>),
    /// i32 representation
    #[cfg(feature = "postgres-only")]
    ArrayI32(Cow<'a, [i32]>),
    /// i16 representation
    #[cfg(feature = "postgres-only")]
    ArrayI16(Cow<'a, [i16]>),
    /// Bool representation
    #[cfg(feature = "postgres-only")]
    ArrayBool(Cow<'a, [bool]>),
    /// f64 representation
    #[cfg(feature = "postgres-only")]
    ArrayF64(Cow<'a, [f64]>),
    /// f32 representation
    #[cfg(feature = "postgres-only")]
    ArrayF32(Cow<'a, [f32]>),
    /// binary representation
    #[cfg(feature = "postgres-only")]
    ArrayBinary(Vec<&'a [u8]>), // TODO: why no Cow?
    /// Naive Time representation
    #[cfg(feature = "chrono")]
    #[cfg(feature = "postgres-only")]
    ArrayChronoNaiveTime(Cow<'a, [chrono::NaiveTime]>),
    /// Naive Date representation
    #[cfg(feature = "chrono")]
    #[cfg(feature = "postgres-only")]
    ArrayChronoNaiveDate(Cow<'a, [chrono::NaiveDate]>),
    /// Naive DateTime representation
    #[cfg(feature = "chrono")]
    #[cfg(feature = "postgres-only")]
    ArrayChronoNaiveDateTime(Cow<'a, [chrono::NaiveDateTime]>),
    /// DateTime representation
    #[cfg(feature = "chrono")]
    #[cfg(feature = "postgres-only")]
    ArrayChronoDateTime(Cow<'a, [chrono::DateTime<chrono::Utc>]>),
    /// time's date representation
    #[cfg(feature = "time")]
    #[cfg(feature = "postgres-only")]
    ArrayTimeDate(Cow<'a, [time::Date]>),
    /// time's time representation
    #[cfg(feature = "time")]
    #[cfg(feature = "postgres-only")]
    ArrayTimeTime(Cow<'a, [time::Time]>),
    /// time's offset datetime representation
    #[cfg(feature = "time")]
    #[cfg(feature = "postgres-only")]
    ArrayTimeOffsetDateTime(Cow<'a, [time::OffsetDateTime]>),
    /// time's primitive datetime representation
    #[cfg(feature = "time")]
    #[cfg(feature = "postgres-only")]
    ArrayTimePrimitiveDateTime(Cow<'a, [time::PrimitiveDateTime]>),
    /// Uuid representation
    #[cfg(feature = "uuid")]
    #[cfg(feature = "postgres-only")]
    ArrayUuid(Cow<'a, [uuid::Uuid]>),
    /// Mac address representation
    #[cfg(feature = "postgres-only")]
    ArrayMacAddress(Cow<'a, [mac_address::MacAddress]>),
    /// IP network presentation
    #[cfg(feature = "postgres-only")]
    ArrayIpNetwork(Cow<'a, [ipnetwork::IpNetwork]>),
    /// Bit vec representation
    #[cfg(feature = "postgres-only")]
    ArrayBitVec(Vec<&'a bit_vec::BitVec>), // TODO: why no Cow?
}
impl Value<'_> {
    /// Convert into an [`sql::Value`](value::Value) instead of an [`sql::Condition`](rorm_db::sql::conditional::Condition) directly.
    pub fn as_sql(&self) -> value::Value<'_> {
        match self {
            Value::Null(null_type) => value::Value::Null(*null_type),
            Value::String(v) => value::Value::String(v.as_ref()),
            Value::Choice(v) => value::Value::Choice(v.as_ref()),
            Value::I64(v) => value::Value::I64(*v),
            Value::I32(v) => value::Value::I32(*v),
            Value::I16(v) => value::Value::I16(*v),
            Value::Bool(v) => value::Value::Bool(*v),
            Value::F64(v) => value::Value::F64(*v),
            Value::F32(v) => value::Value::F32(*v),
            Value::Binary(v) => value::Value::Binary(v.as_ref()),
            #[cfg(feature = "chrono")]
            Value::ChronoNaiveTime(v) => value::Value::ChronoNaiveTime(*v),
            #[cfg(feature = "chrono")]
            Value::ChronoNaiveDate(v) => value::Value::ChronoNaiveDate(*v),
            #[cfg(feature = "chrono")]
            Value::ChronoNaiveDateTime(v) => value::Value::ChronoNaiveDateTime(*v),
            #[cfg(feature = "chrono")]
            Value::ChronoDateTime(v) => value::Value::ChronoDateTime(*v),
            #[cfg(feature = "time")]
            Value::TimeDate(v) => value::Value::TimeDate(*v),
            #[cfg(feature = "time")]
            Value::TimeTime(v) => value::Value::TimeTime(*v),
            #[cfg(feature = "time")]
            Value::TimeOffsetDateTime(v) => value::Value::TimeOffsetDateTime(*v),
            #[cfg(feature = "time")]
            Value::TimePrimitiveDateTime(v) => value::Value::TimePrimitiveDateTime(*v),
            #[cfg(feature = "uuid")]
            Value::Uuid(v) => value::Value::Uuid(*v),
            #[cfg(feature = "postgres-only")]
            Value::MacAddress(v) => value::Value::MacAddress(*v),
            #[cfg(feature = "postgres-only")]
            Value::IpNetwork(v) => value::Value::IpNetwork(*v),
            #[cfg(feature = "postgres-only")]
            Value::BitVec(v) => value::Value::BitVec(v.as_ref()),

            #[cfg(feature = "postgres-only")]
            Value::ArrayNull(v) => value::Value::ArrayNull(*v),
            #[cfg(feature = "postgres-only")]
            Value::ArrayString(v) => value::Value::ArrayString(v.as_ref()),
            #[cfg(feature = "postgres-only")]
            Value::ArrayI64(v) => value::Value::ArrayI64(v.as_ref()),
            #[cfg(feature = "postgres-only")]
            Value::ArrayI32(v) => value::Value::ArrayI32(v.as_ref()),
            #[cfg(feature = "postgres-only")]
            Value::ArrayI16(v) => value::Value::ArrayI16(v.as_ref()),
            #[cfg(feature = "postgres-only")]
            Value::ArrayBool(v) => value::Value::ArrayBool(v.as_ref()),
            #[cfg(feature = "postgres-only")]
            Value::ArrayF64(v) => value::Value::ArrayF64(v.as_ref()),
            #[cfg(feature = "postgres-only")]
            Value::ArrayF32(v) => value::Value::ArrayF32(v.as_ref()),
            #[cfg(feature = "postgres-only")]
            Value::ArrayBinary(v) => value::Value::ArrayBinary(v.as_ref()),
            #[cfg(feature = "chrono")]
            #[cfg(feature = "postgres-only")]
            Value::ArrayChronoNaiveTime(v) => value::Value::ArrayChronoNaiveTime(v.as_ref()),
            #[cfg(feature = "chrono")]
            #[cfg(feature = "postgres-only")]
            Value::ArrayChronoNaiveDate(v) => value::Value::ArrayChronoNaiveDate(v.as_ref()),
            #[cfg(feature = "chrono")]
            #[cfg(feature = "postgres-only")]
            Value::ArrayChronoNaiveDateTime(v) => {
                value::Value::ArrayChronoNaiveDateTime(v.as_ref())
            }
            #[cfg(feature = "chrono")]
            #[cfg(feature = "postgres-only")]
            Value::ArrayChronoDateTime(v) => value::Value::ArrayChronoDateTime(v.as_ref()),
            #[cfg(feature = "time")]
            #[cfg(feature = "postgres-only")]
            Value::ArrayTimeDate(v) => value::Value::ArrayTimeDate(v.as_ref()),
            #[cfg(feature = "time")]
            #[cfg(feature = "postgres-only")]
            Value::ArrayTimeTime(v) => value::Value::ArrayTimeTime(v.as_ref()),
            #[cfg(feature = "time")]
            #[cfg(feature = "postgres-only")]
            Value::ArrayTimeOffsetDateTime(v) => value::Value::ArrayTimeOffsetDateTime(v.as_ref()),
            #[cfg(feature = "time")]
            #[cfg(feature = "postgres-only")]
            Value::ArrayTimePrimitiveDateTime(v) => {
                value::Value::ArrayTimePrimitiveDateTime(v.as_ref())
            }
            #[cfg(feature = "uuid")]
            #[cfg(feature = "postgres-only")]
            Value::ArrayUuid(v) => value::Value::ArrayUuid(v.as_ref()),
            #[cfg(feature = "postgres-only")]
            Value::ArrayMacAddress(v) => value::Value::ArrayMacAddress(v.as_ref()),
            #[cfg(feature = "postgres-only")]
            Value::ArrayIpNetwork(v) => value::Value::ArrayIpNetwork(v.as_ref()),
            #[cfg(feature = "postgres-only")]
            Value::ArrayBitVec(v) => value::Value::ArrayBitVec(v.as_ref()),
        }
    }
}
impl<'c, 'v: 'c> Condition<'c> for Value<'v> {
    fn build(&self, mut builder: ConditionBuilder<'_, 'c>) {
        let value_index = builder.push_value(self.clone());
        builder.push_condition(FlatCondition::Value(value_index));
    }
}

/// A column name
#[derive(Copy, Clone)]
pub struct Column<I: FieldProxyImpl>(pub FieldProxy<I>);

impl<'a, I: FieldProxyImpl> Condition<'a> for Column<I> {
    fn build(&self, mut builder: ConditionBuilder<'_, 'a>) {
        let path_id = builder.add_path::<I::Path>();
        builder.push_condition(FlatCondition::Column(path_id, &<I::Field as Field>::NAME));
    }
}

/// A binary expression
#[derive(Copy, Clone)]
pub struct Binary<A, B> {
    /// SQL operator to use
    pub operator: BinaryOperator,

    /// The expression's first argument
    pub fst_arg: A,

    /// The expression's second argument
    pub snd_arg: B,
}
impl<'a, A: Condition<'a>, B: Condition<'a>> Condition<'a> for Binary<A, B> {
    fn build(&self, mut builder: ConditionBuilder<'_, 'a>) {
        builder.push_condition(FlatCondition::BinaryCondition(self.operator));
        self.fst_arg.build(builder.reborrow());
        self.snd_arg.build(builder.reborrow());
    }
}

/// A ternary expression
#[derive(Copy, Clone)]
pub struct Ternary<A, B, C> {
    /// SQL operator to use
    pub operator: TernaryOperator,

    /// The expression's first argument
    pub fst_arg: A,

    /// The expression's second argument
    pub snd_arg: B,

    /// The expression's third argument
    pub trd_arg: C,
}
impl<'a, A: Condition<'a>, B: Condition<'a>, C: Condition<'a>> Condition<'a> for Ternary<A, B, C> {
    fn build(&self, mut builder: ConditionBuilder<'_, 'a>) {
        builder.push_condition(FlatCondition::TernaryCondition(self.operator));
        self.fst_arg.build(builder.reborrow());
        self.snd_arg.build(builder.reborrow());
        self.trd_arg.build(builder.reborrow());
    }
}

/// A unary expression
#[derive(Copy, Clone)]
pub struct Unary<A> {
    /// SQL operator to use
    pub operator: UnaryOperator,

    /// The expression's first argument
    pub fst_arg: A,
}
impl<'a, A: Condition<'a>> Condition<'a> for Unary<A> {
    fn build(&self, mut builder: ConditionBuilder<'_, 'a>) {
        builder.push_condition(FlatCondition::UnaryCondition(self.operator));
        self.fst_arg.build(builder.reborrow());
    }
}
