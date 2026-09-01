use rorm_declaration::imr::DbType;

use crate::create_column::{self, CreateColumn, CreateColumnImpl, PostgresType};
#[cfg(feature = "postgres")]
use crate::db_specific::postgres;
use crate::error::Error;
use crate::Value;

/**
Representation of operations to execute in the context of an ALTER TABLE statement.
 */
#[derive(Debug)]
pub enum AlterTableOperation<'until_build, 'post_build> {
    /// Use this operation to rename a table
    RenameTo {
        /// New name of the table
        name: String,
    },
    /// Use this operation to rename a column within a table
    RenameColumnTo {
        /// Current column name
        column_name: String,
        /// New column name
        new_column_name: String,
    },
    /// Use this operation to add a column to an existing table.
    AddColumn {
        /// Operation to use for adding the column
        operation: CreateColumnImpl<'until_build, 'post_build>,
    },
    /// Use this operation to drop an existing column.
    DropColumn {
        /// Name of the column to drop
        name: String,
    },
    /**
    Use this operation to change an existing column in place, preserving its data.

    Which change to make is decided by the caller: this operation renders the
    single statement it is given and nothing else.
    */
    AlterColumn {
        /// Name of the column to alter
        name: &'until_build str,

        /// The change to apply to it
        operation: AlterColumnOperation,
    },
}

/**
Representation of a single change to an existing column.

Each variant renders into exactly one `ALTER TABLE` statement, or into none at
all in sqlite: it has no `ALTER COLUMN`, and none of these changes are
observable in a `STRICT` table - [`DbType::VarChar`] and [`DbType::Text`] are
both `TEXT`, every integer is `INTEGER`, both floats are `REAL`, and a maximum
length is never enforced.
 */
#[derive(Copy, Clone, Debug)]
pub enum AlterColumnOperation {
    /// Set the column's type
    ///
    /// The type has to be one which is fully described by itself. A
    /// [`DbType::VarChar`] carries its maximum length and a [`DbType::Choices`]
    /// its enum, so neither can be rendered from the type alone.
    SetType {
        /// The type to change the column to
        data_type: DbType,
    },

    /// Add the check constraint enforcing the column's maximum length
    ///
    /// A column which already has one has to [drop](AlterColumnOperation::DropMaxLength)
    /// it first: a constraint can't be redefined, only replaced.
    SetMaxLength {
        /// The maximum number of characters the column may hold
        max_length: i32,
    },

    /// Drop the check constraint enforcing the column's maximum length
    DropMaxLength,
}

/**
The trait representing an alter table builder
*/
pub trait AlterTable<'post_build> {
    /**
    This method is used to build the alter table statement.
     */
    fn build(self) -> Result<Vec<(String, Vec<Value<'post_build>>)>, Error>;
}

/**
Representation of the data of an ALTER TABLE statement.
 */
