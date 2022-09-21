use crate::{FFISlice, FFIString};
use core::str::Utf8Error;
use std::ops::Deref;

/**
This enum represents a value
 */
#[repr(C)]
#[derive(Copy, Clone)]
pub enum Value<'a> {
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
    /// null representation
    Null,
}

impl<'a> TryFrom<&Value<'a>> for rorm_db::value::Value<'a> {
    type Error = Utf8Error;

    fn try_from(value: &Value<'a>) -> Result<Self, Self::Error> {
        match value {
            Value::Ident(x) => Ok(rorm_db::value::Value::Ident(x.try_into()?)),
            Value::String(x) => Ok(rorm_db::value::Value::String(x.try_into()?)),
            Value::I64(x) => Ok(rorm_db::value::Value::I64(*x)),
            Value::I32(x) => Ok(rorm_db::value::Value::I32(*x)),
            Value::I16(x) => Ok(rorm_db::value::Value::I16(*x)),
            Value::Bool(x) => Ok(rorm_db::value::Value::Bool(*x)),
            Value::F64(x) => Ok(rorm_db::value::Value::F64(*x)),
            Value::F32(x) => Ok(rorm_db::value::Value::F32(*x)),
            Value::Null => Ok(rorm_db::value::Value::Null),
        }
    }
}

/**
This enum represents all available ternary expression.
 */
#[repr(C)]
pub enum TernaryCondition<'a> {
    /// Between represents "{} BETWEEN {} AND {}" from SQL
    Between(Box<[Condition<'a>; 3]>),
    /// Between represents "{} NOT BETWEEN {} AND {}" from SQL
    NotBetween(Box<[Condition<'a>; 3]>),
}

impl<'a> TryFrom<&TernaryCondition<'a>> for rorm_db::conditional::TernaryCondition<'a> {
    type Error = Utf8Error;

    fn try_from(value: &TernaryCondition<'a>) -> Result<Self, Self::Error> {
        match value {
            TernaryCondition::Between(x) => {
                let [a, b, c] = x.as_ref();
                let x_conv = [a.try_into()?, b.try_into()?, c.try_into()?];
                Ok(rorm_db::conditional::TernaryCondition::Between(Box::new(
                    x_conv,
                )))
            }
            TernaryCondition::NotBetween(x) => {
                let [a, b, c] = x.deref();
                let x_conv = [a.try_into()?, b.try_into()?, c.try_into()?];
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

impl<'a> TryFrom<&BinaryCondition<'a>> for rorm_db::conditional::BinaryCondition<'a> {
    type Error = Utf8Error;

    fn try_from(value: &BinaryCondition<'a>) -> Result<Self, Self::Error> {
        match value {
            BinaryCondition::Equals(x) => {
                let [a, b] = x.as_ref();
                let x_conv = [a.try_into()?, b.try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::Equals(Box::new(
                    x_conv,
                )))
            }
            BinaryCondition::NotEquals(x) => {
                let [a, b] = x.as_ref();
                let x_conv = [a.try_into()?, b.try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::NotEquals(Box::new(
                    x_conv,
                )))
            }
            BinaryCondition::Greater(x) => {
                let [a, b] = x.as_ref();
                let x_conv = [a.try_into()?, b.try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::Greater(Box::new(
                    x_conv,
                )))
            }
            BinaryCondition::GreaterOrEquals(x) => {
                let [a, b] = x.as_ref();
                let x_conv = [a.try_into()?, b.try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::GreaterOrEquals(
                    Box::new(x_conv),
                ))
            }
            BinaryCondition::Less(x) => {
                let [a, b] = x.as_ref();
                let x_conv = [a.try_into()?, b.try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::Less(Box::new(
                    x_conv,
                )))
            }
            BinaryCondition::LessOrEquals(x) => {
                let [a, b] = x.as_ref();
                let x_conv = [a.try_into()?, b.try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::LessOrEquals(
                    Box::new(x_conv),
                ))
            }
            BinaryCondition::Like(x) => {
                let [a, b] = x.as_ref();
                let x_conv = [a.try_into()?, b.try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::Like(Box::new(
                    x_conv,
                )))
            }
            BinaryCondition::NotLike(x) => {
                let [a, b] = x.as_ref();
                let x_conv = [a.try_into()?, b.try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::NotLike(Box::new(
                    x_conv,
                )))
            }
            BinaryCondition::Regexp(x) => {
                let [a, b] = x.as_ref();
                let x_conv = [a.try_into()?, b.try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::Regexp(Box::new(
                    x_conv,
                )))
            }
            BinaryCondition::NotRegexp(x) => {
                let [a, b] = x.as_ref();
                let x_conv = [a.try_into()?, b.try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::NotRegexp(Box::new(
                    x_conv,
                )))
            }
            BinaryCondition::In(x) => {
                let [a, b] = x.as_ref();
                let x_conv = [a.try_into()?, b.try_into()?];
                Ok(rorm_db::conditional::BinaryCondition::In(Box::new(x_conv)))
            }
            BinaryCondition::NotIn(x) => {
                let [a, b] = x.as_ref();
                let x_conv = [a.try_into()?, b.try_into()?];
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

impl<'a> TryFrom<&UnaryCondition<'a>> for rorm_db::conditional::UnaryCondition<'a> {
    type Error = Utf8Error;

    fn try_from(value: &UnaryCondition<'a>) -> Result<Self, Self::Error> {
        match value {
            UnaryCondition::IsNull(x) => Ok(rorm_db::conditional::UnaryCondition::IsNull(
                Box::new(x.deref().try_into()?),
            )),
            UnaryCondition::IsNotNull(x) => Ok(rorm_db::conditional::UnaryCondition::IsNotNull(
                Box::new(x.deref().try_into()?),
            )),
            UnaryCondition::Exists(x) => Ok(rorm_db::conditional::UnaryCondition::Exists(
                Box::new(x.deref().try_into()?),
            )),
            UnaryCondition::NotExists(x) => Ok(rorm_db::conditional::UnaryCondition::NotExists(
                Box::new(x.deref().try_into()?),
            )),
            UnaryCondition::Not(x) => Ok(rorm_db::conditional::UnaryCondition::Not(Box::new(
                x.deref().try_into()?,
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
    /// Representation of an unary condition.
    UnaryCondition(UnaryCondition<'a>),
    /// Representation of a binary condition.
    BinaryCondition(BinaryCondition<'a>),
    /// Representation of a ternary condition.
    TernaryCondition(TernaryCondition<'a>),
    /// Representation of a value.
    Value(Value<'a>),
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
