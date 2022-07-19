use anyhow::Context;
use rorm_common::imr::{Annotation, DbType, DefaultValue};

use crate::{DBImpl, SQLCreateTriggerOperation, SQLCreateTriggerPointInTime};

/**
Representation of an annotation
*/
pub struct SQLAnnotation {
    annotation: Annotation,
}

impl SQLAnnotation {
    /**
    Converts the struct into the String for the given dialect.

    `db`: [&DBImpl]: dialect to use
    */
    pub fn build(&self, db: &DBImpl) -> anyhow::Result<String> {
        match db {
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
    name: String,
    data_type: DbType,
    annotations: Vec<SQLAnnotation>,
}

impl SQLCreateColumn {
    pub fn build(self, db: &DBImpl) -> anyhow::Result<(String, Vec<Annotation>)> {
        match db {
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
                        annotation.build(db).with_context(|| {
                            format!("Error while building column {}", self.name)
                        })?,
                    );

                    // If annotation requires a trigger, create those
                    match annotation.annotation {
                        Annotation::AutoUpdateTime => {
                            trigger.push(annotation.annotation.clone());
                        }
                        Annotation::Index(_) => {}
                        Annotation::PrimaryKey => {}
                        _ => {}
                    }
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

pub struct SQLCreateTable {
    pub(crate) dialect: DBImpl,
    pub(crate) name: String,
    pub(crate) columns: Vec<SQLCreateColumn>,
    pub(crate) if_not_exists: bool,
}

impl SQLCreateTable {
    /**
    Add a column to the table.

    - `name`: [&str]: Name of the column
    - `db_type`: [DbType]: Datatype of the column
    - `annotations`: [Vec<Annotation>]: List of annotations to use on the column
    */
    pub fn add_column(mut self, name: &str, db_type: DbType, annotations: Vec<Annotation>) -> Self {
        self.columns.push(SQLCreateColumn {
            name: name.to_string(),
            data_type: db_type,
            annotations: annotations
                .into_iter()
                .map(|x| SQLAnnotation { annotation: x })
                .collect(),
        });
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
        match self.dialect {
            DBImpl::SQLite => {
                let mut columns = vec![];
                let mut trigger = vec![];
                for column in self.columns {
                    let c_name = column.name.clone();

                    let (s, annotations) = column.build(&self.dialect).with_context(|| {
                        format!("Error while building CREATE TABLE {}", self.name)
                    })?;
                    columns.push(s);

                    for x in annotations {
                        match x {
                            Annotation::AutoUpdateTime => {
                                let update_statement = format!(
                                    "UPDATE {} SET {} = CURRENT_TIMESTAMP WHERE id = NEW.id;",
                                    self.name, c_name
                                );

                                trigger.push(
                                    DBImpl::SQLite
                                        .create_trigger(
                                            format!(
                                                "{}_{}_auto_update_time_insert",
                                                &self.name, &c_name
                                            ).as_str(),
                                            self.name.as_str(),
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
                                                &c_name,
                                                &self.name,
                                            )
                                        )?
                                );
                                trigger.push(
                                    DBImpl::SQLite.create_trigger(
                                        format!(
                                            "{}_{}_auto_update_time_update",
                                            &self.name,
                                            &c_name
                                        ).as_str(),
                                        self.name.as_str(),
                                        Some(SQLCreateTriggerPointInTime::After),
                                        SQLCreateTriggerOperation::Update {columns: None},
                                    )
                                        .if_not_exists().
                                        add_statement(
                                            update_statement.clone(),
                                        )
                                        .build()
                                        .with_context(
                                            || format!(
                                                "Couldn't create update trigger for auto_update_time annotation on field {} in table {}",
                                                &c_name,
                                                &self.name
                                            )
                                        )?
                                )
                            }
                            _ => {}
                        }
                    }
                }

                return Ok(format!(
                    r#"CREATE TABLE{} {} ({}) STRICT;{}"#,
                    if self.if_not_exists {
                        " IF NOT EXISTS"
                    } else {
                        ""
                    },
                    self.name,
                    columns.join(","),
                    trigger.join(" "),
                ));
            }
        }
    }
}
