//! Utilities for writing tests

use crate::crud::selector::Selector;
use crate::Model;
use rorm_cli::make_migrations::{run_make_migrations, MakeMigrationsOptions};
use rorm_cli::migrate::apply::apply_operation;
use rorm_db::{Database, DatabaseConfiguration};
use rorm_declaration::config::DatabaseDriver;
use rorm_declaration::imr;
use rorm_declaration::migration::{Migration, MigrationFile};
use rorm_macro::impl_tuple;
use std::env::temp_dir;
use std::error::Error;
use std::fs::File;
use std::future::{poll_fn, Future};
use std::hash::{BuildHasher, Hasher, RandomState};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::path::Path;
use std::pin::pin;
use std::process::{Child, Command, Stdio};
use std::task::Poll;
use std::time::{Duration, Instant};
use std::{fs, io, thread};

/// Error type used by this module's private helpers
type DynError = Box<dyn Error + Send + Sync + 'static>;

/// Creates an in-memory SQLite database
///
/// The database will be initialized to contain the tables for `models`.
#[cfg(feature = "all-drivers")]
pub async fn sqlite_in_memory(models: impl ModelSelection) -> Result<Database, DynError> {
    let db = Database::connect(DatabaseConfiguration {
        min_connections: 1,
        max_connections: 1,
        driver: DatabaseDriver::SQLite {
            filename: ":memory:".to_string(),
        },
    })
    .await?;

    apply_models(&db, models).await?;

    Ok(db)
}

/// Starts a throwaway postgres server in a temporary directory and runs `run` against it
///
/// The database will be initialized to contain the tables for `models`.
///
/// `run`'s output is returned as is.
/// The `Err` variant only reports problems with setting up or tearing down the server.
/// If `run` panics, the server is cleaned up before the panic is resumed.
///
/// # Requirements
///
/// The postgres binaries `initdb`, `postgres` and `pg_ctl` have to be in your `PATH`.
///
/// The server listens on a random loopback address (`127.x.y.z`) and port via TCP.
/// It does not create a unix socket, because some sandboxes forbid them.
///
/// # Caveats
///
/// If the process is killed without unwinding (for example by `SIGKILL` from a test harness' timeout),
/// the server keeps running and its directory is not deleted.
///
/// # Example
///
/// ```no_run
/// use rorm::testing::postgres_tmp_dir;
/// use rorm::Model;
///
/// #[derive(Model)]
/// struct Post {
///     #[rorm(id)]
///     id: i64,
///
///     #[rorm(max_length = 255)]
///     title: String,
/// }
///
/// # async fn test() {
/// postgres_tmp_dir((Post,), |db| async move {
///     let posts = rorm::query(&db, Post).all().await.unwrap();
///     assert!(posts.is_empty());
/// })
/// .await
/// .unwrap();
/// # }
/// ```
pub async fn postgres_tmp_dir<T, F>(
    models: impl ModelSelection,
    run: impl FnOnce(Database) -> F,
) -> Result<T, Box<dyn Error + Send + Sync + 'static>>
where
    F: Future<Output = T>,
{
    let random = RandomState::default().build_hasher().finish();
    let [a, b, c, d, e, ..] = random.to_ne_bytes();
    let host = Ipv4Addr::new(127, a, b, c | 1);
    let port = u16::from_ne_bytes([d, e]) | (1 << 15);

    let root_dir = temp_dir().join(format!(".postgres_{random}"));
    let data_dir = root_dir.join("data");
    let log_file = root_dir.join("postgres.log");

    fs::create_dir(&root_dir)?;
    let mut server = match start_postgres(&data_dir, &log_file, host, port) {
        Ok(server) => server,
        Err(error) => {
            if let Err(error) = fs::remove_dir_all(&root_dir) {
                tracing::warn!(
                    error = DynError::from(error),
                    "Failed to clean up temporary postgres"
                );
            }
            return Err(error);
        }
    };

    let db = match connect_postgres(&mut server, &log_file, host, port).await {
        Ok(db) => db,
        Err(error) => {
            if let Err(error) = stop_postgres(&mut server, &data_dir, &root_dir) {
                tracing::warn!(error, "Failed to clean up temporary postgres");
            }
            return Err(error);
        }
    };

    if let Err(error) = apply_models(&db, models).await {
        db.close().await;
        if let Err(error) = stop_postgres(&mut server, &data_dir, &root_dir) {
            tracing::warn!(error, "Failed to clean up temporary postgres");
        }
        return Err(error);
    }

    let mut future = pin!(run(db.clone()));
    let panic_result =
        poll_fn(
            move |x| match catch_unwind(AssertUnwindSafe(|| future.as_mut().poll(x))) {
                Ok(Poll::Ready(x)) => Poll::Ready(Ok(x)),
                Ok(Poll::Pending) => Poll::Pending,
                Err(x) => Poll::Ready(Err(x)),
            },
        )
        .await;

    db.close().await;
    let stop_result = stop_postgres(&mut server, &data_dir, &root_dir);

    match panic_result {
        Ok(output) => stop_result.map(|()| output),
        Err(panic) => {
            if let Err(error) = stop_result {
                tracing::warn!(error, "Failed to clean up temporary postgres");
            }
            resume_unwind(panic)
        }
    }
}

