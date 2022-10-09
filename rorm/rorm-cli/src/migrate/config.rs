use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::Path;

use anyhow::Context;
use rorm_sql::DBImpl;
use serde::{Deserialize, Serialize};

/**
Outer wrapper for the database configuration file.
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct DatabaseConfigFile {
    pub database: DatabaseConfig,
}

/**
The configuration struct for database related settings
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct DatabaseConfig {
    pub driver: DatabaseDriver,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub last_migration_table_name: String,
}

/**

*/
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "PascalCase")]
pub enum DatabaseDriver {
    SQLite,
    Postgres,
    MySQL,
}

impl From<DatabaseDriver> for DBImpl {
    fn from(v: DatabaseDriver) -> Self {
        match v {
            DatabaseDriver::SQLite => DBImpl::SQLite,
            DatabaseDriver::Postgres => DBImpl::Postgres,
            DatabaseDriver::MySQL => DBImpl::MySQL,
        }
    }
}

/**
Helper method to create a dummy database configuration file
 */
pub fn create_db_config(path: &Path) -> anyhow::Result<()> {
    let db_file = DatabaseConfigFile {
        database: DatabaseConfig {
            driver: DatabaseDriver::SQLite,
            name: "database.sqlite3".to_string(),
            host: "127.0.0.1".to_string(),
            port: 3306,
            user: "user".to_string(),
            password: "change_me".to_string(),
            last_migration_table_name: "_drorm__last_migration".to_string(),
        },
    };

    let toml_str = toml::to_string_pretty(&db_file)
        .with_context(|| "Error while serializing database configuration")?;

    let fh = File::create(path).with_context(|| format!("Couldn't open {:?} for writing", path))?;

    writeln!(&fh, "{}", toml_str).with_context(|| {
        format!(
            "Couldn't write serialized database configuration to {:?}",
            path
        )
    })?;

    Ok(())
}

/**
Helper method to deserialize an existing database configuration file

`path`: [&Path]: Path to the configuration file
 */
pub fn deserialize_db_conf(path: &Path) -> anyhow::Result<DatabaseConfig> {
    let db_conf_toml =
        read_to_string(&path).with_context(|| "Couldn't read database configuration file")?;

    let db_conf = toml::from_str::<DatabaseConfigFile>(db_conf_toml.as_str())
        .with_context(|| "Couldn't deserialize database configuration file")?
        .database;

    Ok(db_conf)
}
