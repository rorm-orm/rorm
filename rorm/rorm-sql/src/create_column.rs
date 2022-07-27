use crate::create_trigger::trigger_annotation_to_trigger;
use anyhow::Context;
use rorm_common::imr::{Annotation, DbType, DefaultValue};

use crate::DBImpl;

/**
Representation of an annotation
 */
pub struct SQLAnnotation {
    pub(crate) annotation: Annotation,
}

impl SQLAnnotation {
    /**
    Converts the struct into the String for the given dialect.

    `dialect`: [&DBImpl]: dialect to use
     */
    pub fn build(&self, dialect: DBImpl) -> anyhow::Result<String> {
        match dialect {
            DBImpl::SQLite => {
                return Ok(match &self.annotation {
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
                        _ => "".to_string(),
                    },
                    Annotation::NotNull => "NOT NULL".to_string(),
                    Annotation::PrimaryKey => "PRIMARY KEY".to_string(),
                    Annotation::Unique => "UNIQUE".to_string(),
                    _ => "".to_string(),
                });
            }
        }
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
    pub fn build(self) -> anyhow::Result<(String, Vec<String>)> {
        match self.dialect {
            DBImpl::SQLite => {
                let db_type = match self.data_type {
                    DbType::VarChar
                    | DbType::VarBinary
                    | DbType::Date
                    | DbType::Datetime
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
                    | DbType::UInt64
                    | DbType::Boolean => "INTEGER",
                    DbType::Float | DbType::Double => "REAL",
                    _ => "",
                };

                let mut annotations = vec![];
                let mut trigger = vec![];
                for annotation in &self.annotations {
                    annotations.push(
                        annotation.build(DBImpl::SQLite).with_context(|| {
                            format!("Error while building column {}", self.name)
                        })?,
                    );

                    // If annotation requires a trigger, create those
                    trigger.extend(trigger_annotation_to_trigger(
                        DBImpl::SQLite,
                        &annotation.annotation,
                        self.table_name.as_str(),
                        self.name.as_str(),
                    )?)
                }

                return Ok((
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
                ));
            }
        }
    }
}
