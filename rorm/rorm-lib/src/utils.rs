use std::slice::from_raw_parts;
use std::string::FromUtf8Error;

/**
Representation of a string.
*/
#[repr(C)]
pub struct FFIString {
    content: *const u8,
    size: usize,
}

impl TryFrom<FFIString> for String {
    type Error = FromUtf8Error;

    fn try_from(value: FFIString) -> Result<Self, Self::Error> {
        unsafe {
            String::from_utf8(from_raw_parts(value.content, value.size).into())
        }
    }
}

impl From<String> for FFIString {
    fn from(s: String) -> Self {
        Self {
            content: s.as_ptr(),
            size: s.len(),
        }
    }
}