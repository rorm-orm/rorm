use rorm_sql::limit_clause::LimitClause;

use crate::executor::{All, One, Optional, Stream};

type Offset = u64;

pub trait GetLimitClause {
    type LimitOrOffset;

    fn get_limit_clause(input: Option<Self::LimitOrOffset>) -> Option<LimitClause>;
}

impl GetLimitClause for Stream {
    type LimitOrOffset = LimitClause;

    fn get_limit_clause(limit: Option<Self::LimitOrOffset>) -> Option<LimitClause> {
        limit
    }
}

impl GetLimitClause for All {
    type LimitOrOffset = LimitClause;

    fn get_limit_clause(limit: Option<Self::LimitOrOffset>) -> Option<LimitClause> {
        limit
    }
}

impl GetLimitClause for Optional {
    type LimitOrOffset = Offset;

    fn get_limit_clause(offset: Option<Self::LimitOrOffset>) -> Option<LimitClause> {
        Some(LimitClause { limit: 1, offset })
    }
}

impl GetLimitClause for One {
    type LimitOrOffset = Offset;

    fn get_limit_clause(offset: Option<Self::LimitOrOffset>) -> Option<LimitClause> {
        Some(LimitClause { limit: 1, offset })
    }
}
