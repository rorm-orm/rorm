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
use std::future::{poll_fn, Future};
use std::hash::{BuildHasher, Hasher, RandomState};
use std::net::{IpAddr, Ipv4Addr};
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::pin::pin;
use std::process::Command;
use std::task::Poll;
use std::{fs, io};

/// Creates an in-memory sqlite database
///
/// The database will be initialized to contain the tables for `models`.
#[cfg(feature = "all-drivers")]
pub async fn sqlite_in_memory(
    models: impl ModelSelection,
) -> Result<Database, Box<dyn Error + Send + Sync + 'static>> {
    let db = Database::connect(DatabaseConfiguration {
        min_connections: 1,
        max_connections: 1,
        driver: DatabaseDriver::SQLite {
            filename: ":memory:".to_string(),
        },
    })
    .await?;

    let models = models.into_imr();
    if !models.is_empty() {
        let migration = models_to_initial_migration(models)?;

        let mut tx = db.start_transaction().await?;
        for operation in migration.operations {
            apply_operation(&mut tx, &operation).await?;
        }
        tx.commit().await?;
    }

    Ok(db)
}

/// Creates a postgres in a temporary directory
///
/// This function depends on the `initdb` and `postgres` commands being installed on the host.
pub async fn postgres_tmp_dir<T, F>(
    run: impl FnOnce(Database) -> F,
) -> Result<T, Box<dyn Error + Send + Sync + 'static>>
where
    F: Future<Output = Result<T, Box<dyn Error + Send + Sync + 'static>>>,
{
    let random = RandomState::default().build_hasher().finish();

    let postgres_dir = temp_dir().join(format!(".postgres_{random}"));
    fs::create_dir(&postgres_dir)?;
    Command::new("initdb")
        .arg("-D")
        .arg(&postgres_dir)
        .output()?;

    let [a, b, c, d, e, ..] = random.to_ne_bytes();
    let host = Ipv4Addr::new(127, a, b, c | 1).to_string();
    let port = u16::from_ne_bytes([d, e]) | (1 << 15);
    let mut server = Command::new("postgres")
        .arg("-D")
        .arg(&postgres_dir)
        .arg("-k")
        .arg(&postgres_dir)
        .arg("-h")
        .arg(&host)
        .arg("-p")
        .arg(port.to_string())
        .spawn()?;
    std::thread::sleep(std::time::Duration::from_secs(1));

    Command::new("createdb")
        .arg("-h")
        .arg(&host)
        .arg("-p")
        .arg(port.to_string())
        .arg("testdb")
        .output()?;

    let db = Database::connect(DatabaseConfiguration {
        min_connections: 1,
        max_connections: 1,
        driver: DatabaseDriver::Postgres {
            name: "testdb".to_string(),
            host,
            port,
            user: "gammelalf".to_string(),
            password: "".to_string(),
        },
    })
    .await?;

    let mut future = pin!(run(db));
    let panic_result =
        poll_fn(
            move |x| match catch_unwind(AssertUnwindSafe(|| future.as_mut().poll(x))) {
                Ok(Poll::Ready(x)) => Poll::Ready(Ok(x)),
                Ok(Poll::Pending) => Poll::Pending,
                Err(x) => Poll::Ready(Err(x)),
            },
        )
        .await;

    server.kill()?;
    fs::remove_dir_all(postgres_dir)?;

    match panic_result {
        Ok(result) => result,
        Err(panic) => resume_unwind(panic),
    }
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
