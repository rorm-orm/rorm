use crate::DBImpl;
use std::fmt::{Debug, Error, Write};

use crate::value::Value;

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

/**
This enum represents all available ternary expression.
*/
#[derive(Debug, PartialEq)]
pub enum TernaryCondition<'a> {
    /// Between represents "{} BETWEEN {} AND {}" from SQL
    Between(Box<[Condition<'a>; 3]>),
    /// Between represents "{} NOT BETWEEN {} AND {}" from SQL
    NotBetween(Box<[Condition<'a>; 3]>),
}

impl<'a> BuildCondition<'a> for TernaryCondition<'a> {
    fn build_to_writer(
        &self,
        writer: &mut impl Write,
        dialect: DBImpl,
        lookup: &mut Vec<Value<'a>>,
    ) -> Result<(), Error> {
        let (keyword, [lhs, mhs, rhs]) = match self {
            TernaryCondition::Between(params) => ("BETWEEN", params.as_ref()),
            TernaryCondition::NotBetween(params) => ("NOT BETWEEN", params.as_ref()),
        };
        write!(writer, "(")?;
        lhs.build_to_writer(writer, dialect, lookup)?;
        write!(writer, " {} ", keyword)?;
        mhs.build_to_writer(writer, dialect, lookup)?;
        write!(writer, " AND ")?;
        rhs.build_to_writer(writer, dialect, lookup)?;
        write!(writer, ")")?;
        Ok(())
    }
}

/**
This enum represents a binary expression.
*/
#[derive(Debug, PartialEq)]
pub enum BinaryCondition<'a> {
    /// Representation of "{} = {}" in SQL
    Equals(Box<[Condition<'a>; 2]>),
    /// Representation of "{} <> {}" in SQL
    NotEquals(Box<[Condition<'a>; 2]>),
    /// Representation of "{} > {}" in SQL
    Greater(Box<[Condition<'a>; 2]>),
    /// Representation of "{} >= {}" in SQL
    GreaterOrEquals(Box<[Condition<'a>; 2]>),
    /// Representation of "{} < {}" in SQL
    Less(Box<[Condition<'a>; 2]>),
    /// Representation of "{} <= {}" in SQL
    LessOrEquals(Box<[Condition<'a>; 2]>),
    /// Representation of "{} LIKE {}" in SQL
    Like(Box<[Condition<'a>; 2]>),
    /// Representation of "{} NOT LIKE {}" in SQL
    NotLike(Box<[Condition<'a>; 2]>),
    /// Representation of "{} REGEXP {}" in SQL
    Regexp(Box<[Condition<'a>; 2]>),
    /// Representation of "{} NOT REGEXP {}" in SQL
    NotRegexp(Box<[Condition<'a>; 2]>),
    /// Representation of "{} IN {}" in SQL
    In(Box<[Condition<'a>; 2]>),
    /// Representation of "{} NOT IN {}" in SQL
    NotIn(Box<[Condition<'a>; 2]>),
}

impl<'a> BuildCondition<'a> for BinaryCondition<'a> {
    fn build_to_writer(
        &self,
        writer: &mut impl Write,
        dialect: DBImpl,
        lookup: &mut Vec<Value<'a>>,
    ) -> Result<(), Error> {
        let (keyword, [lhs, rhs]) = match self {
            BinaryCondition::Equals(params) => ("=", params.as_ref()),
            BinaryCondition::NotEquals(params) => ("<>", params.as_ref()),
            BinaryCondition::Greater(params) => (">", params.as_ref()),
            BinaryCondition::GreaterOrEquals(params) => (">=", params.as_ref()),
            BinaryCondition::Less(params) => ("<", params.as_ref()),
            BinaryCondition::LessOrEquals(params) => ("<=", params.as_ref()),
            BinaryCondition::Like(params) => ("LIKE", params.as_ref()),
            BinaryCondition::NotLike(params) => ("NOT LIKE", params.as_ref()),
            BinaryCondition::Regexp(params) => ("REGEXP", params.as_ref()),
            BinaryCondition::NotRegexp(params) => ("NOT REGEXP", params.as_ref()),
            BinaryCondition::In(params) => ("IN", params.as_ref()),
            BinaryCondition::NotIn(params) => ("NOT IN", params.as_ref()),
        };
        write!(writer, "(")?;
        lhs.build_to_writer(writer, dialect, lookup)?;
        write!(writer, " {} ", keyword)?;
        rhs.build_to_writer(writer, dialect, lookup)?;
        write!(writer, ")")?;
        Ok(())
    }
}

/**
This enum represents all available unary conditions.
*/
#[derive(Debug, PartialEq)]
pub enum UnaryCondition<'a> {
    /// Representation of SQL's "{} IS NULL"
    IsNull(Box<Condition<'a>>),
    /// Representation of SQL's "{} IS NOT NULL"
    IsNotNull(Box<Condition<'a>>),
    /// Representation of SQL's "EXISTS {}"
    Exists(Box<Condition<'a>>),
    /// Representation of SQL's "NOT EXISTS {}"
    NotExists(Box<Condition<'a>>),
    /// Representation of SQL's "NOT {}"
    Not(Box<Condition<'a>>),
}

impl<'a> BuildCondition<'a> for UnaryCondition<'a> {
    fn build_to_writer(
        &self,
        writer: &mut impl Write,
        dialect: DBImpl,
        lookup: &mut Vec<Value<'a>>,
    ) -> Result<(), Error> {
        let (postfix, keyword, value) = match self {
            UnaryCondition::IsNull(value) => (true, "IS NULL", value.as_ref()),
            UnaryCondition::IsNotNull(value) => (true, "IS NOT NULL", value.as_ref()),
            UnaryCondition::Exists(value) => (false, "EXISTS", value.as_ref()),
            UnaryCondition::NotExists(value) => (false, "NOT EXISTS", value.as_ref()),
            UnaryCondition::Not(value) => (false, "NOT", value.as_ref()),
        };
        write!(writer, "(")?;
        if postfix {
            value.build_to_writer(writer, dialect, lookup)?;
            write!(writer, " {}", keyword)?;
        } else {
            write!(writer, "{} ", keyword)?;
            value.build_to_writer(writer, dialect, lookup)?;
        }
        write!(writer, ")")?;
        Ok(())
    }
}

/**
This enum represents a condition tree.
*/
#[derive(Debug, PartialEq)]
pub enum Condition<'a> {
    /// A list of [Condition]s, that get expanded to "{} AND {} ..."
    Conjunction(Vec<Condition<'a>>),
    /// A list of [Condition]s, that get expanded to "{} OR {} ..."
    Disjunction(Vec<Condition<'a>>),
    /// Representation of an unary condition.
    UnaryCondition(UnaryCondition<'a>),
    /// Representation of a binary condition.
    BinaryCondition(BinaryCondition<'a>),
    /// Representation of a ternary condition.
    TernaryCondition(TernaryCondition<'a>),
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
                    Condition::Conjunction(_) => "AND",
                    Condition::Disjunction(_) => "OR",
                    _ => unreachable!("All other possibilities would pass the outer match arm"),
                };
                write!(writer, "(")?;
                if let Some(first) = conditions.first() {
                    first.build_to_writer(writer, dialect, lookup)?;
                    conditions.iter().enumerate().try_for_each(|(idx, cond)| {
                        if idx > 0 {
                            write!(writer, " {}", keyword)?;
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
                Value::Ident(string) => write!(writer, "{}", string),
                _ => {
                    lookup.push(*value);
                    match dialect {
                        #[cfg(feature = "sqlite")]
                        DBImpl::SQLite => {
                            write!(writer, "?")
                        }
                        #[cfg(feature = "mysql")]
                        DBImpl::MySQL => {
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