/// Initializes a postgres data directory and spawns a server on it
fn start_postgres(
    data_dir: &Path,
    log_file: &Path,
    host: Ipv4Addr,
    port: u16,
) -> Result<Child, DynError> {
    let initdb = Command::new("initdb")
        .arg("--pgdata")
        .arg(data_dir)
        .arg("--username=postgres")
        .arg("--auth=trust")
        // Tests don't need durability
        .arg("--no-sync")
        .arg("--no-instructions")
        // Don't depend on the host's locale
        .arg("--encoding=UTF8")
        .arg("--locale=C")
        .output()
        .map_err(|error| command_error("initdb", error))?;
    if !initdb.status.success() {
        return Err(format!(
            "`initdb` failed ({}):\n{}",
            initdb.status,
            String::from_utf8_lossy(&initdb.stderr)
        )
        .into());
    }

    let log = File::create(log_file)?;
    Command::new("postgres")
        .arg("-D")
        .arg(data_dir)
        // Tests don't need durability
        .arg("-F")
        .arg("-h")
        .arg(host.to_string())
        .arg("-p")
        .arg(port.to_string())
        // Don't create a unix socket, some sandboxes forbid them
        .arg("-k")
        .arg("")
        .stdin(Stdio::null())
        .stdout(log.try_clone()?)
        .stderr(log)
        .spawn()
        .map_err(|error| command_error("postgres", error))
}

/// Waits until the postgres server accepts TCP connections and connects to it
///
/// The waiting blocks the current thread,
/// because rorm doesn't depend on an async runtime which could provide a timer.
/// This should be acceptable since the server is usually up within a fraction of a second.
async fn connect_postgres(
    server: &mut Child,
    log_file: &Path,
    host: Ipv4Addr,
    port: u16,
) -> Result<Database, DynError> {
    const TIMEOUT: Duration = Duration::from_secs(10);

    let address = SocketAddr::from((host, port));
    let deadline = Instant::now() + TIMEOUT;
    loop {
        if let Some(status) = server.try_wait()? {
            return Err(format!(
                "`postgres` exited ({status}) before accepting connections:\n{}",
                read_log(log_file)
            )
            .into());
        }
        if TcpStream::connect_timeout(&address, Duration::from_millis(100)).is_ok() {
            break;
        }
        if Instant::now() >= deadline {
            return Err(format!(
                "`postgres` did not accept connections within {TIMEOUT:?}:\n{}",
                read_log(log_file)
            )
            .into());
        }
        thread::sleep(Duration::from_millis(20));
    }

    // The server might still be starting up after it started listening.
    // sqlx retries until the server is ready.
    //
    // The server might also still fail after it started listening on TCP
    // (for example while creating its unix socket),
    // so a connection error should include the log as well.
    Database::connect(DatabaseConfiguration {
        min_connections: 1,
        max_connections: 4,
        driver: DatabaseDriver::Postgres {
            name: "postgres".to_string(),
            host: host.to_string(),
            port,
            user: "postgres".to_string(),
            password: "".to_string(),
        },
    })
    .await
    .map_err(|error| {
        format!(
            "Failed to connect to `postgres`: {error}\n{}",
            read_log(log_file)
        )
        .into()
    })
}

/// Stops a postgres server started by [`start_postgres`] and deletes its directory
fn stop_postgres(server: &mut Child, data_dir: &Path, root_dir: &Path) -> Result<(), DynError> {
    if server.try_wait()?.is_none() {
        // Unlike `Child::kill`, `pg_ctl` waits for all of postgres' processes to exit,
        // so they won't race with deleting the directory.
        // `immediate` skips the shutdown checkpoint, the data is deleted anyway.
        let stopped = Command::new("pg_ctl")
            .arg("stop")
            .arg("--pgdata")
            .arg(data_dir)
            .arg("--mode=immediate")
            .output();
        match stopped {
            Ok(output) if output.status.success() => {}
            Ok(output) => {
                tracing::warn!(
                    "`pg_ctl stop` failed ({}), killing postgres instead:\n{}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr)
                );
                server.kill()?;
            }
            Err(error) => {
                tracing::warn!(
                    "{}, killing postgres instead",
                    command_error("pg_ctl", error)
                );
                server.kill()?;
            }
        }
    }
    server.wait()?;
    fs::remove_dir_all(root_dir)?;
    Ok(())
}

