//! This crate is used to provide C bindings for the `rorm-db` crate.
#![warn(missing_docs)]

/// Utility functions and structs such as the ffi safe string implementation.
pub mod utils;

/// Utility module to provide errors
pub mod errors;

use std::sync::Mutex;
use std::time::Duration;

use rorm_db::{Database, DatabaseBackend, DatabaseConfiguration};
use tokio::runtime::Runtime;

use crate::errors::Error;
use crate::utils::{null_ptr, FFIString, VoidPtr};

static RUNTIME: Mutex<Option<Runtime>> = Mutex::new(None);

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
pub struct DBConnectOptions<'a> {
    backend: DBBackend,
    name: FFIString<'a>,
    host: FFIString<'a>,
    port: u16,
    user: FFIString<'a>,
    password: FFIString<'a>,
    min_connections: u32,
    max_connections: u32,
}

impl From<DBConnectOptions<'_>> for DatabaseConfiguration {
    fn from(config: DBConnectOptions) -> Self {
        Self {
            backend: config.backend.into(),
            name: <&str>::try_from(config.name).unwrap().to_owned(),
            host: <&str>::try_from(config.host).unwrap().to_owned(),
            port: config.port,
            user: <&str>::try_from(config.user).unwrap().to_owned(),
            password: <&str>::try_from(config.password).unwrap().to_owned(),
            min_connections: config.min_connections,
            max_connections: config.max_connections,
        }
    }
}

// ----------------------
// FFI Functions below here.
// ----------------------

/**
This function is used to initialize and start the async runtime.

It is needed as rorm is completely written asynchronously.

**Important**:
Do not forget to stop the runtime using [rorm_runtime_shutdown]!
 */
#[no_mangle]
pub extern "C" fn rorm_runtime_start(
    callback: extern "C" fn(VoidPtr, Error) -> (),
    context: VoidPtr,
) {
    match RUNTIME.lock() {
        Ok(mut guard) => {
            let rt_opt: &mut Option<Runtime> = &mut guard;
            match Runtime::new() {
                Ok(rt) => {
                    *rt_opt = Some(rt);
                }
                Err(err) => callback(
                    context,
                    Error::RuntimeError(err.to_string().as_str().into()),
                ),
            };
        }
        Err(err) => {
            callback(
                context,
                Error::RuntimeError(err.to_string().as_str().into()),
            );
        }
    }
}

/**
Shutdown the runtime.

Specify the amount of time to wait in milliseconds.
 */
#[no_mangle]
pub extern "C" fn rorm_runtime_shutdown(
    duration: u64,
    callback: extern "C" fn(VoidPtr, Error) -> (),
    context: VoidPtr,
) {
    match RUNTIME.lock() {
        Ok(mut guard) => match guard.take() {
            Some(rt) => {
                rt.shutdown_timeout(Duration::from_millis(duration));
                callback(context, Error::NoError);
            }
            None => callback(context, Error::MissingRuntimeError),
        },
        Err(err) => callback(
            context,
            Error::RuntimeError(err.to_string().as_str().into()),
        ),
    };
}

/**
Connect to the database using the provided [DBConnectOptions].

You must provide a callback with the following parameters:

The first parameter is the `context` pointer.
If it is an empty string, the second parameter will hold an error message.

**Important**:
Rust does not manage the memory of the database.
To properly free it, use [rorm_db_free].
*/
#[no_mangle]
pub extern "C" fn rorm_db_connect(
    options: DBConnectOptions,
    callback: extern "C" fn(VoidPtr, Box<Database>, Error) -> (),
    context: VoidPtr,
) {
    let db_options = options.into();

    let fut = async move {
        match Database::connect(db_options).await {
            Ok(db) => {
                let b = Box::new(db);
                callback(context, b, Error::RuntimeError(FFIString::from("")))
            }
            Err(e) => {
                let error = e.to_string();
                callback(
                    context,
                    null_ptr(),
                    Error::RuntimeError(FFIString::from(error.as_str())),
                );
            }
        };
    };

    match RUNTIME.lock() {
        Ok(guard) => match guard.as_ref() {
            Some(rt) => {
                rt.spawn(fut);
            }
            None => callback(
                context,
                null_ptr(),
                Error::RuntimeError(FFIString::from("No runtime running.")),
            ),
        },
        Err(err) => callback(
            context,
            null_ptr(),
            Error::RuntimeError(err.to_string().as_str().into()),
        ),
    };
}

/**
Free the connection to the database.

Takes the pointer to the database instance.

**Important**:
Do not call this function more than once!
 */
#[no_mangle]
pub extern "C" fn rorm_db_free(_: Box<Database>) {}
