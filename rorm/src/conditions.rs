use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use rorm_db::{conditional, value};

/// Basic trait to convert things to rorm-sql types
pub trait AsSql<'a>: 'a {
    /// Type to convert to
    type SQL;

    /// Convert to rorm-sql type
    fn as_sql(&self) -> Self::SQL;
}

mod impl_as_sql {
    use rorm_db::value;

    use super::AsSql;
    use crate::internal::field::{Field, FieldProxy};

    impl<'a> AsSql<'a> for &'a str {
        type SQL = value::Value<'a>;
        fn as_sql(&self) -> Self::SQL {
            value::Value::String(self)
        }
    }

    impl<'a> AsSql<'a> for &'a &[u8] {
        type SQL = value::Value<'a>;
        fn as_sql(&self) -> Self::SQL {
            value::Value::Binary(self)
        }
    }

    impl<'a, F: Field> AsSql<'a> for &'static FieldProxy<F, ()> {
        type SQL = value::Value<'a>;
        fn as_sql(&self) -> Self::SQL {
            value::Value::Ident(self.name())
        }
    }

    macro_rules! impl_numeric {
        ($type:ty, $value_variant:ident) => {
            impl<'a> AsSql<'a> for $type {
                type SQL = value::Value<'a>;
                fn as_sql(&self) -> Self::SQL {
                    value::Value::$value_variant(*self)
                }
            }
        };
    }

    impl_numeric!(i16, I16);
    impl_numeric!(i32, I32);
    impl_numeric!(i64, I64);
    impl_numeric!(f32, F32);
    impl_numeric!(f64, F64);
    impl_numeric!(chrono::NaiveDate, NaiveDate);
    impl_numeric!(chrono::NaiveDateTime, NaiveDateTime);
    impl_numeric!(chrono::NaiveTime, NaiveTime);
}

/// Node in a condition tree
pub trait Condition<'a>: 'a {
    /// Convert the condition into rorm-sql's format
    fn as_sql(&self) -> conditional::Condition<'a>;

    /// Convert the condition into a boxed trait object to erase its concrete type
    fn boxed(self) -> Box<dyn Condition<'a>>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}
impl<'a, T: AsSql<'a, SQL = value::Value<'a>>> Condition<'a> for T {
    fn as_sql(&self) -> conditional::Condition<'a> {
        conditional::Condition::Value(AsSql::as_sql(self))
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
impl<'a> AsSql<'a> for Value<'a> {
    type SQL = value::Value<'a>;

    fn as_sql(&self) -> Self::SQL {
        match *self {
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

/// A column name
#[derive(Copy, Clone)]
pub struct Column<'a> {
    pub(crate) name: &'a str,
}
impl<'a> AsSql<'a> for Column<'a> {
    type SQL = value::Value<'a>;

    fn as_sql(&self) -> Self::SQL {
        value::Value::Ident(self.name)
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
    fn as_sql(&self) -> conditional::Condition<'a> {
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
            self.fst_arg.as_sql(),
            self.snd_arg.as_sql(),
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
    fn as_sql(&self) -> conditional::Condition<'a> {
        conditional::Condition::TernaryCondition((match self.operator {
            TernaryOperator::Between => conditional::TernaryCondition::Between,
            TernaryOperator::NotBetween => conditional::TernaryCondition::NotBetween,
        })(Box::new([
            self.fst_arg.as_sql(),
            self.snd_arg.as_sql(),
            self.trd_arg.as_sql(),
        ])))
    }
}

/// A list of conditions joined by AND or OR
#[derive(Clone)]
pub struct Collection<A> {
    pub(crate) operator: CollectionOperator,
    pub(crate) args: Vec<A>,
}
/// Operator to join a [Collection] with
#[derive(Copy, Clone)]
pub enum CollectionOperator {
    /// Join the list with AND
    And,
    /// Join the list with OR
    Or,
}
impl<'a, A: Condition<'a>> Condition<'a> for Collection<A> {
    fn as_sql(&self) -> conditional::Condition<'a> {
        (match self.operator {
            CollectionOperator::And => conditional::Condition::Conjunction,
            CollectionOperator::Or => conditional::Condition::Disjunction,
        })(
            self.args
                .iter()
                .map(|condition| condition.as_sql())
                .collect(),
        )
    }
}

/// Implement the various condition methods on [FieldProxy]
mod impl_proxy {
    use rorm_declaration::hmr::db_type::{
        Date, DateTime, DbType, Double, Float, Int16, Int32, Int64, Time, VarBinary, VarChar,
    };

    use super::*;
    use crate::internal::field::{Field, FieldProxy};

    pub trait IntoAsValue<'a, D: DbType>: 'a {
        type AsValue: AsSql<'a, SQL = value::Value<'a>>;

        fn into_value(self) -> Self::AsValue;
    }

