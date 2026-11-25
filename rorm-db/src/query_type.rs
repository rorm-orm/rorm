use crate::executor::{FetchAll, FetchGeneric, FetchOne, FetchOptional, FetchStream};
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
    type Result<'exe>: From<FetchGeneric<'exe>>;

    type LimitOrOffset;
    fn get_limit_clause(input: Option<Self::LimitOrOffset>) -> Option<LimitClause>;
}

type Offset = u64;

impl QueryType for Stream {
    type Result<'exe> = FetchStream<'exe>;

    type LimitOrOffset = LimitClause;

    fn get_limit_clause(limit: Option<Self::LimitOrOffset>) -> Option<LimitClause> {
        limit
    }
}

impl QueryType for All {
    type Result<'exe> = FetchAll<'exe>;

    type LimitOrOffset = LimitClause;

    fn get_limit_clause(limit: Option<Self::LimitOrOffset>) -> Option<LimitClause> {
        limit
    }
}

impl QueryType for Optional {
    type Result<'exe> = FetchOptional<'exe>;

    type LimitOrOffset = Offset;

    fn get_limit_clause(offset: Option<Self::LimitOrOffset>) -> Option<LimitClause> {
        Some(LimitClause { limit: 1, offset })
    }
}

impl QueryType for One {
    type Result<'exe> = FetchOne<'exe>;

    type LimitOrOffset = Offset;

    fn get_limit_clause(offset: Option<Self::LimitOrOffset>) -> Option<LimitClause> {
        Some(LimitClause { limit: 1, offset })
    }
}
