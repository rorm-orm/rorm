use rorm_db::join_table::JoinType;
use rorm_db::limit_clause::LimitClause;
use rorm_db::ordering::Ordering;
use rorm_db::{DatabaseConfiguration, DatabaseDriver};

use crate::utils::FFIOption;
use crate::{Error, FFIDate, FFIDateTime, FFISlice, FFIString, FFITime};

/**
Representation of the database backend.

This is used to determine the correct driver and the correct dialect to use.
 */
#[repr(i32)]
pub enum DBBackend {
    /// This exists to forbid default initializations with 0 on C side.
    /// Using this type will result in an [crate::errors::Error::ConfigurationError]
    Invalid,
    /// SQLite backend
    SQLite,
    /// MySQL / MariaDB backend
    MySQL,
    /// Postgres backend
    Postgres,
}

/**
Configuration operation to connect to a database.

Will be converted into [rorm_db::DatabaseConfiguration].

`min_connections` and `max_connections` must not be 0.
 */
#[repr(C)]
pub struct DBConnectOptions<'a> {
    backend: DBBackend,
    name: FFIString<'a>,
    host: FFIString<'a>,
    port: u16,
    user: FFIString<'a>,
    password: FFIString<'a>,
    min_connections: u32,
    max_connections: u32,
}

impl From<DBConnectOptions<'_>> for Result<DatabaseConfiguration, Error<'_>> {
    fn from(config: DBConnectOptions) -> Self {
        if config.min_connections == 0 || config.max_connections == 0 {
            return Err(Error::ConfigurationError(FFIString::from(
                "DBConnectOptions.min_connections and DBConnectOptions.max_connections must not be 0",
            )));
        }

        let d = match config.backend {
            DBBackend::Invalid => {
                return Err(Error::ConfigurationError(FFIString::from(
                    "Invalid database backend selected",
                )))
            }
            DBBackend::SQLite => DatabaseDriver::SQLite {
                filename: <&str>::try_from(config.name).unwrap().to_owned(),
            },
            DBBackend::MySQL => DatabaseDriver::MySQL {
                name: <&str>::try_from(config.name).unwrap().to_owned(),
                host: <&str>::try_from(config.host).unwrap().to_owned(),
                port: config.port,
                user: <&str>::try_from(config.user).unwrap().to_owned(),
                password: <&str>::try_from(config.password).unwrap().to_owned(),
            },
            DBBackend::Postgres => DatabaseDriver::Postgres {
                name: <&str>::try_from(config.name).unwrap().to_owned(),
                host: <&str>::try_from(config.host).unwrap().to_owned(),
                port: config.port,
                user: <&str>::try_from(config.user).unwrap().to_owned(),
                password: <&str>::try_from(config.password).unwrap().to_owned(),
            },
        };

        #[cfg(feature = "logging")]
        return Ok(DatabaseConfiguration {
            driver: d,
            min_connections: config.min_connections,
            max_connections: config.max_connections,
            disable_logging: None,
            statement_log_level: None,
            slow_statement_log_level: None,
        });

        #[cfg(not(feature = "logging"))]
        Ok(DatabaseConfiguration {
            driver: d,
            min_connections: config.min_connections,
            max_connections: config.max_connections,
            disable_logging: Some(true),
            statement_log_level: None,
            slow_statement_log_level: None,
        })
    }
}

/**
This enum represents a value
 */
