use std::fs::{read_dir, read_to_string, DirEntry, File};
use std::io::Write;
use std::path::Path;

use anyhow::{anyhow, Context};
use rorm_declaration::imr::{InternalModelFormat, Model};
use rorm_declaration::migration::{Migration, MigrationFile, Operation};

use crate::utils::re::RE;

/**
This function is used to convert the [InternalModelFormat] into its TOML representation.

`migration` [Migration]: Migration to be converted into TOML
`path` [&str]: The path to write the resulting TOML to
 */
pub fn convert_migration_to_file(migration: Migration, path: &Path) -> anyhow::Result<()> {
    let toml_str = toml::to_string_pretty(&MigrationFile { migration })
        .with_context(|| "Error while serializing migration")?;

    let mut output = File::create(path).with_context(|| {
        format!(
            "Error while opening file {:?} to write migration into",
            path.file_name()
        )
    })?;
    write!(output, "{}", toml_str).with_context(|| "Error while writing to migration file")?;

    Ok(())
}

/**
This function tries to convert a file to a [Migration].

`path` [&DirEntry]: Path to the file that should be parsed.
*/
pub fn convert_file_to_migration(path: &DirEntry) -> anyhow::Result<MigrationFile> {
    let toml_str = read_to_string(path.path()).with_context(|| {
        format!(
            "Error occurred while reading {}",
            path.path().to_str().unwrap()
        )
    })?;

    let mut migration: MigrationFile = toml::from_str(toml_str.as_str()).with_context(|| {
        format!(
            "Error while deserializing migration {:?} from TOML",
            path.file_name()
        )
    })?;

    migration.migration.id = path.path().file_stem().unwrap().to_str().unwrap()[..4].parse()?;
    migration.migration.name = path.path().file_stem().unwrap().to_str().unwrap()[5..].to_string();

    Ok(migration)
}

pub(crate) fn get_migration_files(migration_dir: &str) -> anyhow::Result<Vec<DirEntry>> {
    let dir_entries =
        read_dir(migration_dir).with_context(|| "Error while searching the migration directory")?;

    let file_list: Vec<DirEntry> = dir_entries
        .filter(|x| {
            x.as_ref().unwrap().file_type().unwrap().is_file()
                && RE.migration_allowed_name.is_match(
                    x.as_ref()
                        .unwrap()
                        .file_name()
                        .into_string()
                        .unwrap()
                        .as_str(),
                )
        })
        .map(|x| x.unwrap())
        .collect();

    Ok(file_list)
}

/**
Helper function to retrieve a sorted list of migrations in a given directory.

This strips also migrations, that were replaced.

**Parameter**:
- `migration_dir`: [&str] The directory to search for files.
this point onwards.
*/
pub fn get_existing_migrations(migration_dir: &str) -> anyhow::Result<Vec<Migration>> {
    let migrations = get_all_existing_migrations(migration_dir)?;

    let mut migration_list: Vec<Migration> = vec![];

    // Filter out migrations that replace migrations
    for m in migrations {
        if m.replaces.is_empty() {
            migration_list.push(m);
        }
    }

    let mut sorted_migration_list: Vec<Migration> = vec![];

    let mut current_id = None;
    loop {
        match current_id {
            None => {
                if let Some(&initial) = migration_list
                    .iter()
                    .filter(|x| x.initial)
                    .collect::<Vec<&Migration>>()
                    .first()
                {
                    current_id = Some(initial.id);
                    sorted_migration_list.push(initial.clone());
                    continue;
                }
            }
            Some(curr) => {
                if let Some(&next) = migration_list
                    .iter()
                    .filter(|x| {
                        if let Some(dependency) = x.dependency {
                            dependency == curr
                        } else {
                            false
                        }
                    })
                    .collect::<Vec<&Migration>>()
                    .first()
                {
                    current_id = Some(next.id);
                    sorted_migration_list.push(next.clone());
                    continue;
                }
            }
        }
        break;
    }

    if sorted_migration_list.len() != migration_list.len() {
        return Err(anyhow!("Migrations does not assemble to a coherent list."));
    }

    Ok(sorted_migration_list)
}

/**
Helper function to retrieve an unsorted list of **all** migrations in a given directory.

`migration_dir`: [&str] The directory to search for files.
 */
pub fn get_all_existing_migrations(migration_dir: &str) -> anyhow::Result<Vec<Migration>> {
    let file_list = get_migration_files(migration_dir)?;
    let mut migration_list: Vec<Migration> = vec![];
    for file in &file_list {
        migration_list.push(convert_file_to_migration(file)?.migration);
    }

    Ok(migration_list)
}

