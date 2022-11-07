use std::fmt::Debug;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::Path;

use anyhow::Context;
use rorm_declaration::config::{DatabaseConfig, DatabaseDriver};
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
Converts the [DatabaseDriver] to [DBImpl]
*/
pub fn convert_db_driver_to_db_impl(v: DatabaseDriver) -> DBImpl {
    match v {
        DatabaseDriver::SQLite { .. } => DBImpl::SQLite,
        DatabaseDriver::Postgres { .. } => DBImpl::Postgres,
        DatabaseDriver::MySQL { .. } => DBImpl::MySQL,
    }
}

const EXAMPLE_DATABASE_CONFIG: &str = r#"
# Example database configuration for each supported database.
# Uncomment the database you'd like to use.

[Database]
# -------------------------------
# Example SQLite configuration
# -------------------------------
Driver = "SQLite"
 
# Filename / path of the sqlite database 
Filename = ""

# -------------------------------
# Example MySQL configuration
# -------------------------------
# Driver = "MySQL"
# Name = "dbname"
# Host = "127.0.0.1"
# Port = 3306
# User = "dbuser"
# Password = "super-secure-password"

# -------------------------------
# Example Postgres configuration
# -------------------------------
# Driver = "Postgres"
# Name = "dbname"
# Host = "127.0.0.1"
# Port = 5432
# User = "dbuser"
# Password = "super-secure-password"
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
