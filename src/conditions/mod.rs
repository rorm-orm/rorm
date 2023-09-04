//! A high-level generic condition tree
//!
//! It is basically a generic version of the [`rorm_db::Condition`](conditional::Condition) tree.

use std::borrow::Cow;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use rorm_db::sql::{conditional, value};

use crate::internal::field::RawField;
use crate::internal::query_context::QueryContext;
use crate::internal::relation_path::{JoinAlias, Path};

pub mod collections;

pub use collections::{DynamicCollection, StaticCollection};

use crate::internal::field::access::FieldAccess;

/// A [`Condition`] in a box.
pub type BoxedCondition<'a> = Box<dyn Condition<'a>>;

/// Node in a condition tree
pub trait Condition<'a>: 'a + Send {
    /// Prepare a query context to be able to handle this condition by registering all implicit joins.
    fn add_to_context(&self, context: &mut QueryContext);

    /// Convert the condition into rorm-sql's format using a query context's registered joins.
    fn as_sql(&self, context: &QueryContext) -> conditional::Condition;

    /// Convert the condition into a boxed trait object to erase its concrete type
    fn boxed(self) -> BoxedCondition<'a>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}
impl<'a> Condition<'a> for BoxedCondition<'a> {
    fn add_to_context(&self, context: &mut QueryContext) {
        self.as_ref().add_to_context(context);
    }

    fn as_sql(&self, context: &QueryContext) -> conditional::Condition {
        self.as_ref().as_sql(context)
    }

    fn boxed(self) -> Box<dyn Condition<'a>>
    where
        Self: Sized,
    {
        self
    }
}

/// A value
///
/// However unlike rorm-sql's Value, this does not include an ident.
#[derive(Clone)]
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
    NaiveTime(NaiveTime),
    /// Naive Date representation
    NaiveDate(NaiveDate),
    /// Naive DateTime representation
    NaiveDateTime(NaiveDateTime),
}
impl<'a> Value<'a> {
    /// Convert into an [`sql::Value`](value::Value) instead of an [`sql::Condition`](conditional::Condition) directly.
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
            Value::NaiveTime(v) => value::Value::NaiveTime(*v),
            Value::NaiveDate(v) => value::Value::NaiveDate(*v),
            Value::NaiveDateTime(v) => value::Value::NaiveDateTime(*v),
        }
    }
}
impl<'a> Condition<'a> for Value<'a> {
    fn add_to_context(&self, _context: &mut QueryContext) {}

    fn as_sql(&self, _context: &QueryContext) -> conditional::Condition {
        conditional::Condition::Value(self.as_sql())
    }
}

/// A column name
#[derive(Copy, Clone)]
pub struct Column<A: FieldAccess>(pub A);

impl<'a, A: FieldAccess> Condition<'a> for Column<A> {
    fn add_to_context(&self, context: &mut QueryContext) {
        A::Path::add_to_context(context);
    }

    fn as_sql(&self, _context: &QueryContext) -> conditional::Condition {
        conditional::Condition::Value(value::Value::Column {
            table_name: Some(<A::Path as JoinAlias>::ALIAS),
            column_name: <A::Field as RawField>::NAME,
        })
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
#[derive(Copy, Clone)]
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
    fn add_to_context(&self, context: &mut QueryContext) {
        self.fst_arg.add_to_context(context);
        self.snd_arg.add_to_context(context);
    }

    fn as_sql(&self, context: &QueryContext) -> conditional::Condition {
        conditional::Condition::BinaryCondition((match self.operator {
            BinaryOperator::Equals => conditional::BinaryCondition::Equals,
            BinaryOperator::NotEquals => conditional::BinaryCondition::NotEquals,
            BinaryOperator::Greater => conditional::BinaryCondition::Greater,
            BinaryOperator::GreaterOrEquals => conditional::BinaryCondition::GreaterOrEquals,
            BinaryOperator::Less => conditional::BinaryCondition::Less,
            BinaryOperator::LessOrEquals => conditional::BinaryCondition::LessOrEquals,
            BinaryOperator::Like => conditional::BinaryCondition::Like,
            BinaryOperator::NotLike => conditional::BinaryCondition::NotLike,
            BinaryOperator::Regexp => conditional::BinaryCondition::Regexp,
            BinaryOperator::NotRegexp => conditional::BinaryCondition::NotRegexp,
        })(Box::new([
            self.fst_arg.as_sql(context),
            self.snd_arg.as_sql(context),
        ])))
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
#[derive(Copy, Clone)]
pub enum TernaryOperator {
    /// Between represents "{} BETWEEN {} AND {}" from SQL
    Between,
    /// NotBetween represents "{} NOT BETWEEN {} AND {}" from SQL
    NotBetween,
}
impl<'a, A: Condition<'a>, B: Condition<'a>, C: Condition<'a>> Condition<'a> for Ternary<A, B, C> {
    fn add_to_context(&self, context: &mut QueryContext) {
        self.fst_arg.add_to_context(context);
        self.snd_arg.add_to_context(context);
        self.trd_arg.add_to_context(context);
    }

    fn as_sql(&self, context: &QueryContext) -> conditional::Condition {
        conditional::Condition::TernaryCondition((match self.operator {
            TernaryOperator::Between => conditional::TernaryCondition::Between,
            TernaryOperator::NotBetween => conditional::TernaryCondition::NotBetween,
        })(Box::new([
            self.fst_arg.as_sql(context),
            self.snd_arg.as_sql(context),
            self.trd_arg.as_sql(context),
        ])))
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
#[derive(Copy, Clone)]
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
    fn add_to_context(&self, context: &mut QueryContext) {
        self.fst_arg.add_to_context(context);
    }

    fn as_sql(&self, context: &QueryContext) -> conditional::Condition {
        conditional::Condition::UnaryCondition((match self.operator {
            UnaryOperator::IsNull => conditional::UnaryCondition::IsNull,
            UnaryOperator::IsNotNull => conditional::UnaryCondition::IsNotNull,
            UnaryOperator::Exists => conditional::UnaryCondition::Exists,
            UnaryOperator::NotExists => conditional::UnaryCondition::NotExists,
            UnaryOperator::Not => conditional::UnaryCondition::Not,
        })(Box::new(self.fst_arg.as_sql(context))))
    }
}
