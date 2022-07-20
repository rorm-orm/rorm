use anyhow::Context;
use rorm_sql::DBImpl;

use crate::declaration::{Migration, Operation};

/**
Helper method to convert a migration to a transaction string

`db_impl`: [DBImpl]: The database implementation to use.
`migration`: [&Migration]: Reference to the migration that should be converted.
*/
pub fn migration_to_sql(db_impl: DBImpl, migration: &Migration) -> anyhow::Result<String> {
    let mut transaction = db_impl.start_transaction();
    for operation in &migration.operations {
        match &operation {
            Operation::CreateModel { name, fields } => {
                let mut create_table = db_impl.create_table(name.as_str());

                for field in fields {
                    create_table = create_table.add_column(
                        field.name.as_str(),
                        field.db_type.clone(),
                        field.annotations.clone(),
                    );
                }

                transaction =
                    transaction.add_statement(create_table.build().with_context(|| {
                        format!(
                            "Could not build create table operation for migration {}",
                            migration.id.as_str()
                        )
                    })?);
            }
            Operation::DeleteModel { .. } => {}
            Operation::CreateField { .. } => {}
            Operation::DeleteField { .. } => {}
        }
    }

    Ok(transaction.finish().with_context(|| {
        format!(
            "Could not create transaction for migration {}",
            migration.id.as_str()
        )
    })?)
}
