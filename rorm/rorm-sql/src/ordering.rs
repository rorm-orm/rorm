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
    /// Column to apply the ordering to
    pub column_name: &'until_build str,
}
