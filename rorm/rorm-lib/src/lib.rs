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

use futures::StreamExt;
use rorm_db::row::Row;
use rorm_db::value::Value;
use rorm_db::{Database, DatabaseConfiguration};
use tokio::runtime::Runtime;

use crate::errors::Error;
use crate::representations::{Condition, DBConnectOptions, FFIValue};
use crate::utils::{
    FFIDate, FFIDateTime, FFIOption, FFISlice, FFIString, FFITime, RowList, Stream, VoidPtr,
};

static RUNTIME: Mutex<Option<Runtime>> = Mutex::new(None);

// ----------------------
// FFI Functions below here.
// ----------------------

// --------
// RUNTIME
// --------
/**
This function is used to initialize and start the async runtime.

It is needed as rorm is completely written asynchronously.

**Important**:
Do not forget to stop the runtime using [rorm_runtime_shutdown]!

This function is called completely synchronously.
 */
#[no_mangle]
pub extern "C" fn rorm_runtime_start(
    callback: Option<unsafe extern "C" fn(VoidPtr, Error) -> ()>,
    context: VoidPtr,
) {
    let cb = callback.expect("Callback must not be null");

    match RUNTIME.lock() {
        Ok(mut guard) => {
            let rt_opt: &mut Option<Runtime> = &mut guard;
            match Runtime::new() {
                Ok(rt) => {
                    *rt_opt = Some(rt);

                    unsafe { cb(context, Error::NoError) }
                }
                Err(err) => unsafe {
                    cb(
                        context,
                        Error::RuntimeError(err.to_string().as_str().into()),
                    )
                },
            };
        }
        Err(err) => unsafe {
            cb(
                context,
                Error::RuntimeError(err.to_string().as_str().into()),
            )
        },
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
    callback: Option<unsafe extern "C" fn(VoidPtr, Error) -> ()>,
    context: VoidPtr,
) {
    let cb = callback.expect("Callback must not be null");

    match RUNTIME.lock() {
        Ok(mut guard) => match guard.take() {
            Some(rt) => {
                rt.shutdown_timeout(Duration::from_millis(duration));
                unsafe { cb(context, Error::NoError) }
            }
            None => unsafe { cb(context, Error::MissingRuntimeError) },
        },
        Err(err) => unsafe {
            cb(
                context,
                Error::RuntimeError(err.to_string().as_str().into()),
            )
        },
    };
}

// --------
// DATABASE
// --------

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
    callback: Option<unsafe extern "C" fn(VoidPtr, Option<Box<Database>>, Error) -> ()>,
    context: VoidPtr,
) {
    let cb = callback.expect("Callback must not be null");

    let db_options_conv: Result<DatabaseConfiguration, Error> = options.into();
    if db_options_conv.is_err() {
        unsafe { cb(context, None, db_options_conv.err().unwrap()) }
        return;
    }
    let db_options = db_options_conv.unwrap();

    let fut = async move {
        match Database::connect(db_options).await {
            Ok(db) => {
                let b = Box::new(db);
                unsafe { cb(context, Some(b), Error::NoError) }
            }
            Err(e) => {
                let error = e.to_string();
                unsafe {
                    cb(
                        context,
                        None,
                        Error::RuntimeError(FFIString::from(error.as_str())),
                    )
                }
            }
        };
    };

    let f = |err: String| {
        unsafe { cb(context, None, Error::RuntimeError(err.as_str().into())) };
    };
    spawn_fut!(fut, cb(context, None, Error::MissingRuntimeError), f);
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

// --------
// QUERY
// --------

/**
This function queries the database given the provided parameter and returns one matched row.

**Parameter**:
- `db`: Reference to the Database, provided by [rorm_db_connect].
- `model`: Name of the table to query.
- `columns`: Array of columns to retrieve from the database.
- `condition`: Pointer to a [Condition].
- `callback`: callback function. Takes the `context`, a pointer to a row and an [Error].
- `context`: Pass through void pointer.

**Important**:
- Make sure that `db`, `model`, `columns` and `condition` are
allocated until the callback is executed.

This function is called from an asynchronous context.
 */
#[no_mangle]
pub extern "C" fn rorm_db_query_one(
    db: &'static Database,
    model: FFIString<'static>,
    columns: FFISlice<'static, FFIString<'static>>,
    condition: Option<&'static Condition>,
    callback: Option<unsafe extern "C" fn(VoidPtr, Option<Box<Row>>, Error) -> ()>,
    context: VoidPtr,
) {
    let cb = callback.expect("Callback must not be empty");

    let model_conv = model.try_into();
    if model_conv.is_err() {
        unsafe { cb(context, None, Error::InvalidStringError) };
        return;
    }

    let mut column_vec = vec![];
    {
        let column_slice: &[FFIString] = columns.into();
        for &x in column_slice {
            let x_conv = x.try_into();
            if x_conv.is_err() {
                unsafe { cb(context, None, Error::InvalidStringError) };
                return;
            }
            column_vec.push(x_conv.unwrap());
        }
    }

    let cond;
    if condition.is_none() {
        cond = None;
    } else {
        let cond_conv = condition.unwrap().try_into();
        if cond_conv.is_err() {
            match cond_conv.as_ref().err().unwrap() {
                Error::InvalidStringError
                | Error::InvalidDateError
                | Error::InvalidTimeError
                | Error::InvalidDateTimeError => unsafe {
                    cb(context, None, cond_conv.err().unwrap())
                },
                _ => {}
            }
            return;
        }
        cond = Some(cond_conv.unwrap());
    };

    let fut = async move {
        match cond {
            None => {
                match db
                    .query_one(model_conv.unwrap(), column_vec.as_slice(), None)
                    .await
                {
                    Ok(v) => unsafe { cb(context, Some(Box::new(v)), Error::NoError) },
                    Err(err) => {
                        let ffi_str = err.to_string();
                        unsafe { cb(context, None, Error::DatabaseError(ffi_str.as_str().into())) };
                    }
                };
            }
            Some(c) => match db
                .query_one(model_conv.unwrap(), column_vec.as_slice(), Some(&c))
                .await
            {
                Ok(v) => unsafe { cb(context, Some(Box::new(v)), Error::NoError) },
                Err(err) => {
                    let ffi_str = err.to_string();
                    unsafe { cb(context, None, Error::DatabaseError(ffi_str.as_str().into())) }
                }
            },
        }
    };

    let f = |err: String| {
        unsafe { cb(context, None, Error::RuntimeError(err.as_str().into())) };
    };
    spawn_fut!(fut, cb(context, None, Error::MissingRuntimeError), f);
}

/**
This function queries the database given the provided parameter and returns all matched rows.

**Parameter**:
- `db`: Reference to the Database, provided by [rorm_db_connect].
- `model`: Name of the table to query.
- `columns`: Array of columns to retrieve from the database.
- `condition`: Pointer to a [Condition].
- `callback`: callback function. Takes the `context`, a pointer to a vec of rows and an [Error].
- `context`: Pass through void pointer.

**Important**:
- Make sure that `db`, `model`, `columns` and `condition` are
allocated until the callback is executed.

This function is called from an asynchronous context.
 */
#[no_mangle]
pub extern "C" fn rorm_db_query_all(
    db: &'static Database,
    model: FFIString<'static>,
    columns: FFISlice<'static, FFIString<'static>>,
    condition: Option<&'static Condition>,
    callback: Option<unsafe extern "C" fn(VoidPtr, Option<Box<RowList>>, Error) -> ()>,
    context: VoidPtr,
) {
    let cb = callback.expect("Callback must not be null");

    let model_conv = model.try_into();
    if model_conv.is_err() {
        unsafe { cb(context, None, Error::InvalidStringError) };
        return;
    }

    let mut column_vec = vec![];
    {
        let column_slice: &[FFIString] = columns.into();
        for &x in column_slice {
            let x_conv = x.try_into();
            if x_conv.is_err() {
                unsafe { cb(context, None, Error::InvalidStringError) };
                return;
            }
            column_vec.push(x_conv.unwrap());
        }
    }

    let cond;
    if condition.is_none() {
        cond = None;
    } else {
        let cond_conv = condition.unwrap().try_into();
        if cond_conv.is_err() {
            match cond_conv.as_ref().err().unwrap() {
                Error::InvalidStringError
                | Error::InvalidDateError
                | Error::InvalidTimeError
                | Error::InvalidDateTimeError => unsafe {
                    cb(context, None, cond_conv.err().unwrap())
                },
                _ => {}
            }
            return;
        }
        cond = Some(cond_conv.unwrap());
    };

    let fut = async move {
        let query_res;
        match cond {
            None => {
                query_res = db
                    .query_all(model_conv.unwrap(), column_vec.as_slice(), None)
                    .await;
            }
            Some(cond) => {
                query_res = db
                    .query_all(model_conv.unwrap(), column_vec.as_slice(), Some(&cond))
                    .await;
            }
        };
        match query_res {
            Ok(res) => {
                unsafe { cb(context, Some(Box::new(res)), Error::NoError) };
            }
            Err(err) => {
                let ffi_err = err.to_string();
                unsafe { cb(context, None, Error::RuntimeError(ffi_err.as_str().into())) };
            }
        };
    };

    let f = |err: String| {
        unsafe { cb(context, None, Error::RuntimeError(err.as_str().into())) };
    };
    spawn_fut!(fut, cb(context, None, Error::MissingRuntimeError), f);
}

/**
This function queries the database given the provided parameter.

Returns a pointer to the created stream.

**Parameter**:
- `db`: Reference to the Database, provided by [rorm_db_connect].
- `model`: Name of the table to query.
- `columns`: Array of columns to retrieve from the database.
- `condition`: Pointer to a [Condition].
- `callback`: callback function. Takes the `context`, a stream pointer and an [Error].
- `context`: Pass through void pointer.

This function is called completely synchronously.
 */
#[no_mangle]
pub extern "C" fn rorm_db_query_stream(
    db: &Database,
    model: FFIString,
    columns: FFISlice<FFIString>,
    condition: Option<&Condition>,
    callback: Option<unsafe extern "C" fn(VoidPtr, Option<Box<Stream>>, Error) -> ()>,
    context: VoidPtr,
) {
    let cb = callback.expect("Callback must not be null");

    let model_conv = model.try_into();
    if model_conv.is_err() {
        unsafe { cb(context, None, Error::InvalidStringError) };
        return;
    }

    let column_slice: &[FFIString] = columns.into();
    let mut column_vec = vec![];
    for &x in column_slice {
        let x_conv = x.try_into();
        if x_conv.is_err() {
            unsafe { cb(context, None, Error::InvalidStringError) };
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
            let cond_conv: Result<rorm_db::conditional::Condition, Error> = c.try_into();
            if cond_conv.is_err() {
                match cond_conv.as_ref().err().unwrap() {
                    Error::InvalidStringError
                    | Error::InvalidDateError
                    | Error::InvalidTimeError
                    | Error::InvalidDateTimeError => unsafe {
                        cb(context, None, cond_conv.err().unwrap())
                    },
                    _ => {}
                }
                return;
            }
            query_stream = db.query_stream(
                model_conv.unwrap(),
                column_vec.as_slice(),
                Some(&cond_conv.unwrap()),
            );
        }
    };
    unsafe { cb(context, Some(Box::new(query_stream)), Error::NoError) }
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

/**
Use this function to retrieve a pointer to a row on a stream.

**Parameter**:
- `stream_ptr`: Mutable pointer to the stream that is obtained from [rorm_db_query_stream].
- `callback`: callback function. Takes the `context`, a row pointer and a [Error].
- `context`: Pass through void pointer

**Important**:
- Do not call this function multiple times on the same stream, unless all given callbacks have
returned successfully. Calling the function multiple times on the same stream will result in
undefined behaviour!
- Do not call this function on the same stream if the previous call
returned a [Error::NoRowsLeftInStream].
- Do not use pass the stream to another function unless the callback of the current call is finished

This function is called from an asynchronous context.
 */
#[no_mangle]
pub extern "C" fn rorm_stream_get_row(
    stream_ptr: &'static mut Stream,
    callback: Option<unsafe extern "C" fn(VoidPtr, Option<Box<Row>>, Error) -> ()>,
    context: VoidPtr,
) {
    let cb = callback.expect("Callback must not be null");

    let fut = async move {
        let row_opt = stream_ptr.next().await;
        match row_opt {
            None => unsafe { cb(context, None, Error::NoRowsLeftInStream) },
            Some(row_res) => match row_res {
                Err(err) => unsafe {
                    cb(
                        context,
                        None,
                        Error::DatabaseError(err.to_string().as_str().into()),
                    )
                },
                Ok(row) => unsafe { cb(context, Some(Box::new(row)), Error::NoError) },
            },
        }
    };

    let f = |err: String| {
        unsafe { cb(context, None, Error::RuntimeError(err.as_str().into())) };
    };
    spawn_fut!(fut, cb(context, None, Error::MissingRuntimeError), f);
}

// --------
// DELETE
// --------
/**
This function deletes rows from the database based on the given conditions.

**Parameter**:
- `db`: Reference to the Database, provided by [rorm_db_connect].
- `model`: Name of the table to query.
- `condition`: Pointer to a [Condition].
- `limit`: Optional limit of deletions that should be executed.
- `callback`: callback function. Takes the `context`, a pointer to a vec of rows and an [Error].
- `context`: Pass through void pointer.

Returns the rows affected of the delete statement. Note that this also includes
    relations, etc.

**Important**:
- Make sure that `db`, `model` and `condition` are
allocated until the callback is executed.

This function is called from an asynchronous context.
*/
#[no_mangle]
pub extern "C" fn rorm_db_delete(
    db: &'static Database,
    model: FFIString<'static>,
    condition: Option<&'static Condition>,
    limit: FFIOption<u64>,
    callback: Option<unsafe extern "C" fn(VoidPtr, u64, Error) -> ()>,
    context: VoidPtr,
) {
    let cb = callback.expect("Callback must not be null");

    let model_conv = model.try_into();
    if model_conv.is_err() {
        unsafe { cb(context, u64::MAX, Error::InvalidStringError) };
        return;
    }

    let cond;
    if condition.is_none() {
        cond = None;
    } else {
        let cond_conv = condition.unwrap().try_into();
        if cond_conv.is_err() {
            match cond_conv.as_ref().err().unwrap() {
                Error::InvalidStringError
                | Error::InvalidDateError
                | Error::InvalidTimeError
                | Error::InvalidDateTimeError => unsafe {
                    cb(context, u64::MAX, cond_conv.err().unwrap())
                },
                _ => {}
            }
            return;
        }
        cond = Some(cond_conv.unwrap());
    }

    let limit_conv = limit.into();

    let fut = async move {
        match cond {
            None => match db.delete(model_conv.unwrap(), None, limit_conv).await {
                Ok(rows_affected) => unsafe { cb(context, rows_affected, Error::NoError) },
                Err(err) => {
                    let ffi_err = err.to_string();
                    unsafe {
                        cb(
                            context,
                            u64::MAX,
                            Error::DatabaseError(ffi_err.as_str().into()),
                        )
                    };
                }
            },
            Some(v) => match db.delete(model_conv.unwrap(), Some(&v), limit_conv).await {
                Ok(rows_affected) => unsafe { cb(context, rows_affected, Error::NoError) },
                Err(err) => {
                    let ffi_err = err.to_string();
                    unsafe {
                        cb(
                            context,
                            u64::MAX,
                            Error::DatabaseError(ffi_err.as_str().into()),
                        )
                    };
                }
            },
        }
    };

    let f = |err: String| {
        unsafe { cb(context, u64::MAX, Error::RuntimeError(err.as_str().into())) };
    };
    spawn_fut!(fut, cb(context, u64::MAX, Error::MissingRuntimeError), f);
}

/**
Frees the RowList given as parameter.

This function panics if the pointer to the RowList is invalid.

**Important**:
Do not call this function more than once!

This function is called completely synchronously.
*/
#[no_mangle]
pub extern "C" fn rorm_row_list_free(_: Box<RowList>) {}

/**
Frees the row given as parameter.

The function panics if the provided pointer is invalid.

**Important**:
Do not call this function on the same pointer more than once!

This function is called completely synchronously.
 */
#[no_mangle]
pub extern "C" fn rorm_row_free(_: Box<Row>) {}

// --------
// INSERT
// --------

/**
This function inserts a row into the database.

**Parameter**:
- `db`: Reference to the Database, provided by [rorm_db_connect].
- `model`: Name of the table to query.
- `columns`: Array of columns to insert to the table.
- `row`: List of values to insert. Must be of the same length as `columns`.
- `callback`: callback function. Takes the `context` and an [Error].
- `context`: Pass through void pointer.

**Important**:
- Make sure that `db`, `model`, `columns` and `row` are allocated until the callback is executed.

This function is called from an asynchronous context.
*/
#[no_mangle]
pub extern "C" fn rorm_db_insert(
    db: &'static Database,
    model: FFIString<'static>,
    columns: FFISlice<'static, FFIString<'static>>,
    row: FFISlice<'static, FFIValue<'static>>,
    callback: Option<unsafe extern "C" fn(VoidPtr, Error) -> ()>,
    context: VoidPtr,
) {
    let cb = callback.expect("Callback must not be empty");

    let model_conv = model.try_into();
    if model_conv.is_err() {
        unsafe { cb(context, Error::InvalidStringError) };
        return;
    }
    let model = model_conv.unwrap();

    let mut column_vec = vec![];
    {
        let column_slice: &[FFIString] = columns.into();
        for &x in column_slice {
            let x_conv = x.try_into();
            if x_conv.is_err() {
                unsafe { cb(context, Error::InvalidStringError) };
                return;
            }
            column_vec.push(x_conv.unwrap());
        }
    }

    let mut value_vec = vec![];
    {
        let value_slice: &[FFIValue] = row.into();
        for x in value_slice {
            let x_conv = x.try_into();
            if x_conv.is_err() {
                match x_conv.as_ref().err().unwrap() {
                    Error::InvalidStringError
                    | Error::InvalidDateError
                    | Error::InvalidTimeError
                    | Error::InvalidDateTimeError => unsafe { cb(context, x_conv.err().unwrap()) },
                    _ => {}
                }
                return;
            }
            value_vec.push(x_conv.unwrap());
        }
    }

    let fut = async move {
        match db
            .insert(model, column_vec.as_slice(), value_vec.as_slice())
            .await
        {
            Err(err) => unsafe {
                cb(
                    context,
                    Error::DatabaseError(err.to_string().as_str().into()),
                )
            },
            Ok(_) => unsafe { cb(context, Error::NoError) },
        };
    };

    let f = |err: String| {
        unsafe { cb(context, Error::RuntimeError(err.as_str().into())) };
    };
    spawn_fut!(fut, cb(context, Error::MissingRuntimeError), f);
}

/**
This function inserts multiple rows into the database.

**Parameter**:
- `db`: Reference to the Database, provided by [rorm_db_connect].
- `model`: Name of the table to query.
- `columns`: Array of columns to insert to the table.
- `rows`: List of list of values to insert. The inner lists must be of the same length as `columns`.
- `callback`: callback function. Takes the `context` and an [Error].
- `context`: Pass through void pointer.

**Important**:
- Make sure that `db`, `model`, `columns` and `rows` are allocated until the callback is executed.

This function is called from an asynchronous context.
*/
#[no_mangle]
pub extern "C" fn rorm_db_insert_bulk(
    db: &'static Database,
    model: FFIString<'static>,
    columns: FFISlice<'static, FFIString<'static>>,
    rows: FFISlice<'static, FFISlice<'static, FFIValue<'static>>>,
    callback: Option<unsafe extern "C" fn(VoidPtr, Error) -> ()>,
    context: VoidPtr,
) {
    let cb = callback.expect("Callback must not be empty");

    let model_conv = model.try_into();
    if model_conv.is_err() {
        unsafe { cb(context, Error::InvalidStringError) };
        return;
    }
    let model = model_conv.unwrap();

    let mut column_vec = vec![];
    {
        let column_slice: &[FFIString] = columns.into();
        for &x in column_slice {
            let x_conv = x.try_into();
            if x_conv.is_err() {
                unsafe { cb(context, Error::InvalidStringError) };
                return;
            }
            column_vec.push(x_conv.unwrap());
        }
    }

    let mut rows_vec = vec![];
    {
        let row_slices: &[FFISlice<FFIValue>] = rows.into();
        for row in row_slices {
            let mut row_vec = vec![];
            let row_slice: &[FFIValue] = row.into();
            for x in row_slice {
                let val = x.try_into();
                if val.is_err() {
                    match val.as_ref().err().unwrap() {
                        Error::InvalidStringError
                        | Error::InvalidDateError
                        | Error::InvalidTimeError
                        | Error::InvalidDateTimeError => unsafe { cb(context, val.err().unwrap()) },
                        _ => {}
                    }
                    return;
                }
                row_vec.push(val.unwrap());
            }
            rows_vec.push(row_vec);
        }
    }

    let fut = async move {
        match db
            .insert_bulk(
                model,
                column_vec.as_slice(),
                rows_vec
                    .iter()
                    .map(|x| x.as_slice())
                    .collect::<Vec<&[Value]>>()
                    .as_slice(),
            )
            .await
        {
            Ok(_) => unsafe { cb(context, Error::NoError) },
            Err(err) => {
                let ffi_str = err.to_string();
                unsafe { cb(context, Error::DatabaseError(ffi_str.as_str().into())) };
            }
        }
    };

    let f = |err: String| {
        unsafe { cb(context, Error::RuntimeError(err.as_str().into())) };
    };
    spawn_fut!(fut, cb(context, Error::MissingRuntimeError), f);
}

// --------
// ROW VALUE DECODING
// --------

/**
Tries to retrieve a bool from the given row pointer.

**Parameter**:
- `row_ptr`: Pointer to a row.
- `index`: Name of the column to retrieve from the row.
- `error_ptr`: Pointer to an [Error]. Gets only written to if an error occurs.

This function is called completely synchronously.
*/
#[no_mangle]
pub extern "C" fn rorm_row_get_bool(
    row_ptr: &Row,
    index: FFIString<'_>,
    error_ptr: &mut Error,
) -> bool {
    get_data_from_row!(bool, false, row_ptr, index, error_ptr);
}

/**
Tries to retrieve an i64 from the given row pointer.

**Parameter**:
- `row_ptr`: Pointer to a row.
- `index`: Name of the column to retrieve from the row.
- `error_ptr`: Pointer to an [Error]. Gets only written to if an error occurs.

This function is called completely synchronously.
 */
#[no_mangle]
pub extern "C" fn rorm_row_get_i64(
    row_ptr: &Row,
    index: FFIString<'_>,
    error_ptr: &mut Error,
) -> i64 {
    get_data_from_row!(i64, i64::MAX, row_ptr, index, error_ptr);
}

/**
Tries to retrieve an i32 from the given row pointer.

**Parameter**:
- `row_ptr`: Pointer to a row.
- `index`: Name of the column to retrieve from the row.
- `error_ptr`: Pointer to an [Error]. Gets only written to if an error occurs.

This function is called completely synchronously.
 */
#[no_mangle]
pub extern "C" fn rorm_row_get_i32(
    row_ptr: &Row,
    index: FFIString<'_>,
    error_ptr: &mut Error,
) -> i32 {
    get_data_from_row!(i32, i32::MAX, row_ptr, index, error_ptr);
}

/**
Tries to retrieve an i16 from the given row pointer.

**Parameter**:
- `row_ptr`: Pointer to a row.
- `index`: Name of the column to retrieve from the row.
- `error_ptr`: Pointer to an [Error]. Gets only written to if an error occurs.

This function is called completely synchronously.
 */
#[no_mangle]
pub extern "C" fn rorm_row_get_i16(
    row_ptr: &Row,
    index: FFIString<'_>,
    error_ptr: &mut Error,
) -> i16 {
    get_data_from_row!(i16, i16::MAX, row_ptr, index, error_ptr);
}

/**
Tries to retrieve an FFISlice of a u8 from the given row pointer.

**Parameter**:
- `row_ptr`: Pointer to a row.
- `index`: Name of the column to retrieve from the row.
- `error_ptr`: Pointer to an [Error]. Gets only written to if an error occurs.

This function is called completely synchronously.
*/
#[no_mangle]
pub extern "C" fn rorm_row_get_binary<'a>(
    row_ptr: &'a Row,
    index: FFIString<'a>,
    error_ptr: &mut Error,
) -> FFISlice<'a, u8> {
    let s: &[u8] = &[];
    get_data_from_row!(&[u8], FFISlice::from(s), row_ptr, index, error_ptr);
}

