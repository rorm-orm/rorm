//! This crate is used to provide C bindings for the `rorm-db` crate.
#![warn(missing_docs)]

/**
Utility functions and structs such as the ffi safe string implementation.
*/
pub mod utils;

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use uuid::Uuid;
use once_cell::sync::Lazy;
use tokio::runtime::Runtime;
use rorm_db::{Database, DatabaseBackend, DatabaseConfiguration};

use crate::utils::FFIString;


static RUNTIME: Lazy<Mutex<Option<Runtime>>> = Lazy::new(|| Mutex::new(Some(Runtime::new().expect("Couldn't start runtime"))));

/**
Representation of the database backend.

This is used to determine the correct driver and the correct dialect to use.
*/
#[repr(i32)]
pub enum DBBackend {
    /// SQLite backend
    SQLite,
    /// MySQL / MariaDB backend
    MySQL,
    /// Postgres backend
    Postgres,
}

impl From<DBBackend> for DatabaseBackend {
    fn from(backend: DBBackend) -> Self {
        match backend {
            DBBackend::SQLite => Self::SQLite,
            DBBackend::Postgres => Self::Postgres,
            DBBackend::MySQL => Self::MySQL,
        }
    }
}

/**
Configuration operation to connect to a database.

Will be converted into [rorm_db::DatabaseConfiguration].
*/
#[repr(C)]
pub struct DBConnectOptions {
    backend: DBBackend,
    name: FFIString,
    host: FFIString,
    port: u16,
    user: FFIString,
    password: FFIString,
    min_connections: u32,
    max_connections: u32,
}

impl From<DBConnectOptions> for DatabaseConfiguration {
    fn from(config: DBConnectOptions) -> Self {
        Self {
            backend: config.backend.into(),
            name: String::try_from(config.name).unwrap(),
            host: String::try_from(config.host).unwrap(),
            port: config.port,
            user: String::try_from(config.user).unwrap(),
            password: String::try_from(config.password).unwrap(),
            min_connections: config.min_connections,
            max_connections: config.max_connections
        }
    }
}

static DB_LIST: Lazy<Mutex<HashMap<Uuid, Database>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/**
Connect to the database using the provided [DBConnectOptions].

You must provide a callback with the following signature:
`void (FFIString, FFIString)`.

The first parameter is used as identifier for later operations.
If it is an empty string, the second parameter will hold an error message.
*/
#[no_mangle]
pub extern "C" fn rorm_db_connect(options: DBConnectOptions, callback: extern "C" fn(FFIString, FFIString) -> ()) {
    let db_options = options.into();

    RUNTIME.lock().unwrap().as_ref().unwrap().spawn(async move {
        match Database::connect(db_options).await {
            Ok(db) => {
                let mut l = DB_LIST.lock().unwrap();
                let u = Uuid::new_v4();
                l.insert(u, db);
                callback(u.to_string().into(), String::from("").into());
            }
            Err(e) => {
                callback(FFIString::from(String::from("")), e.to_string().into());
            }
        };
    });
}

/**
Shutdown the runtime.

Specify the amount of time to wait in milliseconds.
*/
#[no_mangle]
pub extern "C" fn rorm_shutdown(duration: u64) {
    RUNTIME.lock().unwrap().take().unwrap().shutdown_timeout(Duration::from_millis(duration));
}
