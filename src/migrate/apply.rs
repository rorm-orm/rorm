//! Contains two functions to apply a single migration or a single operation

use rorm_db::executor::{Executor, Nothing};
use rorm_db::sql::alter_table::{AlterTable, AlterTableOperation};
use rorm_db::sql::create_table::CreateTable;
use rorm_db::sql::drop_table::DropTable;
use rorm_db::sql::insert::Insert;
use rorm_db::sql::value::Value;
use rorm_db::sql::DBImpl;
use rorm_db::transaction::{Transaction, TransactionError};
use rorm_db::Database;
use rorm_declaration::migration::{Migration, Operation};
use thiserror::Error;

/// Applies a single `Migration`, updating the "last migration table".
///
/// This function will start a transaction
/// which is rolled back if any of the migration's operations failed.
///
/// This function won't check the databases current state.
/// It will simply try to apply the migration.
pub async fn apply_migration(
    db: &Database,
    migration: &Migration,
    last_migration_table_name: &str,
) -> Result<(), ApplyMigrationError> {
    let mut tx = db
        .start_transaction()
        .await
        .map_err(|error| ApplyMigrationError {
            error,
            location: ApplyMigrationErrorLocation::StartTransaction,
        })?;

    for (index, operation) in migration.operations.iter().enumerate() {
        apply_operation(&mut tx, operation)
            .await
            .map_err(|error| ApplyMigrationError {
                error,
                location: ApplyMigrationErrorLocation::ApplyOperation(index),
            })?;
    }

    let (query_string, bind_params) = db
        .dialect()
        .insert(
            last_migration_table_name,
            &["migration_id"],
            &[&[Value::I32(migration.id as i32)]],
            None,
        )
        .rollback_transaction()
        .build();

    tx.execute::<Nothing>(query_string, bind_params)
        .await
        .map_err(|error| ApplyMigrationError {
            error,
            location: ApplyMigrationErrorLocation::UpdateLastMigration,
        })?;

    tx.commit().await.map_err(|x| ApplyMigrationError {
        error: match x {
            TransactionError::Database(x) => x,
            TransactionError::Hook(_) => unreachable!("rorm-cli does not use hooks"),
        },
        location: ApplyMigrationErrorLocation::CommitTransaction,
    })?;

    Ok(())
}

/// Error returned by [`apply_migration`].
///
/// It is the raw `error` returned by the database
/// with an additional `location` indicating where in `apply_migration`
/// the error occurred.
#[derive(Debug, Error)]
#[error("{location}: {error}")]
pub struct ApplyMigrationError {
    /// Error returned by the database
    #[source]
    pub error: rorm_db::Error,

    /// Location where the `error` occurred
    pub location: ApplyMigrationErrorLocation,
}

/// Location where an [`ApplyMigrationError`] occurred.
#[derive(Debug, Error)]
pub enum ApplyMigrationErrorLocation {
    /// The error occurred while starting the transaction
    #[error("Failed to start transaction")]
    StartTransaction,

    /// The error occurred while applying an operation
    #[error("Failed to apply operation {}", .0)]
    ApplyOperation(usize),

    /// The error occurred while updating the "last migration table"
    #[error("Failed to update last migration")]
    UpdateLastMigration,

    /// The error occurred while commiting the transaction
    #[error("Failed to commit transaction")]
    CommitTransaction,
}

/// Applies a single migration `Operation`
pub async fn apply_operation(
    tx: &mut Transaction,
    operation: &Operation,
) -> Result<(), rorm_db::Error> {
    let db_impl = tx.dialect();

    match &operation {
        Operation::CreateModel { name, fields } => {
            let mut create_table = db_impl.create_table(name.as_str());

            for field in fields {
                create_table = create_table.add_column(db_impl.create_column(
                    name.as_str(),
                    field.name.as_str(),
                    field.db_type,
                    &field.annotations,
                ));
            }

            let statements = create_table.build()?;

            for (query_string, query_bind_params) in statements {
                tx.execute::<Nothing>(query_string, query_bind_params)
                    .await?;
            }
        }
        Operation::RenameModel { old, new } => {
            let statements = db_impl
                .alter_table(
                    old.as_str(),
                    AlterTableOperation::RenameTo {
                        name: new.to_string(),
                    },
                )
                .build()?;

            for (query_string, query_bind_params) in statements {
                tx.execute::<Nothing>(query_string, query_bind_params)
                    .await?;
            }
        }
        Operation::DeleteModel { name } => {
            let query_string = db_impl.drop_table(name.as_str()).build();

            tx.execute::<Nothing>(query_string, Vec::new()).await?;
        }
        Operation::CreateField { model, field } => {
            let statements = db_impl
                .alter_table(
                    model.as_str(),
                    AlterTableOperation::AddColumn {
                        operation: db_impl.create_column(
                            model.as_str(),
                            field.name.as_str(),
                            field.db_type,
                            &field.annotations,
                        ),
                    },
                )
                .build()?;

            for (query_string, query_bind_params) in statements {
                tx.execute::<Nothing>(query_string, query_bind_params)
                    .await?;
            }
        }
        Operation::RenameField {
            table_name,
            old,
            new,
        } => {
            let statements = db_impl
                .alter_table(
                    table_name.as_str(),
                    AlterTableOperation::RenameColumnTo {
                        column_name: old.to_string(),
                        new_column_name: new.to_string(),
                    },
                )
                .build()?;

            for (query_string, query_bind_params) in statements {
                tx.execute::<Nothing>(query_string, query_bind_params)
                    .await?;
            }
        }
        Operation::DeleteField { model, name } => {
            let statements = db_impl
                .alter_table(
                    model.as_str(),
                    AlterTableOperation::DropColumn { name: name.clone() },
                )
                .build()?;

            for (query_string, query_bind_params) in statements {
                tx.execute::<Nothing>(query_string, query_bind_params)
                    .await?;
            }
        }
        #[allow(unused_variables)]
        Operation::RawSQL {
            mysql,
            postgres,
            sqlite,
            ..
        } => match db_impl {
            #[cfg(feature = "sqlite")]
            DBImpl::SQLite => tx.execute::<Nothing>(sqlite.clone(), Vec::new()).await?,
            #[cfg(feature = "postgres")]
            DBImpl::Postgres => tx.execute::<Nothing>(postgres.clone(), Vec::new()).await?,
        },
    }

    Ok(())
}
