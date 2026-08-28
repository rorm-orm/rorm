use std::borrow::Cow;
use std::fmt::Write;

use rorm_declaration::imr::DefaultValue;

#[cfg(feature = "postgres")]
use crate::create_trigger::trigger_annotation_to_trigger_postgres;
#[cfg(feature = "sqlite")]
use crate::create_trigger::trigger_annotation_to_trigger_sqlite;
#[cfg(feature = "postgres")]
use crate::db_specific::postgres;
#[cfg(feature = "sqlite")]
use crate::db_specific::sqlite;
use crate::error::Error;
use crate::{Annotation, DbType, Value};

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
#[cfg(feature = "postgres")]
pub struct CreateColumnPostgresData<'until_build, 'post_build> {
    pub(crate) name: &'until_build str,
    pub(crate) table_name: &'until_build str,
    pub(crate) data_type: DbType,
    pub(crate) annotations: Vec<SQLAnnotation<'post_build>>,
    pub(crate) pre_statements: Option<&'until_build mut Vec<(String, Vec<Value<'post_build>>)>>,
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
    Postgres representation of the create column operation.
     */
    #[cfg(feature = "postgres")]
    Postgres(CreateColumnPostgresData<'until_build, 'post_build>),
}

impl<'post_build> CreateColumn<'post_build> for CreateColumnImpl<'_, 'post_build> {
    fn build(self, s: &mut String) -> Result<(), Error> {
        match self {
            #[cfg(feature = "sqlite")]
            CreateColumnImpl::SQLite(mut d) => {
                write!(s, "\"{}\" {} ", d.name, sqlite_type(d.data_type)?).unwrap();

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
                            DefaultValue::Integer(i) => write!(s, "DEFAULT {i}").unwrap(),
                            DefaultValue::Float(f) => write!(s, "DEFAULT {f}").unwrap(),
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
                        Annotation::ForeignKey(fk) => write!(
                            s,
                            "REFERENCES \"{}\" (\"{}\") ON DELETE {} ON UPDATE {}",
                            fk.table_name, fk.column_name, fk.on_delete, fk.on_update
                        )
                        .unwrap(),
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
                write!(s, "\"{}\" ", d.name).unwrap();

                match postgres_type(d.data_type, &d.annotations)? {
                    PostgresType::Normal(x) => write!(s, "{x} ").unwrap(),
                    PostgresType::Choices(values) => {
                        if let Some(stmts) = d.pre_statements {
                            stmts.push((
                                format!(
                                    "CREATE TYPE _{}_{} AS ENUM({});",
                                    d.table_name,
                                    d.name,
                                    values
                                        .iter()
                                        .map(|x| { postgres::fmt(x) })
                                        .collect::<Vec<String>>()
                                        .join(", ")
                                ),
                                vec![],
                            ));
                        };
                        write!(s, "_{}_{} ", d.table_name, d.name,).unwrap();
                    }
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
                                    DbType::DateTime => "now()",
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
                            DefaultValue::Integer(i) => write!(s, "DEFAULT {i}").unwrap(),
                            DefaultValue::Float(f) => write!(s, "DEFAULT {f}").unwrap(),
                            DefaultValue::Boolean(b) => {
                                if *b {
                                    write!(s, "DEFAULT true").unwrap();
                                } else {
                                    write!(s, "DEFAULT false").unwrap();
                                }
                            }
                        },
                        Annotation::NotNull => write!(s, "NOT NULL").unwrap(),
                        Annotation::PrimaryKey => write!(s, "PRIMARY KEY").unwrap(),
                        Annotation::Unique => write!(s, "UNIQUE").unwrap(),
                        Annotation::ForeignKey(fk) => write!(
                            s,
                            "REFERENCES \"{}\"(\"{}\") ON DELETE {} ON UPDATE {}",
                            fk.table_name, fk.column_name, fk.on_delete, fk.on_update
                        )
                        .unwrap(),
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

/// Converts a [`DbType`] into the associated sqlite type.
///
/// Note, we create tables in the `STRICT` mode.
/// Only the actual basic datatypes can be used and not their various aliases.
pub fn sqlite_type(data_type: DbType) -> Result<&'static str, Error> {
    #[allow(deprecated)]
    Ok(match data_type {
        DbType::Binary | DbType::Uuid => "BLOB",
        DbType::VarChar
        | DbType::Date
        | DbType::DateTime
        | DbType::Timestamp
        | DbType::Time
        | DbType::Choices => "TEXT",
        DbType::Int8 | DbType::Int16 | DbType::Int32 | DbType::Int64 | DbType::Boolean => "INTEGER",
        DbType::Float | DbType::Double => "REAL",
        DbType::BitVec | DbType::MacAddress | DbType::IpNetwork => {
            return Err(Error::SQLBuildError(format!(
                "{data_type:?} is not available for sqlite"
            )))
        }
    })
}

pub enum PostgresType<'a> {
    Normal(Cow<'static, str>),
    Choices(&'a [String]),
}

/// Converts a [`DbType`] into the associated postgres type.
///
/// Some type have to take the `annotations` into account.
/// The `Choices` need special handling and is returned as its own enum variant.
pub fn postgres_type<'a>(
    data_type: DbType,
    annotations: &'a [SQLAnnotation],
) -> Result<PostgresType<'a>, Error> {
    let auto_increment = annotations
        .iter()
        .any(|x| matches!(x.annotation, Annotation::AutoIncrement));

    let max_length = annotations.iter().find_map(|x| match x.annotation {
        Annotation::MaxLength(x) => Some(*x),
        _ => None,
    });

    let choices = annotations.iter().find_map(|x| match x.annotation {
        Annotation::Choices(x) => Some(x.as_slice()),
        _ => None,
    });

    #[allow(deprecated)]
    Ok(PostgresType::Normal(Cow::Borrowed(match data_type {
        DbType::Uuid => "uuid",
        DbType::MacAddress => "macaddr",
        DbType::IpNetwork => "inet",
        DbType::BitVec => "varbit",
        DbType::Binary => "bytea",
        DbType::Int8 => "smallint",
        DbType::Int16 if auto_increment => "smallserial",
        DbType::Int16 => "smallint",
        DbType::Int32 if auto_increment => "serial",
        DbType::Int32 => "integer",
        DbType::Int64 if auto_increment => "bigserial",
        DbType::Int64 => "bigint",
        DbType::Float => "real",
        DbType::Double => "double precision",
        DbType::Boolean => "boolean",
        DbType::Date => "date",
        DbType::DateTime => "timestamptz",
        DbType::Timestamp => "timestamp ",
        DbType::Time => "time",
        DbType::VarChar => {
            return match max_length {
                Some(x) => Ok(PostgresType::Normal(Cow::Owned(format!(
                    "character varying ({x})"
                )))),
                None => Err(Error::SQLBuildError(
                    "character varying must have a max_length annotation".to_string(),
                )),
            };
        }
        DbType::Choices => {
            return match choices {
                Some(x) => Ok(PostgresType::Choices(x)),
                None => Err(Error::SQLBuildError(
                    "VARCHAR must have a MaxLength annotation".to_string(),
                )),
            };
        }
    })))
}
