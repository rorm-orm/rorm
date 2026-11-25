use crate::executor::{Query, QueryAll, QueryOne, QueryOptional, QueryStream};
use rorm_sql::limit_clause::LimitClause;

/// Marker for [`query`](crate::database::query) to return all rows in a stream
pub struct Stream;

/// Marker for [`query`](crate::database::query) to return all rows in a vec
pub struct All;

/// Marker for [`query`](crate::database::query) to return at most one row
pub struct Optional;

/// Marker for [`query`](crate::database::query) to return exactly one row
pub struct One;

pub trait QueryType {
    type Result<'exe>;
    fn execute(query: Query<'_>) -> Self::Result<'_>;

    type LimitOrOffset;
    fn get_limit_clause(input: Option<Self::LimitOrOffset>) -> Option<LimitClause>;
}

type Offset = u64;

impl QueryType for Stream {
    type Result<'exe> = QueryStream<'exe>;

    fn execute(query: Query<'_>) -> Self::Result<'_> {
        query.stream()
    }

    type LimitOrOffset = LimitClause;

    fn get_limit_clause(limit: Option<Self::LimitOrOffset>) -> Option<LimitClause> {
        limit
    }
}

impl QueryType for All {
    type Result<'exe> = QueryAll<'exe>;
    fn execute(query: Query<'_>) -> Self::Result<'_> {
        query.all()
    }

    type LimitOrOffset = LimitClause;

    fn get_limit_clause(limit: Option<Self::LimitOrOffset>) -> Option<LimitClause> {
        limit
    }
}

impl QueryType for Optional {
    type Result<'exe> = QueryOptional<'exe>;
    fn execute(query: Query<'_>) -> Self::Result<'_> {
        query.optional()
    }

    type LimitOrOffset = Offset;

    fn get_limit_clause(offset: Option<Self::LimitOrOffset>) -> Option<LimitClause> {
        Some(LimitClause { limit: 1, offset })
    }
}

impl QueryType for One {
    type Result<'exe> = QueryOne<'exe>;
    fn execute(query: Query<'_>) -> Self::Result<'_> {
        query.one()
    }

    type LimitOrOffset = Offset;

    fn get_limit_clause(offset: Option<Self::LimitOrOffset>) -> Option<LimitClause> {
        Some(LimitClause { limit: 1, offset })
    }
}
