use std::fs::read_to_string;

use clap::Parser;
use rorm::{config::DatabaseConfig, Database, DatabaseConfiguration};
use serde::{Deserialize, Serialize};

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
    let db = Database::connect(DatabaseConfiguration {
        driver: db_conf_file.database.driver.clone(),
        min_connections: 1,
        max_connections: 1,
    })
    .await?;

    // Perform project-specific operations on the database
    operations::operate(db, &db_conf_file.database.driver.clone()).await
}
