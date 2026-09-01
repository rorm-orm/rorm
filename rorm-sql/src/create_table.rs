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
    pub(crate) pre_statements: Vec<(String, Vec<Value<'post_build>>)>,
    pub(crate) statements: Vec<(String, Vec<Value<'post_build>>)>,
}

/**
The implementation of the [CreateTable] trait for different database dialects.

This should only be constructed via [crate::DBImpl::create_table].
*/
pub enum CreateTableImpl<'until_build, 'post_build> {
    /**
    SQLite representation of the CREATE TABLE operation.
     */
    #[cfg(feature = "sqlite")]
    SQLite(CreateTableData<'until_build, 'post_build>),
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
            #[cfg(feature = "postgres")]
            CreateTableImpl::Postgres(ref mut d) => d.columns.push(column),
        }
        self
    }

    fn if_not_exists(mut self) -> Self {
        match self {
            #[cfg(feature = "sqlite")]
            CreateTableImpl::SQLite(ref mut d) => d.if_not_exists = true,
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
                    if let CreateColumnImpl::SQLite(cci) = &mut x {
                        cci.statements = Some(&mut d.statements)
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
                    #[allow(irrefutable_let_patterns)]
                    if let CreateColumnImpl::Postgres(cci) = &mut x {
                        cci.pre_statements = Some(&mut d.pre_statements);
                        cci.statements = Some(&mut d.statements);
                    }

                    x.build(&mut s)?;

                    if idx != columns_len {
                        write!(s, ", ").unwrap();
                    }
                }

                write!(s, "); ").unwrap();

                let mut statements = d.pre_statements;
                statements.push((s, d.lookup));
                statements.extend(d.statements);

                Ok(statements)
            }
        }
    }
}
