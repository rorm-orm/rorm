/**
All supported orderings
 */
#[derive(Debug, Copy, Clone)]
pub enum Ordering {
    /// Ascending ordering
    Asc,
    /// Descending ordering
    Desc,
}

/**
Representation of an entry in a ORDER BY expression
*/
#[derive(Debug, Copy, Clone)]
pub struct OrderByEntry<'until_build> {
    /// Ordering to apply
    pub ordering: Ordering,
    /// Optional table name
    pub table_name: Option<&'until_build str>,
    /// Column to apply the ordering to
    pub column_name: &'until_build str,
}
