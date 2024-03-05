//! Trait and some implementations used in [`MaxStr`](super::MaxStr)

/// Implementation used by [`MaxStr`](super::MaxStr) to retrieve the wrapped string's length.
///
/// - [`NumBytes`] uses the number of bytes (this is what [`str::len`] does)
/// - [`NumChars`] uses the number of unicode code points
pub trait LenImpl {
    /// Returns the string's length.
    fn len(&self, string: &str) -> usize;
}

/// [`LenImpl`] which uses the number of bytes (this is what [`str::len`] does)
#[derive(Copy, Clone, Debug, Default)]
pub struct NumBytes;

impl LenImpl for NumBytes {
    fn len(&self, string: &str) -> usize {
        string.as_bytes().len()
    }
}

/// [`LenImpl`] which uses the number of unicode code points
#[derive(Copy, Clone, Debug, Default)]
pub struct NumChars;

impl LenImpl for NumChars {
    fn len(&self, string: &str) -> usize {
        string.chars().count()
    }
}

impl<T: LenImpl> LenImpl for &T {
    fn len(&self, string: &str) -> usize {
        T::len(self, string)
    }
}
