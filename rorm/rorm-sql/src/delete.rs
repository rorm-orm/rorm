use std::fmt::Write;

use crate::conditional::{BuildCondition, Condition};
use crate::Value;

/**
Trait representing a delete builder.
*/
pub trait Delete<'until_build, 'post_query>: Sized {
    /**
    Adds the a [Condition] to the delete query.

    **Parameter**:
    - `condition`: Condition to apply to the delete operation
     */
    fn where_clause(self, condition: &'until_build Condition<'post_query>) -> Self;

    /**
    Build the delete operation.

    **Returns**:
    - SQL query string
    - List of [Value] parameters to bind to the query.
    */
    fn build(self) -> (String, Vec<Value<'post_query>>);
}

/**
SQLite representation of the DELETE operation.
*/
#[derive(Debug)]
pub struct DeleteData<'until_build, 'post_query> {
    pub(crate) model: &'until_build str,
    pub(crate) lookup: Vec<Value<'post_query>>,
    pub(crate) where_clause: Option<&'until_build Condition<'post_query>>,
}

/**
Implementation of the [Delete] trait for the different implementations
*/
#[derive(Debug)]
pub enum DeleteImpl<'until_build, 'post_query> {
    /**
    SQLite representation of the DELETE operation.
     */
    #[cfg(feature = "sqlite")]
    SQLite(DeleteData<'until_build, 'post_query>),
    /**
    MySQL representation of the DELETE operation.
     */
    #[cfg(feature = "mysql")]
    MySQL(DeleteData<'until_build, 'post_query>),
    /**
    Postgres representation of the DELETE operation.
     */
    #[cfg(feature = "postgres")]
    Postgres(DeleteData<'until_build, 'post_query>),
}

impl<'until_build, 'post_query> Delete<'until_build, 'post_query>
    for DeleteImpl<'until_build, 'post_query>
{
    fn where_clause(mut self, condition: &'until_build Condition<'post_query>) -> Self {
        match self {
            #[cfg(feature = "sqlite")]
            DeleteImpl::SQLite(ref mut data) => data.where_clause = Some(condition),
            #[cfg(feature = "mysql")]
            DeleteImpl::MySQL(ref mut data) => data.where_clause = Some(condition),
            #[cfg(feature = "postgres")]
            DeleteImpl::Postgres(ref mut data) => data.where_clause = Some(condition),
        };
        self
    }

    fn build(self) -> (String, Vec<Value<'post_query>>) {
        match self {
            #[cfg(feature = "sqlite")]
            DeleteImpl::SQLite(mut d) => {
                let mut s = format!("DELETE FROM {} ", d.model);

                if d.where_clause.is_some() {
                    write!(s, "WHERE {} ", d.where_clause.unwrap().build(&mut d.lookup)).unwrap();
                }

                write!(s, ";").unwrap();
                (s, d.lookup)
            }
            #[cfg(feature = "mysql")]
            DeleteImpl::MySQL(mut d) => {
                let mut s = format!("DELETE FROM {} ", d.model);

                if d.where_clause.is_some() {
                    write!(s, "WHERE {} ", d.where_clause.unwrap().build(&mut d.lookup)).unwrap();
                }

                write!(s, ";").unwrap();
                (s, d.lookup)
            }
            #[cfg(feature = "postgres")]
            DeleteImpl::Postgres(mut d) => {
                let mut s = format!("DELETE FROM \"{}\" ", d.model);

                if d.where_clause.is_some() {
                    write!(s, "WHERE {} ", d.where_clause.unwrap().build(&mut d.lookup)).unwrap();
                }

                write!(s, ";").unwrap();
                (s, d.lookup)
            }
        }
    }
}
