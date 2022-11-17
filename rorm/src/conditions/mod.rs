//! A high-level generic condition tree
//!
//! It is basically a generic version of the [rorm_db::Condition] tree.

use std::marker::PhantomData;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use rorm_db::{conditional, value};
use rorm_declaration::hmr::db_type::{
    Choices, Date, DateTime, DbType, Double, Float, Int16, Int32, Int64, Time, VarBinary, VarChar,
};

use crate::internal::field::{Field, FieldProxy};
use crate::internal::relation_path::Path;

pub mod collections;
use crate::internal::query_context::{QueryContext, QueryContextBuilder};
pub use collections::{DynamicCollection, StaticCollection};

/// A [Condition] in a box.
pub type BoxedCondition<'a> = Box<dyn Condition<'a>>;

/// Node in a condition tree
pub trait Condition<'a>: 'a {
    /// Prepare a query context to be able to handle this condition by registering all implicit joins.
    fn add_to_builder(&self, builder: &mut QueryContextBuilder);

    /// Convert the condition into rorm-sql's format using a query context's registered joins.
    fn as_sql<'c>(&self, context: &'c QueryContext) -> conditional::Condition<'c>
    where
        'a: 'c;

    /// Convert the condition into a boxed trait object to erase its concrete type
    fn boxed(self) -> BoxedCondition<'a>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}
impl<'a> Condition<'a> for BoxedCondition<'a> {
    fn add_to_builder(&self, builder: &mut QueryContextBuilder) {
        self.as_ref().add_to_builder(builder);
    }

    fn as_sql<'c>(&self, context: &'c QueryContext) -> conditional::Condition<'c>
    where
        'a: 'c,
    {
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
#[derive(Copy, Clone)]
pub enum Value<'a> {
    /// null representation
    Null,
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
    /// Naive Time representation
    NaiveTime(NaiveTime),
    /// Naive Date representation
    NaiveDate(NaiveDate),
    /// Naive DateTime representation
    NaiveDateTime(NaiveDateTime),
}
impl<'a> Value<'a> {
    /// Convert into an [sql::Value](value::Value) instead of an [sql::Condition](conditional::Condition) directly.
    pub fn into_sql(self) -> value::Value<'a> {
        match self {
            Value::Null => value::Value::Null,
            Value::String(v) => value::Value::String(v),
            Value::I64(v) => value::Value::I64(v),
            Value::I32(v) => value::Value::I32(v),
            Value::I16(v) => value::Value::I16(v),
            Value::Bool(v) => value::Value::Bool(v),
            Value::F64(v) => value::Value::F64(v),
            Value::F32(v) => value::Value::F32(v),
            Value::Binary(v) => value::Value::Binary(v),
            Value::NaiveTime(v) => value::Value::NaiveTime(v),
            Value::NaiveDate(v) => value::Value::NaiveDate(v),
            Value::NaiveDateTime(v) => value::Value::NaiveDateTime(v),
        }
    }
}
impl<'a> Condition<'a> for Value<'a> {
    fn add_to_builder(&self, _builder: &mut QueryContextBuilder) {}

    fn as_sql<'c>(&self, _context: &'c QueryContext) -> conditional::Condition<'c>
    where
        'a: 'c,
    {
        conditional::Condition::Value(self.into_sql())
    }
}

/// A column name
#[derive(Copy, Clone)]
pub struct Column<F, P> {
    pub(crate) field: PhantomData<F>,
    pub(crate) path: PhantomData<P>,
}
impl<F: Field, P: Path> Column<F, P> {
    /// Construct a new instance
    pub const fn new() -> Self {
        Column {
            field: PhantomData,
            path: PhantomData,
        }
    }
}
impl<'a, F: Field, P: Path> Condition<'a> for Column<F, P> {
    fn add_to_builder(&self, builder: &mut QueryContextBuilder) {
        builder.add_field_proxy::<F, P>();
    }

    fn as_sql<'c>(&self, context: &'c QueryContext) -> conditional::Condition<'c>
    where
        'a: 'c,
    {
        conditional::Condition::Value(value::Value::Ident(context.get_field::<F, P>()))
    }
}

/// A binary expression
#[derive(Copy, Clone)]
pub struct Binary<A, B> {
    pub(crate) operator: BinaryOperator,
    pub(crate) fst_arg: A,
    pub(crate) snd_arg: B,
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
    fn add_to_builder(&self, builder: &mut QueryContextBuilder) {
        self.fst_arg.add_to_builder(builder);
        self.snd_arg.add_to_builder(builder);
    }

