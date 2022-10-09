use rorm_declaration::imr::DefaultValue;

use crate::create_trigger::trigger_annotation_to_trigger;
use crate::{value, Annotation, DBImpl, DbType};

#[cfg(feature = "sqlite")]
use crate::sqlite;

/**
Representation of an annotation
 */
pub struct SQLAnnotation<'post_build> {
    pub(crate) annotation: &'post_build Annotation,
}

impl<'post_build> SQLAnnotation<'post_build> {
    /**
    Converts the struct into the String for the given dialect.

    `dialect`: [crate::DBImpl]: dialect to use
     */
    pub fn build(&self, dialect: DBImpl) -> String {
        match dialect {
            DBImpl::SQLite => match &self.annotation {
                Annotation::AutoIncrement => "AUTOINCREMENT".to_string(),
                Annotation::AutoCreateTime => "DEFAULT CURRENT_TIMESTAMP".to_string(),
                Annotation::DefaultValue(d) => match d {
                    DefaultValue::String(s) => {
                        format!("DEFAULT {}", sqlite::fmt(s))
                    }
                    DefaultValue::Integer(i) => {
                        format!("DEFAULT {}", i)
                    }
                    DefaultValue::Float(f) => format!("DEFAULT {}", f),
                    DefaultValue::Boolean(b) => {
                        if *b {
                            "DEFAULT 1".to_string()
                        } else {
                            "DEFAULT 0".to_string()
                        }
                    }
                },
                Annotation::NotNull => "NOT NULL".to_string(),
                Annotation::PrimaryKey => "PRIMARY KEY".to_string(),
                Annotation::Unique => "UNIQUE".to_string(),
                _ => "".to_string(),
            },
            _ => todo!("Not implemented yet!"),
        }
    }
}

/**
Representation of the creation of a column
 */
pub struct SQLCreateColumn<'post_build> {
    pub(crate) dialect: DBImpl,
    pub(crate) name: String,
    pub(crate) table_name: String,
    pub(crate) data_type: DbType,
    pub(crate) annotations: Vec<SQLAnnotation<'post_build>>,
}

impl<'post_build> SQLCreateColumn<'post_build> {
    /**
    This method is used to build the statement to create a column
    */
    pub fn build(&self, trigger: &mut Vec<(String, Vec<value::Value<'post_build>>)>) -> String {
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

                format!(
                    "{} {} {}",
                    self.name,
                    db_type,
                    self.annotations
                        .iter()
                        .map(|x| {
                            trigger_annotation_to_trigger(
                                DBImpl::SQLite,
                                x.annotation,
                                self.table_name.as_str(),
                                self.name.as_str(),
                                trigger,
                            );
                            x.build(DBImpl::SQLite)
                        })
                        .collect::<Vec<String>>()
                        .join(" ")
                )
            }
            _ => todo!(""),
        }
    }
}
