/**
Representation of a limit / offset clause in SQL.
 */
#[derive(Debug, Clone, Copy)]
pub struct LimitClause {
    /// Limit to set to
    pub limit: u64,
    /// Optional offset to append.
    pub offset: Option<u64>,
}
