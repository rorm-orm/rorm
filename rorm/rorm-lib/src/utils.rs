use std::marker::PhantomData;
use std::slice::from_raw_parts;
use std::str::{from_utf8, Utf8Error};

use chrono::{Datelike, Timelike};
use futures::stream::BoxStream;
use rorm_db::select::LimitClause;
use rorm_db::Row;

use crate::representations::FFILimitClause;
use crate::Error;

/**
Representation of a [chrono::NaiveDate]
*/
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct FFIDate {
    pub(crate) year: i32,
    pub(crate) month: u32,
    pub(crate) day: u32,
}

impl From<chrono::NaiveDate> for FFIDate {
    fn from(value: chrono::NaiveDate) -> Self {
        Self {
            year: value.year(),
            month: value.month(),
            day: value.day(),
        }
    }
}

impl<'a> TryFrom<&'a FFIDate> for chrono::NaiveDate {
    type Error = Error<'a>;

    fn try_from(value: &'a FFIDate) -> Result<Self, Self::Error> {
        match chrono::NaiveDate::from_ymd_opt(value.year, value.month, value.day) {
            None => Err(Error::InvalidDateError),
            Some(v) => Ok(v),
        }
    }
}

/**
Representation of a [chrono::NaiveTime].
*/
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct FFITime {
    pub(crate) hour: u32,
    pub(crate) min: u32,
    pub(crate) sec: u32,
}

impl From<chrono::NaiveTime> for FFITime {
    fn from(value: chrono::NaiveTime) -> Self {
        Self {
            hour: value.hour(),
            min: value.minute(),
            sec: value.second(),
        }
    }
}

impl<'a> TryFrom<&'a FFITime> for chrono::NaiveTime {
    type Error = Error<'a>;

    fn try_from(value: &'a FFITime) -> Result<Self, Self::Error> {
        match chrono::NaiveTime::from_hms_opt(value.hour, value.min, value.sec) {
            None => Err(Error::InvalidTimeError),
            Some(v) => Ok(v),
        }
    }
}

/**
Representation of a [chrono::DateTime].
*/
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct FFIDateTime {
    pub(crate) year: i32,
    pub(crate) month: u32,
    pub(crate) day: u32,
    pub(crate) hour: u32,
    pub(crate) min: u32,
    pub(crate) sec: u32,
}

impl From<chrono::NaiveDateTime> for FFIDateTime {
    fn from(value: chrono::NaiveDateTime) -> Self {
        Self {
            year: value.year(),
            month: value.month(),
            day: value.day(),
            hour: value.hour(),
            min: value.minute(),
            sec: value.second(),
        }
    }
}

impl<'a> TryFrom<&'a FFIDateTime> for chrono::NaiveDateTime {
    type Error = Error<'a>;

    fn try_from(value: &'a FFIDateTime) -> Result<Self, Self::Error> {
        let d = chrono::NaiveDate::from_ymd_opt(value.year, value.month, value.day);
        if d.is_none() {
            return Err(Error::InvalidDateTimeError);
        }
        let dt = d.unwrap().and_hms_opt(value.hour, value.min, value.sec);
        if dt.is_none() {
            return Err(Error::InvalidDateTimeError);
        }
        Ok(dt.unwrap())
    }
}

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

impl<'a, T: 'a> FFISlice<'a, T> {
    pub(crate) fn empty() -> FFISlice<'a, T> {
        let f: &[T] = &[];
        FFISlice::from(f)
    }
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
pub(crate) type Stream<'a> = BoxStream<'a, Result<Row, rorm_db::error::Error>>;

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

macro_rules! ffi_opt_impl {
    ($from:ty, $to:ty) => {
        impl From<Option<$from>> for FFIOption<$to> {
            fn from(value: Option<$from>) -> Self {
                match value {
                    None => FFIOption::None,
                    Some(v) => FFIOption::Some(v.into()),
                }
            }
        }
    };
}

macro_rules! opt_ffi_impl {
    ($from:ty, $to:ty) => {
        impl From<FFIOption<$from>> for Option<$to> {
            fn from(value: FFIOption<$from>) -> Self {
                match value {
                    FFIOption::None => None,
                    FFIOption::Some(v) => Some(v.into()),
                }
            }
        }
    };
}

ffi_opt_impl!(chrono::NaiveTime, FFITime);
ffi_opt_impl!(chrono::NaiveDate, FFIDate);
ffi_opt_impl!(chrono::NaiveDateTime, FFIDateTime);

opt_ffi_impl!(FFILimitClause, LimitClause);

impl<T> From<Option<T>> for FFIOption<T> {
    fn from(option: Option<T>) -> Self {
        match option {
            None => FFIOption::None,
            Some(v) => FFIOption::Some(v),
        }
    }
}

impl<T> From<FFIOption<T>> for Option<T> {
    fn from(option: FFIOption<T>) -> Self {
        match option {
            FFIOption::None => None,
            FFIOption::Some(v) => Some(v),
        }
    }
}

impl<T> From<&FFIOption<T>> for Option<T>
where
    T: Clone,
{
    fn from(v: &FFIOption<T>) -> Self {
        match v {
            FFIOption::None => None,
            FFIOption::Some(v) => Some(v.clone()),
        }
    }
}

/**
This macro is used to simplify pushing futures to the runtime.

**Parameter**:
- `$fut`: Future to push to the runtime.
- `$cb_missing_rt`: Callback to execute if the runtime is missing.
- `$cb_runtime_error`: Function to execute if the runtime could not be locked.
Takes String as parameter.
*/
#[macro_export]
macro_rules! spawn_fut {
    ($fut:expr, $cb_missing_rt:stmt, $cb_runtime_error:expr) => {{
        match RUNTIME.lock() {
            Ok(guard) => match guard.as_ref() {
                Some(rt) => {
                    rt.spawn($fut);
                }
                None => unsafe { $cb_missing_rt },
            },
            Err(err) => {
                let ffi_err = err.to_string();
                $cb_runtime_error(ffi_err);
            }
        }
    }};
}

/**
This function is used to simplify the retrieval of cells from a row.

**Parameter**:
- `D`: The type to decode from db.
- `R`: The type to return.
- `default`: The default value to return in case of an error.
- `row`: Pointer to a row.
- `index`: Name of the column to retrieve the value from.
- `error`: Pointer to write errors to.
 */
pub(crate) fn get_data_from_row<
    'r,
    D: sqlx::Decode<'r, sqlx::Any> + sqlx::Type<sqlx::Any>,
    R: From<D>,
>(
    default: D,
    row: &'r Row,
    index: FFIString<'_>,
    error: &mut Error,
) -> R {
    let index = match index.try_into() {
        Ok(index) => index,
        Err(_) => {
            *error = Error::InvalidStringError;
            return default.into();
        }
    };

    match row.get::<D, &str>(index) {
        Ok(value) => value.into(),
        Err(err) => {
            match err {
                rorm_db::error::Error::SqlxError(err) => match err {
                    sqlx::Error::ColumnIndexOutOfBounds { .. } => {
                        *error = Error::ColumnIndexOutOfBoundsError;
                    }
                    sqlx::Error::ColumnNotFound(_) => {
                        *error = Error::ColumnNotFoundError;
                    }
                    sqlx::Error::ColumnDecode { .. } => {
                        *error = Error::ColumnDecodeError;
                    }
                    _ => unreachable!("This error case should never occur"),
                },
                _ => unreachable!("This error case should never occur"),
            };
            default.into()
        }
    }
}
