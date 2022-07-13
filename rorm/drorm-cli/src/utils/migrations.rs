use std::fs::{read_dir, read_to_string, DirEntry, File};
use std::io::Write;
use std::path::Path;

use anyhow::Context;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::declaration::{Migration, MigrationFile};

pub static RE_ALLOWED_NAME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^[0-9]{4}_\w+\.toml$"#).unwrap());

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

    migration.migration.id = path
        .path()
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    Ok(migration)
}

/**
Helper function to retrieve a sorted list of migrations in a given directory.

`migration_dir`: [&str] The directory to search for files.
*/
pub fn get_existing_migrations(migration_dir: &str) -> anyhow::Result<Vec<Migration>> {
    let dir_entries =
        read_dir(migration_dir).with_context(|| "Error while searching the migration directory")?;

    let mut file_list: Vec<DirEntry> = dir_entries
        .filter(|x| {
            x.as_ref().unwrap().file_type().unwrap().is_file()
                && RE_ALLOWED_NAME.is_match(
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

    file_list.sort_by(|x, y| {
        x.file_name().into_string().unwrap()[0..4]
            .parse::<u16>()
            .unwrap()
            .cmp(
                &y.file_name().into_string().unwrap()[0..4]
                    .parse::<u16>()
                    .unwrap(),
            )
    });

    let mut migration: Vec<Migration> = vec![];
    for file in &file_list {
        migration.push(convert_file_to_migration(file)?.migration);
    }

    Ok(migration)
}
