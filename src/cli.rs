use clap::{Parser, Subcommand};

use crate::init::init;
use crate::make_migrations::{run_make_migrations, MakeMigrationsOptions};
use crate::migrate::{run_migrate, MigrateOptions};
use crate::squash_migrations::squash_migrations;

#[derive(Subcommand)]
pub enum InitDriver {
    /// Initialize a sqlite configuration
    #[cfg(feature = "sqlite")]
    Sqlite {
        /// Name of the sqlite file.
        #[clap(long)]
        #[clap(default_value = "db.sqlite3")]
        filename: String,
    },

    /// Initialize a mysql configuration
    #[cfg(feature = "mysql")]
    Mysql {
        /// The address to use to connect to the database.
        #[clap(long)]
        #[clap(default_value = "127.0.0.1")]
        host: String,

        /// The port to use to connect to the database.
        #[clap(long)]
        #[clap(default_value = "306")]
        port: u16,

        /// The user to use to connect to the database.
        #[clap(long)]
        #[clap(default_value = "dbuser")]
        user: String,

        /// Set the password. To minimize the risk of exposing your password, use --ask-password instead.
        #[clap(long)]
        password: Option<String>,

        /// Ask for the password for the database. If specified with the --password option, this value will be prevalent.
        #[clap(long)]
        ask_password: bool,

        /// The name of the database to connect to.
        #[clap(long)]
        #[clap(default_value = "dbname")]
        name: String,
    },

    /// Initialize a postgres configuration
    #[cfg(feature = "postgres")]
    Postgres {
        /// The address to use to connect to the database.
        #[clap(long)]
        #[clap(default_value = "127.0.0.1")]
        host: String,

        /// The port to use to connect to the database.
        #[clap(long)]
        #[clap(default_value = "5432")]
        port: u16,

        /// The user to use to connect to the database.
        #[clap(long)]
        #[clap(default_value = "dbuser")]
        user: String,

        /// Set the password. To minimize the risk of exposing your password, use --ask-password instead.
        #[clap(long)]
        password: Option<String>,

        /// Ask for the password for the database. If specified with the --password option, this value will be prevalent.
        #[clap(long)]
        ask_password: bool,

        /// The name of the database to connect to.
        #[clap(long)]
        #[clap(default_value = "dbname")]
        name: String,
    },
}

#[derive(Subcommand)]
pub enum Command {
    /// Create the database configuration file
    Init {
        /// Path to the database configuration file that should be created.
        #[clap(long)]
        #[clap(default_value = "./database.toml")]
        database_config: String,

        /// Overwrite the database configuration if it is existent already
        #[clap(short, long)]
        force: bool,

        #[clap(subcommand)]
        driver: InitDriver,
    },

    /// Tool to create migrations
    MakeMigrations {
        /// Location of the intermediate representation of models.
        #[clap(long)]
        #[clap(default_value = "./.models.json")]
        models_file: String,

        /// Destination to / from which migrations are written / read.
        #[clap(short, long)]
        #[clap(default_value = "./migrations/")]
        migration_dir: String,

        /// Use this name as migration name instead of generating one.
        name: Option<String>,

        /// If set, no questions will be asked.
        #[clap(long)]
        non_interactive: bool,

        /// If set, no warnings will be printed.
        #[clap(long)]
        disable_warnings: bool,
    },

    /// Apply migrations
    Migrate {
        /// Destination from which migrations are read.
        #[clap(short, long)]
        #[clap(default_value = "./migrations/")]
        migration_dir: String,

        /// Path to the database configuration file.
        #[clap(long)]
        #[clap(default_value = "./database.toml")]
        database_config: String,

        /// If turned on, all queries to the database will be logged.
        #[clap(long)]
        log_sql: bool,

        /// Only apply the migrations to (inclusive) the given migration.
        #[clap(long)]
        #[clap(id = "MIGRATION_ID")]
        apply_until: Option<u16>,
    },

    /// Squash migrations
    SquashMigrations {
        /// Destination to / from which migrations are written / read.
        #[clap(short, long)]
        #[clap(default_value = "./migrations/")]
        migration_dir: String,

        /// First migration to start squashing from.
        first_migration: u16,

        /// Last migration to squash.
        last_migration: u16,
    },
}

/// CLI tool for rorm
#[derive(Parser)]
#[clap(version)]
#[clap(arg_required_else_help = true)]
#[clap(name = "rorm-cli")]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

impl Cli {
    /// Executes the parsed cli arguments
    pub async fn run(self) -> anyhow::Result<()> {
        match self.command {
            Command::Init {
                force,
                driver,
                database_config,
            } => init(database_config, driver, force)?,
            Command::MakeMigrations {
                models_file,
                migration_dir,
                name,
                non_interactive,
                disable_warnings,
            } => {
                run_make_migrations(MakeMigrationsOptions {
                    models_file,
                    migration_dir,
                    name,
                    non_interactive,
                    warnings_disabled: disable_warnings,
                })?;
            }
            Command::Migrate {
                migration_dir,
                database_config,
                log_sql,
                apply_until,
            } => {
                run_migrate(MigrateOptions {
                    migration_dir,
                    database_config,
                    log_queries: log_sql,
                    apply_until,
                })
                .await?;
            }
            Command::SquashMigrations {
                migration_dir,
                first_migration,
                last_migration,
            } => {
                squash_migrations(migration_dir, first_migration, last_migration).await?;
            }
        }
        Ok(())
    }
}
