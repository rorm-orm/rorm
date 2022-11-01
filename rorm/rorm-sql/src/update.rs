use std::fmt::Write;

use crate::conditional::{BuildCondition, Condition};
use crate::error::Error;
use crate::{OnConflict, Value};

/**
Trait representing a update builder.
*/
pub trait Update<'until_build, 'post_build> {
    /**
    Turns on ROLLBACK mode.

    Only useful in case of an active transaction.

    If the insert fails, the complete transaction will be rolled back.
    The default case is to just stop the transaction, but not rollback any
    prior successful executed queries.
     */
    fn rollback_transaction(self) -> Self;

    /**
    Adds a [Condition] to the update query.
     */
    fn where_clause(self, condition: &'until_build Condition<'post_build>) -> Self;

    /**
    Add an update

    **Parameter**:
    - `column_name`: The column name to set the value to.
    - `column_value`: The value to set the column to.
     */
    fn add_update(self, column_name: &'until_build str, column_value: Value<'post_build>) -> Self;

    /**
    Builds the given statement.

    The query_string as well a list of values to bind are returned.

    This function returns an error, if no update statements are given previously.
     */
    fn build(self) -> Result<(String, Vec<Value<'post_build>>), Error>;
}

/**
Implementation of SQLs UPDATE statement.
 */
#[derive(Debug)]
pub struct UpdateData<'until_build, 'post_build> {
    pub(crate) model: &'until_build str,
    pub(crate) on_conflict: OnConflict,
    pub(crate) updates: Vec<(&'until_build str, Value<'post_build>)>,
    pub(crate) where_clause: Option<&'until_build Condition<'post_build>>,
    pub(crate) lookup: Vec<Value<'post_build>>,
}

/**
Implementation of the [Update] trait for the different implementations
 */
#[derive(Debug)]
pub enum UpdateImpl<'until_build, 'post_build> {
    /**
    SQLite representation of the UPDATE operation.
     */
    #[cfg(feature = "sqlite")]
    SQLite(UpdateData<'until_build, 'post_build>),
    /**
    MySQL representation of the UPDATE operation.
     */
    #[cfg(feature = "mysql")]
    MySQL(UpdateData<'until_build, 'post_build>),
    /**
    Postgres representation of the UPDATE operation.
     */
    #[cfg(feature = "postgres")]
    Postgres(UpdateData<'until_build, 'post_build>),
}

impl<'until_build, 'post_build> Update<'until_build, 'post_build>
    for UpdateImpl<'until_build, 'post_build>
{
    fn rollback_transaction(mut self) -> Self {
        match self {
            #[cfg(feature = "sqlite")]
            UpdateImpl::SQLite(ref mut d) => d.on_conflict = OnConflict::ROLLBACK,
            #[cfg(feature = "mysql")]
            UpdateImpl::MySQL(ref mut d) => d.on_conflict = OnConflict::ROLLBACK,
            #[cfg(feature = "postgres")]
            UpdateImpl::Postgres(ref mut d) => d.on_conflict = OnConflict::ROLLBACK,
        };
        self
    }

    fn where_clause(mut self, condition: &'until_build Condition<'post_build>) -> Self {
        match self {
            #[cfg(feature = "sqlite")]
            UpdateImpl::SQLite(ref mut d) => d.where_clause = Some(condition),
            #[cfg(feature = "mysql")]
            UpdateImpl::MySQL(ref mut d) => d.where_clause = Some(condition),
            #[cfg(feature = "postgres")]
            UpdateImpl::Postgres(ref mut d) => d.where_clause = Some(condition),
        };
        self
    }

    fn add_update(
        mut self,
        column_name: &'until_build str,
        column_value: Value<'post_build>,
    ) -> Self {
        match self {
            #[cfg(feature = "sqlite")]
            UpdateImpl::SQLite(ref mut d) => d.updates.push((column_name, column_value)),
            #[cfg(feature = "mysql")]
            UpdateImpl::MySQL(ref mut d) => d.updates.push((column_name, column_value)),
            #[cfg(feature = "postgres")]
            UpdateImpl::Postgres(ref mut d) => d.updates.push((column_name, column_value)),
        };
        self
    }

    fn build(self) -> Result<(String, Vec<Value<'post_build>>), Error> {
        match self {
            #[cfg(feature = "sqlite")]
            UpdateImpl::SQLite(mut d) => {
                if d.updates.is_empty() {
                    return Err(Error::SQLBuildError(String::from(
                        "There must be at least one update in an UPDATE statement",
                    )));
                }
                let mut s = format!(
                    "UPDATE {}{} SET ",
                    match d.on_conflict {
                        OnConflict::ABORT => "OR ABORT ",
                        OnConflict::ROLLBACK => "OR ROLLBACK ",
                    },
                    d.model,
                );

                let update_index = d.updates.len() - 1;
                for (idx, (name, value)) in d.updates.into_iter().enumerate() {
                    write!(s, "{} = ?", name).unwrap();
                    d.lookup.push(value);
                    if idx != update_index {
                        write!(s, ", ").unwrap();
                    }
                }

                if let Some(condition) = d.where_clause {
                    write!(s, " WHERE {}", condition.build(&mut d.lookup)).unwrap();
                }

                write!(s, ";").unwrap();

                Ok((s, d.lookup))
            }
            #[cfg(feature = "mysql")]
            UpdateImpl::MySQL(mut d) => {
                if d.updates.is_empty() {
                    return Err(Error::SQLBuildError(String::from(
                        "There must be at least one update in an UPDATE statement",
                    )));
                }
                let mut s = format!(
                    "UPDATE {}{} SET ",
                    match d.on_conflict {
                        OnConflict::ABORT => "OR ABORT ",
                        OnConflict::ROLLBACK => "OR ROLLBACK ",
                    },
                    d.model,
                );

                let update_index = d.updates.len() - 1;
                for (idx, (name, value)) in d.updates.into_iter().enumerate() {
                    write!(s, "{} = ?", name).unwrap();
                    d.lookup.push(value);
                    if idx != update_index {
                        write!(s, ", ").unwrap();
                    }
                }

                if let Some(condition) = d.where_clause {
                    write!(s, " WHERE {}", condition.build(&mut d.lookup)).unwrap();
                }

                write!(s, ";").unwrap();

                Ok((s, d.lookup))
            }
            #[cfg(feature = "postgres")]
            UpdateImpl::Postgres(mut d) => {
                if d.updates.is_empty() {
                    return Err(Error::SQLBuildError(String::from(
                        "There must be at least one update in an UPDATE statement",
                    )));
                }
                let mut s = format!("UPDATE \"{}\" SET ", d.model);

                let update_index = d.updates.len() - 1;
                for (idx, (name, value)) in d.updates.into_iter().enumerate() {
                    write!(s, "\"{}\" = ?", name).unwrap();
                    d.lookup.push(value);
                    if idx != update_index {
                        write!(s, ", ").unwrap();
                    }
                }

                if let Some(condition) = d.where_clause {
                    write!(s, " WHERE {}", condition.build(&mut d.lookup)).unwrap();
                }

                write!(s, ";").unwrap();

                Ok((s, d.lookup))
            }
        }
    }
}
