use core::str::Utf8Error;

use rorm_db::{DatabaseBackend, DatabaseConfiguration};

use crate::{Error, FFISlice, FFIString};

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

impl From<DBBackend> for Result<DatabaseBackend, Error<'_>> {
    fn from(value: DBBackend) -> Self {
        match value {
            DBBackend::Invalid => Err(Error::ConfigurationError(FFIString::from(
                "Invalid database backend selected",
            ))),
            DBBackend::SQLite => Ok(DatabaseBackend::SQLite),
            DBBackend::Postgres => Ok(DatabaseBackend::Postgres),
            DBBackend::MySQL => Ok(DatabaseBackend::MySQL),
        }
    }
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
        let db_backend_res: Result<DatabaseBackend, Error> = config.backend.into();
        if db_backend_res.is_err() {
            return Err(db_backend_res.err().unwrap());
        }
        let db_backend: DatabaseBackend = db_backend_res.unwrap();
        if config.min_connections == 0 || config.max_connections == 0 {
            return Err(Error::ConfigurationError(FFIString::from(
                "DBConnectOptions.min_connections and DBConnectOptions.max_connections must not be 0",
            )));
        }

        Ok(DatabaseConfiguration {
            backend: db_backend,
            name: <&str>::try_from(config.name).unwrap().to_owned(),
            host: <&str>::try_from(config.host).unwrap().to_owned(),
            port: config.port,
            user: <&str>::try_from(config.user).unwrap().to_owned(),
            password: <&str>::try_from(config.password).unwrap().to_owned(),
            min_connections: config.min_connections,
            max_connections: config.max_connections,
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
    /// Representation of an identifier, e.g. a column.
    /// This variant will not be escaped, so do not
    /// pass unchecked data to it.
    Ident(FFIString<'a>),
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
}

impl<'a> TryFrom<&FFIValue<'a>> for rorm_db::value::Value<'a> {
    type Error = Utf8Error;

    fn try_from(value: &FFIValue<'a>) -> Result<Self, Self::Error> {
        match value {
            FFIValue::Null => Ok(rorm_db::value::Value::Null),
            FFIValue::Ident(x) => Ok(rorm_db::value::Value::Ident(x.try_into()?)),
            FFIValue::String(x) => Ok(rorm_db::value::Value::String(x.try_into()?)),
            FFIValue::I64(x) => Ok(rorm_db::value::Value::I64(*x)),
            FFIValue::I32(x) => Ok(rorm_db::value::Value::I32(*x)),
            FFIValue::I16(x) => Ok(rorm_db::value::Value::I16(*x)),
            FFIValue::Bool(x) => Ok(rorm_db::value::Value::Bool(*x)),
            FFIValue::F64(x) => Ok(rorm_db::value::Value::F64(*x)),
            FFIValue::F32(x) => Ok(rorm_db::value::Value::F32(*x)),
            FFIValue::Binary(x) => Ok(rorm_db::value::Value::Binary(x.into())),
        }
    }
}

/**
This enum represents all available ternary expression.
 */
#[repr(C)]
pub enum TernaryCondition<'a> {
    /// Between represents "{} BETWEEN {} AND {}" from SQL
    Between([&'a Condition<'a>; 3]),
    /// Between represents "{} NOT BETWEEN {} AND {}" from SQL
    NotBetween([&'a Condition<'a>; 3]),
}

