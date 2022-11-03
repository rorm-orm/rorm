use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs::{create_dir_all, read_to_string};
use std::hash::{Hash, Hasher};
use std::path::Path;

use anyhow::{anyhow, Context};
use rorm_declaration::imr::{Field, InternalModelFormat, Model};
use rorm_declaration::migration::{Migration, Operation};

use crate::linter;
use crate::utils::migrations::{
    convert_migration_to_file, convert_migrations_to_internal_models, get_existing_migrations,
};
use crate::utils::question;
use crate::utils::re::RE;

/// Options struct for [run_make_migrations]
#[derive(Debug)]
pub struct MakeMigrationsOptions {
    /// Path to internal model file
    pub models_file: String,
    /// Path to the migration directory
    pub migration_dir: String,
    /// Alternative name of the migration
    pub name: Option<String>,
    /// If set, no questions are gonna be asked
    pub non_interactive: bool,
    /// If set, all warnings are suppressed
    pub warnings_disabled: bool,
}

/**
Checks the options
*/
pub fn check_options(options: &MakeMigrationsOptions) -> anyhow::Result<()> {
    let models_file = Path::new(options.models_file.as_str());
    if !models_file.exists() || !models_file.is_file() {
        return Err(anyhow!("Models file does not exist"));
    }

    let migration_dir = Path::new(options.migration_dir.as_str());
    if migration_dir.is_file() {
        return Err(anyhow!("Migration directory cannot be created, is a file"));
    }
    if !migration_dir.exists() {
        create_dir_all(migration_dir).with_context(|| "Couldn't create migration directory")?;
    }

    if let Some(name) = &options.name {
        if !RE.migration_allowed_comment.is_match(name.as_str()) {
            return Err(anyhow!(
                "Custom migration name contains illegal characters!"
            ));
        }
    }

    Ok(())
}

/**
A helper function to retrieve the internal models from a given location.

`models_file`: [&str]: The path to the models file.
*/
pub fn get_internal_models(models_file: &str) -> anyhow::Result<InternalModelFormat> {
    let internal_str = read_to_string(Path::new(&models_file))
        .with_context(|| "Couldn't read internal models file")?;
    let internal: InternalModelFormat = serde_json::from_str(internal_str.as_str())
        .with_context(|| "Error deserializing internal models file")?;

    Ok(internal)
}

