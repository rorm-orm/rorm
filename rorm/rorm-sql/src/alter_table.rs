use std::fmt::Write;

use crate::error::Error;
use crate::{value, DBImpl, SQLCreateColumn};

pub trait AlterTable {}

pub struct AlterTableData {}

/**
Representation of operations to execute in the context of an ALTER TABLE statement.
*/
pub enum SQLAlterTableOperation<'post_build> {
    /// Use this operation to rename a table
    RenameTo {
        /// New name of the table
        name: String,
    },
    /// Use this operation to rename a column within a table
    RenameColumnTo {
        /// Current column name
        column_name: String,
        /// New column name
        new_column_name: String,
    },
    /// Use this operation to add a column to an existing table.
    /// Can be generated by using [crate::create_column::SQLCreateColumn]
    AddColumn {
        /// Operation to use for adding the column
        operation: SQLCreateColumn<'post_build>,
    },
    /// Use this operation to drop an existing column.
    DropColumn {
        /// Name of the column to drop
        name: String,
    },
}

impl<'post_build> SQLAlterTableOperation<'post_build> {
    fn build(
        self,
        s: &mut String,
        #[cfg(feature = "mysql")] lookup: &mut Vec<value::Value<'post_build>>,
        statements: &mut Vec<(String, Vec<value::Value<'post_build>>)>,
        dialect: DBImpl,
    ) -> Result<(), Error> {
        match self {
            SQLAlterTableOperation::RenameTo { name } => write!(s, "RENAME TO {}", name).unwrap(),
            SQLAlterTableOperation::RenameColumnTo {
                column_name,
                new_column_name,
            } => match dialect {
                #[cfg(feature = "sqlite")]
                DBImpl::SQLite => {
                    write!(s, "RENAME COLUMN {} TO {}", column_name, new_column_name).unwrap()
                }
                #[cfg(feature = "mysql")]
                DBImpl::MySQL => {
                    write!(s, "RENAME COLUMN {} TO {}", column_name, new_column_name).unwrap()
                }
                #[cfg(feature = "postgres")]
                DBImpl::Postgres => write!(
                    s,
                    "RENAME COLUMN \"{}\" TO \"{}\"",
                    column_name, new_column_name
                )
                .unwrap(),
            },
            #[cfg(any(feature = "postgres", feature = "sqlite"))]
            SQLAlterTableOperation::AddColumn { operation } => {
                write!(s, "ADD COLUMN ").unwrap();
                #[cfg(all(feature = "mysql", any(feature = "postgres", feature = "sqlite")))]
                operation.build(s, lookup, statements)?;
                #[cfg(all(not(feature = "mysql"), any(feature = "postgres", feature = "mysql")))]
                operation.build(s, statements)?;
                #[cfg(not(feature = "mysql"))]
                operation.build(s, statements)?;
            }
            #[cfg(not(any(feature = "postgres", feature = "sqlite")))]
            #[cfg(any(feature = "postgres", feature = "sqlite"))]
            SQLAlterTableOperation::AddColumn { operation } => {
                write!(s, "ADD COLUMN ").unwrap();
                #[cfg(all(feature = "mysql", any(feature = "postgres", feature = "sqlite")))]
                operation.build(s, lookup, statements)?;
                #[cfg(all(not(feature = "mysql"), any(feature = "postgres", feature = "mysql")))]
                operation.build(s, statements)?;
                #[cfg(not(feature = "mysql"))]
                operation.build(s, statements)?;
            }
            SQLAlterTableOperation::DropColumn { name } => {
                write!(s, "DROP COLUMN {}", name).unwrap();
            }
        };
        Ok(())
    }
}

/**
Representation of an ALTER TABLE statement.
*/
pub struct SQLAlterTable<'post_build> {
    pub(crate) dialect: DBImpl,
    /// Name of the table to operate on
    pub(crate) name: String,
    /// Operation to execute
    pub(crate) operation: SQLAlterTableOperation<'post_build>,
    pub(crate) lookup: Vec<value::Value<'post_build>>,
    pub(crate) statements: Vec<(String, Vec<value::Value<'post_build>>)>,
}

impl<'post_build> SQLAlterTable<'post_build> {
    /**
    This method is used to build the alter table statement.
    */
    pub fn build(mut self) -> Result<Vec<(String, Vec<value::Value<'post_build>>)>, Error> {
        let mut s = format!("ALTER TABLE {} ", self.name.as_str());
        #[cfg(feature = "mysql")]
        self.operation
            .build(&mut s, &mut self.lookup, &mut self.statements, self.dialect)?;
        #[cfg(not(feature = "mysql"))]
        self.operation
            .build(&mut s, &mut self.statements, self.dialect)?;
        write!(s, ";").unwrap();

        let mut statements = vec![(s, self.lookup)];
        statements.extend(self.statements);

        Ok(statements)
    }
}
