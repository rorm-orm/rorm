use crate::conditional::BuildCondition;
use crate::{conditional, value, DBImpl};

/**
The representation of a FROM clause
*/
pub enum SQLSelectFrom {}

/**
The representation of a select query.
*/
pub struct SQLSelect<'until_build, 'post_query> {
    pub(crate) dialect: DBImpl,
    pub(crate) resulting_columns: &'until_build [&'until_build str],
    pub(crate) limit: Option<u64>,
    pub(crate) offset: Option<u64>,
    pub(crate) from_clause: String,
    pub(crate) where_clause: Option<&'until_build conditional::Condition<'post_query>>,
    pub(crate) distinct: bool,
    pub(crate) lookup: Vec<value::Value<'post_query>>,
}

impl<'until_build, 'post_query> SQLSelect<'until_build, 'post_query> {
    /**
    Set a limit to the resulting rows.
    */
    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        return self;
    }

    /**
    Set the offset to apply to the resulting rows.
    */
    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        return self;
    }

    /**
    Only retrieve distinct rows.
    */
    pub fn distinct(mut self) -> Self {
        self.distinct = true;
        return self;
    }

    /**
    Set a where clause to the query.
    */
    pub fn where_clause(
        mut self,
        where_clause: &'until_build conditional::Condition<'post_query>,
    ) -> Self {
        self.where_clause = Some(where_clause);
        return self;
    }

    /**
    Build the select query
    */
    pub fn build(mut self) -> (String, Vec<value::Value<'post_query>>) {
        (
            format!(
                "SELECT {} {} FROM {} {};",
                if self.distinct { "DISTINCT" } else { "" },
                self.resulting_columns.join(", "),
                self.from_clause,
                match self.where_clause {
                    None => {
                        "".to_string()
                    }
                    Some(condition) => {
                        format!("WHERE {}", condition.build(&mut self.lookup))
                    }
                },
            ),
            self.lookup,
        )
    }
}
