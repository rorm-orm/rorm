use std::collections::hash_map::DefaultHasher;
use std::fs::{create_dir_all, read_to_string};
use std::hash::{Hash, Hasher};
use std::path::Path;

use anyhow::{anyhow, Context};
use once_cell::sync::Lazy;
use regex::Regex;
use rorm_common::imr::{Field, InternalModelFormat};

use crate::declaration::{Migration, Operation};
use crate::utils::migrations::{
    convert_migration_to_file, convert_migrations_to_internal_models, get_existing_migrations,
};

pub static RE_ALLOWED_NAME: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^[\d\w]+$"#).unwrap());

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

    match &options.name {
        Some(name) => {
            if !RE_ALLOWED_NAME.is_match(name.as_str()) {
                return Err(anyhow!("Custom migration name is not allowed"));
            }
        }
        None => {}
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

    let internal_models = get_internal_models(&options.models_file.as_str())
        .with_context(|| "Couldn't retrieve internal model files.")?;

    let existing_migrations = get_existing_migrations(&options.migration_dir.as_str())
        .with_context(|| "An error occurred while deserializing migrations")?;

    let mut hasher = DefaultHasher::new();
    internal_models.hash(&mut hasher);
    let h = hasher.finish();

    if existing_migrations.len() != 0 {
        let last_migration = &existing_migrations[existing_migrations.len() - 1];

        // If hash matches with the one of the current models, exiting
        if &last_migration.hash == &h {
            println!("No changes - nothing to do.");
            return Ok(());
        }

        let constructed = convert_migrations_to_internal_models(&existing_migrations);

        let mut last_id: u16 = last_migration.id[..4]
            .parse()
            .with_context(|| "Failed converting name of migration to int")?;
        last_id += 1;

        let name = match options.name {
            None => format!("{:04}_placeholder", last_id),
            Some(n) => format!("{:04}_{}", last_id, n),
        };

        let op: Vec<Operation> = vec![];

        let new_migration = Migration {
            hash: h,
            initial: false,
            id: name.clone(),
            dependency: last_migration.id.clone(),
            replaces: vec![],
            operations: op,
        };

        // Write migration to disk
        let path = Path::new(options.migration_dir.as_str()).join(format!("{}.toml", name));
        convert_migration_to_file(new_migration, &path)
            .with_context(|| "Error occurred while converting migration to file")?;
    } else {
        // New migration must be generated as no migration exists

        let name = match options.name {
            None => "0001_initial".to_string(),
            Some(n) => format!("0001_{}", n),
        };

        let new_migration = Migration {
            hash: h,
            initial: true,
            id: name.clone(),
            dependency: "".to_string(),
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
                                db_type: y.db_type.clone(),
                                annotations: y.annotations.clone(),
                                source_defined_at: None,
                            })
                            .collect(),
                    };
                    println!("Created model {}", x.name);
                    return o;
                })
                .collect(),
        };

        // Write migration to disk
        let path = Path::new(options.migration_dir.as_str()).join(format!("{}.toml", name));
        convert_migration_to_file(new_migration, &path)
            .with_context(|| "Error occurred while converting migration to file")?;
    }

    Ok(())
}