/// Reads postgres' log file to include it in an error message
fn read_log(log_file: &Path) -> String {
    fs::read_to_string(log_file)
        .unwrap_or_else(|error| format!("<failed to read {}: {error}>", log_file.display()))
}

/// Converts an error from spawning `program` into a more helpful one
fn command_error(program: &str, error: io::Error) -> DynError {
    if error.kind() == io::ErrorKind::NotFound {
        format!("`{program}` was not found, make sure the postgres binaries are in your PATH")
            .into()
    } else {
        format!("Failed to run `{program}`: {error}").into()
    }
}

/// Creates the tables for `models` in an empty database
async fn apply_models(db: &Database, models: impl ModelSelection) -> Result<(), DynError> {
    let models = models.into_imr();
    if models.is_empty() {
        return Ok(());
    }

    let migration = models_to_initial_migration(models)?;

    let mut tx = db.start_transaction().await?;
    for operation in migration.operations {
        apply_operation(&mut tx, &operation).await?;
    }
    tx.commit().await?;

    Ok(())
}

/// Selection of models to initialize a testing database with
///
/// Possible types are:
/// - `()`: an empty database
/// - `(Model1, Model2, ...)`: a list of models
/// - `Vec<`[`imr::Model`]`>`: manual models
pub trait ModelSelection {
    /// Converts the selection of models into their [`imr`]
    fn into_imr(self) -> Vec<imr::Model>;
}
impl ModelSelection for () {
    fn into_imr(self) -> Vec<imr::Model> {
        Vec::new()
    }
}
impl ModelSelection for Vec<imr::Model> {
    fn into_imr(self) -> Vec<imr::Model> {
        self
    }
}
macro_rules! impl_model_selection {
        ($($index:tt : $S:ident,)+) => {
        impl<$($S: Selector),+> ModelSelection for ($($S,)+) {
            fn into_imr(self) -> Vec<imr::Model> {
                vec![$(<$S::Model as Model>::get_imr(),)+]
            }
        }
    };
}
impl_tuple!(impl_model_selection, 1..33);

/// Converts a list of models into an "initial" migration.
///
/// You can use the [`ModelSelection`] trait to construct the `models` list.
///
/// The returned migration will create all the models' tables assuming an empty database.
pub fn models_to_initial_migration(models: Vec<imr::Model>) -> io::Result<Migration> {
    use std::io::Write;

    let random = RandomState::default().build_hasher().finish();

    let temp_dir = temp_dir();
    let model_file = temp_dir.join(format!(".models_{random}.json"));
    let migration_dir = temp_dir.join(format!(".migrations_{random}"));

    let mut file = fs::File::create(&model_file)?;
    serde_json::to_writer(&mut file, &imr::InternalModelFormat { models })?;
    file.flush()?;
    fs::create_dir(&migration_dir)?;

    run_make_migrations(MakeMigrationsOptions {
        models_file: model_file.display().to_string(),
        migration_dir: migration_dir.display().to_string(),
        name: None,
        non_interactive: true,
        warnings_disabled: false,
    })
    .map_err(io::Error::other)?;

    let MigrationFile { migration } = toml::from_str(&fs::read_to_string(
        migration_dir.join("0001_initial.toml"),
    )?)
    .map_err(io::Error::other)?;

    fs::remove_file(model_file)?;
    fs::remove_dir_all(migration_dir)?;

    Ok(migration)
}

/// Converts any error into something `assert_eq` can handle
///
/// Writing `assert_eq` tests against something that returns an `Result` can be annoying,
/// because most error types don't implement `PartialEq`.
///
/// This function solves it by converting the error into a comparable string.
/// Furthermore, by leaking the string (which should be fine in tests) it enables easy assertions of the error messages:
///
/// ```
/// # use std::io;
/// # use rorm::testing::e;
/// fn will_fail() -> Result<(), io::Error> {
///     Err(io::Error::other("oh no"))
/// }
/// assert_eq!(will_fail().map_err(e), Err("oh no"))
/// ```
///
/// The name was chosen as short as possible, such that more assertions using it can fit in one line.
pub fn e(error: impl ToString + Sized) -> &'static str {
    error.to_string().leak()
}
