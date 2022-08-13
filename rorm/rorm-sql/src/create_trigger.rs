use std::fmt::{Display, Formatter};

use anyhow::{anyhow, Context};

use crate::{Annotation, DBImpl};

/**
Representation of a point in time definition of a create trigger statement
*/
pub enum SQLCreateTriggerPointInTime {
    /// Trigger AFTER operation
    After,
    /// Trigger BEFORE operation
    Before,
    /// Trigger INSTEAD OF operation
    InsteadOf,
}

impl Display for SQLCreateTriggerPointInTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SQLCreateTriggerPointInTime::After => write!(f, "AFTER"),
            SQLCreateTriggerPointInTime::Before => write!(f, "BEFORE"),
            SQLCreateTriggerPointInTime::InsteadOf => write!(f, "INSTEAD OF"),
        }
    }
}

/**
Representation of the operation to execute the trigger on
*/
pub enum SQLCreateTriggerOperation {
    /// Execute a DELETE operation
    Delete,
    /// Execute an INSERT operation
    Insert,
    /// Execute an UPDATE operation
    Update {
        /// Columns to update
        columns: Option<Vec<String>>
    },
}

impl Display for SQLCreateTriggerOperation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SQLCreateTriggerOperation::Delete => write!(f, "DELETE"),
            SQLCreateTriggerOperation::Insert => write!(f, "INSERT"),
            SQLCreateTriggerOperation::Update { columns: None } => write!(f, "UPDATE"),
            SQLCreateTriggerOperation::Update { columns: Some(c) } => {
                write!(f, "UPDATE OF {}", c.join(","))
            }
        }
    }
}

pub(crate) fn trigger_annotation_to_trigger(
    dialect: DBImpl,
    annotation: &Annotation,
    table_name: &str,
    column_name: &str,
) -> anyhow::Result<Vec<String>> {
    let mut trigger: Vec<String> = vec![];
    match dialect {
        DBImpl::SQLite => match annotation {
            Annotation::AutoUpdateTime => {
                let update_statement = format!(
                    "UPDATE {} SET {} = CURRENT_TIMESTAMP WHERE id = NEW.id;",
                    table_name, column_name
                );

                trigger.push(DBImpl::SQLite
                    .create_trigger(
                        format!(
                            "{}_{}_auto_update_time_insert",
                            table_name, column_name
                        ).as_str(),
                        table_name,
                        Some(SQLCreateTriggerPointInTime::After),
                        SQLCreateTriggerOperation::Insert,
                    ).if_not_exists()
                    .add_statement(
                        update_statement.clone(),
                    )
                    .build()
                    .with_context(
                        || format!(
                            "Couldn't create insert trigger for auto_update_time annotation on field {} in table {}",
                            column_name,
                            table_name,
                        )
                    )?);
                trigger.push(
                    DBImpl::SQLite.create_trigger(
                        format!(
                            "{}_{}_auto_update_time_update",
                            table_name,
                            column_name
                        ).as_str(),
                        table_name,
                        Some(SQLCreateTriggerPointInTime::After),
                        SQLCreateTriggerOperation::Update { columns: None },
                    )
                        .if_not_exists().
                        add_statement(
                            update_statement.clone(),
                        )
                        .build()
                        .with_context(
                            || format!(
                                "Couldn't create update trigger for auto_update_time annotation on field {} in table {}",
                                column_name,
                                table_name
                            )
                        )?
                )
            }
            _ => {}
        },
    };
    return Ok(trigger);
}

/**
Representation of a trigger.
*/
pub struct SQLCreateTrigger {
    pub(crate) dialect: DBImpl,
    pub(crate) name: String,
    pub(crate) table_name: String,
    pub(crate) if_not_exists: bool,
    pub(crate) point_in_time: Option<SQLCreateTriggerPointInTime>,
    pub(crate) operation: SQLCreateTriggerOperation,
    pub(crate) statements: Vec<String>,
}

impl SQLCreateTrigger {
    /**
    Create the trigger only, if it does not exists
    */
    pub fn if_not_exists(mut self) -> Self {
        self.if_not_exists = true;
        return self;
    }

    /**
    Adds a statement to a create trigger operation
    */
    pub fn add_statement(mut self, statement: String) -> Self {
        self.statements.push(statement);
        return self;
    }

    /**
    Generate the resulting SQL string
    */
    pub fn build(self) -> anyhow::Result<String> {
        return match self.dialect {
            DBImpl::SQLite => {
                if self.name == "" {
                    return Err(anyhow!("Name of the trigger must not empty"));
                }

                if self.table_name == "" {
                    return Err(anyhow!("Name of the table must not be empty"));
                }

                Ok(format!(
                    "CREATE TRIGGER {} {} {} {} ON {} BEGIN {} END;",
                    if self.if_not_exists {
                        "IF NOT EXISTS"
                    } else {
                        ""
                    },
                    self.name,
                    match self.point_in_time {
                        None => "".to_string(),
                        Some(s) => s.to_string(),
                    },
                    self.operation,
                    self.table_name,
                    self.statements.join(" "),
                ))
            }
        };
    }
}
