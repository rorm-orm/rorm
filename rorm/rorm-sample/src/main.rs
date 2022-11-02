use clap::Parser;
use rorm::DatabaseConfiguration;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::exit;
use toml;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ConfigFile {
    pub database: rorm::config::DatabaseConfig,
}

#[derive(Parser)]
struct Cli {
    /// Specify the database configuration file
    config_file: Option<PathBuf>,
}

#[rorm::rorm_main]
#[tokio::main]
async fn main() {
    // Enable the environment logger
    env_logger::init();

    // Get the config file from the CLI arguments
    let cli: Cli = Cli::parse();
    let path;
    match cli.config_file {
        None => {
            eprintln!("The mandatory argument 'config_file' is missing");
            exit(1);
        }
        Some(p) => path = p.to_str().unwrap().to_string(),
    }

    // Read the config from a TOML file
    let db_conf_file = toml::from_str::<ConfigFile>(
        std::fs::read_to_string(&path)
            .expect("File read error")
            .as_str(),
    )
    .expect("Couldn't deserialize configuration file");

    // Connect to the database to get the database handle using the TOML configuration
    let db = rorm::Database::connect(DatabaseConfiguration {
        driver: db_conf_file.database.driver,
        min_connections: 1,
        max_connections: 1,
    })
    .await
    .expect("error connecting to the database");

    // TODO
}
