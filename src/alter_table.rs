use rorm_declaration::imr::{Annotation, DbType};

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
    Use this operation to change an existing column's type and constraints
    in place, preserving its data.

    The operation asserts the column's max length instead of applying a diff,
    because a migration is applied without any knowledge of the column's
    current definition.

    In sqlite this is a no-op. It has no `ALTER COLUMN`, and none of the
    changes `rorm-cli` considers alterable are observable in a `STRICT` table:
    `VarChar` and `Text` are both `TEXT`, every integer is `INTEGER`, both
    floats are `REAL`, and a max length is never enforced.
    */
    AlterColumn {
        /// Name of the column to alter
        name: &'until_build str,

        /// The type to change the column to, or `None` to leave it alone
        ///
        /// It is never `None` for a [`DbType::VarChar`] column, whose
        /// [`Annotation::MaxLength`] is part of its type.
        data_type: Option<DbType>,

        /// The annotations the column will have
        annotations: &'post_build [Annotation],
    },
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
                    // Deliberately a no-op, see `AlterTableOperation::AlterColumn`.
                    // `rorm-cli` only emits it for changes which leave a
                    // sqlite column's definition untouched.
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
                    AlterTableOperation::AlterColumn {
                        name,
                        data_type,
                        annotations,
                    } => {
                        if let Some(data_type) = data_type {
                            // `smallserial`, `serial` and `bigserial` create a
                            // sequence which `ALTER COLUMN ... TYPE` does not
                            // alter along with its column, leaving it to
                            // overflow at its old type's maximum.
                            if annotations.contains(&Annotation::AutoIncrement) {
                                return Err(Error::SQLBuildError(format!(
                                    "Column \"{name}\" can't be altered: the sequence behind \
                                     an auto_increment column would keep its old type"
                                )));
                            }

                            let PostgresType::Normal(data_type) =
                                create_column::postgres_type(data_type, annotations)?
                            else {
                                return Err(Error::SQLBuildError(format!(
                                    "Column \"{name}\" can't be altered to an enum type"
                                )));
                            };

                            actions.push(format!("ALTER COLUMN \"{name}\" TYPE {data_type}"));
                        }

                        // A `text` column's max length is a check constraint.
                        // It is asserted rather than diffed, because the
                        // column's current definition is unknown - hence the
                        // unconditional drop, which also removes the
                        // constraint of a column which lost its `MaxLength`.
                        //
                        // A `character varying` must not get one: it carries
                        // its max length in its type. `data_type == None`
                        // implies the column is not one, because `rorm-cli`
                        // always sets the type of a `character varying`.
                        let constraint = postgres::max_length_check_name(d.name, name);
                        actions.push(format!("DROP CONSTRAINT IF EXISTS \"{constraint}\""));
                        if matches!(data_type, None | Some(DbType::Text)) {
                            if let Some(max_length) = annotations.iter().find_map(|x| match x {
                                Annotation::MaxLength(x) => Some(x),
                                _ => None,
                            }) {
                                actions.push(format!(
                                    "ADD CONSTRAINT \"{constraint}\" CHECK (length(\"{name}\") <= {max_length})"
                                ));
                            }
                        }
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

    use crate::alter_table::{AlterTable, AlterTableOperation};
    use crate::error::Error;
    use crate::DBImpl;

    /// The statements `db` produces for `operation` on the `user` table
    fn alter(db: DBImpl, operation: AlterTableOperation) -> Vec<String> {
        db.alter_table("user", operation)
            .build()
            .expect("The operation builds")
            .into_iter()
            .map(|(statement, _)| statement)
            .collect()
    }

    fn alter_err(db: DBImpl, operation: AlterTableOperation) -> Error {
        db.alter_table("user", operation)
            .build()
            .expect_err("The operation doesn't build")
    }

    fn alter_column<'a>(
        data_type: Option<DbType>,
        annotations: &'a [Annotation],
    ) -> AlterTableOperation<'a, 'a> {
        AlterTableOperation::AlterColumn {
            name: "login",
            data_type,
            annotations,
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

        /// Sqlite has no `ALTER COLUMN`, and it doesn't need one: every change
        /// rorm-cli considers alterable leaves a sqlite column untouched.
        #[test]
        fn alter_column_is_a_noop() {
            let annotations = [Annotation::MaxLength(255), Annotation::NotNull];
            let cases: [Option<DbType>; 4] = [
                Some(DbType::Text),
                Some(DbType::VarChar),
                Some(DbType::Int64),
                None,
            ];
            for data_type in cases {
                assert_eq!(
                    alter(DBImpl::SQLite, alter_column(data_type, &annotations)),
                    Vec::<String>::new(),
                    "{data_type:?}"
                );
            }
        }

        /// Not even the one case postgres refuses may produce anything
        #[test]
        fn altering_an_auto_increment_column_is_a_noop_too() {
            assert_eq!(
                alter(
                    DBImpl::SQLite,
                    alter_column(Some(DbType::Int64), &[Annotation::AutoIncrement])
                ),
                Vec::<String>::new()
            );
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

        /// The `varchar(n)` -> `text` migration every existing deployment gets
        #[test]
        fn to_text_with_a_max_length() {
            assert_eq!(
                alter(
                    DBImpl::Postgres,
                    alter_column(
                        Some(DbType::Text),
                        &[Annotation::MaxLength(255), Annotation::NotNull]
                    )
                ),
                [
                    r#"ALTER TABLE "user" ALTER COLUMN "login" TYPE text;"#,
                    r#"ALTER TABLE "user" DROP CONSTRAINT IF EXISTS "user_login_max_length";"#,
                    r#"ALTER TABLE "user" ADD CONSTRAINT "user_login_max_length" CHECK (length("login") <= 255);"#,
                ]
            );
        }

        /// Dropping `max_length` altogether has to drop the constraint,
        /// which is why the drop is unconditional.
        #[test]
        fn to_text_without_a_max_length() {
            assert_eq!(
                alter(
                    DBImpl::Postgres,
                    alter_column(Some(DbType::Text), &[Annotation::NotNull])
                ),
                [
                    r#"ALTER TABLE "user" ALTER COLUMN "login" TYPE text;"#,
                    r#"ALTER TABLE "user" DROP CONSTRAINT IF EXISTS "user_login_max_length";"#,
                ]
            );
        }

        /// Changing only the max length must not touch the column's type:
        /// postgres rebuilds a column's indexes on every `ALTER COLUMN TYPE`.
        #[test]
        fn only_the_max_length() {
            assert_eq!(
                alter(
                    DBImpl::Postgres,
                    alter_column(None, &[Annotation::MaxLength(300), Annotation::NotNull])
                ),
                [
                    r#"ALTER TABLE "user" DROP CONSTRAINT IF EXISTS "user_login_max_length";"#,
                    r#"ALTER TABLE "user" ADD CONSTRAINT "user_login_max_length" CHECK (length("login") <= 300);"#,
                ]
            );
        }

        /// A `character varying` carries its max length in its type,
        /// so it may not get a constraint on top of it.
        #[test]
        fn to_varchar_gets_no_constraint() {
            assert_eq!(
                alter(
                    DBImpl::Postgres,
                    alter_column(Some(DbType::VarChar), &[Annotation::MaxLength(300)])
                ),
                [
                    r#"ALTER TABLE "user" ALTER COLUMN "login" TYPE character varying (300);"#,
                    r#"ALTER TABLE "user" DROP CONSTRAINT IF EXISTS "user_login_max_length";"#,
                ]
            );
        }

        /// Widening an integer, the other alterable type change
        #[test]
        fn widening_an_integer() {
            assert_eq!(
                alter(
                    DBImpl::Postgres,
                    alter_column(Some(DbType::Int64), &[Annotation::NotNull])
                ),
                [
                    r#"ALTER TABLE "user" ALTER COLUMN "login" TYPE bigint;"#,
                    r#"ALTER TABLE "user" DROP CONSTRAINT IF EXISTS "user_login_max_length";"#,
                ]
            );
        }

        /// `serial` is not a type but a column with its own sequence, which
        /// `ALTER COLUMN ... TYPE` would leave at its old type.
        #[test]
        fn altering_an_auto_increment_column_is_an_error() {
            assert!(matches!(
                alter_err(
                    DBImpl::Postgres,
                    alter_column(Some(DbType::Int64), &[Annotation::AutoIncrement])
                ),
                Error::SQLBuildError(msg) if msg.contains("auto_increment")
            ));
        }

        /// An enum's type is only known to the column which creates it
        #[test]
        fn altering_to_an_enum_is_an_error() {
            assert!(matches!(
                alter_err(
                    DBImpl::Postgres,
                    alter_column(Some(DbType::Choices), &[Annotation::Choices(vec![])])
                ),
                Error::SQLBuildError(msg) if msg.contains("enum")
            ));
        }
    }
}
