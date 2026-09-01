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
Representation of the data of the creation of a column for the postgres dialect
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
    fn build(self, sql: &mut String) -> Result<(), Error> {
        match self {
            #[cfg(feature = "sqlite")]
            CreateColumnImpl::SQLite(mut column) => {
                write!(
                    sql,
                    "\"{}\" {}",
                    column.name,
                    sqlite_type(column.data_type)?
                )
                .unwrap();

                for x in &column.annotations {
                    let SQLAnnotation { annotation } = x;

                    if let Some(s) = &mut column.statements {
                        trigger_annotation_to_trigger_sqlite(
                            annotation,
                            &column.data_type,
                            column.table_name,
                            column.name,
                            s,
                        );
                    }

                    sql.push(' ');
                    match &annotation {
                        Annotation::AutoIncrement => write!(sql, "AUTOINCREMENT").unwrap(),
                        Annotation::AutoCreateTime => {
                            write!(
                                sql,
                                "DEFAULT {}",
                                match column.data_type {
                                    DbType::Date => "CURRENT_DATE",
                                    DbType::DateTime => "CURRENT_TIMESTAMP",
                                    DbType::Timestamp => "CURRENT_TIMESTAMP",
                                    DbType::Time => "CURRENT_TIME",
                                    _ =>
                                        return Err(Error::SQLBuildError(format!(
                                            "AutoCreateTime not compatible with {:?}",
                                            column.data_type
                                        ))),
                                }
                            )
                            .unwrap();
                        }
                        Annotation::DefaultValue(DefaultValue::String(x)) => {
                            write!(sql, "DEFAULT {}", sqlite::fmt(x)).unwrap()
                        }
                        Annotation::DefaultValue(DefaultValue::Integer(x)) => {
                            write!(sql, "DEFAULT {x}").unwrap()
                        }
                        Annotation::DefaultValue(DefaultValue::Float(x)) => {
                            write!(sql, "DEFAULT {x}").unwrap()
                        }
                        Annotation::DefaultValue(DefaultValue::Boolean(true)) => {
                            write!(sql, "DEFAULT 1").unwrap()
                        }
                        Annotation::DefaultValue(DefaultValue::Boolean(false)) => {
                            write!(sql, "DEFAULT 0").unwrap()
                        }
                        Annotation::NotNull => write!(sql, "NOT NULL").unwrap(),
                        Annotation::PrimaryKey => write!(sql, "PRIMARY KEY").unwrap(),
                        Annotation::Unique => write!(sql, "UNIQUE").unwrap(),
                        Annotation::ForeignKey(fk) => write!(
                            sql,
                            "REFERENCES \"{}\" (\"{}\") ON DELETE {} ON UPDATE {}",
                            fk.table_name, fk.column_name, fk.on_delete, fk.on_update
                        )
                        .unwrap(),
                        _ => {}
                    }
                }

                Ok(())
            }
            #[cfg(feature = "postgres")]
            CreateColumnImpl::Postgres(mut column) => {
                write!(sql, "\"{}\" ", column.name).unwrap();

                match postgres_type(
                    column.data_type,
                    column.annotations.iter().map(|x| x.annotation),
                )? {
                    PostgresType::Normal(x) => write!(sql, "{x}").unwrap(),
                    PostgresType::Choices(values) => {
                        if let Some(stmts) = column.pre_statements {
                            stmts.push((
                                format!(
                                    "CREATE TYPE _{}_{} AS ENUM({});",
                                    column.table_name,
                                    column.name,
                                    values
                                        .iter()
                                        .map(|x| { postgres::fmt(x) })
                                        .collect::<Vec<String>>()
                                        .join(", ")
                                ),
                                vec![],
                            ));
                        };
                        write!(sql, "_{}_{}", column.table_name, column.name,).unwrap();
                    }
                };

                for x in &column.annotations {
                    let SQLAnnotation { annotation } = x;

                    if let Some(s) = &mut column.statements {
                        trigger_annotation_to_trigger_postgres(
                            annotation,
                            column.table_name,
                            column.name,
                            s,
                        );
                    }

                    sql.push(' ');
                    match &annotation {
                        Annotation::AutoCreateTime => {
                            write!(
                                sql,
                                "DEFAULT {}",
                                match column.data_type {
                                    DbType::Date => "CURRENT_DATE",
                                    DbType::DateTime => "now()",
                                    DbType::Timestamp => "CURRENT_TIMESTAMP",
                                    DbType::Time => "CURRENT_TIME",
                                    _ =>
                                        return Err(Error::SQLBuildError(format!(
                                            "AutoCreateTime not compatible with {:?}",
                                            column.data_type
                                        ))),
                                }
                            )
                            .unwrap();
                        }
                        Annotation::DefaultValue(DefaultValue::String(x)) => {
                            write!(sql, "DEFAULT {}", postgres::fmt(x)).unwrap()
                        }
                        Annotation::DefaultValue(DefaultValue::Integer(x)) => {
                            write!(sql, "DEFAULT {x}").unwrap()
                        }
                        Annotation::DefaultValue(DefaultValue::Float(x)) => {
                            write!(sql, "DEFAULT {x}").unwrap()
                        }
                        Annotation::DefaultValue(DefaultValue::Boolean(true)) => {
                            write!(sql, "DEFAULT true").unwrap()
                        }
                        Annotation::DefaultValue(DefaultValue::Boolean(false)) => {
                            write!(sql, "DEFAULT false").unwrap()
                        }
                        Annotation::NotNull => write!(sql, "NOT NULL").unwrap(),
                        Annotation::PrimaryKey => write!(sql, "PRIMARY KEY").unwrap(),
                        Annotation::Unique => write!(sql, "UNIQUE").unwrap(),
                        Annotation::ForeignKey(fk) => write!(
                            sql,
                            "REFERENCES \"{}\"(\"{}\") ON DELETE {} ON UPDATE {}",
                            fk.table_name, fk.column_name, fk.on_delete, fk.on_update
                        )
                        .unwrap(),
                        // A `character varying` carries its maximum length in
                        // its type already, and every non string column has
                        // nothing for `length()` to be applied to.
                        Annotation::MaxLength(max_length) => {
                            if matches!(column.data_type, DbType::Text) {
                                write!(
                                    sql,
                                    "CONSTRAINT \"{}\" CHECK (length(\"{}\") <= {max_length})",
                                    postgres::max_length_check_name(column.table_name, column.name),
                                    column.name,
                                )
                                .unwrap();
                            }
                        }

                        _ => {}
                    };
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
        | DbType::Text
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

/// Return type of [`postgres_type`]
pub enum PostgresType<'a> {
    /// A "normal" postgres identified by a string
    Normal(Cow<'static, str>),

    /// Choices use a custom unique postgres type per column.
    Choices(&'a [String]),
}

/// Converts a [`DbType`] into the associated postgres type.
///
/// Some type have to take the `annotations` into account.
/// The `Choices` need special handling and is returned as its own enum variant.
pub fn postgres_type<'a>(
    data_type: DbType,
    annotations: impl IntoIterator<Item = &'a Annotation> + Clone,
) -> Result<PostgresType<'a>, Error> {
    let auto_increment = annotations
        .clone()
        .into_iter()
        .any(|x| matches!(x, Annotation::AutoIncrement));

    let max_length = annotations.clone().into_iter().find_map(|x| match x {
        Annotation::MaxLength(x) => Some(x),
        _ => None,
    });

    let choices = annotations.clone().into_iter().find_map(|x| match x {
        Annotation::Choices(x) => Some(x.as_slice()),
        _ => None,
    });

    #[allow(deprecated)]
    Ok(PostgresType::Normal(Cow::Borrowed(match data_type {
        DbType::Text => "text",
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
        DbType::Timestamp => "timestamp",
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
