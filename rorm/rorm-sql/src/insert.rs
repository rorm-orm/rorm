use std::fmt::Write;

use crate::on_conflict::OnConflict;
use crate::Value;

/**
Trait representing a insert builder.
 */
pub trait Insert<'post_build> {
    /**
    Turns on ROLLBACK mode.

    Only useful in case of an active transaction.

    If the insert fails, the complete transaction will be rolled back.
    The default case is to just stop the transaction, but not rollback any
    prior successful executed queries.
     */
    fn rollback_transaction(self) -> Self;

    /**
    This method is used to build the INSERT query.
    It returns the build query as well as a vector of values to bind to it.
     */
    fn build(self) -> (String, Vec<Value<'post_build>>);
}

/**
Representation of the data of a INSERT operation in SQL.
*/
#[derive(Debug)]
pub struct InsertData<'until_build, 'post_build> {
    pub(crate) into_clause: &'until_build str,
    pub(crate) columns: &'until_build [&'until_build str],
    pub(crate) row_values: &'until_build [&'until_build [Value<'post_build>]],
    pub(crate) lookup: Vec<Value<'post_build>>,
    pub(crate) on_conflict: OnConflict,
}

/**
Implementation of the [Insert] trait for the different implementations
 */
#[derive(Debug)]
pub enum InsertImpl<'until_build, 'post_build> {
    /**
    SQLite representation of the INSERT operation.
     */
    #[cfg(feature = "sqlite")]
    SQLite(InsertData<'until_build, 'post_build>),
    /**
    MySQL representation of the INSERT operation.
     */
    #[cfg(feature = "mysql")]
    MySQL(InsertData<'until_build, 'post_build>),
    /**
    Postgres representation of the INSERT operation.
     */
    #[cfg(feature = "postgres")]
    Postgres(InsertData<'until_build, 'post_build>),
}

impl<'until_build, 'post_build> Insert<'post_build> for InsertImpl<'until_build, 'post_build> {
    fn rollback_transaction(mut self) -> Self {
        match self {
            #[cfg(feature = "sqlite")]
            InsertImpl::SQLite(ref mut d) => d.on_conflict = OnConflict::ROLLBACK,
            #[cfg(feature = "mysql")]
            InsertImpl::MySQL(ref mut d) => d.on_conflict = OnConflict::ROLLBACK,
            #[cfg(feature = "postgres")]
            InsertImpl::Postgres(ref mut d) => d.on_conflict = OnConflict::ROLLBACK,
        };
        self
    }

    fn build(self) -> (String, Vec<Value<'post_build>>) {
        match self {
            #[cfg(feature = "sqlite")]
            InsertImpl::SQLite(mut d) => {
                let mut s = format!(
                    "INSERT {}INTO {} (",
                    match d.on_conflict {
                        OnConflict::ABORT => "OR ABORT ",
                        OnConflict::ROLLBACK => "OR ROLLBACK ",
                    },
                    d.into_clause,
                );
                for (idx, x) in d.columns.iter().enumerate() {
                    write!(s, "{}", x).unwrap();
                    if idx != d.columns.len() - 1 {
                        write!(s, ", ").unwrap();
                    }
                }
                write!(s, ") VALUES ").unwrap();

                for (idx, x) in d.row_values.iter().enumerate() {
                    write!(s, "(").unwrap();
                    for (idx_2, y) in x.iter().enumerate() {
                        match y {
                            Value::Ident(st) => write!(s, "{}", *st).unwrap(),
                            _ => {
                                d.lookup.push(*y);
                                write!(s, "?").unwrap();
                            }
                        }
                        if idx_2 != x.len() - 1 {
                            write!(s, ", ").unwrap();
                        }
                    }
                    write!(s, ")").unwrap();
                    if idx != d.row_values.len() - 1 {
                        write!(s, ", ").unwrap();
                    }
                }

                write!(s, ";").unwrap();

                (s, d.lookup)
            }
            #[cfg(feature = "mysql")]
            InsertImpl::MySQL(mut d) => {
                let mut s = format!("INSERT INTO {} (", d.into_clause,);
                for (idx, x) in d.columns.iter().enumerate() {
                    write!(s, "{}", x).unwrap();
                    if idx != d.columns.len() - 1 {
                        write!(s, ", ").unwrap();
                    }
                }
                write!(s, ") VALUES ").unwrap();

                for (idx, x) in d.row_values.iter().enumerate() {
                    write!(s, "(").unwrap();
                    for (idx_2, y) in x.iter().enumerate() {
                        match y {
                            Value::Ident(st) => write!(s, "{}", *st).unwrap(),
                            _ => {
                                d.lookup.push(*y);
                                write!(s, "?").unwrap();
                            }
                        }
                        if idx_2 != x.len() - 1 {
                            write!(s, ", ").unwrap();
                        }
                    }
                    write!(s, ")").unwrap();
                    if idx != d.row_values.len() - 1 {
                        write!(s, ", ").unwrap();
                    }
                }

                write!(s, ";").unwrap();

                (s, d.lookup)
            }
            #[cfg(feature = "postgres")]
            InsertImpl::Postgres(mut d) => {
                let mut s = format!("INSERT INTO \"{}\" (", d.into_clause);
                for (idx, x) in d.columns.iter().enumerate() {
                    write!(s, "{}", x).unwrap();
                    if idx != d.columns.len() - 1 {
                        write!(s, ", ").unwrap();
                    }
                }
                write!(s, ") VALUES ").unwrap();

                for (idx, x) in d.row_values.iter().enumerate() {
                    write!(s, "(").unwrap();
                    for (idx_2, y) in x.iter().enumerate() {
                        match y {
                            Value::Ident(st) => write!(s, "\"{}\"", *st).unwrap(),
                            _ => {
                                d.lookup.push(*y);
                                write!(s, "${}", d.lookup.len()).unwrap();
                            }
                        }
                        if idx_2 != x.len() - 1 {
                            write!(s, ", ").unwrap();
                        }
                    }
                    write!(s, ")").unwrap();
                    if idx != d.row_values.len() - 1 {
                        write!(s, ", ").unwrap();
                    }
                }

                write!(s, ";").unwrap();

                (s, d.lookup)
            }
        }
    }
}