/**
Tries to retrieve an f32 from the given row pointer.

**Parameter**:
- `row_ptr`: Pointer to a row.
- `index`: Name of the column to retrieve from the row.
- `error_ptr`: Pointer to an [Error]. Gets only written to if an error occurs.

This function is called completely synchronously.
 */
#[no_mangle]
pub extern "C" fn rorm_row_get_f32(
    row_ptr: &Row,
    index: FFIString<'_>,
    error_ptr: &mut Error,
) -> f32 {
    get_data_from_row!(f32, f32::NAN, row_ptr, index, error_ptr);
}

/**
Tries to retrieve an f64 from the given row pointer.

**Parameter**:
- `row_ptr`: Pointer to a row.
- `index`: Name of the column to retrieve from the row.
- `error_ptr`: Pointer to an [Error]. Gets only written to if an error occurs.

This function is called completely synchronously.
 */
#[no_mangle]
pub extern "C" fn rorm_row_get_f64(
    row_ptr: &Row,
    index: FFIString<'_>,
    error_ptr: &mut Error,
) -> f64 {
    get_data_from_row!(f64, f64::NAN, row_ptr, index, error_ptr);
}

/**
Tries to retrieve an FFIString from the given row pointer.

**Parameter**:
- `row_ptr`: Pointer to a row.
- `index`: Name of the column to retrieve from the row.
- `error_ptr`: Pointer to an [Error]. Gets only written to if an error occurs.

This function is called completely synchronously.
 */
