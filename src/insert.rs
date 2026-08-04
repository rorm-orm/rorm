use std::fmt::Write;

#[cfg(feature = "postgres")]
use crate::db_specific::postgres;
#[cfg(feature = "sqlite")]
use crate::db_specific::sqlite;
use crate::on_conflict::OnConflict;
use crate::value::NullType;
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
    pub(crate) returning_clause: Option<&'until_build [&'until_build str]>,
}

/**
Implementation of the [Insert] trait for the different implementations.

Should only be constructed via [DBImpl::insert](crate::DBImpl::insert).
 */
#[derive(Debug)]
pub enum InsertImpl<'until_build, 'post_build> {
    /**
    SQLite representation of the INSERT operation.
     */
    #[cfg(feature = "sqlite")]
    SQLite(InsertData<'until_build, 'post_build>),
    /**
    Postgres representation of the INSERT operation.
     */
    #[cfg(feature = "postgres")]
    Postgres(InsertData<'until_build, 'post_build>),
}

impl<'post_build> Insert<'post_build> for InsertImpl<'_, 'post_build> {
    fn rollback_transaction(mut self) -> Self {
        match self {
            #[cfg(feature = "sqlite")]
            InsertImpl::SQLite(ref mut d) => d.on_conflict = OnConflict::ROLLBACK,
            #[cfg(feature = "postgres")]
            InsertImpl::Postgres(ref mut d) => d.on_conflict = OnConflict::ROLLBACK,
        };
        self
    }

    fn build(self) -> (String, Vec<Value<'post_build>>) {
        match self {
            #[cfg(feature = "sqlite")]
            InsertImpl::SQLite(mut d) => {
                // Handle case, if no columns should be inserted, aka an empty insert
                if d.columns.is_empty() {
                    let mut s = format!(
                        "INSERT {}INTO \"{}\" DEFAULT VALUES",
                        match d.on_conflict {
                            OnConflict::ABORT => "OR ABORT ",
                            OnConflict::ROLLBACK => "OR ROLLBACK ",
                        },
                        d.into_clause,
                    );

                    if let Some(ret_clause) = d.returning_clause {
                        write!(s, " RETURNING ").unwrap();

                        for (idx, c) in ret_clause.iter().enumerate() {
                            write!(s, "\"{c}\"").unwrap();

                            if idx != ret_clause.len() - 1 {
                                write!(s, ", ").unwrap();
                            }
                        }
                    }
                    write!(s, ";").unwrap();

                    return (s, d.lookup);
                }

                let mut s = format!(
                    "INSERT {}INTO \"{}\" (",
                    match d.on_conflict {
                        OnConflict::ABORT => "OR ABORT ",
                        OnConflict::ROLLBACK => "OR ROLLBACK ",
                    },
                    d.into_clause,
                );
                for (idx, x) in d.columns.iter().enumerate() {
                    write!(s, "\"{x}\"").unwrap();
                    if idx != d.columns.len() - 1 {
                        write!(s, ", ").unwrap();
                    }
                }
                write!(s, ") VALUES ").unwrap();

                for (idx, x) in d.row_values.iter().enumerate() {
                    write!(s, "(").unwrap();
                    for (idx_2, y) in x.iter().enumerate() {
                        match y {
                            #[allow(deprecated)]
                            Value::Ident(st) => write!(s, "\"{}\"", *st).unwrap(),
                            Value::Choice(c) => write!(s, "{}", sqlite::fmt(c)).unwrap(),
                            Value::Null(NullType::Choice) => write!(s, "NULL").unwrap(),
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

                if let Some(ret_clause) = d.returning_clause {
                    write!(s, " RETURNING ").unwrap();

                    for (idx, c) in ret_clause.iter().enumerate() {
                        write!(s, "\"{c}\"").unwrap();

                        if idx != ret_clause.len() - 1 {
                            write!(s, ", ").unwrap();
                        }
                    }
                }

                write!(s, ";").unwrap();

                (s, d.lookup)
            }
            #[cfg(feature = "postgres")]
            InsertImpl::Postgres(mut d) => {
                if d.columns.is_empty() {
                    let mut s = format!("INSERT INTO \"{}\" DEFAULT VALUES", d.into_clause);

                    if let Some(ret_clause) = d.returning_clause {
                        write!(s, " RETURNING ").unwrap();

                        for (idx, c) in ret_clause.iter().enumerate() {
                            write!(s, "\"{c}\"").unwrap();

                            if idx != ret_clause.len() - 1 {
                                write!(s, ", ").unwrap();
                            }
                        }
                    }
                    write!(s, ";").unwrap();

                    return (s, d.lookup);
                }

                let mut s = format!("INSERT INTO \"{}\" (", d.into_clause);
                for (idx, x) in d.columns.iter().enumerate() {
                    write!(s, "\"{x}\"").unwrap();
                    if idx != d.columns.len() - 1 {
                        write!(s, ", ").unwrap();
                    }
                }
                write!(s, ") VALUES ").unwrap();

                for (idx, x) in d.row_values.iter().enumerate() {
                    write!(s, "(").unwrap();
                    for (idx_2, y) in x.iter().enumerate() {
                        match y {
                            #[allow(deprecated)]
                            Value::Ident(st) => write!(s, "\"{}\"", *st).unwrap(),
                            Value::Choice(c) => write!(s, "{}", postgres::fmt(c)).unwrap(),
                            Value::Null(NullType::Choice) => write!(s, "NULL").unwrap(),
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

                if let Some(ret_clause) = d.returning_clause {
                    write!(s, " RETURNING ").unwrap();

                    for (idx, c) in ret_clause.iter().enumerate() {
                        write!(s, "\"{c}\"").unwrap();

                        if idx != ret_clause.len() - 1 {
                            write!(s, ", ").unwrap();
                        }
                    }
                }

                write!(s, ";").unwrap();

                (s, d.lookup)
            }
        }
    }
}
