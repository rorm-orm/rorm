use std::fmt::{Debug, Error, Write};

#[cfg(feature = "postgres")]
use crate::db_specific::postgres;
#[cfg(feature = "sqlite")]
use crate::db_specific::sqlite;
use crate::value::{NullType, Value};
use crate::DBImpl;

/// An expression using a single value
#[derive(Debug, PartialEq, Clone)]
pub struct UnaryExpression<'a> {
    /// Operator applied to the value
    pub operator: UnaryOperator,

    /// Value the operator operates on
    pub value: Box<Condition<'a>>,
}

/// An expression using two values
#[derive(Debug, PartialEq, Clone)]
pub struct BinaryExpression<'a> {
    /// Operator applied to the values
    pub operator: BinaryOperator,

    /// Values the operator operates on
    pub values: Box<[Condition<'a>; 2]>,
}

/// An expression using three values
#[derive(Debug, PartialEq, Clone)]
pub struct TernaryExpression<'a> {
    /// Operator applied to the values
    pub operator: TernaryOperator,

    /// Values the operator operates on
    pub values: Box<[Condition<'a>; 3]>,
}

/// Operator of an [`UnaryExpression`]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum UnaryOperator {
    /// `{} IS NULL"
    IsNull,
    /// `{} IS NOT NULL"
    IsNotNull,
    /// "EXISTS {}`
    Exists,
    /// "NOT EXISTS {}`
    NotExists,
    /// "NOT {}`
    Not,
}

/// Operator of an [`BinaryExpression`]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum BinaryOperator {
    /// `{} = {}`
    Equals,
    /// `{} <> {}`
    NotEquals,
    /// `{} > {}`
    Greater,
    /// `{} >= {}`
    GreaterOrEquals,
    /// `{} < {}`
    Less,
    /// `{} <= {}`
    LessOrEquals,
    /// `{} LIKE {}`
    Like,
    /// `{} NOT LIKE {}`
    NotLike,
    /// `{} REGEXP {}`
    Regexp,
    /// `{} NOT REGEXP {}`
    NotRegexp,
    /// `{} IN {}`
    In,
    /// `{} NOT IN {}`
    NotIn,
    /// `{} ILIKE {}` (postgres feature)
    #[cfg(feature = "postgres-only")]
    ILike,
    /// `{} NOT ILIKE {}` (postgres feature)
    #[cfg(feature = "postgres-only")]
    NotILike,
    /// `{} << {}` for `inet` (postgres feature)
    #[cfg(feature = "postgres-only")]
    Contained,
    /// `{} <<= {}` for `inet` (postgres feature)
    #[cfg(feature = "postgres-only")]
    ContainedOrEquals,
    /// `{} >> {}` for `inet` (postgres feature)
    #[cfg(feature = "postgres-only")]
    Contains,
    /// `{} >>= {}` for `inet` (postgres feature)
    #[cfg(feature = "postgres-only")]
    ContainsOrEquals,
}

/// Operator of an [`TernaryExpression`]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum TernaryOperator {
    /// `{} BETWEEN {} AND {}`
    Between,
    /// `{} NOT BETWEEN {} AND {}`
    NotBetween,
}

/**
Trait implementing constructing sql queries from a condition tree.

This trait auto implements `build` which has a simpler api from the more complex `build_to_writer`.
 */
pub trait BuildCondition<'a>: 'a {
    /**
    This method is used to convert a condition to SQL.
     */
    fn build(&self, dialect: DBImpl, lookup: &mut Vec<Value<'a>>) -> String {
        let mut string = String::new();
        self.build_to_writer(&mut string, dialect, lookup)
            .expect("Writing to a string shouldn't fail");
        string
    }

    /**
    This method is used to convert a condition to SQL without allocating a dedicated string.
     */
    fn build_to_writer(
        &self,
        writer: &mut impl Write,
        dialect: DBImpl,
        lookup: &mut Vec<Value<'a>>,
    ) -> Result<(), Error>;
}

impl<'a> BuildCondition<'a> for TernaryExpression<'a> {
    fn build_to_writer(
        &self,
        writer: &mut impl Write,
        dialect: DBImpl,
        lookup: &mut Vec<Value<'a>>,
    ) -> Result<(), Error> {
        let [lhs, mhs, rhs] = &*self.values;
        let keyword = match self.operator {
            TernaryOperator::Between => "BETWEEN",
            TernaryOperator::NotBetween => "NOT BETWEEN",
        };
        write!(writer, "(")?;
        lhs.build_to_writer(writer, dialect, lookup)?;
        write!(writer, " {keyword} ")?;
        mhs.build_to_writer(writer, dialect, lookup)?;
        write!(writer, " AND ")?;
        rhs.build_to_writer(writer, dialect, lookup)?;
        write!(writer, ")")?;
        Ok(())
    }
}