#[no_mangle]
pub extern "C" fn rorm_row_get_str<'a, 'b>(
    row_ptr: &'a Row,
    index: FFIString<'_>,
    error_ptr: &'b mut Error,
) -> FFIString<'a> {
    get_data_from_row!(&str, FFIString::from(""), row_ptr, index, error_ptr);
}

/**
Tries to retrieve a FFIDate from the given row pointer.

**Parameter**:
- `row_ptr`: Pointer to a row.
- `index`: Name of the column to retrieve from the row.
- `error_ptr`: Pointer to an [Error]. Gets only written to if an error occurs.

This function is called completely synchronously.
*/
#[no_mangle]
pub extern "C" fn rorm_row_get_date(
    row_ptr: &Row,
    index: FFIString<'_>,
    error_ptr: &mut Error,
) -> FFIDate {
    get_data_from_row!(
        chrono::NaiveDate,
        FFIDate {
            day: 0,
            month: 0,
            year: 0,
        },
        row_ptr,
        index,
        error_ptr
    );
}

/**
Tries to retrieve a FFITime from the given row pointer.

**Parameter**:
- `row_ptr`: Pointer to a row.
- `index`: Name of the column to retrieve from the row.
- `error_ptr`: Pointer to an [Error]. Gets only written to if an error occurs.

This function is called completely synchronously.
*/
#[no_mangle]
pub extern "C" fn rorm_row_get_time(
    row_ptr: &Row,
    index: FFIString<'_>,
    error_ptr: &mut Error,
) -> FFITime {
    get_data_from_row!(
        chrono::NaiveTime,
        FFITime {
            hour: 0,
            min: 0,
            sec: 0,
        },
        row_ptr,
        index,
        error_ptr
    )
}

