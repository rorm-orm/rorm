use std::marker::PhantomData;
use std::ptr;
use std::slice::from_raw_parts;
use std::str::{from_utf8, Utf8Error};

use futures::stream::BoxStream;

/**
Representation of a string.
*/
#[repr(C)]
#[derive(Copy, Clone)]
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
///
pub(crate) type VoidPtr = usize;

/// This type alias purely exists only for cbindgen.
pub(crate) type StreamPtr<'a> = BoxStream<'a, Result<rorm_db::row::Row, rorm_db::error::Error>>;

/// Security:
/// Create empty Box, to satisfy callback signature
pub(crate) fn null_ptr<T>() -> Box<T> {
    unsafe { Box::from_raw(ptr::null_mut()) }
}