impl<'a> BuildCondition<'a> for BinaryExpression<'a> {
    fn build_to_writer(
        &self,
        writer: &mut impl Write,
        dialect: DBImpl,
        lookup: &mut Vec<Value<'a>>,
    ) -> Result<(), Error> {
        let [lhs, rhs] = &*self.values;
        let keyword = match self.operator {
            BinaryOperator::Equals => "=",
            BinaryOperator::NotEquals => "<>",
            BinaryOperator::Greater => ">",
            BinaryOperator::GreaterOrEquals => ">=",
            BinaryOperator::Less => "<",
            BinaryOperator::LessOrEquals => "<=",
            BinaryOperator::Like => "LIKE",
            BinaryOperator::NotLike => "NOT LIKE",
            BinaryOperator::Regexp => "REGEXP",
            BinaryOperator::NotRegexp => "NOT REGEXP",
            BinaryOperator::In => "IN",
            BinaryOperator::NotIn => "NOT IN",
            #[cfg(feature = "postgres-only")]
            BinaryOperator::ILike => "ILIKE",
            #[cfg(feature = "postgres-only")]
            BinaryOperator::NotILike => "NOT ILIKE",
            #[cfg(feature = "postgres-only")]
            BinaryOperator::Contained => "<<",
            #[cfg(feature = "postgres-only")]
            BinaryOperator::ContainedOrEquals => "<<=",
            #[cfg(feature = "postgres-only")]
            BinaryOperator::Contains => ">>",
            #[cfg(feature = "postgres-only")]
            BinaryOperator::ContainsOrEquals => ">>=",
        };
        write!(writer, "(")?;
        lhs.build_to_writer(writer, dialect, lookup)?;
        write!(writer, " {keyword} ")?;
        rhs.build_to_writer(writer, dialect, lookup)?;
        #[cfg(feature = "sqlite")]
        if matches!(dialect, DBImpl::SQLite) && matches!(keyword, "LIKE" | "NOT LIKE") {
            // Sqlite does not default it
            write!(writer, " ESCAPE '\'")?;
        }
        write!(writer, ")")?;
        Ok(())
    }
}

impl<'a> BuildCondition<'a> for UnaryExpression<'a> {
    fn build_to_writer(
        &self,
        writer: &mut impl Write,
        dialect: DBImpl,
        lookup: &mut Vec<Value<'a>>,
    ) -> Result<(), Error> {
        let (postfix, keyword) = match self.operator {
            UnaryOperator::IsNull => (true, "IS NULL"),
            UnaryOperator::IsNotNull => (true, "IS NOT NULL"),
            UnaryOperator::Exists => (false, "EXISTS"),
            UnaryOperator::NotExists => (false, "NOT EXISTS"),
            UnaryOperator::Not => (false, "NOT"),
        };
        write!(writer, "(")?;
        if postfix {
            self.value.build_to_writer(writer, dialect, lookup)?;
            write!(writer, " {keyword}")?;
        } else {
            write!(writer, "{keyword} ")?;
            self.value.build_to_writer(writer, dialect, lookup)?;
        }
        write!(writer, ")")?;
        Ok(())
    }
}

