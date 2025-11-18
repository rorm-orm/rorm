use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::exit;

use rorm_declaration::config::{DatabaseConfig, DatabaseDriver};

use crate::cli::InitDriver;
use crate::migrate::config::DatabaseConfigFile;

/// Create the database configuration file
pub fn init(database_configuration: String, driver: InitDriver, force: bool) -> anyhow::Result<()> {
    let p = Path::new(&database_configuration);
    if p.exists() && !force {
        println!("Database configuration at {} does already exists. Use --force to overwrite the existing file.", &database_configuration);
        exit(1);
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

            let mut f = File::create(p)?;
            write!(f, "{}", &serialized)?;

            println!("Configuration was written to {}.", &database_configuration);
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

            let mut f = File::create(p)?;
            write!(f, "{}", &serialized)?;

            println!("Configuration was written to {}.", &database_configuration);
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

            let mut f = File::create(p)?;
            write!(f, "{}", &serialized)?;

            println!("Configuration was written to {}.", &database_configuration);
        }
    };

    Ok(())
}
