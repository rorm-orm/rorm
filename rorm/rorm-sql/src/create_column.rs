use std::fmt::Write;

use rorm_declaration::imr::DefaultValue;

#[cfg(feature = "postgres")]
use crate::create_trigger::trigger_annotation_to_trigger_postgres;

#[cfg(feature = "sqlite")]
use crate::create_trigger::trigger_annotation_to_trigger_sqlite;

use crate::error::Error;
use crate::{Annotation, DbType, Value};

#[cfg(feature = "postgres")]
use crate::db_specific::postgres;

#[cfg(feature = "sqlite")]
use crate::db_specific::sqlite;

/**
Trait representing the create table builder.
*/
pub trait CreateColumn<'post_build>: Sized {
    /**
    Builds the column based on the data.

    **Parameter**:
    - `s`: mutable reference to a String to write the operation to
    */
    fn build(self, s: &mut String) -> Result<(), Error>;
}

/**
Representation of an annotation
 */
#[derive(Debug)]
pub struct SQLAnnotation<'post_build> {
    pub(crate) annotation: &'post_build Annotation,
}

/**
Representation of the data of the creation of a column for the sqlite dialect
 */
#[derive(Debug)]
#[cfg(feature = "sqlite")]
pub struct CreateColumnSQLiteData<'until_build, 'post_build> {
    pub(crate) name: &'until_build str,
    pub(crate) table_name: &'until_build str,
    pub(crate) data_type: DbType,
    pub(crate) annotations: Vec<SQLAnnotation<'post_build>>,
    pub(crate) statements: Option<&'until_build mut Vec<(String, Vec<Value<'post_build>>)>>,
    pub(crate) lookup: Option<&'until_build mut Vec<Value<'post_build>>>,
}

/**
Representation of the data of the creation of a column for the mysql dialect
 */
#[derive(Debug)]
#[cfg(feature = "mysql")]
pub struct CreateColumnMySQLData<'until_build, 'post_build> {
    pub(crate) name: &'until_build str,
    pub(crate) data_type: DbType,
    pub(crate) annotations: Vec<SQLAnnotation<'post_build>>,
    pub(crate) statements: Option<&'until_build mut Vec<(String, Vec<Value<'post_build>>)>>,
    pub(crate) lookup: Option<&'until_build mut Vec<Value<'post_build>>>,
}

/**
Representation of the data of the creation of a column for the mysql dialect
 */
#[derive(Debug)]
#[cfg(feature = "postgres")]
pub struct CreateColumnPostgresData<'until_build, 'post_build> {
    pub(crate) name: &'until_build str,
    pub(crate) table_name: &'until_build str,
    pub(crate) data_type: DbType,
    pub(crate) annotations: Vec<SQLAnnotation<'post_build>>,
    pub(crate) statements: Option<&'until_build mut Vec<(String, Vec<Value<'post_build>>)>>,
}

/**
Representation of the different implementations of the [CreateColumn] trait.

Should only be constructed via [crate::DBImpl::create_column].
*/
#[derive(Debug)]
pub enum CreateColumnImpl<'until_build, 'post_build> {
    /**
    SQLite representation of the create column operation.
     */
    #[cfg(feature = "sqlite")]
    SQLite(CreateColumnSQLiteData<'until_build, 'post_build>),
    /**
    MySQL representation of the create column operation.
     */
    #[cfg(feature = "mysql")]
    MySQL(CreateColumnMySQLData<'until_build, 'post_build>),
    /**
    Postgres representation of the create column operation.
     */
    #[cfg(feature = "postgres")]
    Postgres(CreateColumnPostgresData<'until_build, 'post_build>),
}