/**
This enum represents a condition tree.
*/
#[derive(Debug, PartialEq, Clone)]
pub enum Condition<'a> {
    /// A list of [Condition]s, that get expanded to "{} AND {} ..."
    Conjunction(Vec<Condition<'a>>),
    /// A list of [Condition]s, that get expanded to "{} OR {} ..."
    Disjunction(Vec<Condition<'a>>),
    /// Representation of an unary condition.
    UnaryCondition(UnaryExpression<'a>),
    /// Representation of a binary condition.
    BinaryCondition(BinaryExpression<'a>),
    /// Representation of a ternary condition.
    TernaryCondition(TernaryExpression<'a>),
    /// Representation of a value.
    Value(Value<'a>),
}

impl<'a> BuildCondition<'a> for Condition<'a> {
    fn build_to_writer(
        &self,
        writer: &mut impl Write,
        dialect: DBImpl,
        lookup: &mut Vec<Value<'a>>,
    ) -> Result<(), Error> {
        match self {
            Condition::Conjunction(conditions) | Condition::Disjunction(conditions) => {
                let keyword = match self {
                    Condition::Conjunction(_) => "AND ",
                    Condition::Disjunction(_) => "OR ",
                    _ => unreachable!("All other possibilities would pass the outer match arm"),
                };
                write!(writer, "(")?;
                if let Some(first) = conditions.first() {
                    first.build_to_writer(writer, dialect, lookup)?;
                    conditions.iter().enumerate().try_for_each(|(idx, cond)| {
                        if idx > 0 {
                            write!(writer, " {keyword}")?;
                            cond.build_to_writer(writer, dialect, lookup)?;
                        }
                        Ok(())
                    })?;
                }
                write!(writer, ")")?;
                Ok(())
            }
            Condition::UnaryCondition(unary) => unary.build_to_writer(writer, dialect, lookup),
            Condition::BinaryCondition(binary) => binary.build_to_writer(writer, dialect, lookup),
            Condition::TernaryCondition(ternary) => {
                ternary.build_to_writer(writer, dialect, lookup)
            }
            Condition::Value(value) => match value {
                #[allow(deprecated)]
                Value::Ident(string) => write!(writer, "{string}"),
                Value::Column {
                    table_name,
                    column_name,
                } => match dialect {
                    #[cfg(feature = "sqlite")]
                    DBImpl::SQLite => {
                        if let Some(table_name) = table_name {
                            write!(writer, "\"{table_name}\".")?;
                        }
                        write!(writer, "{column_name}")
                    }
                    #[cfg(feature = "postgres")]
                    DBImpl::Postgres => {
                        if let Some(table_name) = table_name {
                            write!(writer, "\"{table_name}\".")?;
                        }
                        write!(writer, "{column_name}")
                    }
                },
                Value::Choice(c) => match dialect {
                    #[cfg(feature = "sqlite")]
                    DBImpl::SQLite => write!(writer, "{}", sqlite::fmt(c)),
                    #[cfg(feature = "postgres")]
                    DBImpl::Postgres => write!(writer, "{}", postgres::fmt(c)),
                },
                Value::Null(NullType::Choice) => write!(writer, "NULL"),

                _ => {
                    lookup.push(*value);
                    match dialect {
                        #[cfg(feature = "sqlite")]
                        DBImpl::SQLite => {
                            write!(writer, "?")
                        }
                        #[cfg(feature = "postgres")]
                        DBImpl::Postgres => {
                            write!(writer, "${}", lookup.len())
                        }
                    }
                }
            },
        }
    }
}

/**
This macro is used to simplify the creation of conjunctive [Condition]s.
It takes a variadic amount of conditions and places them in a [Condition::Conjunction].

It does **not** try to simplify any conditions where one or no conditions are passed,
so no one gets confused. This also ensures, that the return type of this macro
is always [Condition::Conjunction].

**Usage**:

```
use rorm_sql::and;
use rorm_sql::conditional::Condition;
use rorm_sql::conditional::BinaryCondition;
use rorm_sql::value::Value;

let condition = and!(
    Condition::BinaryCondition(
        BinaryCondition::Equals(Box::new([
            Condition::Value(Value::Ident("id")),
            Condition::Value(Value::I64(23)),
        ]))
    ),
    Condition::BinaryCondition(
        BinaryCondition::Like(Box::new([
            Condition::Value(Value::Ident("foo")),
            Condition::Value(Value::String("%bar")),
        ]))
    ),
);
```
*/
#[macro_export]
macro_rules! and {
    () => {{
        $crate::conditional::Condition::Conjunction(vec![])
    }};
    ($($cond:expr),+ $(,)?) => {{
        $crate::conditional::Condition::Conjunction(vec![$($cond),+])
    }};
}

/**
This macro is used to simplify the creation of disjunctive [Condition]s.
It takes a variadic amount of conditions and places them in a [Condition::Disjunction].

It does **not** try to simplify any conditions where one or no conditions are passed,
so no one gets confused. This also ensures, that the return type of this macro
is always [Condition::Disjunction].

**Usage**:

```
use rorm_sql::or;
use rorm_sql::conditional::Condition;
use rorm_sql::conditional::BinaryCondition;
use rorm_sql::value::Value;

let condition = or!(
    Condition::BinaryCondition(
        BinaryCondition::Equals(Box::new([
            Condition::Value(Value::Ident("id")),
            Condition::Value(Value::I64(23)),
        ]))
    ),
    Condition::BinaryCondition(
        BinaryCondition::Like(Box::new([
            Condition::Value(Value::Ident("foo")),
            Condition::Value(Value::String("%bar")),
        ]))
    ),
);
```
 */
#[macro_export]
macro_rules! or {
    () => {{
        $crate::conditional::Condition::Disjunction(vec![])
    }};
    ($($cond:expr),+ $(,)?) => {{
        $crate::conditional::Condition::Disjunction(vec![$($cond),+])
    }};
}

#[cfg(test)]
mod test {
    use crate::conditional::Condition;
    use crate::value::Value;

    #[test]
    fn empty_and() {
        assert_eq!(and!(), Condition::Conjunction(vec![]))
    }

    #[test]
    fn empty_or() {
        assert_eq!(or!(), Condition::Disjunction(vec![]))
    }

    #[test]
    fn and_01() {
        assert_eq!(
            and!(Condition::Value(Value::String("foo"))),
            Condition::Conjunction(vec![Condition::Value(Value::String("foo"))])
        );
    }
    #[test]
    fn and_02() {
        assert_eq!(
            and!(
                Condition::Value(Value::String("foo")),
                Condition::Value(Value::String("foo"))
            ),
            Condition::Conjunction(vec![
                Condition::Value(Value::String("foo")),
                Condition::Value(Value::String("foo"))
            ])
        );
    }

    #[test]
    fn or_01() {
        assert_eq!(
            or!(Condition::Value(Value::String("foo"))),
            Condition::Disjunction(vec![Condition::Value(Value::String("foo"))])
        );
    }
    #[test]
    fn or_02() {
        assert_eq!(
            or!(
                Condition::Value(Value::String("foo")),
                Condition::Value(Value::String("foo"))
            ),
            Condition::Disjunction(vec![
                Condition::Value(Value::String("foo")),
                Condition::Value(Value::String("foo"))
            ])
        );
    }
}
