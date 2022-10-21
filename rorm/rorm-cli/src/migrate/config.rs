use std::fmt::Debug;
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
The representation of all supported DB drivers
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "Driver")]
pub enum DatabaseDriver {
    #[serde(rename_all = "PascalCase")]
    SQLite { filename: String },
    #[serde(rename_all = "PascalCase")]
    Postgres {
        name: String,
        host: String,
        port: u16,
        user: String,
        password: String,
    },
    #[serde(rename_all = "PascalCase")]
    MySQL {
        name: String,
        host: String,
        port: u16,
        user: String,
        password: String,
    },
}

impl From<DatabaseDriver> for DBImpl {
    fn from(v: DatabaseDriver) -> Self {
        match v {
            DatabaseDriver::SQLite { .. } => DBImpl::SQLite,
            DatabaseDriver::Postgres { .. } => DBImpl::Postgres,
            DatabaseDriver::MySQL { .. } => DBImpl::MySQL,
        }
    }
}

/**
The configuration struct for database related settings
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct DatabaseConfig {
    #[serde(flatten)]
    pub driver: DatabaseDriver,
    pub last_migration_table_name: Option<String>,
}

const EXAMPLE_DATABASE_CONFIG: &str = r#"[Database]
# Valid driver types are: "MySQL", "Postgres" and "SQLite"
Driver = "MySQL"

# Name of the database. 
Name = "dbname"

Host = "127.0.0.1"
Port = 3306
User = "dbuser"
Password = "super-secure-password"
"#;

#[cfg(test)]
mod test {
    use crate::migrate::config::{DatabaseConfigFile, EXAMPLE_DATABASE_CONFIG};

    #[test]
    fn test_example_database_config() {
        let db_conf = toml::from_str::<DatabaseConfigFile>(EXAMPLE_DATABASE_CONFIG);
        assert!(db_conf.is_ok());
    }
}

/**
Helper method to create a dummy database configuration file
 */
pub fn create_db_config(path: &Path) -> anyhow::Result<()> {
    let fh = File::create(path).with_context(|| format!("Couldn't open {:?} for writing", path))?;

    writeln!(&fh, "{}", EXAMPLE_DATABASE_CONFIG).with_context(|| {
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
