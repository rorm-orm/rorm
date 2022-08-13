use crate::DBImpl;

use std::collections::HashMap;

/**
The representation of a FROM clause
*/
pub enum SQLSelectFrom {}

/**
The representation of a select query.
*/
pub struct SQLSelect {
    pub(crate) dialect: DBImpl,
    pub(crate) resulting_columns: Vec<String>,
    pub(crate) limit: Option<u64>,
    pub(crate) offset: Option<u64>,
    pub(crate) from_clause: String,
    pub(crate) where_clause: Option<String>,
    pub(crate) distinct: bool,
}

impl SQLSelect {
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
    Add an identifier to the retrieving columns.

    matches:
    SELECT {columns} FROM ...;
    */
    pub fn add_column(mut self, col: String) -> Self {
        self.resulting_columns.push(col);
        return self;
    }

    /**
    Set a where clause to the query.
    */
    pub fn where_clause(mut self, where_clause: String) -> Self {
        self.where_clause = Some(where_clause);
        return self;
    }

    /**
    Build the select query
    */
    pub fn build(self) -> anyhow::Result<(String, HashMap<String, i64>)> {
        let lookup = HashMap::new();

        return match self.dialect {
            DBImpl::SQLite => Ok((
                format!(
                    "SELECT {} {} FROM {} {};",
                    if self.distinct { "DISTINCT" } else { "" },
                    self.resulting_columns.join(", "),
                    self.from_clause,
                    match self.where_clause {
                        None => {"".to_string()}
                        Some( where_clause ) => {format!("WHERE {}", where_clause)}
                    },
                ),
                lookup,
            )),
        };
    }
}
