use std::marker::PhantomData;
use std::ptr;
use std::slice::from_raw_parts;
use std::str::{from_utf8, Utf8Error};

use futures::stream::BoxStream;

/**
Representation of a string.
*/
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct FFIString<'a> {
    content: *const u8,
    size: usize,
    lifetime: PhantomData<&'a ()>,
}

impl<'a> TryFrom<FFIString<'a>> for &'a str {
    type Error = Utf8Error;

    fn try_from(value: FFIString) -> Result<Self, Self::Error> {
        from_utf8(unsafe { from_raw_parts(value.content, value.size) })
    }
}

impl<'a> TryFrom<&FFIString<'a>> for &'a str {
    type Error = Utf8Error;

    fn try_from(value: &FFIString<'a>) -> Result<Self, Self::Error> {
        from_utf8(unsafe { from_raw_parts(value.content, value.size) })
    }
}

impl<'a> From<&'a str> for FFIString<'a> {
    fn from(s: &'a str) -> Self {
        Self {
            content: s.as_ptr(),
            size: s.len(),
            lifetime: PhantomData,
        }
    }
}

/**
Representation of an FFI safe slice.
*/
#[repr(C)]
#[derive(Copy, Clone)]
pub struct FFISlice<'a, T> {
    content: *const T,
    size: usize,
    lifetime: PhantomData<&'a ()>,
}

impl<'a, T> From<FFISlice<'a, T>> for &'a [T] {
    fn from(s: FFISlice<'a, T>) -> Self {
        unsafe { from_raw_parts(s.content, s.size) }
    }
}

impl<'a, T> From<&FFISlice<'a, T>> for &'a [T] {
    fn from(s: &FFISlice<'a, T>) -> Self {
        unsafe { from_raw_parts(s.content, s.size) }
    }
}

impl<'a, T> From<&'a [T]> for FFISlice<'a, T> {
    fn from(s: &'a [T]) -> Self {
        Self {
            content: s.as_ptr(),
            size: s.len(),
            lifetime: PhantomData,
        }
    }
}

/// This type alias purely exists only for cbindgen.
/// It renames all VoidPtr to void* as rusts' implementation of *const ()
/// does not implement the Send trait.
pub(crate) type VoidPtr = usize;

/// This type alias purely exists only for cbindgen.
/// cbindgen:ignore
pub(crate) type Stream<'a> = BoxStream<'a, Result<rorm_db::row::Row, rorm_db::error::Error>>;

/// Security:
/// Create empty Box, to satisfy callback signature
pub(crate) fn null_ptr<T>() -> Box<T> {
    unsafe { Box::from_raw(ptr::null_mut()) }
}

/**
Helper type to wrap [Option] ffi safe.
*/
#[repr(C)]
pub enum FFIOption<T> {
    /// None value
    None,
    /// Some value
    Some(T),
}

impl<T> From<Option<T>> for FFIOption<T> {
    fn from(option: Option<T>) -> Self {
        match option {
            None => FFIOption::None,
            Some(v) => FFIOption::Some(v),
        }
    }
}

/**
This macro is used to simplify the retrieval of cells from a row.

**Parameter**:
- `$data_type`: The type to build the conversion for.
- `$default_value`: The default value to insert in case of an error.
- `$row_ptr`: The pointer to a row
- `$index`: Name of the column to retrieve the value from
- `$callback`: The callback to execute. Must be of the form fn(VoidPtr, $data_type, Error) -> ()
- `$context`: Pass through void pointer
*/
#[macro_export]
macro_rules! get_data_from_row {
    ($data_type:ty, $default_value:expr, $row_ptr:expr, $index:expr, $callback:expr, $context:expr) => {{
        let index_conv: Result<&str, Utf8Error> = $index.try_into();
        if index_conv.is_err() {
            $callback($context, $default_value, Error::InvalidStringError);
            return;
        }
        let value_res: Result<$data_type, rorm_db::error::Error> =
            $row_ptr.get(index_conv.unwrap());
        if value_res.is_err() {
            match value_res.err().unwrap() {
                rorm_db::error::Error::SqlxError(err) => match err {
                    sqlx::Error::ColumnIndexOutOfBounds { .. } => {
                        $callback($context, $default_value, Error::ColumnIndexOutOfBoundsError);
                    }
                    sqlx::Error::ColumnNotFound(_) => {
                        $callback($context, $default_value, Error::ColumnNotFoundError);
                    }
                    sqlx::Error::ColumnDecode { .. } => {
                        $callback($context, $default_value, Error::ColumnDecodeError);
                    }
                    _ => todo!("This error case should never occur"),
                },
                _ => todo!("This error case should never occur"),
            };
            return;
        }

        $callback($context, value_res.unwrap().into(), Error::NoError);
    }};
}
