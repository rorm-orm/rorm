use std::fmt::Write;

use crate::on_conflict::OnConflict;
use crate::{DBImpl, Value};

/**
Representation of the INSERT operation in SQL.
*/
pub struct SQLInsert<'until_build, 'post_build> {
    pub(crate) dialect: DBImpl,
    pub(crate) into_clause: String,
    pub(crate) columns: &'until_build [&'until_build str],
    pub(crate) row_values: &'until_build [&'until_build [Value<'post_build>]],
    pub(crate) lookup: Vec<Value<'post_build>>,
    pub(crate) on_conflict: OnConflict,
}

impl<'until_build, 'post_build> SQLInsert<'until_build, 'post_build> {
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
    This method is used to build the INSERT query.

    It returns the build query as well as a vector of values to bind to it.
    */
    pub fn build(mut self) -> (String, Vec<Value<'post_build>>) {
        let mut s = format!(
            "INSERT {}INTO ",
            match self.dialect {
                DBImpl::SQLite | DBImpl::MySQL => match self.on_conflict {
                    OnConflict::ABORT => "OR ABORT ",
                    OnConflict::ROLLBACK => "OR ROLLBACK ",
                },
                DBImpl::Postgres => "",
            },
        );

        match self.dialect {
            DBImpl::SQLite | DBImpl::MySQL => write!(s, "{} (", self.into_clause).unwrap(),
            DBImpl::Postgres => write!(s, "\"{}\" (", self.into_clause).unwrap(),
        }

        for (idx, x) in self.columns.iter().enumerate() {
            write!(s, "{}", x).unwrap();
            if idx != self.columns.len() - 1 {
                write!(s, ", ").unwrap();
            }
        }

        write!(s, ") VALUES ").unwrap();

        for (idx, x) in self.row_values.iter().enumerate() {
            write!(s, "(").unwrap();
            for (idx_2, y) in x.iter().enumerate() {
                match y {
                    Value::Ident(st) => match self.dialect {
                        DBImpl::SQLite | DBImpl::MySQL => write!(s, "{}", *st).unwrap(),
                        DBImpl::Postgres => write!(s, "\"{}\"", *st).unwrap(),
                    },
                    _ => {
                        self.lookup.push(*y);
                        match self.dialect {
                            DBImpl::SQLite | DBImpl::MySQL => write!(s, "?").unwrap(),
                            DBImpl::Postgres => write!(s, "${}", self.lookup.len()).unwrap(),
                        }
                    }
                }
                if idx_2 != x.len() - 1 {
                    write!(s, ", ").unwrap();
                }
            }
            write!(s, ")").unwrap();
            if idx != self.row_values.len() - 1 {
                write!(s, ", ").unwrap();
            }
        }

        write!(s, ";").unwrap();

        (s, self.lookup)
    }
}
