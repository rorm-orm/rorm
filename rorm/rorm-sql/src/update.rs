use std::fmt::Write;

use crate::conditional::BuildCondition;
use crate::error::Error;
use crate::{conditional, value, DBImpl, OnConflict};

/**
Implementation of SQLs UPDATE statement.
*/
pub struct SQLUpdate<'until_build, 'post_build> {
    pub(crate) dialect: DBImpl,
    pub(crate) model: &'until_build str,
    pub(crate) on_conflict: OnConflict,
    pub(crate) single_column_updates: Vec<(&'until_build str, value::Value<'post_build>)>,
    pub(crate) multiple_column_updates:
        Vec<(&'until_build [&'until_build str], value::Value<'post_build>)>,
    pub(crate) where_clause: Option<&'until_build conditional::Condition<'post_build>>,
    pub(crate) lookup: Vec<value::Value<'post_build>>,
}

impl<'until_build, 'post_build> SQLUpdate<'until_build, 'post_build> {
    /**
    Turns on ROLLBACK mode.

    Only useful in case of an active transaction.

    If the insert fails, the complete transaction will be rolled back.
    The default case is to just stop the transaction, but not rollback any
    prior successful executed queries.
     */
    pub fn rollback_transaction(mut self) -> Self {
        self.on_conflict = OnConflict::ROLLBACK;
        self
    }

    /**
    Adds a [conditional::Condition] to the update query.
     */
    pub fn where_clause(
        mut self,
        condition: &'until_build conditional::Condition<'post_build>,
    ) -> Self {
        self.where_clause = Some(condition);
        self
    }

    /**
    Add an update for a single column.

    **Parameter**:
    - `column_name`: The column name to set the value to.
    - `column_value`: The value to set the column to.
    */
    pub fn add_single_col_update(
        mut self,
        column_name: &'until_build str,
        column_value: value::Value<'post_build>,
    ) -> Self {
        self.single_column_updates.push((column_name, column_value));
        self
    }

    /**
    Add an update for multiple columns.

    **Parameter**:
    - `column_names`: The column names to set the value to.
    - `column_value`: The value to set the columns to.
     */
    pub fn add_multiple_col_update(
        mut self,
        column_names: &'until_build [&'until_build str],
        column_value: value::Value<'post_build>,
    ) -> Self {
        self.multiple_column_updates
            .push((column_names, column_value));
        self
    }

    /**
    Builds the given statement.

    The query_string as well a list of values to bind are returned.

    This function returns an error, if no update statements are given previously.
    */
    pub fn build(mut self) -> Result<(String, Vec<value::Value<'post_build>>), Error> {
        if self.single_column_updates.is_empty() && self.multiple_column_updates.is_empty() {
            return Err(Error::SQLBuildError(String::from(
                "There must be at least one update in an UPDATE statement",
            )));
        }

        let mut s = format!(
            "UPDATE {}",
            match self.dialect {
                DBImpl::SQLite | DBImpl::MySQL => match self.on_conflict {
                    OnConflict::ABORT => "OR ABORT ",
                    OnConflict::ROLLBACK => "OR ROLLBACK ",
                },
                DBImpl::Postgres => "",
            },
        );

        match self.dialect {
            DBImpl::SQLite | DBImpl::MySQL => write!(s, "{} SET ", self.model).unwrap(),
            DBImpl::Postgres => write!(s, "\"{}\" SET ", self.model).unwrap(),
        }

        let multiple_columns_empty = self.multiple_column_updates.is_empty();
        let multiple_columns_max_index = self.multiple_column_updates.len() - 1;
        for (idx, (names, value)) in self.multiple_column_updates.into_iter().enumerate() {
            write!(s, "(").unwrap();
            for (idx, name) in names.iter().enumerate() {
                match self.dialect {
                    DBImpl::SQLite | DBImpl::MySQL => write!(s, "{}", name).unwrap(),
                    DBImpl::Postgres => write!(s, "\"{}\"", name).unwrap(),
                }

                if idx != names.len() - 1 {
                    write!(s, ", ").unwrap();
                }
            }

            self.lookup.push(value);
            match self.dialect {
                DBImpl::SQLite | DBImpl::MySQL => write!(s, ") = ?").unwrap(),
                DBImpl::Postgres => write!(s, ") = ${}", self.lookup.len()).unwrap(),
            }

            if idx != multiple_columns_max_index {
                write!(s, ", ").unwrap();
            }
        }

        if !multiple_columns_empty {
            write!(s, ", ").unwrap();
        }

        let single_column_max_index = self.single_column_updates.len() - 1;
        for (idx, (name, value)) in self.single_column_updates.into_iter().enumerate() {
            match self.dialect {
                DBImpl::SQLite | DBImpl::MySQL => write!(s, "{}", name).unwrap(),
                DBImpl::Postgres => write!(s, "\"{}\"", name).unwrap(),
            }

            self.lookup.push(value);
            match self.dialect {
                DBImpl::SQLite | DBImpl::MySQL => write!(s, " = ?").unwrap(),
                DBImpl::Postgres => write!(s, " = ${}", self.lookup.len()).unwrap(),
            }

            if idx != single_column_max_index {
                write!(s, ", ").unwrap();
            }
        }

        match self.where_clause {
            None => {}
            Some(cond) => write!(s, " WHERE {}", cond.build(&mut self.lookup)).unwrap(),
        }

        write!(s, ";").unwrap();

        Ok((s, self.lookup))
    }
}
