use std::fmt::Write;

use crate::create_column::{CreateColumn, CreateColumnImpl};
use crate::error::Error;
use crate::Value;

/**
Representation of operations to execute in the context of an ALTER TABLE statement.
 */
#[derive(Debug)]
pub enum AlterTableOperation<'until_build, 'post_build> {
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
    AddColumn {
        /// Operation to use for adding the column
        operation: CreateColumnImpl<'until_build, 'post_build>,
    },
    /// Use this operation to drop an existing column.
    DropColumn {
        /// Name of the column to drop
        name: String,
    },
}

/**
The trait representing an alter table builder
*/
pub trait AlterTable<'post_build> {
    /**
    This method is used to build the alter table statement.
     */
    fn build(self) -> Result<Vec<(String, Vec<Value<'post_build>>)>, Error>;
}

/**
Representation of the data of an ALTER TABLE statement.
 */
#[derive(Debug)]
pub struct AlterTableData<'until_build, 'post_build> {
    /// Name of the table to operate on
    pub(crate) name: &'until_build str,
    /// Operation to execute
    pub(crate) operation: AlterTableOperation<'until_build, 'post_build>,
    pub(crate) lookup: Vec<Value<'post_build>>,
    pub(crate) statements: Vec<(String, Vec<Value<'post_build>>)>,
}

/**
Implementation of the [AlterTable] trait for the different database dialects
 */
#[derive(Debug)]
pub enum AlterTableImpl<'until_build, 'post_build> {
    /**
    SQLite representation of the ALTER TABLE operation.
     */
    #[cfg(feature = "sqlite")]
    SQLite(AlterTableData<'until_build, 'post_build>),
    /**
    MySQL representation of the ALTER TABLE operation.
     */
    #[cfg(feature = "mysql")]
    MySQL(AlterTableData<'until_build, 'post_build>),
    /**
    Postgres representation of the ALTER TABLE operation.
     */
    #[cfg(feature = "postgres")]
    Postgres(AlterTableData<'until_build, 'post_build>),
}

impl<'until_build, 'post_build> AlterTable<'post_build>
    for AlterTableImpl<'until_build, 'post_build>
{
    fn build(self) -> Result<Vec<(String, Vec<Value<'post_build>>)>, Error> {
        match self {
            #[cfg(feature = "sqlite")]
            AlterTableImpl::SQLite(mut d) => {
                let mut s = format!("ALTER TABLE {} ", d.name);

                match d.operation {
                    AlterTableOperation::RenameTo { name } => {
                        write!(s, "RENAME TO {}", name).unwrap();
                    }
                    AlterTableOperation::RenameColumnTo {
                        column_name,
                        new_column_name,
                    } => write!(s, "RENAME COLUMN {} TO {}", column_name, new_column_name).unwrap(),
                    AlterTableOperation::AddColumn { mut operation } => {
                        write!(s, "ADD COLUMN ").unwrap();

                        #[cfg(any(feature = "mysql", feature = "postgres"))]
                        if let CreateColumnImpl::SQLite(ref mut ccd) = operation {
                            ccd.statements = Some(&mut d.statements);
                            ccd.lookup = Some(&mut d.lookup);
                        }
                        #[cfg(not(any(feature = "mysql", feature = "postgres")))]
                        {
                            let CreateColumnImpl::SQLite(ref mut ccd) = operation;
                            ccd.statements = Some(&mut d.statements);
                            ccd.lookup = Some(&mut d.lookup);
                        }

                        operation.build(&mut s)?;
                    }
                    AlterTableOperation::DropColumn { name } => {
                        write!(s, "DROP COLUMN {}", name).unwrap()
                    }
                };

                write!(s, ";").unwrap();

                let mut statements = vec![(s, d.lookup)];
                statements.extend(d.statements);

                Ok(statements)
            }
            #[cfg(feature = "mysql")]
            AlterTableImpl::MySQL(mut d) => {
                let mut s = format!("ALTER TABLE {} ", d.name);

                match d.operation {
                    AlterTableOperation::RenameTo { name } => {
                        write!(s, "RENAME TO {}", name).unwrap();
                    }
                    AlterTableOperation::RenameColumnTo {
                        column_name,
                        new_column_name,
                    } => write!(s, "RENAME COLUMN {} TO {}", column_name, new_column_name).unwrap(),
                    AlterTableOperation::AddColumn { mut operation } => {
                        write!(s, "ADD COLUMN ").unwrap();

                        #[cfg(any(feature = "sqlite", feature = "postgres"))]
                        if let CreateColumnImpl::MySQL(ref mut ccd) = operation {
                            ccd.statements = Some(&mut d.statements);
                            ccd.lookup = Some(&mut d.lookup);
                        }
                        #[cfg(not(any(feature = "sqlite", feature = "postgres")))]
                        {
                            let CreateColumnImpl::MySQL(ref mut ccd) = operation;
                            ccd.statements = Some(&mut d.statements);
                            ccd.lookup = Some(&mut d.lookup);
                        }

                        operation.build(&mut s)?;
                    }
                    AlterTableOperation::DropColumn { name } => {
                        write!(s, "DROP COLUMN {}", name).unwrap()
                    }
                };

                write!(s, ";").unwrap();

                let mut statements = vec![(s, d.lookup)];
                statements.extend(d.statements);

                Ok(statements)
            }
            #[cfg(feature = "postgres")]
            AlterTableImpl::Postgres(mut d) => {
                let mut s = format!("ALTER TABLE {} ", d.name);

                match d.operation {
                    AlterTableOperation::RenameTo { name } => {
                        write!(s, "RENAME TO {}", name).unwrap();
                    }

                    AlterTableOperation::RenameColumnTo {
                        column_name,
                        new_column_name,
                    } => {
                        write!(
                            s,
                            "RENAME COLUMN \"{}\" TO \"{}\"",
                            column_name, new_column_name
                        )
                        .unwrap();
                    }
                    AlterTableOperation::AddColumn { mut operation } => {
                        write!(s, "ADD COLUMN ").unwrap();

                        #[cfg(any(feature = "sqlite", feature = "mysql"))]
                        if let CreateColumnImpl::Postgres(ref mut ccd) = operation {
                            ccd.statements = Some(&mut d.statements);
                        }
                        #[cfg(not(any(feature = "sqlite", feature = "mysql")))]
                        {
                            let CreateColumnImpl::Postgres(ref mut ccd) = operation;
                            ccd.statements = Some(&mut d.statements);
                        }

                        operation.build(&mut s)?;
                    }
                    AlterTableOperation::DropColumn { name } => {
                        write!(s, "DROP COLUMN {}", name).unwrap()
                    }
                };

                write!(s, ";").unwrap();

                let mut statements = vec![(s, d.lookup)];
                statements.extend(d.statements);

                Ok(statements)
            }
        }
    }
}