/**
Helper function to converts a list of migrations to an internal model.

`migrations`: [Vec<Migration>]: List of migrations
 */
pub fn convert_migrations_to_internal_models(
    migrations: &[Migration],
) -> anyhow::Result<InternalModelFormat> {
    let mut m = vec![];

    let res: anyhow::Result<_> = migrations.iter().try_for_each(|x| {
        x.operations.iter().try_for_each(|y| match y {
            Operation::CreateModel { name, fields } => {
                m.push(Model {
                    name: name.clone(),
                    fields: fields.clone(),
                    source_defined_at: None,
                });
                Ok(())
            }
            Operation::RenameModel { old, new } => {
                m = m
                    .iter()
                    .map(|z| {
                        let mut a = z.clone();
                        if &a.name == old {
                            a.name = new.to_string();
                        }
                        a
                    })
                    .collect();
                Ok(())
            }
            Operation::DeleteModel { name } => {
                m.retain(|z| z.name != *name);
                Ok(())
            }
            Operation::CreateField { model, field } => {
                for i in &mut m {
                    if i.name == *model {
                        i.fields.push(field.clone());
                    }
                }
                Ok(())
            }
            Operation::RenameField {
                table_name,
                old,
                new,
            } => {
                m = m
                    .iter()
                    .map(|z| {
                        let mut a = z.clone();
                        if &a.name == table_name {
                            a.fields = a
                                .fields
                                .iter()
                                .map(|b| {
                                    let mut c = b.clone();
                                    if &c.name == old {
                                        c.name = new.to_string();
                                    }
                                    c
                                })
                                .collect();
                        }
                        a
                    })
                    .collect();
                Ok(())
            }
            Operation::DeleteField { model, name } => {
                for i in &mut m {
                    if i.name == *model {
                        i.fields.retain(|z| z.name != *name);
                    }
                }
                Ok(())
            }
            Operation::RawSQL { .. } => Err(anyhow!(
                r#"RawSQL migration found!

Can not proceed to generate migrations as the current database state can not be determined anymore!
You can still write migrations with all available operations yourself.

To use the make-migrations feature again, delete all RawSQL operations."#
            )),
        })
    });

    res?;

    Ok(InternalModelFormat { models: m })
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use rorm_declaration::migration::Migration;
    use temp_dir::TempDir;

    use crate::utils::migrations::{convert_migration_to_file, get_existing_migrations};

    #[test]
    fn test_get_existing_migrations_non_initial() {
        let tmp = TempDir::new().expect("Could not create a temporary directory");
        let p = tmp.path().join("0001_not_initial.toml");

        let migration = Migration {
            hash: "".to_string(),
            initial: false,
            id: 0,
            name: "".to_string(),
            dependency: None,
            replaces: vec![],
            operations: vec![],
        };

        convert_migration_to_file(migration, Path::new(p.to_str().unwrap()))
            .expect("Could not write to file");

        assert!(get_existing_migrations(tmp.path().to_str().unwrap()).is_err());
    }

    #[test]
    fn test_get_existing_migrations_initial() {
        let tmp = TempDir::new().expect("Could not create a temporary directory");
        let p = tmp.path().join("0001_initial.toml");

        let migration = Migration {
            hash: "".to_string(),
            initial: true,
            id: 0,
            name: "".to_string(),
            dependency: None,
            replaces: vec![],
            operations: vec![],
        };

        convert_migration_to_file(migration, Path::new(p.to_str().unwrap()))
            .expect("Could not write to file");

        assert!(get_existing_migrations(tmp.path().to_str().unwrap()).is_ok());
    }

    #[test]
    fn test_get_existing_migrations_multiple_connected() {
        let tmp = TempDir::new().expect("Could not create a temporary directory");
        let p = tmp.path().join("0001_initial.toml");
        let p_2 = tmp.path().join("00002_foobar.toml");

        let migration = Migration {
            hash: "".to_string(),
            initial: true,
            id: 0,
            name: "".to_string(),
            dependency: None,
            replaces: vec![],
            operations: vec![],
        };

        convert_migration_to_file(migration, Path::new(p.to_str().unwrap()))
            .expect("Could not write to file");

        let migration = Migration {
            hash: "".to_string(),
            initial: false,
            id: 0,
            name: "".to_string(),
            dependency: Some(1),
            replaces: vec![],
            operations: vec![],
        };

        convert_migration_to_file(migration, Path::new(p_2.to_str().unwrap()))
            .expect("Could not write to file");

        assert!(get_existing_migrations(tmp.path().to_str().unwrap()).is_ok());
    }
}
