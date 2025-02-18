use std::future::poll_fn;
use std::time::Duration;

use rorm_declaration::config::DatabaseDriver;
use rorm_sql::value::Value;
use sqlx::ConnectOptions;
use tracing::log::LevelFilter;

use crate::database::{Database, DatabaseConfiguration};
use crate::error::Error;
use crate::internal::any::{AnyExecutor, AnyPool};
use crate::internal::utils;
use crate::row::Row;
use crate::transaction::Transaction;

pub(crate) type Impl = AnyPool;

/// All statements that take longer to execute than this value are considered
/// as slow statements.
const SLOW_STATEMENTS: Duration = Duration::from_millis(300);

/// Implementation of [Database::connect]
pub(crate) async fn connect(configuration: DatabaseConfiguration) -> Result<Impl, Error> {
    if configuration.max_connections < configuration.min_connections {
        return Err(Error::ConfigurationError(String::from(
            "max_connections must not be less than min_connections",
        )));
    }

    if configuration.min_connections == 0 {
        return Err(Error::ConfigurationError(String::from(
            "min_connections must not be 0",
        )));
    }

    macro_rules! pool_options {
        ($Pool:ty) => {
            <$Pool>::new()
                .min_connections(configuration.min_connections)
                .max_connections(configuration.max_connections)
        };
    }

    let pool: Impl = match &configuration.driver {
        #[cfg(feature = "sqlite")]
        DatabaseDriver::SQLite { filename } => {
            if filename.is_empty() {
                return Err(Error::ConfigurationError(String::from(
                    "filename must not be empty",
                )));
            }
            let connect_options = sqlx::sqlite::SqliteConnectOptions::new()
                .create_if_missing(true)
                .filename(filename)
                .log_statements(LevelFilter::Off)
                .log_slow_statements(LevelFilter::Warn, SLOW_STATEMENTS);
            Impl::Sqlite(
                pool_options!(sqlx::sqlite::SqlitePoolOptions)
                    .connect_with(connect_options)
                    .await?,
            )
        }
        #[cfg(feature = "postgres")]
        DatabaseDriver::Postgres {
            host,
            port,
            name,
            user,
            password,
        } => {
            if name.is_empty() {
                return Err(Error::ConfigurationError(String::from(
                    "name must not be empty",
                )));
            }
            let connect_options = sqlx::postgres::PgConnectOptions::new()
                .host(host.as_str())
                .port(*port)
                .username(user.as_str())
                .password(password.as_str())
                .database(name.as_str())
                .log_statements(LevelFilter::Off)
                .log_slow_statements(LevelFilter::Warn, SLOW_STATEMENTS);
            Impl::Postgres(
                pool_options!(sqlx::postgres::PgPoolOptions)
                    .connect_with(connect_options)
                    .await?,
            )
        }
        #[cfg(feature = "mysql")]
        DatabaseDriver::MySQL {
            name,
            host,
            port,
            user,
            password,
        } => {
            if name.is_empty() {
                return Err(Error::ConfigurationError(String::from(
                    "name must not be empty",
                )));
            }
            let connect_options = sqlx::mysql::MySqlConnectOptions::new()
                .host(host.as_str())
                .port(*port)
                .username(user.as_str())
                .password(password.as_str())
                .database(name.as_str())
                .log_statements(LevelFilter::Off)
                .log_slow_statements(LevelFilter::Warn, SLOW_STATEMENTS);
            Impl::MySql(
                pool_options!(sqlx::mysql::MySqlPoolOptions)
                    .connect_with(connect_options)
                    .await?,
            )
        }
    };

    Ok(pool)
}

/// Implementation of [Database::raw_sql]
pub async fn raw_sql<'a>(
    db: &Database,
    query_string: &'a str,
    bind_params: Option<&[Value<'a>]>,
    transaction: Option<&mut Transaction>,
) -> Result<Vec<Row>, Error> {
    let mut query = if let Some(transaction) = transaction {
        transaction.0.query(query_string)
    } else {
        db.0.query(query_string)
    };

    if let Some(params) = bind_params {
        for param in params {
            utils::bind_param(&mut query, *param);
        }
    }

    let mut stream = query.fetch_many();
    let mut rows = Vec::new();
    while let Some(either) = poll_fn(|ctx| stream.as_mut().poll_next(ctx))
        .await
        .transpose()?
    {
        match either {
            sqlx::Either::Left(_result) => {}
            sqlx::Either::Right(row) => {
                rows.push(Row(row));
            }
        }
    }
    Ok(rows)
}

/// Implementation of [Database::start_transaction]
pub async fn start_transaction(db: &Database) -> Result<Transaction, Error> {
    Ok(Transaction(db.0.begin().await?))
}

/// Implementation of [Database::close]
pub async fn close(db: Database) {
    db.0.close().await;
}

/// Checked in [Database::drop]
pub fn is_closed(db: &Database) -> bool {
    db.0.is_closed()
}