/**
Tries to retrieve a FFIDateTime from the given row pointer.

**Parameter**:
- `row_ptr`: Pointer to a row.
- `index`: Name of the column to retrieve from the row.
- `error_ptr`: Pointer to an [Error]. Gets only written to if an error occurs.

This function is called completely synchronously.
*/
#[no_mangle]
pub extern "C" fn rorm_row_get_datetime(
    row_ptr: &Row,
    index: FFIString<'_>,
    error_ptr: &mut Error,
) -> FFIDateTime {
    get_data_from_row!(
        chrono::NaiveDateTime,
        FFIDateTime {
            year: 0,
            month: 0,
            day: 0,
            hour: 0,
            min: 0,
            sec: 0,
        },
        row_ptr,
        index,
        error_ptr
    )
}

/**
Tries to retrieve a nullable bool from the given row pointer.

**Parameter**:
- `row_ptr`: Pointer to a row.
- `index`: Name of the column to retrieve from the row.
- `error_ptr`: Pointer to an [Error]. Gets only written to if an error occurs.

This function is called completely synchronously.
 */
#[no_mangle]
pub extern "C" fn rorm_row_get_null_bool(
    row_ptr: &Row,
    index: FFIString<'_>,
    error_ptr: &mut Error,
) -> FFIOption<bool> {
    get_data_from_row!(Option<bool>, FFIOption::None, row_ptr, index, error_ptr);
}