#[repr(C)]
#[derive(Copy, Clone)]
pub enum FFIValue<'a> {
    /// null representation
    Null,
    /// Representation of an identifier.
    /// This variant will not be escaped, so do not
    /// pass unchecked data to it.
    Ident(FFIString<'a>),
    /// Representation of a column.
    Column {
        /// Optional table name
        table_name: FFIOption<FFIString<'a>>,
        /// Name of the column
        column_name: FFIString<'a>,
    },
    /// String representation
    String(FFIString<'a>),
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
    /// Binary representation
    Binary(FFISlice<'a, u8>),
    /// Representation of time without timezones
    NaiveTime(FFITime),
    /// Representation of dates without timezones
    NaiveDate(FFIDate),
    /// Representation of datetimes without timezones
    NaiveDateTime(FFIDateTime),
}

impl<'a> TryFrom<&'a FFIValue<'a>> for rorm_db::value::Value<'a> {
    type Error = Error<'a>;

    fn try_from(value: &'a FFIValue<'a>) -> Result<Self, Self::Error> {
        match value {
            FFIValue::Null => Ok(rorm_db::value::Value::Null),
            FFIValue::Ident(x) => Ok(rorm_db::value::Value::Ident(
                x.try_into().map_err(|_| Error::InvalidStringError)?,
            )),
            FFIValue::Column {
                table_name,
                column_name,
            } => {
                let table_name = match table_name {
                    FFIOption::None => None,
                    FFIOption::Some(v) => {
                        Some(v.try_into().map_err(|_| Error::InvalidStringError)?)
                    }
                };
                Ok(rorm_db::value::Value::Column {
                    table_name,
                    column_name: column_name
                        .try_into()
                        .map_err(|_| Error::InvalidStringError)?,
                })
            }
            FFIValue::String(x) => Ok(rorm_db::value::Value::String(
                x.try_into().map_err(|_| Error::InvalidStringError)?,
            )),
            FFIValue::I64(x) => Ok(rorm_db::value::Value::I64(*x)),
            FFIValue::I32(x) => Ok(rorm_db::value::Value::I32(*x)),
            FFIValue::I16(x) => Ok(rorm_db::value::Value::I16(*x)),
            FFIValue::Bool(x) => Ok(rorm_db::value::Value::Bool(*x)),
            FFIValue::F64(x) => Ok(rorm_db::value::Value::F64(*x)),
            FFIValue::F32(x) => Ok(rorm_db::value::Value::F32(*x)),
            FFIValue::Binary(x) => Ok(rorm_db::value::Value::Binary(x.into())),
            FFIValue::NaiveTime(x) => Ok(rorm_db::value::Value::NaiveTime(x.try_into()?)),
            FFIValue::NaiveDate(x) => Ok(rorm_db::value::Value::NaiveDate(x.try_into()?)),
            FFIValue::NaiveDateTime(x) => Ok(rorm_db::value::Value::NaiveDateTime(x.try_into()?)),
        }
    }
}

/**
This enum represents all available ternary expression.
 */
#[repr(C)]
pub enum FFITernaryCondition<'a> {
    /// Between represents "{} BETWEEN {} AND {}" from SQL
    Between([&'a FFICondition<'a>; 3]),
    /// Between represents "{} NOT BETWEEN {} AND {}" from SQL
    NotBetween([&'a FFICondition<'a>; 3]),
}

impl<'a> TryFrom<&FFITernaryCondition<'a>> for rorm_db::conditional::TernaryCondition<'a> {
    type Error = Error<'a>;

    fn try_from(value: &FFITernaryCondition<'a>) -> Result<Self, Self::Error> {
        match value {
            FFITernaryCondition::Between(x) => {
                let [a, b, c] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?, (*c).try_into()?];
                Ok(rorm_db::conditional::TernaryCondition::Between(Box::new(
                    x_conv,
                )))
            }
            FFITernaryCondition::NotBetween(x) => {
                let [a, b, c] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?, (*c).try_into()?];
                Ok(rorm_db::conditional::TernaryCondition::NotBetween(
                    Box::new(x_conv),
                ))
            }
        }
    }
}

/**
This enum represents a binary expression.
 */
#[repr(C)]
pub enum FFIBinaryCondition<'a> {
    /// Representation of "{} = {}" in SQL
    Equals([&'a FFICondition<'a>; 2]),
    /// Representation of "{} <> {}" in SQL
    NotEquals([&'a FFICondition<'a>; 2]),
    /// Representation of "{} > {}" in SQL
    Greater([&'a FFICondition<'a>; 2]),
    /// Representation of "{} >= {}" in SQL
    GreaterOrEquals([&'a FFICondition<'a>; 2]),
    /// Representation of "{} < {}" in SQL
    Less([&'a FFICondition<'a>; 2]),
    /// Representation of "{} <= {}" in SQL
    LessOrEquals([&'a FFICondition<'a>; 2]),
    /// Representation of "{} LIKE {}" in SQL
    Like([&'a FFICondition<'a>; 2]),
    /// Representation of "{} NOT LIKE {}" in SQL
    NotLike([&'a FFICondition<'a>; 2]),
    /// Representation of "{} REGEXP {}" in SQL
    Regexp([&'a FFICondition<'a>; 2]),
    /// Representation of "{} NOT REGEXP {}" in SQL
    NotRegexp([&'a FFICondition<'a>; 2]),
    /// Representation of "{} IN {}" in SQL
    In([&'a FFICondition<'a>; 2]),
    /// Representation of "{} NOT IN {}" in SQL
    NotIn([&'a FFICondition<'a>; 2]),
}