/**
Runs the make-migrations tool
*/
pub fn run_make_migrations(options: MakeMigrationsOptions) -> anyhow::Result<()> {
    check_options(&options).with_context(|| "Error while checking options")?;

    let internal_models = get_internal_models(&options.models_file)
        .with_context(|| "Couldn't retrieve internal model files.")?;

    linter::check_internal_models(&internal_models).with_context(|| "Model checks failed.")?;

    let existing_migrations = get_existing_migrations(&options.migration_dir)
        .with_context(|| "An error occurred while deserializing migrations")?;

    let mut hasher = DefaultHasher::new();
    internal_models.hash(&mut hasher);
    let h = hasher.finish();

    let mut new_migration = None;

    if !existing_migrations.is_empty() {
        let last_migration = &existing_migrations[existing_migrations.len() - 1];

        // If hash matches with the one of the current models, exiting
        if last_migration.hash == h.to_string() {
            println!("No changes - nothing to do.");
            return Ok(());
        }

        let constructed = convert_migrations_to_internal_models(&existing_migrations)
            .with_context(|| "Error while parsing existing migration files")?;

        let last_id: u16 = last_migration.id + 1;
        let name = options.name.as_ref().map_or("placeholder", |x| x.as_str());

        let mut op: Vec<Operation> = vec![];

        let old_lookup: HashMap<String, &Model> = constructed
            .models
            .iter()
            .map(|x| (x.name.clone(), x))
            .collect();

        let new_lookup: HashMap<String, &Model> = internal_models
            .models
            .iter()
            .map(|x| (x.name.clone(), x))
            .collect();

        // Old -> New
        let mut renamed_models: Vec<(&Model, &Model)> = vec![];
        let mut new_models: Vec<&Model> = vec![];
        let mut deleted_models: Vec<&Model> = vec![];

        // Mapping: Model name -> (Old field name, New field name)
        let mut renamed_fields: HashMap<String, Vec<(&Field, &Field)>> = HashMap::new();
        let mut new_fields: HashMap<String, Vec<&Field>> = HashMap::new();
        let mut deleted_fields: HashMap<String, Vec<&Field>> = HashMap::new();
        // Mapping: Model name -> (Old field, new field)
        let mut altered_fields: HashMap<String, Vec<(&Field, &Field)>> = HashMap::new();

        // Check if any new models exist
        internal_models.models.iter().for_each(|x| {
            if !old_lookup.iter().any(|(a, _)| x.name == *a) {
                new_models.push(x);
            }
        });

        // Check if any old model got deleted
        constructed.models.iter().for_each(|x| {
            if !new_lookup.iter().any(|(a, _)| x.name == *a) {
                deleted_models.push(x);
            }
        });

        // Iterate over all models, that are in the constructed
        // as well as in the new internal models
        internal_models
            .models
            .iter()
            .filter(|x| old_lookup.get(x.name.as_str()).is_some())
            .for_each(|x| {
                // Check if a new field has been added
                x.fields.iter().for_each(|y| {
                    if !old_lookup[x.name.as_str()]
                        .fields
                        .iter()
                        .any(|z| z.name == y.name)
                    {
                        if new_fields.get(x.name.as_str()).is_none() {
                            new_fields.insert(x.name.clone(), vec![]);
                        }
                        new_fields.get_mut(x.name.as_str()).unwrap().push(y);
                    }
                });

                // Check if a existing field got deleted
                old_lookup[x.name.as_str()].fields.iter().for_each(|y| {
                    if !x.fields.iter().any(|z| z.name == y.name) {
                        if deleted_fields.get(x.name.as_str()).is_none() {
                            deleted_fields.insert(x.name.clone(), vec![]);
                        }
                        deleted_fields.get_mut(x.name.as_str()).unwrap().push(y);
                    }
                });

                // Check if a existing field got altered
                old_lookup[x.name.as_str()].fields.iter().for_each(|y| {
                    x.fields.iter().filter(|z| y.name == z.name).for_each(|z| {
                        // Check for differences
                        if y.db_type != z.db_type || y.annotations != z.annotations {
                            if altered_fields.get(x.name.as_str()).is_none() {
                                altered_fields.insert(x.name.clone(), vec![]);
                            }
                            altered_fields.get_mut(&x.name).unwrap().push((y, z));
                        }
                    });
                });
            });

        // Check if a model was renamed
        if !new_models.is_empty() && !deleted_models.is_empty() {
            for x in &new_models {
                for y in &deleted_models {
                    if x.fields == y.fields
                        && question(
                            format!("Did you rename the model {} to {}?", &y.name, &x.name)
                                .as_str(),
                        )
                    {
                        println!("Renamed model {} to {}.", &y.name, &x.name);
                        renamed_models.push((y, x));
                    }
                }
            }
        }
        // Remove renamed models from new and deleted lists
        for (old, new) in &renamed_models {
            new_models.retain(|x| x != new);
            deleted_models.retain(|x| x != old);

            // Create migration operations for renamed models
            op.push(Operation::RenameModel {
                old: old.name.clone(),
                new: new.name.clone(),
            })
        }

        // Create migration operations for new models
        new_models.iter().for_each(|x| {
            op.push(Operation::CreateModel {
                name: x.name.clone(),
                fields: x.fields.clone(),
            });
            println!("Created model {}", x.name);
        });

        // Create migration operations for deleted models
        deleted_models.iter().for_each(|x| {
            op.push(Operation::DeleteModel {
                name: x.name.clone(),
            });
            println!("Deleted model {}", x.name);
        });

        for (x, new_fields) in &new_fields {
            if let Some(old_fields) = deleted_fields.get(x) {
                for new_field in new_fields {
                    for old_field in old_fields {
                        if new_field.db_type == old_field.db_type
                            && new_field.annotations == old_field.annotations
                            && question(
                                format!(
                                    "Did you rename the field {} of model {} to {}?",
                                    &old_field.name, &x, &new_field.name
                                )
                                .as_str(),
                            )
                        {
                            if !renamed_fields.contains_key(x) {
                                renamed_fields.insert(x.clone(), vec![]);
                            }
                            let f = renamed_fields.get_mut(x).unwrap();
                            f.push((old_field, new_field));
                            println!(
                                "Renamed field {} of model {} to {}.",
                                &new_field.name, &x, &old_field.name
                            );
                        }
                    }
                }
            }
        }
        // Remove renamed fields in existing models from new and deleted lists
        renamed_fields.iter().for_each(|(model_name, fields)| {
            for (old_field, new_field) in fields {
                new_fields
                    .get_mut(model_name)
                    .unwrap()
                    .retain(|x| x.name != new_field.name);
                deleted_fields
                    .get_mut(model_name)
                    .unwrap()
                    .retain(|x| x.name != old_field.name);

                // Create migration operation for renamed fields on existing models
                op.push(Operation::RenameField {
                    table_name: model_name.clone(),
                    old: old_field.name.clone(),
                    new: new_field.name.clone(),
                })
            }
        });

        // Create migration operations for new fields in existing models
        new_fields.iter().for_each(|(x, y)| {
            y.iter().for_each(|z| {
                op.push(Operation::CreateField {
                    model: x.clone(),
                    field: (*z).clone(),
                });
                println!("Added field {} to model {}", z.name, x);
            })
        });

        // Create migration operations for deleted fields in existing models
        deleted_fields.iter().for_each(|(x, y)| {
            y.iter().for_each(|z| {
                op.push(Operation::DeleteField {
                    model: x.clone(),
                    name: z.name.clone(),
                });
                println!("Deleted field {} from model {}", z.name, x);
            })
        });

        // Create migration operations for altered fields in existing models
        altered_fields.iter().for_each(|(model, af)| {
            af.iter().for_each(|(old, new)| {
                // Check datatype
                if old.db_type != new.db_type {
                    match (old.db_type, new.db_type) {
                        // TODO:
                        // There are cases where columns can be altered
                        // e.g. i8 -> i16 or float -> double

                        // Default case
                        (_, _) => {
                            op.push(Operation::DeleteField {
                                model: model.clone(),
                                name: old.name.clone(),
                            });
                            op.push(Operation::CreateField {
                                model: model.clone(),
                                field: (*new).clone(),
                            });
                            println!("Recreated field {} on model {}", &new.name, &model);
                        }
                    }
                } else {
                    // As the datatypes match, there must be a change in the annotations
                    op.push(Operation::DeleteField {
                        model: model.clone(),
                        name: old.name.clone(),
                    });
                    op.push(Operation::CreateField {
                        model: model.clone(),
                        field: (*new).clone(),
                    });
                    println!("Recreated field {} on model {}", &new.name, &model);
                }
            });
        });

        new_migration = Some(Migration {
            hash: h.to_string(),
            initial: false,
            id: last_id,
            name: name.to_string(),
            dependency: Some(last_migration.id),
            replaces: vec![],
            operations: op,
        });
    } else {
        // If there are no models yet, no migrations must be created
        if internal_models.models.is_empty() {
            println!("No models found.");
        // New migration must be generated as no migration exists
        } else {
            new_migration = Some(Migration {
                hash: h.to_string(),
                initial: true,
                id: 1,
                name: match &options.name {
                    None => "initial".to_string(),
                    Some(n) => n.clone(),
                },
                dependency: None,
                replaces: vec![],
                operations: internal_models
                    .models
                    .iter()
                    .map(|x| {
                        let o = Operation::CreateModel {
                            name: x.name.clone(),
                            fields: x
                                .fields
                                .iter()
                                .map(|y| Field {
                                    name: y.name.clone(),
                                    db_type: y.db_type,
                                    annotations: y.annotations.clone(),
                                    source_defined_at: None,
                                })
                                .collect(),
                        };
                        println!("Created model {}", x.name);
                        o
                    })
                    .collect(),
            });
        }
    }

    if let Some(migration) = new_migration {
        // Write migration to disk
        let path = Path::new(options.migration_dir.as_str())
            .join(format!("{:04}_{}.toml", migration.id, &migration.name));
        convert_migration_to_file(migration, &path)
            .with_context(|| "Error occurred while converting migration to file")?;
    }

    println!("Done.");

    Ok(())
}
