use std::fmt::{Display, Formatter};

#[cfg(any(feature = "postgres", feature = "sqlite"))]
use rorm_declaration::imr::Annotation;
#[cfg(feature = "sqlite")]
use rorm_declaration::imr::DbType;

#[cfg(feature = "sqlite")]
use crate::DBImpl;
#[cfg(any(feature = "sqlite", feature = "postgres"))]
use crate::Value;

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
        columns: Option<Vec<String>>,
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

// TODO: Make this more beautiful :D
#[cfg(feature = "postgres")]
pub(crate) fn trigger_annotation_to_trigger_postgres(
    annotation: &Annotation,
    table_name: &str,
    column_name: &str,
    statements: &mut Vec<(String, Vec<Value>)>,
) {
    match annotation {
        Annotation::AutoUpdateTime => {
            statements.push(
                (
                    format!(
                        "CREATE OR REPLACE FUNCTION {}_{}_auto_update_time_update_procedure() RETURNS TRIGGER AS $$ BEGIN NEW.{} = now(); RETURN NEW; END; $$ language 'plpgsql';",
                        table_name,
                        column_name,
                        column_name,
                    ),
                    vec![],
                )
            );
            statements.push((
                format!(
                    "DROP TRIGGER IF EXISTS {}_{}_auto_update_time_update ON \"{}\";",
                    table_name, column_name, table_name
                ),
                vec![],
            ));
            statements.push(
                (
                    format!(
                        "CREATE TRIGGER {}_{}_auto_update_time_update BEFORE UPDATE ON \"{}\" FOR EACH ROW WHEN (OLD IS DISTINCT FROM NEW) EXECUTE PROCEDURE {}_{}_auto_update_time_update_procedure();",
                        table_name,
                        column_name,
                        table_name,
                        table_name,
                        column_name,
                    ),
                    vec![],
                )
            );
        }
        _ => {}
    }
}

// TODO: Make this more beautiful :D
#[cfg(feature = "sqlite")]
pub(crate) fn trigger_annotation_to_trigger_sqlite(
    annotation: &Annotation,
    db_type: &DbType,
    table_name: &str,
    column_name: &str,
    statements: &mut Vec<(String, Vec<Value>)>,
) {
    match annotation {
        Annotation::AutoUpdateTime => {
            let update_statement = format!(
                "UPDATE {} SET {} = {} WHERE ROWID = NEW.ROWID;",
                table_name,
                column_name,
                match db_type {
                    DbType::Date => "CURRENT_DATE",
                    DbType::DateTime => "CURRENT_TIMESTAMP",
                    DbType::Timestamp => "CURRENT_TIMESTAMP",
                    DbType::Time => "CURRENT_TIME",
                    _ => "",
                }
            );
            statements.push((
                DBImpl::SQLite
                    .create_trigger(
                        format!("{}_{}_auto_update_time", table_name, column_name).as_str(),
                        table_name,
                        Some(SQLCreateTriggerPointInTime::After),
                        SQLCreateTriggerOperation::Update { columns: None },
                    )
                    .for_each_row()
                    .if_not_exists()
                    .add_statement(update_statement.clone())
                    .build(),
                vec![],
            ))
        }
        _ => {}
    }
}

/**
Representation of a trigger.
*/
pub struct SQLCreateTrigger {
    pub(crate) name: String,
    pub(crate) table_name: String,
    pub(crate) if_not_exists: bool,
    pub(crate) point_in_time: Option<SQLCreateTriggerPointInTime>,
    pub(crate) operation: SQLCreateTriggerOperation,
    pub(crate) statements: Vec<String>,
    pub(crate) for_each_row: bool,
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
    Executes the given trigger statement for each row individually.
    */
    pub fn for_each_row(mut self) -> Self {
        self.for_each_row = true;
        self
    }

    /**
    Generate the resulting SQL string
    */
    pub fn build(self) -> String {
        format!(
            "CREATE TRIGGER {} {} {} {} ON {}{} BEGIN {} END;",
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
            match self.for_each_row {
                true => " FOR EACH ROW",
                false => "",
            },
            self.statements.join(" "),
        )
    }
}
