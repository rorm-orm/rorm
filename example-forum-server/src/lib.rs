mod handler;
mod models;
mod test;

use std::borrow::Cow;
use std::fs;

use anyhow::Context;
use clap::{Parser, Subcommand};
use rorm::conditions::{Binary, BinaryOperator, Column, Value};
use rorm::config::DatabaseConfig;
use rorm::{Database, DatabaseConfiguration, DatabaseDriver};
use tokio::net::TcpListener;
use tower_sessions::{MemoryStore, SessionManagerLayer};
use tracing::info;

use crate::models::user::User;

/// The cli
#[derive(Parser)]
pub struct Cli {
    /// The path to the database config file
    #[clap(long)]
    pub db_config: Option<String>,

    /// The available subcommands
    #[clap(subcommand)]
    pub command: Command,
}

/// All available commands
#[derive(Subcommand)]
pub enum Command {
    /// Run the migrations on the database
    Migrate {
        /// The directory where the migration files are located in
        migrations_dir: String,
    },
    /// Create new migrations
    #[cfg(debug_assertions)]
    MakeMigrations {
        /// The directory where the migration files are located in
        migrations_dir: String,
    },
    /// Start the server
    Start,
    /// Tests the server by sending it requests
    Test,
}

pub async fn run_main(cli: Cli) -> anyhow::Result<()> {
    let db_driver = match cli.db_config {
        Some(path) => {
            serde_json::from_reader(fs::File::open(path).context("Failed to open db config")?)
                .context("Failed to read db config")?
        }
        None => DatabaseDriver::Postgres {
            name: "".to_string(),
            host: "".to_string(),
            port: 0,
            user: "".to_string(),
            password: "".to_string(),
        },
    };
    let db = Database::connect(DatabaseConfiguration::new(db_driver.clone()))
        .await
        .context("Failed to connect to db")?;

    rorm::query(&db, User)
        .condition(Binary {
            operator: BinaryOperator::EqualsAny,
            fst_arg: Column(User.id),
            snd_arg: Value::ArrayI64(Cow::Borrowed(&[1, 2, 3])),
        })
        .all()
        .await?;

    match cli.command {
        #[cfg(debug_assertions)]
        Command::MakeMigrations { migrations_dir } => {
            use std::io::Write;

            const MODELS: &str = ".models.json";

            let mut file = fs::File::create(MODELS)?;
            rorm::write_models(&mut file)?;
            file.flush()?;

            rorm::cli::make_migrations::run_make_migrations(
                rorm::cli::make_migrations::MakeMigrationsOptions {
                    models_file: MODELS.to_string(),
                    migration_dir: migrations_dir,
                    name: None,
                    non_interactive: false,
                    warnings_disabled: false,
                },
            )?;

            fs::remove_file(MODELS)?;
        }
        Command::Migrate { migrations_dir } => {
            rorm::cli::migrate::run_migrate_custom(
                DatabaseConfig {
                    driver: db_driver,
                    last_migration_table_name: None,
                },
                migrations_dir,
                None,
            )
            .await?
        }
        Command::Start => {
            info!("Starting server on http://localhost:8000");

            axum::serve(
                TcpListener::bind(("localhost", 8000))
                    .await
                    .context("Failed to bind to localhost:8000")?,
                handler::get_router()
                    .with_state(db)
                    .layer(SessionManagerLayer::new(MemoryStore::default()).with_secure(false)),
            )
            .await?
        }
        Command::Test => test::run_test_client().await?,
    }

    Ok(())
}
