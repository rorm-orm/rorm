//! This modules holds the definition of the configuration used by [rorm-cli]

use serde::{Deserialize, Serialize};

/**
The representation of all supported DB drivers
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "Driver")]
pub enum DatabaseDriver {
    /// Representation of the SQLite driver
    #[serde(rename_all = "PascalCase")]
    SQLite {
        /// The filename of the sqlite database
        filename: String,
    },
    /// Representation of the Postgres driver
    #[serde(rename_all = "PascalCase")]
    Postgres {
        /// Name of the database
        name: String,
        /// Host of the database
        host: String,
        /// Port of the database
        port: u16,
        /// User to connect to the database
        user: String,
        /// Password to connect to the database
        password: String,
    },
    /// Representation of the MySQL / MariaDB driver
    #[serde(rename_all = "PascalCase")]
    MySQL {
        /// Name of the database
        name: String,
        /// Host of the database
        host: String,
        /// Port of the database
        port: u16,
        /// User to connect to the database
        user: String,
        /// Password to connect to the database
        password: String,
    },
}

/**
The configuration struct for database related settings
 */
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct DatabaseConfig {
    /// The representation of the SQL driver. Will be flattened
    #[serde(flatten)]
    pub driver: DatabaseDriver,
    /// The name of the migration table. Only used be [rorm-cli].
    pub last_migration_table_name: Option<String>,
}
