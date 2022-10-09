use std::fmt::Write;

use rorm_declaration::imr::DefaultValue;

use crate::create_trigger::trigger_annotation_to_trigger;
use crate::error::Error;
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
    pub fn build(&self, s: &mut String, dialect: DBImpl) {
        match &self.annotation {
            Annotation::AutoIncrement => match dialect {
                DBImpl::SQLite => write!(s, "AUTOINCREMENT").unwrap(),
                DBImpl::MySQL => write!(s, "AUTO_INCREMENT").unwrap(),
                _ => todo!("Not implemented yet!"),
            },
            Annotation::AutoCreateTime => write!(s, "DEFAULT CURRENT_TIMESTAMP").unwrap(),
            Annotation::AutoUpdateTime => match dialect {
                DBImpl::SQLite => {}
                DBImpl::MySQL => write!(s, "ON UPDATE CURRENT_TIMESTAMP").unwrap(),
                _ => todo!("Not implemented yet!"),
            },
            Annotation::DefaultValue(d) => match d {
                DefaultValue::String(dv) => match dialect {
                    #[cfg(feature = "sqlite")]
                    DBImpl::SQLite => write!(s, "DEFAULT {}", sqlite::fmt(dv)).unwrap(),
                    DBImpl::MySQL => write!(s, "DEFAULT QUOTE({})", dv).unwrap(),
                    _ => todo!("Not implemented yet!"),
                },
                DefaultValue::Integer(i) => write!(s, "DEFAULT {}", i).unwrap(),
                DefaultValue::Float(f) => write!(s, "DEFAULT {}", f).unwrap(),
                DefaultValue::Boolean(b) => {
                    if *b {
                        write!(s, "DEFAULT 1").unwrap();
                    } else {
                        write!(s, "DEFAULT 0").unwrap();
                    }
                }
            },
            Annotation::NotNull => write!(s, "NOT NULL").unwrap(),
            Annotation::PrimaryKey => write!(s, "PRIMARY KEY").unwrap(),
            Annotation::Unique => write!(s, "UNIQUE").unwrap(),
            _ => {}
        };
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
    pub fn build(
        &self,
        s: &mut String,
        trigger: &mut Vec<(String, Vec<value::Value<'post_build>>)>,
    ) -> Result<(), Error> {
        write!(s, "{} ", self.name).unwrap();

        match self.dialect {
            #[cfg(feature = "sqlite")]
            DBImpl::SQLite => match self.data_type {
                DbType::VarBinary => write!(s, "BLOB ").unwrap(),
                DbType::VarChar
                | DbType::Date
                | DbType::DateTime
                | DbType::Timestamp
                | DbType::Time
                | DbType::Choices => write!(s, "TEXT ").unwrap(),
                DbType::Int8
                | DbType::Int16
                | DbType::Int32
                | DbType::Int64
                | DbType::UInt8
                | DbType::UInt16
                | DbType::UInt32
                | DbType::Boolean => write!(s, "INTEGER ").unwrap(),
                DbType::Float | DbType::Double => write!(s, "REAL ").unwrap(),
            },
            DBImpl::MySQL => match self.data_type {
                DbType::VarChar => {
                    let a_opt = self
                        .annotations
                        .iter()
                        .filter(|x| x.annotation.eq_shallow(&Annotation::MaxLength(0)))
                        .next();

                    if let Some(a) = a_opt {
                        if let Annotation::MaxLength(max_length) = a.annotation {
                            write!(s, "VARCHAR({}) ", max_length).unwrap();
                        } else {
                            return Err(Error::SQLBuildError(
                                "VARCHAR must have a max_length annotation".to_string(),
                            ));
                        }
                    } else {
                        return Err(Error::SQLBuildError(
                            "VARCHAR must have a max_length annotation".to_string(),
                        ));
                    }
                }
                DbType::VarBinary => {
                    let a_opt = self
                        .annotations
                        .iter()
                        .filter(|x| x.annotation.eq_shallow(&Annotation::MaxLength(0)))
                        .next();

                    if let Some(a) = a_opt {
                        if let Annotation::MaxLength(max_length) = a.annotation {
                            write!(s, "VARBINARY({}) ", max_length).unwrap();
                        } else {
                            return Err(Error::SQLBuildError(
                                "VARBINARY must have a max_length annotation".to_string(),
                            ));
                        }
                    } else {
                        return Err(Error::SQLBuildError(
                            "VARBINARY must have a max_length annotation".to_string(),
                        ));
                    }
                }
                DbType::Int8 | DbType::UInt8 => write!(s, "TINYINT(255) ").unwrap(),
                DbType::Int16 | DbType::UInt16 => write!(s, "SMALLINT(255) ").unwrap(),
                DbType::Int32 | DbType::UInt32 => write!(s, "INT(255) ").unwrap(),
                DbType::Int64 => write!(s, "BIGINT(255) ").unwrap(),
                DbType::Float => write!(s, "FLOAT(24) ").unwrap(),
                DbType::Double => write!(s, "DOUBLE(53) ").unwrap(),
                DbType::Boolean => write!(s, "BOOL ").unwrap(),
                DbType::Date => write!(s, "DATE ").unwrap(),
                DbType::DateTime => write!(s, "DATETIME ").unwrap(),
                DbType::Timestamp => write!(s, "TIMESTAMP ").unwrap(),
                DbType::Time => write!(s, "TIME ").unwrap(),
                DbType::Choices => {
                    let a_opt = self
                        .annotations
                        .iter()
                        .filter(|x| {
                            x.annotation
                                .eq_shallow(&Annotation::Choices(Default::default()))
                        })
                        .next();

                    if let Some(a) = a_opt {
                        if let Annotation::Choices(values) = a.annotation {
                            write!(
                                s,
                                "VARCHAR({}) ",
                                values
                                    .iter()
                                    .map(|x| format!("QUOTE({})", x))
                                    .collect::<Vec<String>>()
                                    .join(", ")
                            )
                            .unwrap();
                        } else {
                            return Err(Error::SQLBuildError(
                                "VARCHAR must have a MaxLength annotation".to_string(),
                            ));
                        }
                    } else {
                        return Err(Error::SQLBuildError(
                            "VARCHAR must have a MaxLength annotation".to_string(),
                        ));
                    }
                }
            },
            _ => todo!("Not implemented yet!"),
        };

        for (idx, x) in self.annotations.iter().enumerate() {
            trigger_annotation_to_trigger(
                self.dialect,
                x.annotation,
                &self.table_name,
                &self.name,
                trigger,
            );
            x.build(s, self.dialect);
            if idx != self.annotations.len() - 1 {
                write!(s, " ").unwrap();
            }
        }

        Ok(())
    }
}
