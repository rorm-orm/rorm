use crate::value::Value;

/**
This enum represents all available ternary expression.
*/
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
pub enum UnaryCondition<'a> {
    /// Representation of SQL's "{} IS NULL"
    IsNull(Box<Condition<'a>>),
    /// Representation of SQL's "{} IS NOT NULL"
    IsNotNull(Box<Condition<'a>>),
    /// Representation of SQL's "EXISTS {}"
    Exists(Box<Condition<'a>>),
    /// Representation of SQL's "NOT EXISTS {}"
    NotExists(Box<Condition<'a>>),
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
        }
    }
}

/**
This enum represents a condition tree.
*/
pub enum Condition<'a> {
    /// A list of [Condition]s, that get expanded to "{} AND {} ..."
    Conjunction(Box<[Condition<'a>]>),
    /// A list of [Condition]s, that get expanded to "{} OR {} ..."
    Disjunction(Box<[Condition<'a>]>),
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
