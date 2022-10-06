use rorm_declaration::imr::DefaultValue;

use crate::create_trigger::trigger_annotation_to_trigger;
use crate::{Annotation, DBImpl, DbType};

/**
Representation of an annotation
 */
pub struct SQLAnnotation {
    pub(crate) annotation: Annotation,
}

impl SQLAnnotation {
    /**
    Converts the struct into the String for the given dialect.

    `dialect`: [crate::DBImpl]: dialect to use
     */
    pub fn build(&self, dialect: DBImpl) -> String {
        return match dialect {
            DBImpl::SQLite => match &self.annotation {
                Annotation::AutoIncrement => "AUTOINCREMENT".to_string(),
                Annotation::AutoCreateTime => "DEFAULT CURRENT_TIMESTAMP".to_string(),
                Annotation::DefaultValue(d) => match d {
                    DefaultValue::String(s) => format!("DEFAULT {}", s),
                    DefaultValue::Integer(i) => format!("DEFAULT {}", i),
                    DefaultValue::Float(f) => format!("DEFAULT {}", f),
                    DefaultValue::Boolean(b) => {
                        if *b {
                            "DEFAULT 1".to_string()
                        } else {
                            "DEFAULT 0".to_string()
                        }
                    }
                    _ => {
                        todo!("not intended");
                    }
                },
                Annotation::NotNull => "NOT NULL".to_string(),
                Annotation::PrimaryKey => "PRIMARY KEY".to_string(),
                Annotation::Unique => "UNIQUE".to_string(),
                _ => "".to_string(),
            },
            _ => todo!("Not implemented yet!"),
        };
    }
}

/**
Representation of the creation of a column
 */
pub struct SQLCreateColumn {
    pub(crate) dialect: DBImpl,
    pub(crate) name: String,
    pub(crate) table_name: String,
    pub(crate) data_type: DbType,
    pub(crate) annotations: Vec<SQLAnnotation>,
}

impl SQLCreateColumn {
    /**
    This method is used to build the statement to create a column
    */
    pub fn build(self) -> (String, Vec<String>) {
        match self.dialect {
            DBImpl::SQLite => {
                let db_type = match self.data_type {
                    DbType::VarBinary => "BLOB",
                    DbType::VarChar
                    | DbType::Date
                    | DbType::DateTime
                    | DbType::Timestamp
                    | DbType::Time
                    | DbType::Choices
                    | DbType::Set => "TEXT",
                    DbType::Int8
                    | DbType::Int16
                    | DbType::Int32
                    | DbType::Int64
                    | DbType::UInt8
                    | DbType::UInt16
                    | DbType::UInt32
                    | DbType::Boolean => "INTEGER",
                    DbType::Float | DbType::Double => "REAL",
                };

                let mut annotations = vec![];
                let mut trigger = vec![];
                for annotation in &self.annotations {
                    annotations.push(annotation.build(DBImpl::SQLite));

                    // If annotation requires a trigger, create those
                    trigger.extend(trigger_annotation_to_trigger(
                        DBImpl::SQLite,
                        &annotation.annotation,
                        self.table_name.as_str(),
                        self.name.as_str(),
                    ))
                }

                (
                    format!(
                        "{} {}{}",
                        self.name,
                        db_type,
                        if annotations.len() > 0 {
                            format!(" {}", annotations.join(" "))
                        } else {
                            annotations.join(" ")
                        }
                    ),
                    trigger,
                )
            }
            _ => todo!(""),
        }
    }
}
