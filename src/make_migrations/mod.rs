use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs::{create_dir_all, read_to_string};
use std::hash::{Hash, Hasher};
use std::path::Path;

use anyhow::{anyhow, Context};
use rorm_declaration::imr::{Annotation, Field, InternalModelFormat, Model};
use rorm_declaration::migration::{Migration, Operation};
use tracing::info;

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

/// Checks the options
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

/// A helper function to retrieve the internal models from a given location.
///
/// `models_file`: [&str]: The path to the models file.
pub fn get_internal_models(models_file: &str) -> anyhow::Result<InternalModelFormat> {
    let internal_str = read_to_string(Path::new(&models_file))
        .with_context(|| "Couldn't read internal models file")?;
    let internal: InternalModelFormat = serde_json::from_str(internal_str.as_str())
        .with_context(|| "Error deserializing internal models file")?;

    Ok(internal)
}

/// Runs the make-migrations tool
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
            info!("No changes - nothing to do.");
            return Ok(());
        }

        let constructed = convert_migrations_to_internal_models(&existing_migrations)
            .with_context(|| "Error while parsing existing migration files")?;

        let last_id: u16 = last_migration.id + 1;
        let name = options.name.as_deref().unwrap_or("placeholder");

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
        for new_model in &internal_models.models {
            if !old_lookup.iter().any(|(a, _)| new_model.name == *a) {
                new_models.push(new_model);
            }
        }

        // Check if any old model got deleted
        for old_model in &constructed.models {
            if !new_lookup.iter().any(|(a, _)| old_model.name == *a) {
                deleted_models.push(old_model);
            }
        }

        // Iterate over all models, that are in the constructed
        // as well as in the new internal models
        for new_model in &internal_models.models {
            let Some(old_model) = old_lookup.get(&new_model.name) else {
                continue;
            };

            // Check if a new field has been added
            for new_field in &new_model.fields {
                if !old_model.fields.iter().any(|z| z.name == new_field.name) {
                    new_fields
                        .entry(new_model.name.clone())
                        .or_default()
                        .push(new_field);
                }
            }

            // Check if a existing field got deleted
            for old_field in &old_model.fields {
                if !new_model.fields.iter().any(|z| z.name == old_field.name) {
                    deleted_fields
                        .entry(new_model.name.clone())
                        .or_default()
                        .push(old_field);
                }
            }

            // Check if a existing field got altered
            for old_field in &old_model.fields {
                for new_field in &new_model.fields {
                    if old_field.name != new_field.name {
                        continue;
                    }

                    // Check for differences
                    if old_field.db_type != new_field.db_type
                        || old_field.annotations != new_field.annotations
                    {
                        altered_fields
                            .entry(new_model.name.clone())
                            .or_default()
                            .push((old_field, new_field));
                    }
                }
            }
        }

        // Check if a model was renamed
        if !new_models.is_empty() && !deleted_models.is_empty() {
            for new_model in &new_models {
                for old_model in &deleted_models {
                    if new_model.fields == old_model.fields
                        && question(
                            format!(
                                "Did you rename the model {} to {}?",
                                old_model.name, new_model.name
                            )
                            .as_str(),
                        )
                    {
                        info!("Renamed model {} to {}.", old_model.name, new_model.name);
                        renamed_models.push((old_model, new_model));
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

        let mut references: HashMap<String, Vec<Field>> = HashMap::new();

        // Create migration operations for new models
        for new_model in &new_models {
            let mut normal_fields = vec![];

            for new_field in &new_model.fields {
                if new_field
                    .annotations
                    .iter()
                    .any(|x| matches!(x, Annotation::ForeignKey(_)))
                {
                    references
                        .entry(new_model.name.clone())
                        .or_default()
                        .push(new_field.clone());
                } else {
                    normal_fields.push(new_field.clone());
                }
            }

            op.push(Operation::CreateModel {
                name: new_model.name.clone(),
                fields: normal_fields,
            });
            info!("Created model {}", new_model.name);
        }

        // Create referencing fields for new models
        for (model, fields) in references {
            for field in fields {
                op.push(Operation::CreateField {
                    model: model.clone(),
                    field,
                });
            }
        }

        // Create migration operations for deleted models
        for deleted_model in &deleted_models {
            op.push(Operation::DeleteModel {
                name: deleted_model.name.clone(),
            });
            info!("Deleted model {}", deleted_model.name);
        }

        for (model_name, new_fields) in &new_fields {
            if let Some(old_fields) = deleted_fields.get(model_name) {
                for new_field in new_fields {
                    for old_field in old_fields {
                        if new_field.db_type == old_field.db_type
                            && new_field.annotations == old_field.annotations
                            && question(
                                format!(
                                    "Did you rename the field {} of model {model_name} to {}?",
                                    old_field.name, new_field.name
                                )
                                .as_str(),
                            )
                        {
                            renamed_fields
                                .entry(model_name.clone())
                                .or_default()
                                .push((old_field, new_field));
                            info!(
                                "Renamed field {} of model {model_name} to {}.",
                                old_field.name, new_field.name
                            );
                        }
                    }
                }
            }
        }
        // Remove renamed fields in existing models from new and deleted lists
        for (model_name, fields) in &renamed_fields {
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
        }

        // Create migration operations for new fields in existing models
        for (model_name, fields) in &new_fields {
            for field in fields {
                op.push(Operation::CreateField {
                    model: model_name.clone(),
                    field: (*field).clone(),
                });
                info!("Added field {} to model {}", field.name, model_name);
            }
        }

        // Create migration operations for deleted fields in existing models
        for (model_name, fields) in &deleted_fields {
            for field in fields {
                op.push(Operation::DeleteField {
                    model: model_name.clone(),
                    name: field.name.clone(),
                });
                info!("Deleted field {} from model {}", field.name, model_name);
            }
        }

        // Create migration operations for altered fields in existing models
        for (model, af) in &altered_fields {
            for (old, new) in af {
                // Check datatype
                if old.db_type != new.db_type {
                    #[expect(clippy::match_single_binding, reason = "It will be extended™")]
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
                            info!("Recreated field {} on model {}", &new.name, &model);
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
                    info!("Recreated field {} on model {}", &new.name, &model);
                }
            }
        }

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
            info!("No models found.");
        // New migration must be generated as no migration exists
        } else {
            let mut operations = vec![];
            let mut references: HashMap<String, Vec<Field>> = HashMap::new();

            operations.extend(internal_models.models.iter().map(|model| {
                let mut normal_fields = vec![];

                for field in &model.fields {
                    if field
                        .annotations
                        .iter()
                        .any(|x| matches!(x, Annotation::ForeignKey(_)))
                    {
                        references
                            .entry(model.name.clone())
                            .or_default()
                            .push(field.clone());
                    } else {
                        normal_fields.push(field.clone());
                    }
                }

                info!("Created model {}", model.name);
                Operation::CreateModel {
                    name: model.name.clone(),
                    fields: normal_fields,
                }
            }));

            operations.extend(references.into_iter().flat_map(|(model, fields)| {
                fields
                    .iter()
                    .map(|field| Operation::CreateField {
                        model: model.clone(),
                        field: field.clone(),
                    })
                    .collect::<Vec<Operation>>()
            }));

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
                operations,
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

    info!("Done.");

    Ok(())
}