#[derive(Debug)]
pub struct AlterTableData<'until_build, 'post_build> {
    /// Name of the table to operate on
    pub(crate) name: &'until_build str,
    /// Operation to execute
    pub(crate) operation: AlterTableOperation<'until_build, 'post_build>,
    pub(crate) lookup: Vec<Value<'post_build>>,
    pub(crate) statements: Vec<(String, Vec<Value<'post_build>>)>,
}

/**
Implementation of the [AlterTable] trait for the different database dialects.

Should only be constructed via [crate::DBImpl::alter_table].
 */
#[derive(Debug)]
pub enum AlterTableImpl<'until_build, 'post_build> {
    /**
    SQLite representation of the ALTER TABLE operation.
     */
    #[cfg(feature = "sqlite")]
    SQLite(AlterTableData<'until_build, 'post_build>),
    /**
    Postgres representation of the ALTER TABLE operation.
     */
    #[cfg(feature = "postgres")]
    Postgres(AlterTableData<'until_build, 'post_build>),
}

impl<'post_build> AlterTable<'post_build> for AlterTableImpl<'_, 'post_build> {
    fn build(self) -> Result<Vec<(String, Vec<Value<'post_build>>)>, Error> {
        match self {
            #[cfg(feature = "sqlite")]
            AlterTableImpl::SQLite(mut d) => {
                // The actions to execute, each as its own ALTER TABLE statement.
                //
                // It stays empty if the operation is a no-op for this dialect,
                // in which case no statement is produced at all.
                let mut actions: Vec<String> = Vec::new();

                match d.operation {
                    AlterTableOperation::RenameTo { name } => {
                        actions.push(format!("RENAME TO \"{name}\""));
                    }
                    AlterTableOperation::RenameColumnTo {
                        column_name,
                        new_column_name,
                    } => actions.push(format!(
                        "RENAME COLUMN \"{column_name}\" TO \"{new_column_name}\""
                    )),
                    AlterTableOperation::AddColumn { mut operation } => {
                        let mut action = String::from("ADD COLUMN ");

                        if let CreateColumnImpl::SQLite(ccd) = &mut operation {
                            ccd.statements = Some(&mut d.statements);
                            ccd.lookup = Some(&mut d.lookup);
                        }

                        operation.build(&mut action)?;
                        actions.push(action);
                    }
                    AlterTableOperation::DropColumn { name } => {
                        actions.push(format!("DROP COLUMN \"{name}\""))
                    }
                    // Deliberately a no-op, see `AlterColumnOperation`.
                    // Sqlite has no `ALTER COLUMN` and needs none for these
                    // changes: not one of them is observable in a `STRICT` table.
                    AlterTableOperation::AlterColumn { .. } => {}
                };

                Ok(finish(d.name, actions, d.lookup, d.statements))
            }
            #[cfg(feature = "postgres")]
            AlterTableImpl::Postgres(mut d) => {
                // The actions to execute, each as its own ALTER TABLE statement
                let mut actions: Vec<String> = Vec::new();

                match d.operation {
                    AlterTableOperation::RenameTo { name } => {
                        actions.push(format!("RENAME TO \"{name}\""));
                    }
                    AlterTableOperation::RenameColumnTo {
                        column_name,
                        new_column_name,
                    } => {
                        actions.push(format!(
                            "RENAME COLUMN \"{column_name}\" TO \"{new_column_name}\""
                        ));
                    }
                    AlterTableOperation::AddColumn { mut operation } => {
                        let mut action = String::from("ADD COLUMN ");

                        #[allow(irrefutable_let_patterns)]
                        if let CreateColumnImpl::Postgres(ccd) = &mut operation {
                            ccd.statements = Some(&mut d.statements);
                        }

                        operation.build(&mut action)?;
                        actions.push(action);
                    }
                    AlterTableOperation::DropColumn { name } => {
                        actions.push(format!("DROP COLUMN \"{name}\""))
                    }
                    AlterTableOperation::AlterColumn { name, operation } => {
                        actions.push(match operation {
                            AlterColumnOperation::SetType { data_type } => {
                                // A `character varying` carries its maximum
                                // length and a `Choices` its enum, so for
                                // neither does the type describe the column.
                                #[allow(deprecated)]
                                let unrenderable = match data_type {
                                    DbType::VarChar => {
                                        Some("its maximum length is part of its type")
                                    }
                                    DbType::Choices => {
                                        Some("its enum type belongs to the column creating it")
                                    }
                                    _ => None,
                                };
                                if let Some(reason) = unrenderable {
                                    return Err(Error::SQLBuildError(format!(
                                        "Column \"{name}\" can't be given the type \
                                         {data_type:?}: {reason}"
                                    )));
                                }

                                let data_type = match create_column::postgres_type(data_type, [])? {
                                    PostgresType::Normal(x) => x,
                                    PostgresType::Choices(_) => {
                                        unreachable!("Choices is rejected above")
                                    }
                                };

                                format!("ALTER COLUMN \"{name}\" TYPE {data_type}")
                            }
                            AlterColumnOperation::SetMaxLength { max_length } => format!(
                                "ADD CONSTRAINT \"{}\" CHECK (length(\"{name}\") <= {max_length})",
                                postgres::max_length_check_name(d.name, name),
                            ),
                            AlterColumnOperation::DropMaxLength => format!(
                                "DROP CONSTRAINT \"{}\"",
                                postgres::max_length_check_name(d.name, name),
                            ),
                        });
                    }
                };

                Ok(finish(d.name, actions, d.lookup, d.statements))
            }
        }
    }
}

/// Wraps every action into its own `ALTER TABLE` statement
/// and appends the statements the operation produced on the side.
#[cfg(any(feature = "sqlite", feature = "postgres"))]
fn finish<'post_build>(
    table: &str,
    actions: Vec<String>,
    lookup: Vec<Value<'post_build>>,
    side_statements: Vec<(String, Vec<Value<'post_build>>)>,
) -> Vec<(String, Vec<Value<'post_build>>)> {
    let mut statements: Vec<(String, Vec<Value<'post_build>>)> = actions
        .into_iter()
        .map(|action| (format!("ALTER TABLE \"{table}\" {action};"), Vec::new()))
        .collect();

    // Only `AddColumn` can bind values, and it produces a single action
    if let Some((_, first)) = statements.first_mut() {
        *first = lookup;
    }

    statements.extend(side_statements);
    statements
}

#[cfg(test)]
mod test {
    use rorm_declaration::imr::{Annotation, DbType};

    use crate::alter_table::{AlterColumnOperation, AlterTable, AlterTableOperation};
    use crate::error::Error;
    use crate::DBImpl;

    /// Collapses insignificant whitespace in `sql`
    ///
    /// Sql ignores whitespace, so the builders are free to leave a separator
    /// behind an annotation which rendered nothing, and the assertions here
    /// shouldn't have to spell those out. (Which also means they must not
    /// contain string literals.)
    fn normalize(sql: &str) -> String {
        sql.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .replace(" ;", ";")
            .replace(" ,", ",")
    }

    /// The statements `db` produces for `operation` on the `user` table
    fn alter(db: DBImpl, operation: AlterTableOperation) -> Vec<String> {
        db.alter_table("user", operation)
            .build()
            .expect("The operation builds")
            .into_iter()
            .map(|(statement, _)| normalize(&statement))
            .collect()
    }

    fn alter_err(db: DBImpl, operation: AlterTableOperation) -> Error {
        db.alter_table("user", operation)
            .build()
            .expect_err("The operation doesn't build")
    }

    fn alter_column(operation: AlterColumnOperation) -> AlterTableOperation<'static, 'static> {
        AlterTableOperation::AlterColumn {
            name: "login",
            operation,
        }
    }

    /// Both dialects share these, so they are asserted for whichever is built
    fn assert_common(db: DBImpl) {
        assert_eq!(
            alter(
                db,
                AlterTableOperation::RenameTo {
                    name: "person".to_string()
                }
            ),
            [r#"ALTER TABLE "user" RENAME TO "person";"#]
        );
        assert_eq!(
            alter(
                db,
                AlterTableOperation::RenameColumnTo {
                    column_name: "login".to_string(),
                    new_column_name: "username".to_string(),
                }
            ),
            [r#"ALTER TABLE "user" RENAME COLUMN "login" TO "username";"#]
        );
        assert_eq!(
            alter(
                db,
                AlterTableOperation::DropColumn {
                    name: "login".to_string()
                }
            ),
            [r#"ALTER TABLE "user" DROP COLUMN "login";"#]
        );
    }

    #[cfg(feature = "sqlite")]
    mod sqlite {
        use super::*;

        #[test]
        fn the_existing_operations_are_unchanged() {
            assert_common(DBImpl::SQLite);
            assert_eq!(
                alter(
                    DBImpl::SQLite,
                    AlterTableOperation::AddColumn {
                        operation: DBImpl::SQLite.create_column(
                            "user",
                            "login",
                            DbType::Text,
                            &[Annotation::MaxLength(255), Annotation::NotNull],
                        ),
                    }
                ),
                [r#"ALTER TABLE "user" ADD COLUMN "login" TEXT NOT NULL;"#]
            );
        }

        /// Sqlite has no `ALTER COLUMN` and needs none: not one of these
        /// changes is observable in a `STRICT` table.
        #[test]
        fn every_alter_column_operation_is_a_noop() {
            let cases = [
                AlterColumnOperation::SetType {
                    data_type: DbType::Text,
                },
                AlterColumnOperation::SetType {
                    data_type: DbType::Int64,
                },
                AlterColumnOperation::SetMaxLength { max_length: 255 },
                AlterColumnOperation::DropMaxLength,
            ];
            for operation in cases {
                assert_eq!(
                    alter(DBImpl::SQLite, alter_column(operation)),
                    Vec::<String>::new(),
                    "{operation:?}"
                );
            }
        }

        /// Not even the types postgres refuses may produce anything
        #[test]
        fn an_unrenderable_type_is_a_noop_too() {
            #[allow(deprecated)]
            for data_type in [DbType::VarChar, DbType::Choices] {
                assert_eq!(
                    alter(
                        DBImpl::SQLite,
                        alter_column(AlterColumnOperation::SetType { data_type })
                    ),
                    Vec::<String>::new(),
                    "{data_type:?}"
                );
            }
        }
    }

    #[cfg(feature = "postgres")]
    mod postgres {
        use super::*;

        #[test]
        fn the_existing_operations_are_unchanged() {
            assert_common(DBImpl::Postgres);
            assert_eq!(
                alter(
                    DBImpl::Postgres,
                    AlterTableOperation::AddColumn {
                        operation: DBImpl::Postgres.create_column(
                            "user",
                            "login",
                            DbType::Text,
                            &[Annotation::MaxLength(255), Annotation::NotNull],
                        ),
                    }
                ),
                [
                    r#"ALTER TABLE "user" ADD COLUMN "login" text CONSTRAINT "user_login_max_length" CHECK (length("login") <= 255) NOT NULL;"#
                ]
            );
        }

        /// Every operation renders into exactly one statement,
        /// just like the four which existed before.
        #[test]
        fn every_operation_is_a_single_statement() {
            assert_eq!(
                alter(
                    DBImpl::Postgres,
                    alter_column(AlterColumnOperation::SetType {
                        data_type: DbType::Text
                    })
                ),
                [r#"ALTER TABLE "user" ALTER COLUMN "login" TYPE text;"#]
            );
            assert_eq!(
                alter(
                    DBImpl::Postgres,
                    alter_column(AlterColumnOperation::SetMaxLength { max_length: 255 })
                ),
                [
                    r#"ALTER TABLE "user" ADD CONSTRAINT "user_login_max_length" CHECK (length("login") <= 255);"#
                ]
            );
            assert_eq!(
                alter(
                    DBImpl::Postgres,
                    alter_column(AlterColumnOperation::DropMaxLength)
                ),
                [r#"ALTER TABLE "user" DROP CONSTRAINT "user_login_max_length";"#]
            );
        }

        /// The caller decides whether a constraint exists, so the drop is
        /// unqualified: a `DROP CONSTRAINT IF EXISTS` would paper over a
        /// migration which got its own delta wrong.
        #[test]
        fn dropping_a_max_length_is_not_conditional() {
            let statements = alter(
                DBImpl::Postgres,
                alter_column(AlterColumnOperation::DropMaxLength),
            );
            assert!(!statements[0].contains("IF EXISTS"), "{statements:?}");
        }

        /// Widening an integer, the other alterable type change
        #[test]
        fn widening_an_integer() {
            assert_eq!(
                alter(
                    DBImpl::Postgres,
                    alter_column(AlterColumnOperation::SetType {
                        data_type: DbType::Int64
                    })
                ),
                [r#"ALTER TABLE "user" ALTER COLUMN "login" TYPE bigint;"#]
            );
        }

        /// `serial` is not a type but a column with its own sequence, so an
        /// `auto_increment` column must never be given one. That is a migration
        /// decision though, so rorm-cli makes it - here the base type is
        /// rendered, which is the only correct thing for an `ALTER COLUMN`.
        #[test]
        fn a_type_is_never_rendered_as_serial() {
            for (data_type, expected) in [
                (DbType::Int16, "smallint"),
                (DbType::Int32, "integer"),
                (DbType::Int64, "bigint"),
            ] {
                assert_eq!(
                    alter(
                        DBImpl::Postgres,
                        alter_column(AlterColumnOperation::SetType { data_type })
                    ),
                    [format!(
                        r#"ALTER TABLE "user" ALTER COLUMN "login" TYPE {expected};"#
                    )]
                );
            }
        }

        /// A `character varying` carries its maximum length in its type,
        /// so the type alone doesn't describe the column.
        #[test]
        fn setting_a_varchar_type_is_an_error() {
            #[allow(deprecated)]
            let operation = AlterColumnOperation::SetType {
                data_type: DbType::VarChar,
            };
            assert!(matches!(
                alter_err(DBImpl::Postgres, alter_column(operation)),
                Error::SQLBuildError(msg) if msg.contains("maximum length")
            ));
        }

        /// An enum's type is only known to the column which creates it
        #[test]
        fn setting_an_enum_type_is_an_error() {
            assert!(matches!(
                alter_err(
                    DBImpl::Postgres,
                    alter_column(AlterColumnOperation::SetType {
                        data_type: DbType::Choices
                    })
                ),
                Error::SQLBuildError(msg) if msg.contains("enum type")
            ));
        }
    }
}
