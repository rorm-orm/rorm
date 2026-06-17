use std::fmt::Write;

use crate::conditional::{BuildCondition, Condition};
use crate::join_table::{JoinTable, JoinTableImpl};
use crate::limit_clause::LimitClause;
use crate::ordering::{OrderByEntry, Ordering};
use crate::select_column::{SelectColumn, SelectColumnImpl};
use crate::{DBImpl, Value};

/// Select builder
///
/// Can be constructed via [`DBImpl::select`]
#[derive(Debug)]
pub struct Select<'until_build, 'post_query> {
    // Set on construction
    pub(crate) db_impl: DBImpl,
    pub(crate) resulting_columns: &'until_build [SelectColumnImpl<'until_build>],
    pub(crate) from_clause: &'until_build str,
    pub(crate) join_tables: &'until_build [JoinTableImpl<'until_build, 'post_query>],
    pub(crate) order_by_clause: &'until_build [OrderByEntry<'until_build>],

    // Set by builder
    pub(crate) where_clause: Option<&'until_build Condition<'post_query>>,
    pub(crate) distinct: bool,
    pub(crate) limit: Option<u64>,
    pub(crate) offset: Option<u64>,

    #[cfg(feature = "postgres-only")]
    pub(crate) locking_clause: Option<LockingClause>,
}

/// A `SELECT` statement's locking clause
#[cfg(feature = "postgres-only")]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct LockingClause {
    /// Strength of the row lock
    pub strength: LockStrength,

    /// How to acquire the row lock
    pub acquire: LockAcquire,
}

#[cfg(feature = "postgres-only")]
/// Strength of a `SELECT` statement's row lock
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum LockStrength {
    /// `SELECT ... FOR UPDATE;`
    Update,
    /// `SELECT ... FOR NO KEY UPDATE;`
    NoKeyUpdate,
    /// `SELECT ... FOR SHARE;`
    Share,
    /// `SELECT ... FOR KEY SHARE;`
    KeyShare,
}

#[cfg(feature = "postgres-only")]
/// How to acquire a `SELECT` statement's row lock
#[derive(Default, Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum LockAcquire {
    /// `SELECT ... FOR <strength>;`
    #[default]
    Wait,
    /// `SELECT ... FOR <strength> NOWAIT;`
    NoWait,
    /// `SELECT ... FOR <strength> SKIP LOCKED;`
    SkipLocked,
}

impl<'until_build, 'post_build> Select<'until_build, 'post_build> {
    /// Set a limit to the resulting rows.
    pub fn limit_clause(mut self, limit: LimitClause) -> Self {
        self.limit = Some(limit.limit);
        self.offset = limit.offset;
        self
    }

    /// Only retrieve distinct rows.
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        self
    }

    /// Set a where clause to the query.
    pub fn where_clause(mut self, where_clause: &'until_build Condition<'post_build>) -> Self {
        self.where_clause = Some(where_clause);
        self
    }

    /// Set a lock on the resulting rows
    #[cfg(feature = "postgres-only")]
    pub fn locking_clause(mut self, locking: LockingClause) -> Self {
        self.locking_clause = Some(locking);
        self
    }

    /// Build the select query
    pub fn build(self) -> (String, Vec<Value<'post_build>>) {
        let mut sql;
        let mut values = Vec::new();

        match self.db_impl {
            #[cfg(feature = "sqlite")]
            DBImpl::SQLite => {
                sql = format!("SELECT{} ", if self.distinct { " DISTINCT" } else { "" });

                let column_len = self.resulting_columns.len();
                for (idx, column) in self.resulting_columns.iter().enumerate() {
                    column.build(&mut sql);

                    if idx != column_len - 1 {
                        write!(sql, ", ").unwrap();
                    }
                }

                write!(sql, " FROM \"{}\"", self.from_clause).unwrap();

                for x in self.join_tables {
                    write!(sql, " ").unwrap();
                    x.build(&mut sql, &mut values);
                }

                if let Some(c) = self.where_clause {
                    write!(sql, " WHERE {}", c.build(DBImpl::SQLite, &mut values)).unwrap()
                };

                if !self.order_by_clause.is_empty() {
                    write!(sql, " ORDER BY ").unwrap();

                    let order_by_len = self.order_by_clause.len();
                    for (idx, entry) in self.order_by_clause.iter().enumerate() {
                        if let Some(table_name) = entry.table_name {
                            write!(sql, "{table_name}.").unwrap();
                        };
                        write!(
                            sql,
                            "{}{}",
                            entry.column_name,
                            match entry.ordering {
                                Ordering::Asc => "",
                                Ordering::Desc => " DESC",
                            }
                        )
                        .unwrap();

                        if idx != order_by_len - 1 {
                            write!(sql, ", ").unwrap();
                        }
                    }
                };

                if let Some(limit) = self.limit {
                    write!(sql, " LIMIT {limit}").unwrap();
                    if let Some(offset) = self.offset {
                        write!(sql, " OFFSET {offset}").unwrap();
                    }
                };

                write!(sql, ";").unwrap();
            }
            #[cfg(feature = "postgres")]
            DBImpl::Postgres => {
                sql = format!("SELECT{} ", if self.distinct { " DISTINCT" } else { "" });

                let column_len = self.resulting_columns.len();
                for (idx, column) in self.resulting_columns.iter().enumerate() {
                    column.build(&mut sql);

                    if idx != column_len - 1 {
                        write!(sql, ", ").unwrap();
                    }
                }

                write!(sql, " FROM \"{}\"", self.from_clause).unwrap();

                for x in self.join_tables {
                    write!(sql, " ").unwrap();
                    x.build(&mut sql, &mut values);
                }

                if let Some(c) = self.where_clause {
                    write!(sql, " WHERE {}", c.build(DBImpl::Postgres, &mut values)).unwrap()
                };

                if !self.order_by_clause.is_empty() {
                    write!(sql, " ORDER BY ").unwrap();

                    let order_by_len = self.order_by_clause.len();
                    for (idx, entry) in self.order_by_clause.iter().enumerate() {
                        if let Some(table_name) = entry.table_name {
                            write!(sql, "\"{table_name}\".").unwrap();
                        };
                        write!(
                            sql,
                            "\"{}\"{}",
                            entry.column_name,
                            match entry.ordering {
                                Ordering::Asc => "",
                                Ordering::Desc => " DESC",
                            }
                        )
                        .unwrap();

                        if idx != order_by_len - 1 {
                            write!(sql, ", ").unwrap();
                        }
                    }
                };

                if let Some(limit) = self.limit {
                    write!(sql, " LIMIT {limit}").unwrap();
                    if let Some(offset) = self.offset {
                        write!(sql, " OFFSET {offset}").unwrap();
                    }
                };

                #[cfg(feature = "postgres-only")]
                if let Some(locking) = self.locking_clause {
                    write!(
                        sql,
                        " FOR {}{}",
                        match locking.strength {
                            LockStrength::Update => "UPDATE",
                            LockStrength::NoKeyUpdate => "NO KEY UPDATE",
                            LockStrength::Share => "SHARE",
                            LockStrength::KeyShare => "KEY SHARE",
                        },
                        match locking.acquire {
                            LockAcquire::Wait => "",
                            LockAcquire::NoWait => " NOWAIT",
                            LockAcquire::SkipLocked => " SKIP LOCKED",
                        }
                    )
                    .unwrap();
                }

                write!(sql, ";").unwrap();
            }
        }

        (sql, values)
    }
}
