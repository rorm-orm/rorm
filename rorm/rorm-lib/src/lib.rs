//! This crate is used to provide C bindings for the `rorm-db` crate.
#![deny(missing_docs)]

/// Utility module to provide errors
pub mod errors;
/// Module that holds the definitions for conditions.
pub mod representations;
/// Utility functions and structs such as the ffi safe string implementation.
pub mod utils;

use std::str::Utf8Error;
use std::sync::Mutex;
use std::time::Duration;

use rorm_db::{Database, DatabaseBackend, DatabaseConfiguration};
use tokio::runtime::Runtime;

use crate::errors::Error;
use crate::representations::Condition;
use crate::utils::{null_ptr, FFISlice, FFIString, Stream, VoidPtr};

static RUNTIME: Mutex<Option<Runtime>> = Mutex::new(None);

/**
Representation of the database backend.

This is used to determine the correct driver and the correct dialect to use.
*/
#[repr(i32)]
pub enum DBBackend {
    /// This exists to forbid default initializations with 0 on C side.
    /// Using this type will result in an [crate::errors::Error::ConfigurationError]
    Invalid,
    /// SQLite backend
    SQLite,
    /// MySQL / MariaDB backend
    MySQL,
    /// Postgres backend
    Postgres,
}

impl From<DBBackend> for Result<DatabaseBackend, Error<'_>> {
    fn from(value: DBBackend) -> Self {
        match value {
            DBBackend::Invalid => Err(Error::ConfigurationError(FFIString::from(
                "Invalid database backend selected",
            ))),
            DBBackend::SQLite => Ok(DatabaseBackend::SQLite),
            DBBackend::Postgres => Ok(DatabaseBackend::Postgres),
            DBBackend::MySQL => Ok(DatabaseBackend::MySQL),
        }
    }
}

/**
Configuration operation to connect to a database.

Will be converted into [rorm_db::DatabaseConfiguration].

`min_connections` and `max_connections` must not be 0.
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

impl From<DBConnectOptions<'_>> for Result<DatabaseConfiguration, Error<'_>> {
    fn from(config: DBConnectOptions) -> Self {
        let db_backend_res: Result<DatabaseBackend, Error> = config.backend.into();
        if db_backend_res.is_err() {
            return Err(db_backend_res.err().unwrap());
        }
        let db_backend: DatabaseBackend = db_backend_res.unwrap();
        if config.min_connections == 0 || config.max_connections == 0 {
            return Err(Error::ConfigurationError(FFIString::from(
                "DBConnectOptions.min_connections and DBConnectOptions.max_connections must not be 0",
            )));
        }

        Ok(DatabaseConfiguration {
            backend: db_backend,
            name: <&str>::try_from(config.name).unwrap().to_owned(),
            host: <&str>::try_from(config.host).unwrap().to_owned(),
            port: config.port,
            user: <&str>::try_from(config.user).unwrap().to_owned(),
            password: <&str>::try_from(config.password).unwrap().to_owned(),
            min_connections: config.min_connections,
            max_connections: config.max_connections,
        })
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

This function is called completely synchronously.
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

                    callback(context, Error::NoError);
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

If no runtime is currently existing, a [Error::MissingRuntimeError] will be returned.
If the runtime could not be locked, a [Error::RuntimeError]
containing further information will be returned.

This function is called completely synchronously.
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
The second parameter will be a pointer to the Database connection.
It will be needed to make queries.
The last parameter holds an [Error] enum.

**Important**:
Rust does not manage the memory of the database.
To properly free it, use [rorm_db_free].

This function is called from an asynchronous context.
*/
#[no_mangle]
pub extern "C" fn rorm_db_connect(
    options: DBConnectOptions,
    callback: extern "C" fn(VoidPtr, Box<Database>, Error) -> (),
    context: VoidPtr,
) {
    let db_options_conv: Result<DatabaseConfiguration, Error> = options.into();
    if db_options_conv.is_err() {
        callback(context, null_ptr(), db_options_conv.err().unwrap());
        return;
    }
    let db_options = db_options_conv.unwrap();

    let fut = async move {
        match Database::connect(db_options).await {
            Ok(db) => {
                let b = Box::new(db);
                callback(context, b, Error::NoError)
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

This function is called completely synchronously.
 */
#[no_mangle]
pub extern "C" fn rorm_db_free(_: Box<Database>) {}

/**
This function queries the database given the provided parameter.

Returns a pointer to the created stream.

**Parameter**:
- `box`: Reference to the Database, provided by [rorm_db_connect].
- `model`: Name of the table to query.
- `columns`: Array of columns to retrieve from the database.
- `condition`: Pointer to a [Condition].
- `callback`: callback function. Takes the `context`, a stream pointer and the [Error].
- `context`: Pass through void pointer.

This function is called completely synchronously.
*/
#[no_mangle]
pub extern "C" fn rorm_db_query_stream(
    db: Box<Database>,
    model: FFIString,
    columns: FFISlice<FFIString>,
    condition: Option<&Condition>,
    callback: extern "C" fn(VoidPtr, Box<Stream>, Error) -> (),
    context: VoidPtr,
) {
    let model_conv: Result<&str, Utf8Error> = model.try_into();
    if model_conv.is_err() {
        callback(context, null_ptr(), Error::InvalidStringError);
        return;
    }

    let column_slice: &[FFIString] = columns.into();
    let mut column_vec = vec![];
    for &x in column_slice {
        let x_conv: Result<&str, Utf8Error> = x.try_into();
        if x_conv.is_err() {
            callback(context, null_ptr(), Error::InvalidStringError);
            return;
        }
        column_vec.push(x_conv.unwrap());
    }

    let query_stream;
    match condition {
        None => {
            query_stream = db.query_stream(model_conv.unwrap(), column_vec.as_slice(), None);
        }
        Some(c) => {
            let cond_conv: Result<rorm_db::conditional::Condition, Utf8Error> = c.try_into();
            if cond_conv.is_err() {
                callback(context, null_ptr(), Error::InvalidStringError);
                return;
            }
            query_stream = db.query_stream(
                model_conv.unwrap(),
                column_vec.as_slice(),
                Some(&cond_conv.unwrap()),
            );
        }
    };
    callback(context, Box::new(query_stream), Error::NoError);
}

/**
Frees the stream given as parameter.

This function panics if the pointer to the stream is invalid.

**Important**:
Do not call this function more than once!

This function is called completely synchronously.
*/
#[no_mangle]
pub extern "C" fn rorm_stream_free(_: Box<Stream>) {}
