pub mod linter;
pub mod make_migrations;
pub mod merge_migrations;
pub mod migrate;
pub mod squash_migrations;
pub mod utils;

use clap::{ArgAction, Parser, Subcommand};

use crate::make_migrations::{run_make_migrations, MakeMigrationsOptions};
use crate::migrate::{run_migrate, MigrateOptions};

#[derive(Subcommand)]
enum Commands {
    #[clap(about = "Tool to create migrations")]
    MakeMigrations {
        #[clap(long = "models-file")]
        #[clap(default_value_t=String::from("./.models.json"))]
        #[clap(help = "Location of the intermediate representation of models.")]
        models_file: String,

        #[clap(short = 'm', long = "migration-dir")]
        #[clap(default_value_t=String::from("./migrations/"))]
        #[clap(help = "Destination to / from which migrations are written / read.")]
        migration_dir: String,

        #[clap(help = "Use this name as migration name instead of generating one.")]
        name: Option<String>,

        #[clap(long = "non-interactive")]
        #[clap(action = ArgAction::SetTrue)]
        #[clap(help = "If set, no questions will be asked.")]
        non_interactive: bool,

        #[clap(long = "disable-warnings")]
        #[clap(action = ArgAction::SetTrue)]
        #[clap(help = "If set, no warnings will be printed.")]
        warnings_disabled: bool,
    },

    #[clap(about = "Apply migrations")]
    Migrate {
        #[clap(short = 'm', long = "migration-dir")]
        #[clap(default_value_t=String::from("./migrations/"))]
        #[clap(help = "Destination to / from which migrations are written / read.")]
        migration_dir: String,

        #[clap(long = "database-config")]
        #[clap(default_value_t=String::from("./database.toml"))]
        #[clap(help = "Path to the database configuration file.")]
        database_config: String,

        #[clap(long = "log-sql")]
        #[clap(action = ArgAction::SetTrue)]
        #[clap(help = "If turned on, all queries to the database will be logged")]
        log_queries: bool,
    },

    #[clap(about = "Squash migrations")]
    SquashMigrations {
        #[clap(short = 'm', long = "migration-dir")]
        #[clap(default_value_t = String::from("./migrations/"))]
        #[clap(help = "Destination to / from which migrations are written / read.")]
        migration_dir: String,
    },

    #[clap(about = "Merge migrations")]
    MergeMigrations {},
}

#[derive(Parser)]
#[clap(version = "0.1.0", about = "CLI tool for rorm", long_about = None)]
#[clap(arg_required_else_help = true)]
#[clap(name = "rorm-cli")]
struct Cli {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli: Cli = Cli::parse();

    match cli.command {
        Some(Commands::MakeMigrations {
            models_file,
            migration_dir,
            name,
            non_interactive,
            warnings_disabled,
        }) => {
            run_make_migrations(MakeMigrationsOptions {
                models_file,
                migration_dir,
                name,
                non_interactive,
                warnings_disabled,
            })?;
        }
        Some(Commands::Migrate {
            migration_dir,
            database_config,
            log_queries,
        }) => {
            run_migrate(MigrateOptions {
                migration_dir,
                database_config,
                log_queries,
            })
            .await?;
        }
        Some(Commands::SquashMigrations { migration_dir }) => {}
        _ => {}
    }
    Ok(())
}
