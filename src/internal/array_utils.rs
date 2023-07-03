//! Two traits which are workaround for the not yet ideal support for arrays in traits as associated types.

/// Implemented on any array exposing `Self::LEN` and `Self::Item`
pub trait Array: IntoIterator<Item = <Self as Array>::Item> {
    /// The array's length
    const LEN: usize;

    /// The array's item type
    type Item;
}
impl<T, const N: usize> Array for [T; N] {
    const LEN: usize = N;
    type Item = T;
}

/// Implemented on any array to restrict concrete size
pub trait IntoArray<const N: usize>: Array {
    /// "Convert" the generic into the actual array
    fn into_array(self) -> [<Self as Array>::Item; N];
}
impl<T, const N: usize> IntoArray<N> for [T; N] {
    fn into_array(self) -> [T; N] {
        self
    }
}