/**
Tries to retrieve a nullable i64 from the given row pointer.

**Parameter**:
- `row_ptr`: Pointer to a row.
- `index`: Name of the column to retrieve from the row.
- `error_ptr`: Pointer to an [Error]. Gets only written to if an error occurs.

This function is called completely synchronously.
 */
#[no_mangle]
pub extern "C" fn rorm_row_get_null_i64(
    row_ptr: &Row,
    index: FFIString<'_>,
    error_ptr: &mut Error,
) -> FFIOption<i64> {
    get_data_from_row!(Option<i64>, FFIOption::None, row_ptr, index, error_ptr);
}

/**
Tries to retrieve a nullable i32 from the given row pointer.

**Parameter**:
- `row_ptr`: Pointer to a row.
- `index`: Name of the column to retrieve from the row.
- `error_ptr`: Pointer to an [Error]. Gets only written to if an error occurs.

This function is called completely synchronously.
 */
#[no_mangle]
pub extern "C" fn rorm_row_get_null_i32(
    row_ptr: &Row,
    index: FFIString<'_>,
    error_ptr: &mut Error,
) -> FFIOption<i32> {
    get_data_from_row!(Option<i32>, FFIOption::None, row_ptr, index, error_ptr);
}

/**
Tries to retrieve a nullable i16 from the given row pointer.

**Parameter**:
- `row_ptr`: Pointer to a row.
- `index`: Name of the column to retrieve from the row.
- `error_ptr`: Pointer to an [Error]. Gets only written to if an error occurs.

This function is called completely synchronously.
 */
