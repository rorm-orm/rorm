/**
This enum represents a value
*/
pub enum ConditionValue<'a> {
    /// String representation
    String(&'a str),
    /// i64 representation
    I64(i64),
    /// i32 representation
    I32(i32),
    /// i16 representation
    I16(i16),
    /// i8 representation
    I8(i8),
}

/**
This enum represents all available ternary expression.
*/
pub enum TernaryCondition<'a> {
    /// Between represents "{} BETWEEN {} AND {}" from SQL
    Between(Box<[Condition<'a>; 3]>),
    /// Between represents "{} NOT BETWEEN {} AND {}" from SQL
    NotBetween(Box<[Condition<'a>; 3]>),
}

impl TernaryCondition<'_> {
    /**
    This method is used to convert the current enum to SQL.
    */
    pub fn build(&self) -> String {
        match self {
            TernaryCondition::Between(params) => format!(
                "{} BETWEEN {} AND {}",
                params[0].build(),
                params[1].build(),
                params[2].build(),
            ),
            TernaryCondition::NotBetween(params) => format!(
                "{} NOT BETWEEN {} AND {}",
                params[0].build(),
                params[1].build(),
                params[2].build(),
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

impl BinaryCondition<'_> {
    /**
    This method is used to convert the current enum to SQL.
    */
    pub fn build(&self) -> String {
        match self {
            BinaryCondition::Equals(params) => {
                format!("{} = {}", params[0].build(), params[1].build())
            }
            BinaryCondition::NotEquals(params) => {
                format!("{} <> {}", params[0].build(), params[1].build())
            }
            BinaryCondition::Greater(params) => {
                format!("{} > {}", params[0].build(), params[1].build())
            }
            BinaryCondition::GreaterOrEquals(params) => {
                format!("{} >= {}", params[0].build(), params[1].build())
            }
            BinaryCondition::Less(params) => {
                format!("{} < {}", params[0].build(), params[1].build())
            }
            BinaryCondition::LessOrEquals(params) => {
                format!("{} <= {}", params[0].build(), params[1].build())
            }
            BinaryCondition::Like(params) => {
                format!("{} LIKE {}", params[0].build(), params[1].build())
            }
            BinaryCondition::NotLike(params) => {
                format!("{} NOT LIKE {}", params[0].build(), params[1].build())
            }
            BinaryCondition::Regexp(params) => {
                format!("{} REGEXP {}", params[0].build(), params[1].build())
            }
            BinaryCondition::NotRegexp(params) => {
                format!("{} NOT REGEXP {}", params[0].build(), params[1].build())
            }
            BinaryCondition::In(params) => {
                format!("{} IN {}", params[0].build(), params[1].build())
            }
            BinaryCondition::NotIn(params) => {
                format!("{} NOT IN {}", params[0].build(), params[1].build())
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

impl UnaryCondition<'_> {
    /**
    This method is used to convert the [UnaryCondition] to SQL.
    */
    pub fn build(&self) -> String {
        match self {
            UnaryCondition::IsNull(value) => format!("{} IS NULL", value.build()),
            UnaryCondition::IsNotNull(value) => format!("{} IS NOT NULL", value.build()),
            UnaryCondition::Exists(value) => format!("EXISTS {}", value.build()),
            UnaryCondition::NotExists(value) => format!("NOT EXISTS {}", value.build()),
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
    Value(ConditionValue<'a>),
}

impl Condition<'_> {
    /**
    This method is used to convert the condition into SQL.
    */
    pub fn build(&self) -> String {
        match self {
            Condition::Conjunction(conditions) => format!(
                "({})",
                conditions
                    .iter()
                    .map(|x| x.build())
                    .collect::<Vec<String>>()
                    .join(" AND ")
            ),
            Condition::Disjunction(conditions) => format!(
                "({})",
                conditions
                    .iter()
                    .map(|x| x.build())
                    .collect::<Vec<String>>()
                    .join(" OR ")
            ),
            Condition::UnaryCondition(unary) => unary.build(),
            Condition::BinaryCondition(binary) => binary.build(),
            Condition::TernaryCondition(ternary) => ternary.build(),
            Condition::Value(expression) => match expression {
                //TODO: Use ? representation
                ConditionValue::String(str) => str.to_string(),
                ConditionValue::I64(int) => int.to_string(),
                ConditionValue::I32(int) => int.to_string(),
                ConditionValue::I16(int) => int.to_string(),
                ConditionValue::I8(int) => int.to_string(),
            },
        }
    }
}
