use crate::executor::{FetchAll, FetchGeneric, FetchOne, FetchOptional, FetchStream};
use rorm_sql::limit_clause::LimitClause;

pub trait QueryType<'exe>: From<FetchGeneric<'exe>> {
    type LimitOrOffset;
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
