use std::fmt::Write;
use std::fmt::{Display, Formatter};

use crate::conditional::{BuildCondition, Condition};
use crate::value::Value;
use crate::DBImpl;

/**
Definition of all available Join types
*/
#[derive(Copy, Clone, Debug)]
pub enum JoinType {
    /// Normal join operation.
    ///
    /// Equivalent to INNER JOIN
    Join,
    /// Cartesian product of the tables
    CrossJoin,
    /// Given:
    /// T1 LEFT JOIN T2 ON ..
    ///
    /// First, an inner join is performed.
    /// Then, for each row in T1 that does not satisfy the join condition with any row in T2,
    /// a joined row is added with null values in columns of T2.
    LeftJoin,
    /// Given:
    /// T1 RIGHT JOIN T2 ON ..
    ///
    /// First, an inner join is performed.
    /// Then, for each row in T2 that does not satisfy the join condition with any row in T1,
    /// a joined row is added with null values in columns of T1.
    RightJoin,
    /// Given:
    /// T1 FULL JOIN T2 ON ..
    ///
    /// First, an inner join is performed.
    /// Then, for each row in T2 that does not satisfy the join condition with any row in T1,
    /// a joined row is added with null values in columns of T1.
    /// Also, for each row in T1 that does not satisfy the join condition with any row in T2,
    /// a joined row is added with null values in columns of T2.
    FullJoin,
}

impl Display for JoinType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            JoinType::Join => write!(f, "JOIN"),
            JoinType::CrossJoin => write!(f, "CROSS JOIN"),
            JoinType::LeftJoin => write!(f, "LEFT JOIN"),
            JoinType::RightJoin => write!(f, "RIGHT JOIN"),
            JoinType::FullJoin => write!(f, "FULL JOIN"),
        }
    }
}

/**
Trait representing a join table builder.
 */
pub trait JoinTable<'post_query> {
    /**
    Method to build a join table expression.

    **Parameter**:
    - `s`: Mutable reference to String to write to.
    - `lookup`: List of values for bind parameter.
    */
    fn build(self, s: &mut String, lookup: &mut Vec<Value<'post_query>>);
}

/**
Data of a JOIN expression.
*/
#[derive(Debug, Copy, Clone)]
pub struct JoinTableData<'until_build, 'post_query> {
    /// Type of the join operation
    pub join_type: JoinType,
    /// Name of the join table
    pub table_name: &'until_build str,
    /// Alias for the join table
    pub join_alias: &'until_build str,
    /// Condition to apply the join on
    pub join_condition: &'until_build Condition<'post_query>,
}

/**
Representation of the JOIN expression

Should only be constructed via [DBImpl::join_table].
*/
#[derive(Debug, Copy, Clone)]
pub enum JoinTableImpl<'until_build, 'post_query> {
    /**
    SQLite representation of a JOIN expression.
     */
    #[cfg(feature = "sqlite")]
    SQLite(JoinTableData<'until_build, 'post_query>),
    /**
    MySQL representation of a JOIN expression.
     */
    #[cfg(feature = "mysql")]
    MySQL(JoinTableData<'until_build, 'post_query>),
    /**
    Postgres representation of a JOIN expression.
     */
    #[cfg(feature = "postgres")]
    Postgres(JoinTableData<'until_build, 'post_query>),
}

impl<'until_build, 'post_query> JoinTable<'post_query>
    for JoinTableImpl<'until_build, 'post_query>
{
    fn build(self, s: &mut String, lookup: &mut Vec<Value<'post_query>>) {
        match self {
            #[cfg(feature = "sqlite")]
            JoinTableImpl::SQLite(d) => write!(
                s,
                "{} {} AS {} ON {}",
                d.join_type,
                d.table_name,
                d.join_alias,
                d.join_condition.build(DBImpl::SQLite, lookup)
            )
            .unwrap(),
            #[cfg(feature = "mysql")]
            JoinTableImpl::MySQL(d) => write!(
                s,
                "{} {} AS {} ON {}",
                d.join_type,
                d.table_name,
                d.join_alias,
                d.join_condition.build(DBImpl::MySQL, lookup)
            )
            .unwrap(),
            #[cfg(feature = "postgres")]
            JoinTableImpl::Postgres(d) => write!(
                s,
                "{} \"{}\" AS {} ON {}",
                d.join_type,
                d.table_name,
                d.join_alias,
                d.join_condition.build(DBImpl::Postgres, lookup)
            )
            .unwrap(),
        }
    }
}
