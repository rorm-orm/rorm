use rorm_declaration::migration::{Migration, Operation};
use rorm_sql::alter_table::SQLAlterTableOperation;
use rorm_sql::{value, DBImpl};

/**
Helper method to convert a migration to a transaction string

`db_impl`: [DBImpl]: The database implementation to use.
`migration`: [&Migration]: Reference to the migration that should be converted.
*/
pub fn migration_to_sql(
    db_impl: DBImpl,
    migration: &Migration,
) -> anyhow::Result<(String, Vec<value::Value>)> {
    let mut transaction = db_impl.start_transaction();
    let mut bind_params = vec![];

    for operation in &migration.operations {
        match &operation {
            Operation::CreateModel { name, fields } => {
                let mut create_table = db_impl.create_table(name.as_str());

                for field in fields {
                    create_table = create_table.add_column(db_impl.create_column(
                        name.as_str(),
                        field.name.as_str(),
                        field.db_type.clone(),
                        &field.annotations,
                    ));
                }

                let (query_string, query_bind_params) = create_table.build();

                transaction = transaction.add_statement(query_string);
                bind_params.extend(query_bind_params);
            }
            Operation::RenameModel { old, new } => {
                let (query_string, query_bind_params) = db_impl
                    .alter_table(
                        old.as_str(),
                        SQLAlterTableOperation::RenameTo {
                            name: new.to_string(),
                        },
                    )
                    .build();

                transaction = transaction.add_statement(query_string);
                bind_params.extend(query_bind_params);
            }
            Operation::DeleteModel { name } => {
                transaction = transaction.add_statement(db_impl.drop_table(name.as_str()).build())
            }
            Operation::CreateField { model, field } => {
                let (query_string, query_bind_params) = db_impl
                    .alter_table(
                        model.as_str(),
                        SQLAlterTableOperation::AddColumn {
                            operation: db_impl.create_column(
                                model.as_str(),
                                field.name.as_str(),
                                field.db_type.clone(),
                                &field.annotations,
                            ),
                        },
                    )
                    .build();

                transaction = transaction.add_statement(query_string);
                bind_params.extend(query_bind_params);
            }
            Operation::RenameField {
                table_name,
                old,
                new,
            } => {
                let (query_string, query_bind_params) = db_impl
                    .alter_table(
                        table_name.as_str(),
                        SQLAlterTableOperation::RenameColumnTo {
                            column_name: old.to_string(),
                            new_column_name: new.to_string(),
                        },
                    )
                    .build();

                transaction = transaction.add_statement(query_string);
                bind_params.extend(query_bind_params);
            }
            Operation::DeleteField { model, name } => {
                let (query_string, query_bind_params) = db_impl
                    .alter_table(
                        model.as_str(),
                        SQLAlterTableOperation::DropColumn { name: name.clone() },
                    )
                    .build();
                transaction = transaction.add_statement(query_string);
                bind_params.extend(query_bind_params);
            }
        }
    }

    Ok((transaction.finish(), bind_params))
}
