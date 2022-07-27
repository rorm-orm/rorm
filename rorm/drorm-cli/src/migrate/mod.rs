pub mod config;
pub mod sql_builder;

use std::path::Path;

use anyhow::{anyhow, Context};
use rorm_common::imr::{Annotation, DbType};
use rorm_sql::DBImpl;
use sqlx::sqlite::{SqliteConnectOptions, SqliteRow};
use sqlx::{query, Row, SqlitePool};

use crate::declaration::Migration;
use crate::migrate::config::DatabaseDriver::SQLite;
use crate::migrate::config::{create_db_config, deserialize_db_conf, DatabaseDriver};
use crate::migrate::sql_builder::migration_to_sql;
use crate::utils::migrations::get_existing_migrations;

/**
Options for running migrations
*/
pub struct MigrateOptions {
    /// Directory, migrations exist in
    pub migration_dir: String,

    /// Path to the database configuration file
    pub database_config: String,
}

/**
Helper method to apply one migration. Writes also to last migration table.

`migration`: [&Migration]: Reference to the migration to apply.
`pool`: [&SqlitePool]: Pool to apply the migration onto.
`last_migration_table_name`: [&str]: Name of the table to insert successful applied migrations into.
*/
pub async fn apply_migration_sqlite(
    migration: &Migration,
    pool: &SqlitePool,
    last_migration_table_name: &str,
) -> anyhow::Result<()> {
    let q = migration_to_sql(DBImpl::SQLite, migration)?;
    query(q.as_str())
        .execute(pool)
        .await
        .with_context(|| format!("Error while applying migration {}", migration.id))?;
    query(
        format!(
            "INSERT INTO {} (migration_name) VALUES (?);",
            last_migration_table_name
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
        println!("Couldn't find the database configuration file, creating one and exiting");
        create_db_config(&db_conf_path)?;
        return Ok(());
    }

    let db_conf = deserialize_db_conf(&db_conf_path)?;

    let existing_migrations = get_existing_migrations(options.migration_dir.as_str())
        .with_context(|| "Couldn't retrieve existing migrations")?;

    match db_conf.driver {
        SQLite => {
            let pool = SqlitePool::connect_with(
                SqliteConnectOptions::default()
                    .create_if_missing(true)
                    .filename(db_conf.name),
            )
            .await
            .with_context(|| "Couldn't initialize pool connection")?;

            query(
                DBImpl::SQLite
                    .create_table(db_conf.last_migration_table_name.as_str())
                    .add_column(DBImpl::SQLite.create_column(
                        db_conf.last_migration_table_name.as_str(),
                        "id",
                        DbType::UInt64,
                        vec![
                            Annotation::NotNull,
                            Annotation::PrimaryKey,
                            Annotation::AutoIncrement,
                        ],
                    ))
                    .add_column(DBImpl::SQLite.create_column(
                        db_conf.last_migration_table_name.as_str(),
                        "updated_at",
                        DbType::VarChar,
                        vec![Annotation::AutoUpdateTime],
                    ))
                    .add_column(DBImpl::SQLite.create_column(
                        db_conf.last_migration_table_name.as_str(),
                        "migration_name",
                        DbType::VarChar,
                        vec![Annotation::NotNull],
                    ))
                    .if_not_exists()
                    .build()
                    .with_context(|| "Error while creating last migration table")?
                    .as_str(),
            )
            .execute(&pool)
            .await
            .with_context(|| "Couldn't create internal last migration table")?;

            let last_migration: Option<String> = query(
                format!(
                    "SELECT migration_name FROM {} ORDER BY id DESC LIMIT 1;",
                    &db_conf.last_migration_table_name
                )
                .as_str(),
            )
            .map(|x: SqliteRow| x.get(0))
            .fetch_optional(&pool)
            .await
            .with_context(|| {
                "Couldn't fetch information about successful migrations from migration table"
            })?;

            match last_migration {
                None => {
                    // Apply all migrations
                    for migration in &existing_migrations {
                        apply_migration_sqlite(
                            migration,
                            &pool,
                            db_conf.last_migration_table_name.as_str(),
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
                                apply_migration_sqlite(
                                    migration,
                                    &pool,
                                    db_conf.last_migration_table_name.as_str(),
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
        }
        DatabaseDriver::Postgres => {}
        DatabaseDriver::MySQL => {}
    }

    Ok(())
}
