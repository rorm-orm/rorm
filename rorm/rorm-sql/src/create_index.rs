use anyhow::anyhow;

use crate::DBImpl;

/**
Representation of a create index operation
*/
pub struct SQLCreateIndex {
    pub(crate) dialect: DBImpl,
    pub(crate) name: String,
    pub(crate) table_name: String,
    pub(crate) unique: bool,
    pub(crate) if_not_exists: bool,
    pub(crate) columns: Vec<String>,
    pub(crate) condition: Option<String>,
}

impl SQLCreateIndex {
    /**
    Creates a unique index.
        Null values are considered different from all other null values.
    */
    pub fn unique(mut self) -> Self {
        self.unique = true;
        return self;
    }

    /**
    Creates the index only if it doesn't exist yet.
    */
    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        return self;
    }

    /**
    Adds a column to the index.
    */
    pub fn add_column(mut self, column: &str) -> Self {
        self.columns.push(column.to_string());
        return self;
    }

    /**
    Sets the condition to apply. This will build a partial index.
    */
    pub fn set_condition(mut self, condition: &str) -> Self {
        self.condition = Some(condition.to_string());
        return self;
    }

    /**
    This method is used to build the create index operation
    */
    pub fn build(self) -> anyhow::Result<String> {
        if self.columns.len() == 0 {
            return Err(anyhow!(
                "Couldn't create index on {}: Missing column(s) to create the index on",
                self.table_name
            ));
        }

        Ok(match self.dialect {
            DBImpl::SQLite => format!(
                "CREATE {} INDEX {} {} ON {} ({}) {};",
                if self.unique { "UNIQUE" } else { "" },
                if self.if_not_exists {
                    "IF NOT EXISTS"
                } else {
                    ""
                },
                self.name,
                self.table_name,
                self.columns.join(","),
                match self.condition {
                    None => "".to_string(),
                    Some(s) => s,
                }
            ),
            _ => todo!("Not implemented yet!")
        })
    }
}
