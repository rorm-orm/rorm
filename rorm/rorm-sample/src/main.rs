use std::fs::read_to_string;

use clap::Parser;
use log::LevelFilter;
use rorm::{config::DatabaseConfig, Database, DatabaseConfiguration, DatabaseDriver};
use serde::{Deserialize, Serialize};

pub mod forum;
mod operations;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ConfigFile {
    pub database: DatabaseConfig,
}

#[derive(Parser)]
struct Cli {
    /// Specify the database configuration file
    config_file: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum DatabaseVariant {
    MySQL,
    Postgres,
    SQLite,
}

impl From<&DatabaseDriver> for DatabaseVariant {
    fn from(db: &DatabaseDriver) -> Self {
        match db {
            DatabaseDriver::Postgres { .. } => DatabaseVariant::Postgres,
            DatabaseDriver::MySQL { .. } => DatabaseVariant::MySQL,
            DatabaseDriver::SQLite { .. } => DatabaseVariant::SQLite,
        }
    }
}

#[rorm::rorm_main]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Enable the environment logger
    env_logger::init();

    // Get the config file from the CLI arguments
    let path = Cli::parse().config_file;

    // Read the config from a TOML file
    let db_conf_file: ConfigFile = toml::from_str(&read_to_string(&path)?)?;

    // Connect to the database to get the database handle using the TOML configuration
    let db_variant = (&db_conf_file.database.driver).into();
    let db = Database::connect(DatabaseConfiguration {
        driver: db_conf_file.database.driver,
        min_connections: 1,
        max_connections: 1,
        disable_logging: Some(false),
        statement_log_level: Some(LevelFilter::Debug),
        slow_statement_log_level: Some(LevelFilter::Error),
    })
    .await?;

    // Perform project-specific operations on the database
    operations::operate(db, db_variant).await
}
