use crate::conditional::{BuildCondition, Condition};
use crate::Value;

/**
Trait representing a select builder.
 */
pub trait Select<'until_build, 'post_query> {
    /**
    Set a limit to the resulting rows.
     */
    fn limit(self, limit: u64) -> Self;

    /**
    Set the offset to apply to the resulting rows.
     */
    fn offset(self, offset: u64) -> Self;

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
    pub(crate) resulting_columns: &'until_build [&'until_build str],
    pub(crate) limit: Option<u64>,
    pub(crate) offset: Option<u64>,
    pub(crate) from_clause: &'until_build str,
    pub(crate) where_clause: Option<&'until_build Condition<'post_query>>,
    pub(crate) distinct: bool,
    pub(crate) lookup: Vec<Value<'post_query>>,
}

/**
Implementation of the [Select] trait for the different implementations
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
    fn limit(mut self, limit: u64) -> Self {
        match self {
            #[cfg(feature = "sqlite")]
            SelectImpl::SQLite(ref mut d) => d.limit = Some(limit),
            #[cfg(feature = "mysql")]
            SelectImpl::MySQL(ref mut d) => d.limit = Some(limit),
            #[cfg(feature = "postgres")]
            SelectImpl::Postgres(ref mut d) => d.limit = Some(limit),
        };
        self
    }

    fn offset(mut self, offset: u64) -> Self {
        match self {
            #[cfg(feature = "sqlite")]
            SelectImpl::SQLite(ref mut d) => d.offset = Some(offset),
            #[cfg(feature = "mysql")]
            SelectImpl::MySQL(ref mut d) => d.offset = Some(offset),
            #[cfg(feature = "postgres")]
            SelectImpl::Postgres(ref mut d) => d.offset = Some(offset),
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
            SelectImpl::SQLite(mut d) => (
                format!(
                    "SELECT {} {} FROM {} {};",
                    if d.distinct { "DISTINCT" } else { "" },
                    d.resulting_columns.join(", "),
                    d.from_clause,
                    match d.where_clause {
                        None => {
                            "".to_string()
                        }
                        Some(condition) => {
                            format!("WHERE {}", condition.build(&mut d.lookup))
                        }
                    },
                ),
                d.lookup,
            ),
            #[cfg(feature = "mysql")]
            SelectImpl::MySQL(mut d) => (
                format!(
                    "SELECT {} {} FROM {} {};",
                    if d.distinct { "DISTINCT" } else { "" },
                    d.resulting_columns.join(", "),
                    d.from_clause,
                    match d.where_clause {
                        None => {
                            "".to_string()
                        }
                        Some(condition) => {
                            format!("WHERE {}", condition.build(&mut d.lookup))
                        }
                    },
                ),
                d.lookup,
            ),
            #[cfg(feature = "postgres")]
            SelectImpl::Postgres(mut d) => (
                format!(
                    "SELECT {} {} FROM {} {};",
                    if d.distinct { "DISTINCT" } else { "" },
                    d.resulting_columns.join(", "),
                    format!("\"{}\"", d.from_clause),
                    match d.where_clause {
                        None => {
                            "".to_string()
                        }
                        Some(condition) => {
                            format!("WHERE {}", condition.build(&mut d.lookup))
                        }
                    },
                ),
                d.lookup,
            ),
        }
    }
}
