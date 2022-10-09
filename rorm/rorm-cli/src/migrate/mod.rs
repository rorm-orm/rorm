pub mod config;
pub mod sql_builder;

use std::path::Path;

use anyhow::{anyhow, Context};
use rorm_declaration::imr::{Annotation, DbType};
use rorm_declaration::migration::Migration;
use rorm_sql::DBImpl;
use sqlx::any::{AnyPoolOptions, AnyRow};
use sqlx::mysql::MySqlConnectOptions;
use sqlx::postgres::PgConnectOptions;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{query, AnyPool, Row};

use crate::log_sql;
use crate::migrate::config::{create_db_config, deserialize_db_conf, DatabaseDriver};
use crate::migrate::sql_builder::migration_to_sql;
use crate::utils::bind;
use crate::utils::migrations::get_existing_migrations;

/**
Options for running migrations
*/
pub struct MigrateOptions {
    /// Directory, migrations exist in
    pub migration_dir: String,

    /// Path to the database configuration file
    pub database_config: String,

    /// Log all SQL statements
    pub log_queries: bool,
}

/**
Helper method to apply one migration. Writes also to last migration table.

`migration`: [&Migration]: Reference to the migration to apply.
`pool`: [&SqlitePool]: Pool to apply the migration onto.
`last_migration_table_name`: [&str]: Name of the table to insert successful applied migrations into.
*/
pub async fn apply_migration(
    dialect: DBImpl,
    migration: &Migration,
    pool: &AnyPool,
    last_migration_table_name: &str,
    do_log: bool,
) -> anyhow::Result<()> {
    let mut tx = pool
        .begin()
        .await
        .with_context(|| format!("Error while starting transaction {}", migration.id))?;

    migration_to_sql(&mut tx, dialect, migration, do_log).await?;

    tx.commit().await.with_context(|| {
        format!(
            "Error while committing transaction {}",
            last_migration_table_name
        )
    })?;

    query(
        log_sql!(
            format!(
                "INSERT INTO {} (migration_name) VALUES (?);",
                last_migration_table_name
            ),
            do_log
        )
        .as_str(),
    )
    .bind(migration.id.as_str())
    .execute(pool)
    .await
    .with_context(|| {
        format!(
            "Error while inserting applied migration {} into last migration table",
            last_migration_table_name
        )
    })?;

    println!("Applied migration {}", migration.id.as_str());
    Ok(())
}

/**
Applies migrations on the given database
*/
pub async fn run_migrate(options: MigrateOptions) -> anyhow::Result<()> {
    let db_conf_path = Path::new(options.database_config.as_str());

    if !&db_conf_path.exists() {
        println!(
            "Couldn't find the database configuration file, created {} and exiting",
            options.database_config.as_str()
        );
        create_db_config(db_conf_path)?;
        return Ok(());
    }

    let db_conf = deserialize_db_conf(db_conf_path)?;

    let existing_migrations = get_existing_migrations(options.migration_dir.as_str())
        .with_context(|| "Couldn't retrieve existing migrations")?;

    let pool_options = AnyPoolOptions::new().min_connections(1).max_connections(10);
    let pool;

    match db_conf.driver {
        DatabaseDriver::SQLite => {
            let connect_options = SqliteConnectOptions::new()
                .create_if_missing(true)
                .filename(db_conf.name.as_str());
            pool = pool_options.connect_with(connect_options.into()).await?;
        }
        DatabaseDriver::Postgres => {
            let connect_options = PgConnectOptions::new()
                .host(db_conf.host.as_str())
                .port(db_conf.port)
                .username(db_conf.user.as_str())
                .password(db_conf.password.as_str())
                .database(db_conf.name.as_str());
            pool = pool_options.connect_with(connect_options.into()).await?;
        }
        DatabaseDriver::MySQL => {
            let connect_options = MySqlConnectOptions::new()
                .host(db_conf.host.as_str())
                .port(db_conf.port)
                .username(db_conf.user.as_str())
                .password(db_conf.password.as_str())
                .database(db_conf.name.as_str());
            pool = pool_options.connect_with(connect_options.into()).await?;
        }
    }

    let db_impl = DBImpl::from(db_conf.driver);
    let (query_str, bind_params) = db_impl
        .create_table(db_conf.last_migration_table_name.as_str())
        .add_column(db_impl.create_column(
            db_conf.last_migration_table_name.as_str(),
            "id",
            DbType::Int64,
            &[Annotation::PrimaryKey, Annotation::AutoIncrement],
        ))
        .add_column(db_impl.create_column(
            db_conf.last_migration_table_name.as_str(),
            "updated_at",
            DbType::DateTime,
            &[Annotation::AutoUpdateTime],
        ))
        .add_column(db_impl.create_column(
            db_conf.last_migration_table_name.as_str(),
            "migration_name",
            DbType::VarChar,
            &[Annotation::NotNull, Annotation::MaxLength(255)],
        ))
        .if_not_exists()
        .build()?;

    if options.log_queries {
        println!("{}", query_str.as_str());
    }

    let mut q = query(query_str.as_str());

    for x in bind_params {
        q = bind::bind_param(q, x);
    }

    q.execute(&pool)
        .await
        .with_context(|| "Couldn't create internal last migration table")?;

    let last_migration: Option<String> = query(
        log_sql!(
            format!(
                "SELECT migration_name FROM {} ORDER BY id DESC LIMIT 1;",
                &db_conf.last_migration_table_name
            ),
            options.log_queries
        )
        .as_str(),
    )
    .map(|x: AnyRow| x.get(0))
    .fetch_optional(&pool)
    .await
    .with_context(|| {
        "Couldn't fetch information about successful migrations from migration table"
    })?;

    match last_migration {
        None => {
            // Apply all migrations
            for migration in &existing_migrations {
                apply_migration(
                    db_conf.driver.into(),
                    migration,
                    &pool,
                    db_conf.last_migration_table_name.as_str(),
                    options.log_queries,
                )
                .await?;
            }
        }
        Some(id) => {
            // Search for last applied migration
            if existing_migrations.iter().any(|x| x.id == id) {
                let mut apply = false;
                for (idx, migration) in existing_migrations.iter().enumerate() {
                    if apply {
                        apply_migration(
                            db_conf.driver.into(),
                            migration,
                            &pool,
                            db_conf.last_migration_table_name.as_str(),
                            options.log_queries,
                        )
                        .await?;
                        continue;
                    }

                    if migration.id == id {
                        apply = true;

                        if idx == existing_migrations.len() - 1 {
                            println!("All migration have already been applied.");
                        }
                    }
                }
            } else {
                // If last applied migration could not be found in existing migrations,
                // panic as there's no way to determine what to do next
                return Err(anyhow!(
                    r#"Last applied migration {} was not found in current migrations.
 
Can not proceed any further without damaging data.
To correct, empty the {} table or reset the whole database."#,
                    id.as_str(),
                    db_conf.last_migration_table_name.as_str()
                ));
            }
        }
    }

    Ok(())
}
