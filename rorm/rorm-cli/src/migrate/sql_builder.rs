use rorm_declaration::migration::{Migration, Operation};
use rorm_sql::alter_table::SQLAlterTableOperation;
use rorm_sql::DBImpl;
use sqlx::{Any, Transaction};

use crate::utils::bind;

/**
Helper method to convert a migration to a transaction string

`db_impl`: [DBImpl]: The database implementation to use.
`migration`: [&Migration]: Reference to the migration that should be converted.
*/
pub async fn migration_to_sql<'a>(
    tx: &'a mut Transaction<'_, Any>,
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
                    if do_log {
                        println!("{}", query_string.as_str());
                    }

                    let mut q = sqlx::query(query_string.as_str());
                    for x in query_bind_params {
                        q = bind::bind_param(q, x);
                    }
                    q.execute(&mut *tx).await?;
                }
            }
            Operation::RenameModel { old, new } => {
                let statements = db_impl
                    .alter_table(
                        old.as_str(),
                        SQLAlterTableOperation::RenameTo {
                            name: new.to_string(),
                        },
                    )
                    .build()?;

                for (query_string, query_bind_params) in statements {
                    if do_log {
                        println!("{}", query_string.as_str());
                    }

                    let mut q = sqlx::query(query_string.as_str());
                    for x in query_bind_params {
                        q = bind::bind_param(q, x);
                    }
                    q.execute(&mut *tx).await?;
                }
            }
            Operation::DeleteModel { name } => {
                let query_string = db_impl.drop_table(name.as_str()).build();

                if do_log {
                    println!("{}", query_string.as_str());
                }

                sqlx::query(query_string.as_str()).execute(&mut *tx).await?;
            }
            Operation::CreateField { model, field } => {
                let statements = db_impl
                    .alter_table(
                        model.as_str(),
                        SQLAlterTableOperation::AddColumn {
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
                    if do_log {
                        println!("{}", query_string.as_str());
                    }

                    let mut q = sqlx::query(query_string.as_str());
                    for x in query_bind_params {
                        q = bind::bind_param(q, x);
                    }
                    q.execute(&mut *tx).await?;
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
                        SQLAlterTableOperation::RenameColumnTo {
                            column_name: old.to_string(),
                            new_column_name: new.to_string(),
                        },
                    )
                    .build()?;

                for (query_string, query_bind_params) in statements {
                    if do_log {
                        println!("{}", query_string.as_str());
                    }

                    let mut q = sqlx::query(query_string.as_str());
                    for x in query_bind_params {
                        q = bind::bind_param(q, x);
                    }
                    q.execute(&mut *tx).await?;
                }
            }
            Operation::DeleteField { model, name } => {
                let statements = db_impl
                    .alter_table(
                        model.as_str(),
                        SQLAlterTableOperation::DropColumn { name: name.clone() },
                    )
                    .build()?;

                for (query_string, query_bind_params) in statements {
                    if do_log {
                        println!("{}", query_string.as_str());
                    }

                    let mut q = sqlx::query(query_string.as_str());
                    for x in query_bind_params {
                        q = bind::bind_param(q, x);
                    }
                    q.execute(&mut *tx).await?;
                }
            }
        }
    }

    Ok(())
}