    fn as_sql<'c>(&self, context: &'c QueryContext) -> conditional::Condition<'c>
    where
        'a: 'c,
    {
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
    pub(crate) operator: TernaryOperator,
    pub(crate) fst_arg: A,
    pub(crate) snd_arg: B,
    pub(crate) trd_arg: C,
}
/// A ternary operator
#[derive(Copy, Clone)]
pub enum TernaryOperator {
    /// Between represents "{} BETWEEN {} AND {}" from SQL
    Between,
    /// Between represents "{} NOT BETWEEN {} AND {}" from SQL
    NotBetween,
}
impl<'a, A: Condition<'a>, B: Condition<'a>, C: Condition<'a>> Condition<'a> for Ternary<A, B, C> {
    fn add_to_builder(&self, builder: &mut QueryContextBuilder) {
        self.fst_arg.add_to_builder(builder);
        self.snd_arg.add_to_builder(builder);
        self.trd_arg.add_to_builder(builder);
    }

    fn as_sql<'c>(&self, context: &'c QueryContext) -> conditional::Condition<'c>
    where
        'a: 'c,
    {
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

/// Mark common rust types as convertable into certain condition values.
///
/// This trait is used to simplify rorm's api and not internally.
pub trait IntoSingleValue<'a, D: DbType>: 'a {
    /// The condition tree node type
    ///
    /// Either [Value] or [Column]
    type Condition: Condition<'a>;

    /// Convert into a condition tree node
    ///
    /// Call this when the result is used in the generic condition tree,
    /// i.e. when you need to preserve a column's data.
    fn into_condition(self) -> Self::Condition;

    /// Convert into an sql value
    ///
    /// Call this when the result is passed to the db and columns don't need different treatment than values.
    ///
    /// This method will probably be refactored or removed.
    // TODO it's used in update which theoretically should handle joins and therefore needs the distinction
    fn into_value(self) -> value::Value<'a>;
}

impl<'a, S: AsRef<str> + ?Sized> IntoSingleValue<'a, VarChar> for &'a S {
    type Condition = Value<'a>;
    fn into_condition(self) -> Self::Condition {
        Value::String(self.as_ref())
    }
    fn into_value(self) -> value::Value<'a> {
        IntoSingleValue::<'a, VarChar>::into_condition(self).into_sql()
    }
}

impl<'a, S: AsRef<str> + ?Sized> IntoSingleValue<'a, Choices> for &'a S {
    type Condition = Value<'a>;
    fn into_condition(self) -> Self::Condition {
        Value::String(self.as_ref())
    }
    fn into_value(self) -> value::Value<'a> {
        IntoSingleValue::<'a, Choices>::into_condition(self).into_sql()
    }
}

impl<'a, S: AsRef<[u8]> + ?Sized> IntoSingleValue<'a, VarBinary> for &'a S {
    type Condition = Value<'a>;
    fn into_condition(self) -> Self::Condition {
        Value::Binary(self.as_ref())
    }
    fn into_value(self) -> value::Value<'a> {
        self.into_condition().into_sql()
    }
}

impl<'a, F: Field, P: Path> IntoSingleValue<'a, F::DbType> for &'static FieldProxy<F, P> {
    type Condition = Column<F, P>;
    fn into_condition(self) -> Self::Condition {
        Column::new()
    }
    fn into_value(self) -> value::Value<'a> {
        value::Value::Ident(self.name())
    }
}

macro_rules! impl_numeric {
    ($type:ty, $value_variant:ident, $db_type:ident) => {
        impl<'a> IntoSingleValue<'a, $db_type> for $type {
            type Condition = Value<'a>;
            fn into_condition(self) -> Self::Condition {
                Value::$value_variant(self)
            }
            fn into_value(self) -> value::Value<'a> {
                self.into_condition().into_sql()
            }
        }
    };
}
impl_numeric!(i16, I16, Int16);
impl_numeric!(i32, I32, Int32);
impl_numeric!(i64, I64, Int64);
impl_numeric!(f32, F32, Float);
impl_numeric!(f64, F64, Double);
impl_numeric!(chrono::NaiveDate, NaiveDate, Date);
impl_numeric!(chrono::NaiveDateTime, NaiveDateTime, DateTime);
impl_numeric!(chrono::NaiveTime, NaiveTime, Time);

/// Implement the various condition methods on [FieldProxy]
mod impl_proxy {
    use super::*;

    // Helper methods hiding most of the verbosity in creating Conditions
    impl<F: Field, P: Path> FieldProxy<F, P> {
        fn __column(&self) -> Column<F, P> {
            Column::new()
        }

        /*
        fn __unary<'a>(
            &self,
            variant: impl Fn(Box<Condition<'a>>) -> UnaryCondition<'a>,
        ) -> Condition<'a> {
            Condition::UnaryCondition(variant(Box::new(self.__column())))
        }
        */

