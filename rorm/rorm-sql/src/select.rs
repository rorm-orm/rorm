use std::fmt::Write;

use crate::conditional::{BuildCondition, Condition};
use crate::join_table::{JoinTable, JoinTableImpl};
use crate::limit_clause::LimitClause;
use crate::ordering::{OrderByEntry, Ordering};
use crate::select_column::{SelectColumn, SelectColumnImpl};
use crate::{DBImpl, Value};

/**
Trait representing a select builder.
 */
pub trait Select<'until_build, 'post_query> {
    /**
    Set a limit to the resulting rows.
     */
    fn limit_clause(self, limit: LimitClause) -> Self;

    /**
    Only retrieve distinct rows.
     */
    fn distinct(self) -> Self;

    /**
    Set a where clause to the query.
     */
    fn where_clause(self, where_clause: &'until_build Condition<'post_query>) -> Self;

    /**
    Build the select query
     */
    fn build(self) -> (String, Vec<Value<'post_query>>);
}

/**
Representation of the data of a SELECT operation in SQL.
 */
#[derive(Debug)]
pub struct SelectData<'until_build, 'post_query> {
    pub(crate) resulting_columns: &'until_build [SelectColumnImpl<'until_build>],
    pub(crate) limit: Option<u64>,
    pub(crate) offset: Option<u64>,
    pub(crate) from_clause: &'until_build str,
    pub(crate) where_clause: Option<&'until_build Condition<'post_query>>,
    pub(crate) distinct: bool,
    pub(crate) lookup: Vec<Value<'post_query>>,
    pub(crate) join_tables: &'until_build [JoinTableImpl<'until_build, 'post_query>],
    pub(crate) order_by_clause: &'until_build [OrderByEntry<'until_build>],
}

/**
Implementation of the [Select] trait for the different implementations.

Should only be constructed via [DBImpl::select]
 */
#[derive(Debug)]
pub enum SelectImpl<'until_build, 'post_query> {
    /**
    SQLite representation of the SELECT operation.
     */
    #[cfg(feature = "sqlite")]
    SQLite(SelectData<'until_build, 'post_query>),
    /**
    MySQL representation of the SELECT operation.
     */
    #[cfg(feature = "mysql")]
    MySQL(SelectData<'until_build, 'post_query>),
    /**
    Postgres representation of the SELECT operation.
     */
    #[cfg(feature = "postgres")]
    Postgres(SelectData<'until_build, 'post_query>),
}

