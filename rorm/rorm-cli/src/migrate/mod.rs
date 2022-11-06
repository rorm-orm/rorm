pub mod config;
pub mod sql_builder;

use std::path::Path;

use anyhow::{anyhow, Context};
use rorm_declaration::config::DatabaseDriver;
use rorm_declaration::imr::{Annotation, DbType};
use rorm_declaration::migration::Migration;
use rorm_sql::create_table::CreateTable;
use rorm_sql::insert::Insert;
use rorm_sql::DBImpl;
use sqlx::any::{AnyPoolOptions, AnyRow};
use sqlx::mysql::MySqlConnectOptions;
use sqlx::postgres::PgConnectOptions;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{query, Any, AnyPool, Pool, Row};

use crate::log_sql;
use crate::migrate::config::{convert_db_driver_to_db_impl, create_db_config, deserialize_db_conf};
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

    /// Apply only to (inclusive) the given id, if set
    pub apply_until: Option<u16>,
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

    if let Err(e) = migration_to_sql(&mut tx, dialect, migration, do_log).await {
        tx.rollback()
            .await
            .with_context(|| "Error while rollback in transaction")?;
        return Err(e);
    }

    let v: &[&[rorm_sql::value::Value]] = &[&[rorm_sql::value::Value::I32(migration.id as i32)]];
    let (query_string, bind_params) = dialect
        .insert(last_migration_table_name, &["migration_id"], v)
        .rollback_transaction()
        .build();

    if do_log {
        println!("{}", query_string);
    }

    let mut q = query(query_string.as_str());

    for x in bind_params {
        q = bind::bind_param(q, x);
    }
    q.execute(&mut tx).await.with_context(|| {
        format!(
            "Error while inserting applied migration {} into last migration table",
            last_migration_table_name
        )
    })?;

    println!("Applied migration {:04}_{}", migration.id, migration.name);

    tx.commit().await.with_context(|| {
        format!(
            "Error while committing transaction {}",
            last_migration_table_name
        )
    })?;

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

    let p = Path::new(options.migration_dir.as_str());
    if !p.exists() || p.is_file() {
        println!(
            "Couldn't find the migration directory in {} \n\n\
            You can specify an alternative path with --migration-dir <PATH>",
            options.migration_dir.as_str()
        );
        return Ok(());
    }

    let existing_migrations = get_existing_migrations(options.migration_dir.as_str())
        .with_context(|| "Couldn't retrieve existing migrations")?;

    if existing_migrations.is_empty() {
        println!("No migrations found.\nExiting.");
        return Ok(());
    }

    let pool_options = AnyPoolOptions::new().min_connections(1).max_connections(1);

    let pool: Pool<Any> = match &db_conf.driver {
        DatabaseDriver::SQLite { filename } => {
            let connect_options = SqliteConnectOptions::new()
                .create_if_missing(true)
                .filename(filename.as_str());
            pool_options.connect_with(connect_options.into()).await?
        }
        DatabaseDriver::Postgres {
            name,
            host,
            port,
            user,
            password,
        } => {
            let connect_options = PgConnectOptions::new()
                .host(host.as_str())
                .port(*port)
                .username(user.as_str())
                .password(password.as_str())
                .database(name.as_str());
            pool_options.connect_with(connect_options.into()).await?
        }
        DatabaseDriver::MySQL {
            name,
            host,
            port,
            user,
            password,
        } => {
            let connect_options = MySqlConnectOptions::new()
                .host(host.as_str())
                .port(*port)
                .username(user.as_str())
                .password(password.as_str())
                .database(name.as_str());
            pool_options.connect_with(connect_options.into()).await?
        }
    };

    let last_migration_table_name = db_conf
        .last_migration_table_name
        .as_ref()
        .map_or("_rorm__last_migration", |x| x.as_str());

    let db_impl = convert_db_driver_to_db_impl(db_conf.driver);
    let statements = db_impl
        .create_table(last_migration_table_name)
        .add_column(db_impl.create_column(
            last_migration_table_name,
            "id",
            DbType::Int64,
            &[Annotation::PrimaryKey, Annotation::AutoIncrement],
        ))
        .add_column(db_impl.create_column(
            last_migration_table_name,
            "updated_at",
            DbType::DateTime,
            &[Annotation::AutoUpdateTime],
        ))
        .add_column(db_impl.create_column(
            last_migration_table_name,
            "migration_id",
            DbType::Int32,
            &[Annotation::NotNull],
        ))
        .if_not_exists()
        .build()?;

    let mut tx = pool
        .begin()
        .await
        .with_context(|| "Could not create transaction")?;

    for (query_string, bind_params) in statements {
        if options.log_queries {
            println!("{}", query_string.as_str());
        }

        let mut q = sqlx::query(query_string.as_str());
        for x in bind_params {
            q = bind::bind_param(q, x);
        }
        q.execute(&mut tx)
            .await
            .with_context(|| "Couldn't create internal last migration table")?;
    }

    tx.commit()
        .await
        .with_context(|| "Couldn't create internal last migration table")?;

    let last_migration: Option<i32> = query(
        log_sql!(
            format!(
                "SELECT migration_id FROM {} ORDER BY id DESC LIMIT 1;",
                &last_migration_table_name
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
                    db_impl,
                    migration,
                    &pool,
                    last_migration_table_name,
                    options.log_queries,
                )
                .await?;

                if let Some(apply_until) = options.apply_until {
                    if migration.id == apply_until {
                        println!(
                            "Applied all migrations until (inclusive) migration {:04}",
                            apply_until
                        );
                        break;
                    }
                }
            }
        }
        Some(id) => {
            let id = id as u16;
            // Search for last applied migration
            if existing_migrations.iter().any(|x| x.id == id) {
                let mut apply = false;
                for (idx, migration) in existing_migrations.iter().enumerate() {
                    if apply {
                        apply_migration(
                            db_impl,
                            migration,
                            &pool,
                            last_migration_table_name,
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

                    if let Some(apply_until) = options.apply_until {
                        if migration.id == apply_until {
                            if apply {
                                println!(
                                    "Applied all migrations until (inclusive) migration {:04}",
                                    apply_until
                                );
                            } else {
                                println!(
                                    "All migrations until (inclusive) migration {:04} have already been applied",
                                    apply_until
                                );
                            }
                            break;
                        } else if migration.id > apply_until {
                            break;
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
                    id,
                    last_migration_table_name
                ));
            }
        }
    }

    Ok(())
}