impl<'a> TryFrom<&TernaryCondition<'a>> for rorm_db::conditional::TernaryCondition<'a> {
    type Error = Utf8Error;

    fn try_from(value: &TernaryCondition<'a>) -> Result<Self, Self::Error> {
        match value {
            TernaryCondition::Between(x) => {
                let [a, b, c] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?, (*c).try_into()?];
                Ok(rorm_db::conditional::TernaryCondition::Between(Box::new(
                    x_conv,
                )))
            }
            TernaryCondition::NotBetween(x) => {
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
pub enum BinaryCondition<'a> {
    /// Representation of "{} = {}" in SQL
    Equals([&'a Condition<'a>; 2]),
    /// Representation of "{} <> {}" in SQL
    NotEquals([&'a Condition<'a>; 2]),
    /// Representation of "{} > {}" in SQL
    Greater([&'a Condition<'a>; 2]),
    /// Representation of "{} >= {}" in SQL
    GreaterOrEquals([&'a Condition<'a>; 2]),
    /// Representation of "{} < {}" in SQL
    Less([&'a Condition<'a>; 2]),
    /// Representation of "{} <= {}" in SQL
    LessOrEquals([&'a Condition<'a>; 2]),
    /// Representation of "{} LIKE {}" in SQL
    Like([&'a Condition<'a>; 2]),
    /// Representation of "{} NOT LIKE {}" in SQL
    NotLike([&'a Condition<'a>; 2]),
    /// Representation of "{} REGEXP {}" in SQL
    Regexp([&'a Condition<'a>; 2]),
    /// Representation of "{} NOT REGEXP {}" in SQL
    NotRegexp([&'a Condition<'a>; 2]),
    /// Representation of "{} IN {}" in SQL
    In([&'a Condition<'a>; 2]),
    /// Representation of "{} NOT IN {}" in SQL
    NotIn([&'a Condition<'a>; 2]),
}

impl<'a> TryFrom<&BinaryCondition<'a>> for rorm_db::conditional::BinaryCondition<'a> {
    type Error = Utf8Error;

    fn try_from(value: &BinaryCondition<'a>) -> Result<Self, Self::Error> {
        match value {
            BinaryCondition::Equals(x) => {
                let [a, b] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::Equals(Box::new(
                    x_conv,
                )))
            }
            BinaryCondition::NotEquals(x) => {
                let [a, b] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::NotEquals(Box::new(
                    x_conv,
                )))
            }
            BinaryCondition::Greater(x) => {
                let [a, b] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::Greater(Box::new(
                    x_conv,
                )))
            }
            BinaryCondition::GreaterOrEquals(x) => {
                let [a, b] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::GreaterOrEquals(
                    Box::new(x_conv),
                ))
            }
            BinaryCondition::Less(x) => {
                let [a, b] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::Less(Box::new(
                    x_conv,
                )))
            }
            BinaryCondition::LessOrEquals(x) => {
                let [a, b] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::LessOrEquals(
                    Box::new(x_conv),
                ))
            }
            BinaryCondition::Like(x) => {
                let [a, b] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::Like(Box::new(
                    x_conv,
                )))
            }
            BinaryCondition::NotLike(x) => {
                let [a, b] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::NotLike(Box::new(
                    x_conv,
                )))
            }
            BinaryCondition::Regexp(x) => {
                let [a, b] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::Regexp(Box::new(
                    x_conv,
                )))
            }
            BinaryCondition::NotRegexp(x) => {
                let [a, b] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::NotRegexp(Box::new(
                    x_conv,
                )))
            }
            BinaryCondition::In(x) => {
                let [a, b] = x;
                let x_conv = [(*a).try_into()?, (*b).try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::In(Box::new(x_conv)))
            }
            BinaryCondition::NotIn(x) => {
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
pub enum UnaryCondition<'a> {
    /// Representation of SQL's "{} IS NULL"
    IsNull(&'a Condition<'a>),
    /// Representation of SQL's "{} IS NOT NULL"
    IsNotNull(&'a Condition<'a>),
    /// Representation of SQL's "EXISTS {}"
    Exists(&'a Condition<'a>),
    /// Representation of SQL's "NOT EXISTS {}"
    NotExists(&'a Condition<'a>),
    /// Representation of SQL's "NOT {}"
    Not(&'a Condition<'a>),
}

impl<'a> TryFrom<&UnaryCondition<'a>> for rorm_db::conditional::UnaryCondition<'a> {
    type Error = Utf8Error;

    fn try_from(value: &UnaryCondition<'a>) -> Result<Self, Self::Error> {
        match value {
            UnaryCondition::IsNull(x) => Ok(rorm_db::conditional::UnaryCondition::IsNull(
                Box::new((*x).try_into()?),
            )),
            UnaryCondition::IsNotNull(x) => Ok(rorm_db::conditional::UnaryCondition::IsNotNull(
                Box::new((*x).try_into()?),
            )),
            UnaryCondition::Exists(x) => Ok(rorm_db::conditional::UnaryCondition::Exists(
                Box::new((*x).try_into()?),
            )),
            UnaryCondition::NotExists(x) => Ok(rorm_db::conditional::UnaryCondition::NotExists(
                Box::new((*x).try_into()?),
            )),
            UnaryCondition::Not(x) => Ok(rorm_db::conditional::UnaryCondition::Not(Box::new(
                (*x).try_into()?,
            ))),
        }
    }
}

/**
This enum represents a condition tree.
 */
#[repr(C)]
pub enum Condition<'a> {
    /// A list of [Condition]s, that get expanded to "{} AND {} ..."
    Conjunction(FFISlice<'a, Condition<'a>>),
    /// A list of [Condition]s, that get expanded to "{} OR {} ..."
    Disjunction(FFISlice<'a, Condition<'a>>),
    /// Representation of a unary condition.
    UnaryCondition(UnaryCondition<'a>),
    /// Representation of a binary condition.
    BinaryCondition(BinaryCondition<'a>),
    /// Representation of a ternary condition.
    TernaryCondition(TernaryCondition<'a>),
    /// Representation of a value.
    Value(FFIValue<'a>),
}

impl<'a> TryFrom<&Condition<'a>> for rorm_db::conditional::Condition<'a> {
    type Error = Utf8Error;

    fn try_from(value: &Condition<'a>) -> Result<Self, Self::Error> {
        match value {
            Condition::Conjunction(x) => {
                let x_conv: &[Condition] = x.into();
                let mut x_vec = vec![];
                for cond in x_conv {
                    x_vec.push(cond.try_into()?);
                }
                Ok(rorm_db::conditional::Condition::Conjunction(x_vec))
            }
            Condition::Disjunction(x) => {
                let x_conv: &[Condition] = x.into();
                let mut x_vec = vec![];
                for cond in x_conv {
                    x_vec.push(cond.try_into()?);
                }
                Ok(rorm_db::conditional::Condition::Disjunction(x_vec))
            }
            Condition::UnaryCondition(x) => Ok(rorm_db::conditional::Condition::UnaryCondition(
                x.try_into()?,
            )),
            Condition::BinaryCondition(x) => Ok(rorm_db::conditional::Condition::BinaryCondition(
                x.try_into()?,
            )),
            Condition::TernaryCondition(x) => Ok(
                rorm_db::conditional::Condition::TernaryCondition(x.try_into()?),
            ),
            Condition::Value(x) => Ok(rorm_db::conditional::Condition::Value(x.try_into()?)),
        }
    }
}