    impl<'a, S: AsRef<str> + ?Sized> IntoAsValue<'a, VarChar> for &'a S {
        type AsValue = Value<'a>;
        fn into_value(self) -> Self::AsValue {
            Value::String(self.as_ref())
        }
    }

    impl<'a, S: AsRef<[u8]> + ?Sized> IntoAsValue<'a, VarBinary> for &'a S {
        type AsValue = Value<'a>;
        fn into_value(self) -> Self::AsValue {
            Value::Binary(self.as_ref())
        }
    }

    impl<'a, F: Field> IntoAsValue<'a, F::DbType> for &'static FieldProxy<F, ()> {
        type AsValue = Column<'a>;
        fn into_value(self) -> Self::AsValue {
            Column { name: self.name() }
        }
    }

    macro_rules! impl_numeric {
        ($type:ty, $value_variant:ident, $db_type:ident) => {
            impl<'a> IntoAsValue<'a, $db_type> for $type {
                type AsValue = Value<'a>;
                fn into_value(self) -> Self::AsValue {
                    Value::$value_variant(self)
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

    // Helper methods hiding most of the verbosity in creating Conditions
    impl<F: Field> FieldProxy<F, ()> {
        fn __column(&self) -> Column<'static> {
            Column { name: self.name() }
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
        ) -> Binary<Column<'a>, B> {
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
        ) -> Ternary<Column<'a>, B, C> {
            Ternary {
                operator,
                fst_arg: self.__column(),
                snd_arg,
                trd_arg,
            }
        }
    }

    impl<F: Field> FieldProxy<F, ()> {
        /// Check if this field's value lies between two other values
        pub fn between<'a, T1: IntoAsValue<'a, F::DbType>, T2: IntoAsValue<'a, F::DbType>>(
            &self,
            lower: T1,
            upper: T2,
        ) -> Ternary<Column<'a>, T1::AsValue, T2::AsValue> {
            self.__ternary(
                TernaryOperator::Between,
                lower.into_value(),
                upper.into_value(),
            )
        }

        /// Check if this field's value does not lie between two other values
        pub fn not_between<'a, T1: IntoAsValue<'a, F::DbType>, T2: IntoAsValue<'a, F::DbType>>(
            &self,
            lower: T1,
            upper: T2,
        ) -> Ternary<Column<'a>, T1::AsValue, T2::AsValue> {
            self.__ternary(
                TernaryOperator::NotBetween,
                lower.into_value(),
                upper.into_value(),
            )
        }

        /// Check if this field's value is equal to another value
        pub fn equals<'a, T: IntoAsValue<'a, F::DbType>>(
            &self,
            arg: T,
        ) -> Binary<Column<'a>, T::AsValue> {
            self.__binary(BinaryOperator::Equals, arg.into_value())
        }

        /// Check if this field's value is not equal to another value
        pub fn not_equals<'a, T: IntoAsValue<'a, F::DbType>>(
            &self,
            arg: T,
        ) -> Binary<Column<'a>, T::AsValue> {
            self.__binary(BinaryOperator::NotEquals, arg.into_value())
        }

        /// Check if this field's value is greater than another value
        pub fn greater<'a, T: IntoAsValue<'a, F::DbType>>(
            &self,
            arg: T,
        ) -> Binary<Column<'a>, T::AsValue> {
            self.__binary(BinaryOperator::Greater, arg.into_value())
        }

        /// Check if this field's value is greater than or equal to another value
        pub fn greater_or_equals<'a, T: IntoAsValue<'a, F::DbType>>(
            &self,
            arg: T,
        ) -> Binary<Column<'a>, T::AsValue> {
            self.__binary(BinaryOperator::GreaterOrEquals, arg.into_value())
        }

        /// Check if this field's value is less than another value
        pub fn less<'a, T: IntoAsValue<'a, F::DbType>>(
            &self,
            arg: T,
        ) -> Binary<Column<'a>, T::AsValue> {
            self.__binary(BinaryOperator::Less, arg.into_value())
        }

        /// Check if this field's value is less than or equal to another value
        pub fn less_or_equals<'a, T: IntoAsValue<'a, F::DbType>>(
            &self,
            arg: T,
        ) -> Binary<Column<'a>, T::AsValue> {
            self.__binary(BinaryOperator::LessOrEquals, arg.into_value())
        }

        /// Check if this field's value is similar to another value
        pub fn like<'a, T: IntoAsValue<'a, F::DbType>>(
            &self,
            arg: T,
        ) -> Binary<Column<'a>, T::AsValue> {
            self.__binary(BinaryOperator::Like, arg.into_value())
        }

        /// Check if this field's value is not similar to another value
        pub fn not_like<'a, T: IntoAsValue<'a, F::DbType>>(
            &self,
            arg: T,
        ) -> Binary<Column<'a>, T::AsValue> {
            self.__binary(BinaryOperator::NotLike, arg.into_value())
        }

        /// Check if this field's value is matched by a regex
        pub fn regexp<'a, T: IntoAsValue<'a, F::DbType>>(
            &self,
            arg: T,
        ) -> Binary<Column<'a>, T::AsValue> {
            self.__binary(BinaryOperator::Regexp, arg.into_value())
        }

        /// Check if this field's value is not matched by a regex
        pub fn not_regexp<'a, T: IntoAsValue<'a, F::DbType>>(
            &self,
            arg: T,
        ) -> Binary<Column<'a>, T::AsValue> {
            self.__binary(BinaryOperator::NotRegexp, arg.into_value())
        }

        // TODO in, not_in: requires different trait than IntoCondValue
        // TODO is_null, is_not_null: check AsDbType::NULLABLE in type constraint, new Nullable trait?
    }
}
