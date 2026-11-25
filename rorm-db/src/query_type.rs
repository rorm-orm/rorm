use crate::executor::{FetchAll, FetchGeneric, FetchOne, FetchOptional, FetchStream};
use rorm_sql::limit_clause::LimitClause;

/// Defines how [`query`](crate::database::query) returns its resulting rows.
///
/// This trait is implemented on the following structs:
///
/// - [`FetchOptional`] retrieves an optional row
/// - [`FetchOne`] retrieves a single row
/// - [`FetchStream`] retrieves many rows in a stream
/// - [`FetchAll`] retrieves many rows in a vector
pub trait QueryType<'exe>: From<FetchGeneric<'exe>> {
    #[doc(hidden)]
    type LimitOrOffset;

    #[doc(hidden)]
    fn get_limit_clause(input: Option<Self::LimitOrOffset>) -> Option<LimitClause>;
}

type Offset = u64;

impl<'exe> QueryType<'exe> for FetchStream<'exe> {
    type LimitOrOffset = LimitClause;

    fn get_limit_clause(limit: Option<Self::LimitOrOffset>) -> Option<LimitClause> {
        limit
    }
}

impl<'exe> QueryType<'exe> for FetchAll<'exe> {
    type LimitOrOffset = LimitClause;

    fn get_limit_clause(limit: Option<Self::LimitOrOffset>) -> Option<LimitClause> {
        limit
    }
}

impl<'exe> QueryType<'exe> for FetchOptional<'exe> {
    type LimitOrOffset = Offset;

    fn get_limit_clause(offset: Option<Self::LimitOrOffset>) -> Option<LimitClause> {
        Some(LimitClause { limit: 1, offset })
    }
}

impl<'exe> QueryType<'exe> for FetchOne<'exe> {
    type LimitOrOffset = Offset;

    fn get_limit_clause(offset: Option<Self::LimitOrOffset>) -> Option<LimitClause> {
        Some(LimitClause { limit: 1, offset })
    }
}
