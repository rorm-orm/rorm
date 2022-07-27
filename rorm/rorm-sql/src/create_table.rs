use anyhow::Context;

use crate::{DBImpl, SQLCreateColumn};

pub struct SQLCreateTable {
    pub(crate) dialect: DBImpl,
    pub(crate) name: String,
    pub(crate) columns: Vec<SQLCreateColumn>,
    pub(crate) if_not_exists: bool,
}

impl SQLCreateTable {
    /**
    Add a column to the table.
    */
    pub fn add_column(mut self, column: SQLCreateColumn) -> Self {
        self.columns.push(column);
        return self;
    }

    /**
    Sets the IF NOT EXISTS trait on the table
    */
    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        return self;
    }

    /**
    This method is used to convert the current state for the given dialect in a [String].
    */
    pub fn build(self) -> anyhow::Result<String> {
        return match self.dialect {
            DBImpl::SQLite => {
                let mut columns = vec![];
                let mut trigger = vec![];
                for column in self.columns {
                    let (s, c_trigger) = column.build().with_context(|| {
                        format!("Error while building CREATE TABLE {}", self.name)
                    })?;
                    columns.push(s);

                    trigger.extend(c_trigger);
                }

                Ok(format!(
                    r#"CREATE TABLE{} {} ({}) STRICT;{}"#,
                    if self.if_not_exists {
                        " IF NOT EXISTS"
                    } else {
                        ""
                    },
                    self.name,
                    columns.join(","),
                    trigger.join(" "),
                ))
            }
        };
    }
}
