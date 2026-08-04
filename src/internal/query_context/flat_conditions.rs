//! Flat generic-less representation of a condition tree
//!
//! This representation is used inside the [`QueryContext`]
//! to store a generic [`Condition`](crate::conditions::Condition) using a concrete type
//! before handing it over to `rorm-sql`.
//!
//! There has to be a representation in between because `rorm-sql` doesn't take ownership
//! and the `Condition` [`Column`](crate::conditions::Column) requires generating join aliases (owned strings)
//! after the use constructed his condition tree.

use crate::conditions::collections::CollectionOperator;
use crate::conditions::{BinaryOperator, TernaryOperator, UnaryOperator};
use crate::fields::utils::column_name::ColumnName;
use crate::internal::query_context::QueryContext;
use crate::internal::relation_path::PathId;

mod sql {
    pub use crate::db::sql::conditional::*;
    pub use crate::db::sql::value::*;
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum FlatCondition {
    StartCollection(CollectionOperator),
    EndCollection,
    UnaryCondition(UnaryOperator),
    BinaryCondition(BinaryOperator),
    TernaryCondition(TernaryOperator),
    Value(usize),
    Column(PathId, &'static ColumnName),
}

/// Error returned by [`QueryContext::try_get_condition`]
///
/// The error's cause is either a bad `Condition` implementation
/// or an invalid index passed to the method.
///
/// I.e. it's always some programmer's fault.
#[derive(Copy, Clone, Debug)]
pub enum GetConditionError {
    /// Unexpected end of slice
    MissingNodes,

    /// Unexpected `FlatCondition::EndCollection` i.e. end without previous start
    CollectionEnd,

    /// Invalid value index
    UnknownValue,

    /// Invalid table key
    UnknownAlias,
}

impl QueryContext<'_> {
    pub(super) fn get_condition_inner(
        &self,
        head: FlatCondition,
        tail: &mut impl Iterator<Item = FlatCondition>,
    ) -> Result<sql::Condition<'_>, GetConditionError> {
        use GetConditionError::*;

        Ok(match head {
            FlatCondition::StartCollection(op) => {
                let op = match op {
                    CollectionOperator::And => sql::Condition::Conjunction,
                    CollectionOperator::Or => sql::Condition::Disjunction,
                };
                let mut args = Vec::new();
                loop {
                    let head = tail.next().ok_or(MissingNodes)?;
                    if matches!(head, FlatCondition::EndCollection) {
                        break;
                    } else {
                        args.push(self.get_condition_inner(head, tail)?);
                    }
                }
                op(args)
            }
            FlatCondition::EndCollection => return Err(CollectionEnd),
            FlatCondition::UnaryCondition(operator) => {
                sql::Condition::UnaryCondition(sql::UnaryExpression {
                    operator,
                    value: Box::new(
                        self.get_condition_inner(tail.next().ok_or(MissingNodes)?, tail)?,
                    ),
                })
            }
            FlatCondition::BinaryCondition(operator) => {
                sql::Condition::BinaryCondition(sql::BinaryExpression {
                    operator,
                    values: Box::new([
                        self.get_condition_inner(tail.next().ok_or(MissingNodes)?, tail)?,
                        self.get_condition_inner(tail.next().ok_or(MissingNodes)?, tail)?,
                    ]),
                })
            }
            FlatCondition::TernaryCondition(operator) => {
                sql::Condition::TernaryCondition(sql::TernaryExpression {
                    operator,
                    values: Box::new([
                        self.get_condition_inner(tail.next().ok_or(MissingNodes)?, tail)?,
                        self.get_condition_inner(tail.next().ok_or(MissingNodes)?, tail)?,
                        self.get_condition_inner(tail.next().ok_or(MissingNodes)?, tail)?,
                    ]),
                })
            }
            FlatCondition::Value(index) => {
                sql::Condition::Value(self.values.get(index).ok_or(UnknownValue)?.as_sql())
            }
            FlatCondition::Column(table_name, column_name) => {
                sql::Condition::Value(sql::Value::Column {
                    table_name: Some(self.join_aliases.get(&table_name).ok_or(UnknownAlias)?),
                    column_name,
                })
            }
        })
    }
}