impl<'until_build, 'post_build> Select<'until_build, 'post_build>
    for SelectImpl<'until_build, 'post_build>
{
    fn limit_clause(mut self, limit: LimitClause) -> Self {
        match self {
            #[cfg(feature = "sqlite")]
            SelectImpl::SQLite(ref mut d) => {
                d.limit = Some(limit.limit);
                d.offset = limit.offset;
            }
            #[cfg(feature = "mysql")]
            SelectImpl::MySQL(ref mut d) => {
                d.limit = Some(limit.limit);
                d.offset = limit.offset;
            }
            #[cfg(feature = "postgres")]
            SelectImpl::Postgres(ref mut d) => {
                d.limit = Some(limit.limit);
                d.offset = limit.offset;
            }
        };
        self
    }

    fn distinct(mut self) -> Self {
        match self {
            #[cfg(feature = "sqlite")]
            SelectImpl::SQLite(ref mut d) => d.distinct = true,
            #[cfg(feature = "mysql")]
            SelectImpl::MySQL(ref mut d) => d.distinct = true,
            #[cfg(feature = "postgres")]
            SelectImpl::Postgres(ref mut d) => d.distinct = true,
        };
        self
    }

    fn where_clause(mut self, where_clause: &'until_build Condition<'post_build>) -> Self {
        match self {
            #[cfg(feature = "sqlite")]
            SelectImpl::SQLite(ref mut d) => d.where_clause = Some(where_clause),
            #[cfg(feature = "mysql")]
            SelectImpl::MySQL(ref mut d) => d.where_clause = Some(where_clause),
            #[cfg(feature = "postgres")]
            SelectImpl::Postgres(ref mut d) => d.where_clause = Some(where_clause),
        };
        self
    }

    fn build(self) -> (String, Vec<Value<'post_build>>) {
        match self {
            #[cfg(feature = "sqlite")]
            SelectImpl::SQLite(mut d) => {
                let mut s = format!("SELECT{} ", if d.distinct { " DISTINCT" } else { "" });

                let column_len = d.resulting_columns.len();
                for (idx, column) in d.resulting_columns.iter().enumerate() {
                    column.build(&mut s);

                    if idx != column_len - 1 {
                        write!(s, ", ").unwrap();
                    }
                }

                write!(s, " FROM {}", d.from_clause).unwrap();

                for x in d.join_tables {
                    write!(s, " ").unwrap();
                    x.build(&mut s, &mut d.lookup);
                }

                if let Some(c) = d.where_clause {
                    write!(s, " WHERE {}", c.build(DBImpl::SQLite, &mut d.lookup)).unwrap()
                };

                if !d.order_by_clause.is_empty() {
                    write!(s, " ORDER BY ").unwrap();

                    let order_by_len = d.order_by_clause.len();
                    for (idx, entry) in d.order_by_clause.iter().enumerate() {
                        write!(
                            s,
                            "{}{}",
                            entry.column_name,
                            match entry.ordering {
                                Ordering::Asc => "",
                                Ordering::Desc => " DESC",
                            }
                        )
                        .unwrap();

                        if idx != order_by_len - 1 {
                            write!(s, ", ").unwrap();
                        }
                    }
                };

                if let Some(limit) = d.limit {
                    write!(s, " LIMIT {}", limit).unwrap();
                    if let Some(offset) = d.offset {
                        write!(s, " OFFSET {}", offset).unwrap();
                    }
                };

                write!(s, ";").unwrap();

                (s, d.lookup)
            }
            #[cfg(feature = "mysql")]
            SelectImpl::MySQL(mut d) => {
                let mut s = format!("SELECT{} ", if d.distinct { " DISTINCT" } else { "" });

                let column_len = d.resulting_columns.len();
                for (idx, column) in d.resulting_columns.iter().enumerate() {
                    column.build(&mut s);

                    if idx != column_len - 1 {
                        write!(s, ", ").unwrap();
                    }
                }

                write!(s, " FROM {}", d.from_clause).unwrap();

                for x in d.join_tables {
                    write!(s, " ").unwrap();
                    x.build(&mut s, &mut d.lookup);
                }

                if let Some(c) = d.where_clause {
                    write!(s, " WHERE {}", c.build(DBImpl::MySQL, &mut d.lookup)).unwrap()
                };

                if !d.order_by_clause.is_empty() {
                    write!(s, " ORDER BY ").unwrap();

                    let order_by_len = d.order_by_clause.len();
                    for (idx, entry) in d.order_by_clause.iter().enumerate() {
                        write!(
                            s,
                            "{}{}",
                            entry.column_name,
                            match entry.ordering {
                                Ordering::Asc => "",
                                Ordering::Desc => " DESC",
                            }
                        )
                        .unwrap();

                        if idx != order_by_len - 1 {
                            write!(s, ", ").unwrap();
                        }
                    }
                };

                if let Some(limit) = d.limit {
                    write!(s, " LIMIT {}", limit).unwrap();
                    if let Some(offset) = d.offset {
                        write!(s, " OFFSET {}", offset).unwrap();
                    }
                };

                write!(s, ";").unwrap();

                (s, d.lookup)
            }
            #[cfg(feature = "postgres")]
            SelectImpl::Postgres(mut d) => {
                let mut s = format!("SELECT{} ", if d.distinct { " DISTINCT" } else { "" });

                let column_len = d.resulting_columns.len();
                for (idx, column) in d.resulting_columns.iter().enumerate() {
                    column.build(&mut s);

                    if idx != column_len - 1 {
                        write!(s, ", ").unwrap();
                    }
                }

                write!(s, " FROM \"{}\"", d.from_clause).unwrap();

                for x in d.join_tables {
                    write!(s, " ").unwrap();
                    x.build(&mut s, &mut d.lookup);
                }

                if let Some(c) = d.where_clause {
                    write!(s, " WHERE {}", c.build(DBImpl::Postgres, &mut d.lookup)).unwrap()
                };

                if !d.order_by_clause.is_empty() {
                    write!(s, " ORDER BY ").unwrap();

                    let order_by_len = d.order_by_clause.len();
                    for (idx, entry) in d.order_by_clause.iter().enumerate() {
                        write!(
                            s,
                            "\"{}\"{}",
                            entry.column_name,
                            match entry.ordering {
                                Ordering::Asc => "",
                                Ordering::Desc => " DESC",
                            }
                        )
                        .unwrap();

                        if idx != order_by_len - 1 {
                            write!(s, ", ").unwrap();
                        }
                    }
                };

                if let Some(limit) = d.limit {
                    write!(s, " LIMIT {}", limit).unwrap();
                    if let Some(offset) = d.offset {
                        write!(s, " OFFSET {}", offset).unwrap();
                    }
                };

                write!(s, ";").unwrap();

                (s, d.lookup)
            }
        }
    }
}
