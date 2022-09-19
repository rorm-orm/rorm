use crate::value::Value;

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

impl<'a> TernaryCondition<'a> {
    /**
    This method is used to convert the current enum to SQL.
    */
    pub fn build(&self, lookup: &mut Vec<Value<'a>>) -> String {
        match self {
            TernaryCondition::Between(params) => format!(
                "{} BETWEEN {} AND {}",
                params[0].build(lookup),
                params[1].build(lookup),
                params[2].build(lookup),
            ),
            TernaryCondition::NotBetween(params) => format!(
                "{} NOT BETWEEN {} AND {}",
                params[0].build(lookup),
                params[1].build(lookup),
                params[2].build(lookup),
            ),
        }
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

impl<'a> BinaryCondition<'a> {
    /**
    This method is used to convert the current enum to SQL.
    */
    pub fn build(&self, lookup: &mut Vec<Value<'a>>) -> String {
        match self {
            BinaryCondition::Equals(params) => {
                format!("{} = {}", params[0].build(lookup), params[1].build(lookup))
            }
            BinaryCondition::NotEquals(params) => {
                format!("{} <> {}", params[0].build(lookup), params[1].build(lookup))
            }
            BinaryCondition::Greater(params) => {
                format!("{} > {}", params[0].build(lookup), params[1].build(lookup))
            }
            BinaryCondition::GreaterOrEquals(params) => {
                format!("{} >= {}", params[0].build(lookup), params[1].build(lookup))
            }
            BinaryCondition::Less(params) => {
                format!("{} < {}", params[0].build(lookup), params[1].build(lookup))
            }
            BinaryCondition::LessOrEquals(params) => {
                format!("{} <= {}", params[0].build(lookup), params[1].build(lookup))
            }
            BinaryCondition::Like(params) => {
                format!(
                    "{} LIKE {}",
                    params[0].build(lookup),
                    params[1].build(lookup)
                )
            }
            BinaryCondition::NotLike(params) => {
                format!(
                    "{} NOT LIKE {}",
                    params[0].build(lookup),
                    params[1].build(lookup)
                )
            }
            BinaryCondition::Regexp(params) => {
                format!(
                    "{} REGEXP {}",
                    params[0].build(lookup),
                    params[1].build(lookup)
                )
            }
            BinaryCondition::NotRegexp(params) => {
                format!(
                    "{} NOT REGEXP {}",
                    params[0].build(lookup),
                    params[1].build(lookup)
                )
            }
            BinaryCondition::In(params) => {
                format!("{} IN {}", params[0].build(lookup), params[1].build(lookup))
            }
            BinaryCondition::NotIn(params) => {
                format!(
                    "{} NOT IN {}",
                    params[0].build(lookup),
                    params[1].build(lookup)
                )
            }
        }
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

impl<'a> UnaryCondition<'a> {
    /**
    This method is used to convert the [UnaryCondition] to SQL.
    */
    pub fn build(&self, lookup: &mut Vec<Value<'a>>) -> String {
        match self {
            UnaryCondition::IsNull(value) => format!("{} IS NULL", value.build(lookup)),
            UnaryCondition::IsNotNull(value) => format!("{} IS NOT NULL", value.build(lookup)),
            UnaryCondition::Exists(value) => format!("EXISTS {}", value.build(lookup)),
            UnaryCondition::NotExists(value) => format!("NOT EXISTS {}", value.build(lookup)),
            UnaryCondition::Not(value) => format!("NOT {}", value.build(lookup)),
        }
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

impl<'a> Condition<'a> {
    /**
    This method is used to convert the condition into SQL.
    */
    pub fn build(&self, lookup: &mut Vec<Value<'a>>) -> String {
        match self {
            Condition::Conjunction(conditions) => format!(
                "({})",
                conditions
                    .iter()
                    .map(|x| x.build(lookup))
                    .collect::<Vec<String>>()
                    .join(" AND ")
            ),
            Condition::Disjunction(conditions) => format!(
                "({})",
                conditions
                    .iter()
                    .map(|x| x.build(lookup))
                    .collect::<Vec<String>>()
                    .join(" OR ")
            ),
            Condition::UnaryCondition(unary) => unary.build(lookup),
            Condition::BinaryCondition(binary) => binary.build(lookup),
            Condition::TernaryCondition(ternary) => ternary.build(lookup),
            Condition::Value(expression) => match expression {
                Value::Ident(x) => x.to_string(),
                _ => {
                    lookup.push(*expression);
                    return "?".to_string();
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
    () => {
        Condition::Conjunction(vec![])
    };
    ($($cond:expr),+ $(,)?) => (
        Condition::Conjunction(vec![$($cond),+])
    );
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
    () => {
        Condition::Disjunction(vec![])
    };
    ($($cond:expr),+ $(,)?) => (
        Condition::Disjunction(vec![$($cond),+])
    );
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
