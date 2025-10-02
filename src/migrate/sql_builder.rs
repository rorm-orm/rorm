use rorm_db::executor::{Executor, Nothing};
use rorm_db::sql::alter_table::{AlterTable, AlterTableOperation};
use rorm_db::sql::create_table::CreateTable;
use rorm_db::sql::drop_table::DropTable;
use rorm_db::sql::value::Value;
use rorm_db::sql::DBImpl;
use rorm_db::transaction::Transaction;
use rorm_declaration::migration::{Migration, Operation};

/// Helper method to convert a migration to a transaction string
///
/// - `db_impl`: [`DBImpl`]: The database implementation to use.
/// - `migration`: [`&Migration`](Migration): Reference to the migration that should be converted.
pub async fn migration_to_sql<'a>(
    tx: &'a mut Transaction,
    db_impl: DBImpl,
    migration: &'a Migration,
    do_log: bool,
) -> anyhow::Result<()> {
    for operation in &migration.operations {
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
                    execute_statement(tx, query_string, query_bind_params, do_log).await?;
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
                    execute_statement(tx, query_string, query_bind_params, do_log).await?;
                }
            }
            Operation::DeleteModel { name } => {
                let query_string = db_impl.drop_table(name.as_str()).build();

                if do_log {
                    println!("{}", query_string.as_str());
                }

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
                    execute_statement(tx, query_string, query_bind_params, do_log).await?;
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
                    execute_statement(tx, query_string, query_bind_params, do_log).await?;
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
                    execute_statement(tx, query_string, query_bind_params, do_log).await?;
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
                DBImpl::SQLite => {
                    execute_statement(tx, sqlite.clone(), Vec::new(), do_log).await?;
                }
                #[cfg(feature = "postgres")]
                DBImpl::Postgres => {
                    execute_statement(tx, postgres.clone(), Vec::new(), do_log).await?;
                }
                #[cfg(feature = "mysql")]
                DBImpl::MySQL => {
                    execute_statement(tx, mysql.clone(), Vec::new(), do_log).await?;
                }
            },
        }
    }

    Ok(())
}

async fn execute_statement(
    tx: &mut Transaction,
    query_string: String,
    query_bind_params: Vec<Value<'_>>,
    do_log: bool,
) -> Result<(), rorm_db::Error> {
    if do_log {
        println!("{}", query_string.as_str());
    }
    tx.execute::<Nothing>(query_string, query_bind_params).await
}
