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
    pub(crate) where_clause: Option<&'until_build Condition<'post_query>>,

    // Set by builder
    pub(crate) order_by_clause: &'until_build [OrderByEntry<'until_build>],
    pub(crate) distinct: bool,
    pub(crate) limit: Option<u64>,
    pub(crate) offset: Option<u64>,
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

                write!(sql, ";").unwrap();
            }
        }

        (sql, values)
    }
}