#[no_mangle]
pub extern "C" fn rorm_row_get_null_i16(
    row_ptr: &Row,
    index: FFIString<'_>,
    error_ptr: &mut Error,
) -> FFIOption<i16> {
    get_data_from_row!(Option<i16>, FFIOption::None, row_ptr, index, error_ptr);
}

/**
Tries to retrieve a nullable FFISlice of a u8 from the given row pointer.

**Parameter**:
- `row_ptr`: Pointer to a row.
- `index`: Name of the column to retrieve from the row.
- `error_ptr`: Pointer to an [Error]. Gets only written to if an error occurs.

This function is called completely synchronously.
 */
#[no_mangle]
pub extern "C" fn rorm_row_get_null_binary<'a>(
    row_ptr: &'a Row,
    index: FFIString<'a>,
    error_ptr: &mut Error,
) -> FFIOption<FFISlice<'a, u8>> {
    let index_conv: Result<&str, Utf8Error> = index.try_into();
    if index_conv.is_err() {
        *error_ptr = Error::InvalidStringError;
        return FFIOption::None;
    }
    let value_res: Result<Option<&[u8]>, rorm_db::error::Error> = row_ptr.get(index_conv.unwrap());
    if value_res.is_err() {
        match value_res.err().unwrap() {
            rorm_db::error::Error::SqlxError(err) => match err {
                sqlx::Error::ColumnIndexOutOfBounds { .. } => {
                    *error_ptr = Error::ColumnIndexOutOfBoundsError;
                }
                sqlx::Error::ColumnNotFound(_) => {
                    *error_ptr = Error::ColumnNotFoundError;
                }
                sqlx::Error::ColumnDecode { .. } => {
                    *error_ptr = Error::ColumnDecodeError;
                }
                _ => todo!("This error case should never occur"),
            },
            _ => todo!("This error case should never occur"),
        };
        return FFIOption::None;
    }

    return match value_res.unwrap() {
        None => FFIOption::None,
        Some(v) => FFIOption::Some(v.into()),
    };
}

