use std::fmt::Write;

use crate::error::Error;
use crate::{value, DBImpl, SQLCreateColumn};

/**
The representation of an create table operation.
*/
pub struct SQLCreateTable<'post_build> {
    pub(crate) dialect: DBImpl,
    pub(crate) name: String,
    pub(crate) columns: Vec<SQLCreateColumn<'post_build>>,
    pub(crate) if_not_exists: bool,
    pub(crate) lookup: Vec<value::Value<'post_build>>,
    pub(crate) statements: Vec<(String, Vec<value::Value<'post_build>>)>,
}

impl<'post_build> SQLCreateTable<'post_build> {
    /**
    Add a column to the table.
    */
    pub fn add_column(mut self, column: SQLCreateColumn<'post_build>) -> Self {
        self.columns.push(column);
        self
    }

    /**
    Sets the IF NOT EXISTS trait on the table
    */
    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        self
    }

    /**
    This method is used to convert the current state for the given dialect in a
    list of tuples.

    Each tuple consists of the query string and the corresponding bind parameters.
    */
    pub fn build(mut self) -> Result<Vec<(String, Vec<value::Value<'post_build>>)>, Error> {
        let mut s = format!(
            "CREATE TABLE{} ",
            if self.if_not_exists {
                " IF NOT EXISTS"
            } else {
                ""
            }
        );

        match self.dialect {
            DBImpl::SQLite | DBImpl::MySQL => write!(s, "{} (", self.name).unwrap(),
            DBImpl::Postgres => write!(s, "\"{}\" (", self.name).unwrap(),
        }

        for (idx, x) in self.columns.iter().enumerate() {
            x.build(&mut s, &mut self.lookup, &mut self.statements)?;
            if idx != self.columns.len() - 1 {
                write!(s, ", ").unwrap();
            }
        }

        write!(
            s,
            ") {}; ",
            match self.dialect {
                DBImpl::SQLite => "STRICT",
                _ => "",
            }
        )
        .unwrap();

        let mut statements = vec![(s, self.lookup)];
        statements.extend(self.statements);

        Ok(statements)
    }
}
