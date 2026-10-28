//! Utilities for implementing multi-column fields

/// Constructs arrays by concatenating others
pub struct ArrayBuilder<T, const N: usize> {
    array: [T; N],
    index: usize,
}
impl<T, const N: usize> ArrayBuilder<T, N> {
    /// Constructs a new `ArrayBuilder`
    pub const fn new(array: [T; N]) -> Self {
        Self { array, index: 0 }
    }

    /// Extends `self` by another array
    pub fn extend<const M: usize>(&mut self, other: [T; M]) {
        if M > N - self.index {
            panic!("Called ArrayBuild::extend with to many items (Array already contains {} / {N} elements, can't add {M} more)", self.index);
        }
        for item in other {
            self.array[self.index] = item;
            self.index += 1;
        }
    }

    /// Returns the final array
    pub fn finish(self) -> [T; N] {
        if self.index != N {
            panic!("Called ArrayBuild::finish but the array is not finished yet ({} / {N} items are initialized)", self.index);
        }
        self.array
    }

    /// Extends `self` by another array
    pub const fn extend_const<const M: usize>(&mut self, other: [T; M])
    where
        T: Copy,
    {
        if M > N - self.index {
            panic!("Called ArrayBuild::extend_const with to many items");
        }
        let mut other = other.as_slice();
        while let [item, remaining @ ..] = other {
            other = remaining;
            self.array[self.index] = *item;
            self.index += 1;
        }
    }

    /// Returns the final array
    pub const fn finish_const(self) -> [T; N]
    where
        T: Copy,
    {
        if self.index != N {
            panic!("Called ArrayBuild::finish_const but the array is not finished yet");
        }
        self.array
    }
}