impl<'a> TryFrom<&FFIBinaryCondition<'a>> for rorm_db::conditional::BinaryCondition<'a> {
    type Error = Error<'a>;

    fn try_from(value: &FFIBinaryCondition<'a>) -> Result<Self, Self::Error> {
        match value {
            FFIBinaryCondition::Equals(x) => {
                let [a, b] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::Equals(Box::new(
                    x_conv,
                )))
            }
            FFIBinaryCondition::NotEquals(x) => {
                let [a, b] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::NotEquals(Box::new(
                    x_conv,
                )))
            }
            FFIBinaryCondition::Greater(x) => {
                let [a, b] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::Greater(Box::new(
                    x_conv,
                )))
            }
            FFIBinaryCondition::GreaterOrEquals(x) => {
                let [a, b] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::GreaterOrEquals(
                    Box::new(x_conv),
                ))
            }
            FFIBinaryCondition::Less(x) => {
                let [a, b] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::Less(Box::new(
                    x_conv,
                )))
            }
            FFIBinaryCondition::LessOrEquals(x) => {
                let [a, b] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::LessOrEquals(
                    Box::new(x_conv),
                ))
            }
            FFIBinaryCondition::Like(x) => {
                let [a, b] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::Like(Box::new(
                    x_conv,
                )))
            }
            FFIBinaryCondition::NotLike(x) => {
                let [a, b] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::NotLike(Box::new(
                    x_conv,
                )))
            }
            FFIBinaryCondition::Regexp(x) => {
                let [a, b] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::Regexp(Box::new(
                    x_conv,
                )))
            }
            FFIBinaryCondition::NotRegexp(x) => {
                let [a, b] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::NotRegexp(Box::new(
                    x_conv,
                )))
            }
            FFIBinaryCondition::In(x) => {
                let [a, b] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::In(Box::new(x_conv)))
            }
            FFIBinaryCondition::NotIn(x) => {
                let [a, b] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::NotIn(Box::new(
                    x_conv,
                )))
            }
        }
    }
}

/**
This enum represents all available unary conditions.
 */
#[repr(C)]
pub enum FFIUnaryCondition<'a> {
    /// Representation of SQL's "{} IS NULL"
    IsNull(&'a FFICondition<'a>),
    /// Representation of SQL's "{} IS NOT NULL"
    IsNotNull(&'a FFICondition<'a>),
    /// Representation of SQL's "EXISTS {}"
    Exists(&'a FFICondition<'a>),
    /// Representation of SQL's "NOT EXISTS {}"
    NotExists(&'a FFICondition<'a>),
    /// Representation of SQL's "NOT {}"
    Not(&'a FFICondition<'a>),
}

impl<'a> TryFrom<&FFIUnaryCondition<'a>> for rorm_db::conditional::UnaryCondition<'a> {
    type Error = Error<'a>;

    fn try_from(value: &FFIUnaryCondition<'a>) -> Result<Self, Self::Error> {
        match value {
            FFIUnaryCondition::IsNull(x) => Ok(rorm_db::conditional::UnaryCondition::IsNull(
                Box::new((*x).try_into()?),
            )),
            FFIUnaryCondition::IsNotNull(x) => Ok(rorm_db::conditional::UnaryCondition::IsNotNull(
                Box::new((*x).try_into()?),
            )),
            FFIUnaryCondition::Exists(x) => Ok(rorm_db::conditional::UnaryCondition::Exists(
                Box::new((*x).try_into()?),
            )),
            FFIUnaryCondition::NotExists(x) => Ok(rorm_db::conditional::UnaryCondition::NotExists(
                Box::new((*x).try_into()?),
            )),
            FFIUnaryCondition::Not(x) => Ok(rorm_db::conditional::UnaryCondition::Not(Box::new(
                (*x).try_into()?,
            ))),
        }
    }
}

/**
This enum represents a condition tree.
 */