impl<'until_build, 'post_build> CreateColumn<'post_build>
    for CreateColumnImpl<'until_build, 'post_build>
{
    fn build(self, s: &mut String) -> Result<(), Error> {
        match self {
            #[cfg(feature = "sqlite")]
            CreateColumnImpl::SQLite(mut d) => {
                write!(
                    s,
                    "{} {} ",
                    d.name,
                    match d.data_type {
                        DbType::VarBinary => "BLOB",
                        DbType::VarChar
                        | DbType::Date
                        | DbType::DateTime
                        | DbType::Timestamp
                        | DbType::Time
                        | DbType::Choices => "TEXT",
                        DbType::Int8
                        | DbType::Int16
                        | DbType::Int32
                        | DbType::Int64
                        | DbType::Boolean => "INTEGER",
                        DbType::Float | DbType::Double => "REAL",
                    }
                )
                .unwrap();

                for (idx, x) in d.annotations.iter().enumerate() {
                    if let Some(ref mut s) = d.statements {
                        trigger_annotation_to_trigger_sqlite(
                            x.annotation,
                            &d.data_type,
                            d.table_name,
                            d.name,
                            s,
                        );
                    }

                    match &x.annotation {
                        Annotation::AutoIncrement => write!(s, "AUTOINCREMENT").unwrap(),
                        Annotation::AutoCreateTime => {
                            write!(
                                s,
                                "DEFAULT {}",
                                match d.data_type {
                                    DbType::Date => "CURRENT_DATE",
                                    DbType::DateTime => "CURRENT_TIMESTAMP",
                                    DbType::Timestamp => "CURRENT_TIMESTAMP",
                                    DbType::Time => "CURRENT_TIME",
                                    _ => "",
                                }
                            )
                            .unwrap();
                        }
                        Annotation::DefaultValue(d) => match d {
                            DefaultValue::String(dv) => {
                                write!(s, "DEFAULT {}", sqlite::fmt(dv)).unwrap()
                            }
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
                        Annotation::ForeignKey(fk) => {
                            write!(s, "REFERENCES {} ({})", fk.table_name, fk.column_name).unwrap()
                        }
                        _ => {}
                    }

                    if idx != d.annotations.len() - 1 {
                        write!(s, " ").unwrap();
                    }
                }

                Ok(())
            }
            #[cfg(feature = "mysql")]
            CreateColumnImpl::MySQL(mut d) => {
                write!(s, "{} ", d.name).unwrap();

                match d.data_type {
                    DbType::VarChar => {
                        let a_opt = d
                            .annotations
                            .iter()
                            .find(|x| x.annotation.eq_shallow(&Annotation::MaxLength(0)));

                        if let Some(a) = a_opt {
                            if let Annotation::MaxLength(max_length) = a.annotation {
                                write!(s, "VARCHAR({}) ", max_length).unwrap();
                            } else {
                                return Err(Error::SQLBuildError(String::from(
                                    "VARCHAR must have a max_length annotation",
                                )));
                            }
                        } else {
                            return Err(Error::SQLBuildError(String::from(
                                "VARCHAR must have a max_length annotation",
                            )));
                        }
                    }
                    DbType::VarBinary => {
                        let a_opt = d
                            .annotations
                            .iter()
                            .find(|x| x.annotation.eq_shallow(&Annotation::MaxLength(0)));

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
                    DbType::Int8 => write!(s, "TINYINT(255) ").unwrap(),
                    DbType::Int16 => write!(s, "SMALLINT(255) ").unwrap(),
                    DbType::Int32 => write!(s, "INT(255) ").unwrap(),
                    DbType::Int64 => write!(s, "BIGINT(255) ").unwrap(),
                    DbType::Float => write!(s, "FLOAT(24) ").unwrap(),
                    DbType::Double => write!(s, "DOUBLE(53) ").unwrap(),
                    DbType::Boolean => write!(s, "BOOL ").unwrap(),
                    DbType::Date => write!(s, "DATE ").unwrap(),
                    DbType::DateTime => write!(s, "DATETIME ").unwrap(),
                    DbType::Timestamp => write!(s, "TIMESTAMP ").unwrap(),
                    DbType::Time => write!(s, "TIME ").unwrap(),
                    DbType::Choices => {
                        let a_opt = d.annotations.iter().find(|x| {
                            x.annotation
                                .eq_shallow(&Annotation::Choices(Default::default()))
                        });

                        if let Some(a) = a_opt {
                            if let Annotation::Choices(values) = a.annotation {
                                write!(
                                    s,
                                    "VARCHAR({}) ",
                                    values
                                        .iter()
                                        .map(|x| {
                                            if let Some(l) = &mut d.lookup {
                                                l.push(Value::String(x));
                                            }
                                            String::from("?")
                                        })
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
                };

                for (idx, x) in d.annotations.iter().enumerate() {
                    match &x.annotation {
                        Annotation::AutoIncrement => write!(s, "AUTO_INCREMENT").unwrap(),
                        Annotation::AutoCreateTime => {
                            write!(
                                s,
                                "DEFAULT {}",
                                match d.data_type {
                                    DbType::Date => "CURRENT_DATE",
                                    DbType::DateTime => "CURRENT_TIMESTAMP",
                                    DbType::Timestamp => "CURRENT_TIMESTAMP",
                                    DbType::Time => "CURRENT_TIME",
                                    _ => "",
                                }
                            )
                            .unwrap();
                        }
                        Annotation::AutoUpdateTime => write!(
                            s,
                            "ON UPDATE {}",
                            match d.data_type {
                                DbType::Date => "CURRENT_DATE",
                                DbType::DateTime => "CURRENT_TIMESTAMP",
                                DbType::Timestamp => "CURRENT_TIMESTAMP",
                                DbType::Time => "CURRENT_TIME",
                                _ => "",
                            }
                        )
                        .unwrap(),
                        Annotation::DefaultValue(v) => match v {
                            DefaultValue::String(dv) => {
                                if let Some(l) = &mut d.lookup {
                                    l.push(Value::String(dv))
                                }
                                write!(s, "DEFAULT ?").unwrap();
                            }
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
                        Annotation::ForeignKey(fk) => {
                            write!(s, "REFERENCES {}({})", fk.table_name, fk.column_name).unwrap()
                        }
                        _ => {}
                    }

                    if idx != d.annotations.len() - 1 {
                        write!(s, " ").unwrap();
                    }
                }

                Ok(())
            }
            #[cfg(feature = "postgres")]
            CreateColumnImpl::Postgres(mut d) => {
                write!(s, "{} ", d.name).unwrap();

                match d.data_type {
                    DbType::VarChar | DbType::Choices => {
                        let a_opt = d
                            .annotations
                            .iter()
                            .find(|x| x.annotation.eq_shallow(&Annotation::MaxLength(0)));

                        if let Some(a) = a_opt {
                            if let Annotation::MaxLength(max_length) = a.annotation {
                                write!(s, "character varying ({}) ", max_length).unwrap();
                            } else {
                                return Err(Error::SQLBuildError(
                                    "character varying must have a max_length annotation"
                                        .to_string(),
                                ));
                            }
                        } else {
                            return Err(Error::SQLBuildError(
                                "character varying must have a max_length annotation".to_string(),
                            ));
                        }
                    }
                    DbType::VarBinary => write!(s, "bytea ").unwrap(),
                    DbType::Int8 => write!(s, "smallint ").unwrap(),
                    DbType::Int16 => {
                        if d.annotations
                            .iter()
                            .any(|x| x.annotation.eq_shallow(&Annotation::AutoIncrement))
                        {
                            write!(s, "smallserial ").unwrap();
                        } else {
                            write!(s, "smallint ").unwrap();
                        }
                    }
                    DbType::Int32 => {
                        if d.annotations
                            .iter()
                            .any(|x| x.annotation.eq_shallow(&Annotation::AutoIncrement))
                        {
                            write!(s, "serial ").unwrap();
                        } else {
                            write!(s, "integer ").unwrap();
                        }
                    }
                    DbType::Int64 => {
                        if d.annotations
                            .iter()
                            .any(|x| x.annotation.eq_shallow(&Annotation::AutoIncrement))
                        {
                            write!(s, "bigserial ").unwrap();
                        } else {
                            write!(s, "bigint ").unwrap();
                        }
                    }
                    DbType::Float => write!(s, "real ").unwrap(),
                    DbType::Double => write!(s, "double precision ").unwrap(),
                    DbType::Boolean => write!(s, "boolean ").unwrap(),
                    DbType::Date => write!(s, "date ").unwrap(),
                    DbType::DateTime | DbType::Timestamp => write!(s, "timestamp ").unwrap(),
                    DbType::Time => write!(s, "time ").unwrap(),
                };

                for (idx, x) in d.annotations.iter().enumerate() {
                    if let Some(ref mut s) = d.statements {
                        trigger_annotation_to_trigger_postgres(
                            x.annotation,
                            d.table_name,
                            d.name,
                            s,
                        );
                    }

                    match &x.annotation {
                        Annotation::AutoCreateTime => {
                            write!(
                                s,
                                "DEFAULT {}",
                                match d.data_type {
                                    DbType::Date => "CURRENT_DATE",
                                    DbType::DateTime => "CURRENT_TIMESTAMP",
                                    DbType::Timestamp => "CURRENT_TIMESTAMP",
                                    DbType::Time => "CURRENT_TIME",
                                    _ => "",
                                }
                            )
                            .unwrap();
                        }
                        Annotation::DefaultValue(d) => match d {
                            DefaultValue::String(dv) => {
                                write!(s, "DEFAULT {}", postgres::fmt(dv)).unwrap()
                            }
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
                        Annotation::ForeignKey(fk) => {
                            write!(s, "REFERENCES \"{}\"({})", fk.table_name, fk.column_name)
                                .unwrap()
                        }
                        _ => {}
                    };

                    if idx != d.annotations.len() - 1 {
                        write!(s, " ").unwrap();
                    }
                }

                Ok(())
            }
        }
    }
}
