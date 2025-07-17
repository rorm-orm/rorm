//! A high-level generic condition tree
//!
//! It is basically a generic version of the [`rorm_sql::Condition`](rorm_db::sql::conditional::Condition) tree.

use std::borrow::Cow;
use std::sync::Arc;

// use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
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
    BitVec(crate::fields::types::postgres_only::BitCow<'a>),
}
impl Value<'_> {
    /// Convert into an [`sql::Value`](value::Value) instead of an [`sql::Condition`](rorm_db::sql::conditional::Condition) directly.
    pub fn as_sql(&self) -> value::Value {
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
        }
    }
}
impl<'a> Condition<'a> for Value<'a> {
    fn build(&self, mut builder: ConditionBuilder<'_, 'a>) {
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
/// A binary operator
#[derive(Copy, Clone, Debug)]
pub enum BinaryOperator {
    /// Representation of "{} = {}" in SQL
    Equals,
    /// Representation of "{} <> {}" in SQL
    NotEquals,
    /// Representation of "{} > {}" in SQL
    Greater,
    /// Representation of "{} >= {}" in SQL
    GreaterOrEquals,
    /// Representation of "{} < {}" in SQL
    Less,
    /// Representation of "{} <= {}" in SQL
    LessOrEquals,
    /// Representation of "{} LIKE {}" in SQL
    Like,
    /// Representation of "{} NOT LIKE {}" in SQL
    NotLike,
    /// Representation of "{} REGEXP {}" in SQL
    Regexp,
    /// Representation of "{} NOT REGEXP {}" in SQL
    NotRegexp,
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
/// A ternary operator
#[derive(Copy, Clone, Debug)]
pub enum TernaryOperator {
    /// Between represents "{} BETWEEN {} AND {}" from SQL
    Between,
    /// NotBetween represents "{} NOT BETWEEN {} AND {}" from SQL
    NotBetween,
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
/// A unary operator
#[derive(Copy, Clone, Debug)]
pub enum UnaryOperator {
    /// Representation of SQL's "{} IS NULL"
    IsNull,
    /// Representation of SQL's "{} IS NOT NULL"
    IsNotNull,
    /// Representation of SQL's "EXISTS {}"
    Exists,
    /// Representation of SQL's "NOT EXISTS {}"
    NotExists,
    /// Representation of SQL's "NOT {}"
    Not,
}
impl<'a, A: Condition<'a>> Condition<'a> for Unary<A> {
    fn build(&self, mut builder: ConditionBuilder<'_, 'a>) {
        builder.push_condition(FlatCondition::UnaryCondition(self.operator));
        self.fst_arg.build(builder.reborrow());
    }
}