#[repr(C)]
pub enum FFICondition<'a> {
    /// A list of [Condition]s, that get expanded to "{} AND {} ..."
    Conjunction(FFISlice<'a, FFICondition<'a>>),
    /// A list of [Condition]s, that get expanded to "{} OR {} ..."
    Disjunction(FFISlice<'a, FFICondition<'a>>),
    /// Representation of a unary condition.
    UnaryCondition(FFIUnaryCondition<'a>),
    /// Representation of a binary condition.
    BinaryCondition(FFIBinaryCondition<'a>),
    /// Representation of a ternary condition.
    TernaryCondition(FFITernaryCondition<'a>),
    /// Representation of a value.
    Value(FFIValue<'a>),
}

impl<'a> TryFrom<&'a FFICondition<'a>> for rorm_db::conditional::Condition<'a> {
    type Error = Error<'a>;

    fn try_from(value: &'a FFICondition<'a>) -> Result<Self, Self::Error> {
        match value {
            FFICondition::Conjunction(x) => {
                let x_conv: &[FFICondition] = x.into();
                let mut x_vec = vec![];
                for cond in x_conv {
                    x_vec.push(cond.try_into()?);
                }
                Ok(rorm_db::conditional::Condition::Conjunction(x_vec))
            }
            FFICondition::Disjunction(x) => {
                let x_conv: &[FFICondition] = x.into();
                let mut x_vec = vec![];
                for cond in x_conv {
                    x_vec.push(cond.try_into()?);
                }
                Ok(rorm_db::conditional::Condition::Disjunction(x_vec))
            }
            FFICondition::UnaryCondition(x) => Ok(rorm_db::conditional::Condition::UnaryCondition(
                x.try_into()?,
            )),
            FFICondition::BinaryCondition(x) => Ok(
                rorm_db::conditional::Condition::BinaryCondition(x.try_into()?),
            ),
            FFICondition::TernaryCondition(x) => Ok(
                rorm_db::conditional::Condition::TernaryCondition(x.try_into()?),
            ),
            FFICondition::Value(x) => Ok(rorm_db::conditional::Condition::Value(x.try_into()?)),
        }
    }
}

/**
Representation of an update.

Consists of a column and the value to set to this column.
*/
#[repr(C)]
pub struct FFIUpdate<'a> {
    pub(crate) column: FFIString<'a>,
    pub(crate) value: FFIValue<'a>,
}

/**
Representation of a join type.
*/
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum FFIJoinType {
    /// Normal join operation.
    ///
    /// Equivalent to INNER JOIN
    Join,
    /// Cartesian product of the tables
    CrossJoin,
    /// Given:
    /// T1 LEFT JOIN T2 ON ..
    ///
    /// First, an inner join is performed.
    /// Then, for each row in T1 that does not satisfy the join condition with any row in T2,
    /// a joined row is added with null values in columns of T2.
    LeftJoin,
    /// Given:
    /// T1 RIGHT JOIN T2 ON ..
    ///
    /// First, an inner join is performed.
    /// Then, for each row in T2 that does not satisfy the join condition with any row in T1,
    /// a joined row is added with null values in columns of T1.
    RightJoin,
    /// Given:
    /// T1 FULL JOIN T2 ON ..
    ///
    /// First, an inner join is performed.
    /// Then, for each row in T2 that does not satisfy the join condition with any row in T1,
    /// a joined row is added with null values in columns of T1.
    /// Also, for each row in T1 that does not satisfy the join condition with any row in T2,
    /// a joined row is added with null values in columns of T2.
    FullJoin,
}

impl From<FFIJoinType> for JoinType {
    fn from(v: FFIJoinType) -> Self {
        match v {
            FFIJoinType::Join => JoinType::Join,
            FFIJoinType::CrossJoin => JoinType::CrossJoin,
            FFIJoinType::LeftJoin => JoinType::LeftJoin,
            FFIJoinType::RightJoin => JoinType::RightJoin,
            FFIJoinType::FullJoin => JoinType::FullJoin,
        }
    }
}

/**
FFI representation of a Join expression.
*/
#[repr(C)]
pub struct FFIJoin<'a> {
    /// Type of the join operation
    pub(crate) join_type: FFIJoinType,
    /// Name of the join table
    pub(crate) table_name: FFIString<'a>,
    /// Alias for the join table
    pub(crate) join_alias: FFIString<'a>,
    /// Condition to apply the join on
    pub(crate) join_condition: &'a FFICondition<'a>,
}

/**
FFI representation of a Limit clause.
*/
#[repr(C)]
pub struct FFILimitClause {
    pub(crate) limit: u64,
    pub(crate) offset: FFIOption<u64>,
}

impl From<FFILimitClause> for LimitClause {
    fn from(v: FFILimitClause) -> Self {
        Self {
            limit: v.limit,
            offset: v.offset.into(),
        }
    }
}

/**
FFI representation of a [SelectColumnImpl]
*/
#[repr(C)]
pub struct FFIColumnSelector<'a> {
    pub(crate) table_name: FFIOption<FFIString<'a>>,
    pub(crate) column_name: FFIString<'a>,
    pub(crate) select_alias: FFIOption<FFIString<'a>>,
}

/**
Representation of the [Ordering]
*/
#[repr(C)]
#[derive(Copy, Clone)]
pub enum FFIOrdering {
    /// Ascending
    Asc,
    /// Descending
    Desc,
}

impl From<FFIOrdering> for Ordering {
    fn from(v: FFIOrdering) -> Self {
        match v {
            FFIOrdering::Asc => Self::Asc,
            FFIOrdering::Desc => Self::Desc,
        }
    }
}

/**
Representation of a [OrderByEntry]
*/
#[repr(C)]
#[derive(Copy, Clone)]
pub struct FFIOrderByEntry<'a> {
    pub(crate) ordering: FFIOrdering,
    pub(crate) column_name: FFIString<'a>,
}