/**
Tries to retrieve a nullable f32 from the given row pointer.

**Parameter**:
- `row_ptr`: Pointer to a row.
- `index`: Name of the column to retrieve from the row.
- `error_ptr`: Pointer to an [Error]. Gets only written to if an error occurs.

This function is called completely synchronously.
 */
#[no_mangle]
pub extern "C" fn rorm_row_get_null_f32(
    row_ptr: &Row,
    index: FFIString<'_>,
    error_ptr: &mut Error,
) -> FFIOption<f32> {
    get_data_from_row!(Option<f32>, FFIOption::None, row_ptr, index, error_ptr);
}

/**
Tries to retrieve a nullable f64 from the given row pointer.

**Parameter**:
- `row_ptr`: Pointer to a row.
- `index`: Name of the column to retrieve from the row.
- `error_ptr`: Pointer to an [Error]. Gets only written to if an error occurs.

This function is called completely synchronously.
 */
#[no_mangle]
pub extern "C" fn rorm_row_get_null_f64(
    row_ptr: &Row,
    index: FFIString<'_>,
    error_ptr: &mut Error,
) -> FFIOption<f64> {
    get_data_from_row!(Option<f64>, FFIOption::None, row_ptr, index, error_ptr);
}

/**
Tries to retrieve a nullable FFIString from the given row pointer.

**Parameter**:
- `row_ptr`: Pointer to a row.
- `index`: Name of the column to retrieve from the row.
- `error_ptr`: Pointer to an [Error]. Gets only written to if an error occurs.

This function is called completely synchronously.
 */
