use std::fs;
use std::path::Path;

use rorm_declaration::config::{DatabaseConfig, DatabaseDriver};
use tracing::{error, info};

use crate::cli::InitDriver;
use crate::migrate::config::DatabaseConfigFile;

/// Create the database configuration file
pub fn init(database_configuration: String, driver: InitDriver, force: bool) -> anyhow::Result<()> {
    let p = Path::new(&database_configuration);
    if p.exists() && !force {
        error!("Database configuration at {} does already exists. Use --force to overwrite the existing file.", &database_configuration);
        return Ok(());
    }

    match driver {
        #[cfg(feature = "sqlite")]
        InitDriver::Sqlite { filename } => {
            let config_file = DatabaseConfigFile {
                database: DatabaseConfig {
                    driver: DatabaseDriver::SQLite { filename },
                    last_migration_table_name: None,
                },
            };

            let serialized = toml::to_string_pretty(&config_file)?;

            fs::write(p, serialized)?;
        }
        #[cfg(feature = "mysql")]
        InitDriver::Mysql {
            host,
            port,
            user,
            password,
            ask_password,
            name,
        } => {
            let pw = if ask_password {
                rpassword::prompt_password("Enter the password for the database:")?
            } else {
                password.unwrap_or_default()
            };

            let config_file = DatabaseConfigFile {
                database: DatabaseConfig {
                    driver: DatabaseDriver::MySQL {
                        host,
                        port,
                        user,
                        password: pw,
                        name,
                    },
                    last_migration_table_name: None,
                },
            };

            let serialized = toml::to_string_pretty(&config_file)?;

            fs::write(p, serialized)?;
        }
        #[cfg(feature = "postgres")]
        InitDriver::Postgres {
            host,
            port,
            user,
            password,
            ask_password,
            name,
        } => {
            let pw = if ask_password {
                rpassword::prompt_password("Enter the password for the database:")?
            } else {
                password.unwrap_or_default()
            };

            let config_file = DatabaseConfigFile {
                database: DatabaseConfig {
                    driver: DatabaseDriver::Postgres {
                        host,
                        port,
                        user,
                        password: pw,
                        name,
                    },
                    last_migration_table_name: None,
                },
            };

            let serialized = toml::to_string_pretty(&config_file)?;

            fs::write(p, serialized)?;
        }
    }

    info!("Configuration was written to {}.", &database_configuration);

    Ok(())
}
