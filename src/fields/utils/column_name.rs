//! A column's name

use std::fmt;
use std::ops::Deref;

/// A column's name
#[derive(Copy, Clone)]
pub struct ColumnName(ConstString<63>);
impl ColumnName {
    /// Constructs an empty `ColumnName`
    ///
    /// This function should only be used to get a placeholder value.
    pub const fn placeholder() -> Self {
        Self(ConstString::new())
    }

    /// Constructs a new `ColumnName`
    ///
    /// # Panics
    /// if the resulting column name would be larger than 63.
    pub const fn new(name: &str) -> Self {
        let mut this = ConstString::<63>::new();
        if this.try_push_str(name).is_err() {
            panic!("Column names can't be larger than 63 bytes. Please choose a shorter name or consider using #[rorm(rename = \"...\")].");
        }
        Self(this)
    }

    /// Extracts a string slice containing the entire `ColumnName`.
    pub const fn as_str<'a>(&'a self) -> &'a str {
        self.0.as_str()
    }

    /// Extracts a byte slice containing the entire `ColumnName`.
    pub const fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Joins the name of a sub filed onto `self`
    ///
    /// # Panics
    /// if the resulting column name would be larger than 63.
    pub const fn join(mut self, sub_field: &str) -> Self {
        let r1 = self.0.try_push_str("_");
        let r2 = self.0.try_push_str(sub_field);
        if r1.is_err() || r2.is_err() {
            panic!("Column names can't be larger than 63 bytes. Please choose a shorter name or consider using #[rorm(rename = \"...\")].");
        }
        self
    }
}

impl Deref for ColumnName {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl fmt::Debug for ColumnName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for ColumnName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

/// A UTF-8–encoded, growable string for const expressions.
///
/// ## Required invariant
/// Every instance, which is observable from the public API,
/// - must have valid utf8 in its bytes.
/// - must have a valid utf8 boundary at len.
#[derive(Copy, Clone)]
struct ConstString<const MAX_LEN: usize> {
    len: u8,
    bytes: [u8; MAX_LEN],
}

impl<const MAX_LEN: usize> ConstString<MAX_LEN> {
    /// Creates a new empty `ConstString`.
    pub const fn new() -> Self {
        if MAX_LEN > u8::MAX as usize {
            panic!("ConstString only supports a MAX_LENGTH of at most 255");
        }
        Self {
            len: 0,
            bytes: [0; MAX_LEN],
        }
    }

    /// Returns the string's length
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    /// Extracts a string slice containing the entire `ConstString`.
    pub const fn as_str<'a>(&'a self) -> &'a str {
        let bytes = self.as_bytes();

        // SAFETY:
        // `push_str` only adds a complete `str` or aborts.
        // Therefore whenever you've got an instance of `ConstString`, its bytes must form valid utf8
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }

    /// Extracts a byte slice containing the entire `ConstString`.
    pub const fn as_bytes<'a>(&'a self) -> &'a [u8] {
        let mut len = self.len();
        if len > self.bytes.len() {
            len = self.bytes.len();
        }

        // SAFETY: - `&self.bytes` is a valid reference
        //         - `len` is not larger than `self.bytes.len()`
        //         - the lifetime `'a` is bound to the input and output
        unsafe { std::slice::from_raw_parts::<'a, u8>(self.bytes.as_ptr(), len) }
    }

    /// Appends a given string slice onto the end of this `ConstString`,
    ///
    /// # Errors
    /// if the resulting string would be larger than `MAX_LEN`.
    pub const fn try_push_str(&mut self, string: &str) -> Result<(), ()> {
        let mut bytes = string.as_bytes();
        while let [byte, remaining @ ..] = bytes {
            bytes = remaining;

            if self.len() == MAX_LEN {
                return Err(());
            }

            self.bytes[self.len()] = *byte;
            self.len += 1;
        }
        Ok(())
    }
}
