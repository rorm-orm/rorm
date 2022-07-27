use crate::{DBImpl, SQLCreateColumn};

/**
Representation of operations to execute in the context of an ALTER TABLE statement.
*/
pub enum SQLAlterTableOperation {
    /// Use this operation to rename a table
    RenameTo { name: String },
    /// Use this operation to rename a column within a table
    RenameColumnTo {
        column_name: String,
        new_column_name: String,
    },
    /// Use this operation to add a column to an existing table.
    /// Can be generated by using [crate::create_table::SQLCreateColumn]
    AddColumn { operation: SQLCreateColumn },
    /// Use this operation to drop an existing column.
    DropColumn { name: String },
}

impl SQLAlterTableOperation {
    fn build(self) -> anyhow::Result<(String, Option<Vec<String>>)> {
        Ok(match self {
            SQLAlterTableOperation::RenameTo { name } => (format!("RENAME TO {}", name), None),
            SQLAlterTableOperation::RenameColumnTo {
                column_name,
                new_column_name,
            } => (
                format!("RENAME COLUMN {} TO {}", column_name, new_column_name),
                None,
            ),
            SQLAlterTableOperation::AddColumn { operation } => {
                let (sql, annotation) = operation.build()?;
                (format!("ADD COLUMN {}", sql), Some(annotation))
            }
            SQLAlterTableOperation::DropColumn { name } => (format!("DROP COLUMN {}", name), None),
        })
    }
}

/**
Representation of an ALTER TABLE statement.
*/
pub struct SQLAlterTable {
    pub(crate) dialect: DBImpl,
    /// Name of the table to operate on
    pub(crate) name: String,
    /// Operation to execute
    pub(crate) operation: SQLAlterTableOperation,
}

impl SQLAlterTable {
    /**
    This method is used to build the alter table statement.
    */
    pub fn build(self) -> anyhow::Result<String> {
        Ok(match self.dialect {
            DBImpl::SQLite => {
                let (sql, trigger) = self.operation.build()?;
                format!(
                    "ALTER TABLE {} {};{}",
                    self.name.as_str(),
                    sql,
                    match trigger {
                        None => {
                            "".to_string()
                        }
                        Some(t) => {
                            t.join(" ")
                        }
                    }
                )
            }
        })
    }
}