        fn __binary<'a, B: Condition<'a>>(
            &self,
            operator: BinaryOperator,
            snd_arg: B,
        ) -> Binary<Column<F, P>, B> {
            Binary {
                operator,
                fst_arg: self.__column(),
                snd_arg,
            }
        }

        fn __ternary<'a, B: Condition<'a>, C: Condition<'a>>(
            &self,
            operator: TernaryOperator,
            snd_arg: B,
            trd_arg: C,
        ) -> Ternary<Column<F, P>, B, C> {
            Ternary {
                operator,
                fst_arg: self.__column(),
                snd_arg,
                trd_arg,
            }
        }

        /// Check if this field's value lies between two other values
        pub fn between<
            'a,
            T1: IntoSingleValue<'a, F::DbType>,
            T2: IntoSingleValue<'a, F::DbType>,
        >(
            &self,
            lower: T1,
            upper: T2,
        ) -> Ternary<Column<F, P>, T1::Condition, T2::Condition> {
            self.__ternary(
                TernaryOperator::Between,
                lower.into_condition(),
                upper.into_condition(),
            )
        }

        /// Check if this field's value does not lie between two other values
        pub fn not_between<
            'a,
            T1: IntoSingleValue<'a, F::DbType>,
            T2: IntoSingleValue<'a, F::DbType>,
        >(
            &self,
            lower: T1,
            upper: T2,
        ) -> Ternary<Column<F, P>, T1::Condition, T2::Condition> {
            self.__ternary(
                TernaryOperator::NotBetween,
                lower.into_condition(),
                upper.into_condition(),
            )
        }

        /// Check if this field's value is equal to another value
        pub fn equals<'a, T: IntoSingleValue<'a, F::DbType>>(
            &self,
            arg: T,
        ) -> Binary<Column<F, P>, T::Condition> {
            self.__binary(BinaryOperator::Equals, arg.into_condition())
        }

        /// Check if this field's value is not equal to another value
        pub fn not_equals<'a, T: IntoSingleValue<'a, F::DbType>>(
            &self,
            arg: T,
        ) -> Binary<Column<F, P>, T::Condition> {
            self.__binary(BinaryOperator::NotEquals, arg.into_condition())
        }

        /// Check if this field's value is greater than another value
        pub fn greater<'a, T: IntoSingleValue<'a, F::DbType>>(
            &self,
            arg: T,
        ) -> Binary<Column<F, P>, T::Condition> {
            self.__binary(BinaryOperator::Greater, arg.into_condition())
        }

        /// Check if this field's value is greater than or equal to another value
        pub fn greater_or_equals<'a, T: IntoSingleValue<'a, F::DbType>>(
            &self,
            arg: T,
        ) -> Binary<Column<F, P>, T::Condition> {
            self.__binary(BinaryOperator::GreaterOrEquals, arg.into_condition())
        }

        /// Check if this field's value is less than another value
        pub fn less<'a, T: IntoSingleValue<'a, F::DbType>>(
            &self,
            arg: T,
        ) -> Binary<Column<F, P>, T::Condition> {
            self.__binary(BinaryOperator::Less, arg.into_condition())
        }

        /// Check if this field's value is less than or equal to another value
        pub fn less_or_equals<'a, T: IntoSingleValue<'a, F::DbType>>(
            &self,
            arg: T,
        ) -> Binary<Column<F, P>, T::Condition> {
            self.__binary(BinaryOperator::LessOrEquals, arg.into_condition())
        }

        /// Check if this field's value is similar to another value
        pub fn like<'a, T: IntoSingleValue<'a, F::DbType>>(
            &self,
            arg: T,
        ) -> Binary<Column<F, P>, T::Condition> {
            self.__binary(BinaryOperator::Like, arg.into_condition())
        }

        /// Check if this field's value is not similar to another value
        pub fn not_like<'a, T: IntoSingleValue<'a, F::DbType>>(
            &self,
            arg: T,
        ) -> Binary<Column<F, P>, T::Condition> {
            self.__binary(BinaryOperator::NotLike, arg.into_condition())
        }

        /// Check if this field's value is matched by a regex
        pub fn regexp<'a, T: IntoSingleValue<'a, F::DbType>>(
            &self,
            arg: T,
        ) -> Binary<Column<F, P>, T::Condition> {
            self.__binary(BinaryOperator::Regexp, arg.into_condition())
        }

        /// Check if this field's value is not matched by a regex
        pub fn not_regexp<'a, T: IntoSingleValue<'a, F::DbType>>(
            &self,
            arg: T,
        ) -> Binary<Column<F, P>, T::Condition> {
            self.__binary(BinaryOperator::NotRegexp, arg.into_condition())
        }

        // TODO in, not_in: requires different trait than IntoCondValue
        // TODO is_null, is_not_null: check AsDbType::NULLABLE in type constraint, new Nullable trait?
    }
}