#[no_mangle]
pub extern "C" fn rorm_row_get_null_str<'a, 'b>(
    row_ptr: &'a Row,
    index: FFIString<'_>,
    error_ptr: &'b mut Error,
) -> FFIOption<FFIString<'a>> {
    let index_conv: Result<&str, Utf8Error> = index.try_into();
    if index_conv.is_err() {
        *error_ptr = Error::InvalidStringError;
        return FFIOption::None;
    }
    let value_res: Result<Option<&str>, rorm_db::error::Error> = row_ptr.get(index_conv.unwrap());
    if value_res.is_err() {
        match value_res.err().unwrap() {
            rorm_db::error::Error::SqlxError(err) => match err {
                sqlx::Error::ColumnIndexOutOfBounds { .. } => {
                    *error_ptr = Error::ColumnIndexOutOfBoundsError;
                }
                sqlx::Error::ColumnNotFound(_) => {
                    *error_ptr = Error::ColumnNotFoundError;
                }
                sqlx::Error::ColumnDecode { .. } => {
                    *error_ptr = Error::ColumnDecodeError;
                }
                _ => todo!("This error case should never occur"),
            },
            _ => todo!("This error case should never occur"),
        };
        return FFIOption::None;
    }

    return match value_res.unwrap() {
        None => FFIOption::None,
        Some(v) => match v.try_into() {
            Err(_) => {
                *error_ptr = Error::InvalidStringError;
                return FFIOption::None;
            }
            Ok(v) => FFIOption::Some(v),
        },
    };
}

/**
Tries to retrieve a nullable FFIDate from the given row pointer.

**Parameter**:
- `row_ptr`: Pointer to a row.
- `index`: Name of the column to retrieve from the row.
- `error_ptr`: Pointer to an [Error]. Gets only written to if an error occurs.

This function is called completely synchronously.
 */
#[no_mangle]
pub extern "C" fn rorm_row_get_null_date(
    row_ptr: &Row,
    index: FFIString<'_>,
    error_ptr: &mut Error,
) -> FFIOption<FFIDate> {
    get_data_from_row!(
        Option<chrono::NaiveDate>,
        FFIOption::None,
        row_ptr,
        index,
        error_ptr
    );
}

/**
Tries to retrieve a nullable FFITime from the given row pointer.

**Parameter**:
- `row_ptr`: Pointer to a row.
- `index`: Name of the column to retrieve from the row.
- `error_ptr`: Pointer to an [Error]. Gets only written to if an error occurs.

This function is called completely synchronously.
 */
#[no_mangle]
pub extern "C" fn rorm_row_get_null_time(
    row_ptr: &Row,
    index: FFIString<'_>,
    error_ptr: &mut Error,
) -> FFIOption<FFITime> {
    get_data_from_row!(
        Option<chrono::NaiveTime>,
        FFIOption::None,
        row_ptr,
        index,
        error_ptr
    );
}

/**
Tries to retrieve a nullable FFIDate from the given row pointer.

**Parameter**:
- `row_ptr`: Pointer to a row.
- `index`: Name of the column to retrieve from the row.
- `error_ptr`: Pointer to an [Error]. Gets only written to if an error occurs.

This function is called completely synchronously.
 */
#[no_mangle]
pub extern "C" fn rorm_row_get_null_datetime(
    row_ptr: &Row,
    index: FFIString<'_>,
    error_ptr: &mut Error,
) -> FFIOption<FFIDateTime> {
    get_data_from_row!(
        Option<chrono::NaiveDateTime>,
        FFIOption::None,
        row_ptr,
        index,
        error_ptr
    );
}
