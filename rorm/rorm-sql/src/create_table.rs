use std::fmt::Write;

use crate::create_column::{CreateColumn, CreateColumnImpl};
use crate::error::Error;
use crate::Value;

/**
The trait representing a create table builder
*/
pub trait CreateTable<'until_build, 'post_build> {
    /**
    Add a column to the table.
     */
    fn add_column(self, column: CreateColumnImpl<'until_build, 'post_build>) -> Self;

    /**
    Sets the IF NOT EXISTS trait on the table
     */
    fn if_not_exists(self) -> Self;

    /**
    This method is used to convert the current state for the given dialect in a
    list of tuples.

    Each tuple consists of the query string and the corresponding bind parameters.
     */
    fn build(self) -> Result<Vec<(String, Vec<Value<'post_build>>)>, Error>;
}

/**
The representation of an create table operation.
*/
pub struct CreateTableData<'until_build, 'post_build> {
    pub(crate) name: &'until_build str,
    pub(crate) columns: Vec<CreateColumnImpl<'until_build, 'post_build>>,
    pub(crate) if_not_exists: bool,
    pub(crate) lookup: Vec<Value<'post_build>>,
    pub(crate) statements: Vec<(String, Vec<Value<'post_build>>)>,
}

/**
The implementation of the [CreateTable] trait for different database dialects.
*/
pub enum CreateTableImpl<'until_build, 'post_build> {
    /**
    SQLite representation of the CREATE TABLE operation.
     */
    #[cfg(feature = "sqlite")]
    SQLite(CreateTableData<'until_build, 'post_build>),
    /**
    MySQL representation of the CREATE TABLE operation.
     */
    #[cfg(feature = "mysql")]
    MySQL(CreateTableData<'until_build, 'post_build>),
    /**
    Postgres representation of the CREATE TABLE operation.
     */
    #[cfg(feature = "postgres")]
    Postgres(CreateTableData<'until_build, 'post_build>),
}

impl<'until_build, 'post_build> CreateTable<'until_build, 'post_build>
    for CreateTableImpl<'until_build, 'post_build>
{
    fn add_column(mut self, column: CreateColumnImpl<'until_build, 'post_build>) -> Self {
        match self {
            #[cfg(feature = "sqlite")]
            CreateTableImpl::SQLite(ref mut d) => d.columns.push(column),
            #[cfg(feature = "mysql")]
            CreateTableImpl::MySQL(ref mut d) => d.columns.push(column),
            #[cfg(feature = "postgres")]
            CreateTableImpl::Postgres(ref mut d) => d.columns.push(column),
        }
        self
    }

    fn if_not_exists(mut self) -> Self {
        match self {
            #[cfg(feature = "sqlite")]
            CreateTableImpl::SQLite(ref mut d) => d.if_not_exists = true,
            #[cfg(feature = "mysql")]
            CreateTableImpl::MySQL(ref mut d) => d.if_not_exists = true,
            #[cfg(feature = "postgres")]
            CreateTableImpl::Postgres(ref mut d) => d.if_not_exists = true,
        }
        self
    }

    fn build(self) -> Result<Vec<(String, Vec<Value<'post_build>>)>, Error> {
        match self {
            #[cfg(feature = "sqlite")]
            CreateTableImpl::SQLite(mut d) => {
                let mut s = format!(
                    "CREATE TABLE{} {} (",
                    if d.if_not_exists {
                        " IF NOT EXISTS"
                    } else {
                        ""
                    },
                    d.name
                );

                let columns_len = d.columns.len() - 1;
                for (idx, mut x) in d.columns.into_iter().enumerate() {
                    #[cfg(any(feature = "mysql", feature = "postgres"))]
                    if let CreateColumnImpl::SQLite(ref mut cci) = x {
                        cci.statements = Some(&mut d.statements)
                    }
                    #[cfg(not(any(feature = "mysql", feature = "postgres")))]
                    {
                        let CreateColumnImpl::SQLite(ref mut cci) = x;
                        cci.statements = Some(&mut d.statements);
                    }

                    x.build(&mut s)?;

                    if idx != columns_len {
                        write!(s, ", ").unwrap();
                    }
                }

                write!(s, ") STRICT; ").unwrap();

                let mut statements = vec![(s, d.lookup)];
                statements.extend(d.statements);

                Ok(statements)
            }
            #[cfg(feature = "mysql")]
            CreateTableImpl::MySQL(mut d) => {
                let mut s = format!(
                    "CREATE TABLE{} {} (",
                    if d.if_not_exists {
                        " IF NOT EXISTS"
                    } else {
                        ""
                    },
                    d.name
                );

                let columns_len = d.columns.len() - 1;
                for (idx, mut x) in d.columns.into_iter().enumerate() {
                    #[cfg(any(feature = "postgres", feature = "sqlite"))]
                    if let CreateColumnImpl::MySQL(ref mut cci) = x {
                        cci.statements = Some(&mut d.statements);
                    }
                    #[cfg(not(any(feature = "postgres", feature = "sqlite")))]
                    {
                        let CreateColumnImpl::MySQL(ref mut cci) = x;
                        cci.statements = Some(&mut d.statements);
                    }

                    x.build(&mut s)?;

                    if idx != columns_len {
                        write!(s, ", ").unwrap();
                    }
                }

                write!(s, "); ").unwrap();

                let mut statements = vec![(s, d.lookup)];
                statements.extend(d.statements);

                Ok(statements)
            }
            #[cfg(feature = "postgres")]
            CreateTableImpl::Postgres(mut d) => {
                let mut s = format!(
                    "CREATE TABLE{} \"{}\" (",
                    if d.if_not_exists {
                        " IF NOT EXISTS"
                    } else {
                        ""
                    },
                    d.name
                );

                let columns_len = d.columns.len() - 1;
                for (idx, mut x) in d.columns.into_iter().enumerate() {
                    #[cfg(any(feature = "sqlite", feature = "mysql"))]
                    if let CreateColumnImpl::Postgres(ref mut cci) = x {
                        cci.statements = Some(&mut d.statements);
                    }
                    #[cfg(not(any(feature = "sqlite", feature = "mysql")))]
                    {
                        let CreateColumnImpl::Postgres(ref mut cci) = x;
                        cci.statements = Some(&mut d.statements);
                    }

                    x.build(&mut s)?;

                    if idx != columns_len {
                        write!(s, ", ").unwrap();
                    }
                }

                write!(s, "); ").unwrap();

                let mut statements = vec![(s, d.lookup)];
                statements.extend(d.statements);

                Ok(statements)
            }
        }
    }
}
