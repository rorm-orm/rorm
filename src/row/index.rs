/// The index used by [`Row::get`](crate::Row::get)
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum RowIndex<'i> {
    /// Index using the columns' positions
    Position(usize),
    /// Index using the columns' names
    Name(&'i str),
}

impl RowIndex<'_> {
    /// Converts the index into its owned version
    pub fn into_owned(self) -> OwnedRowIndex {
        match self {
            RowIndex::Position(index) => index.into(),
            RowIndex::Name(index) => index.into(),
        }
    }
}

/// Owned version of the [`RowIndex`]
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum OwnedRowIndex {
    /// Index using the columns' positions
    Position(usize),
    /// Index using the columns' names
    Name(Box<str>),
}

impl OwnedRowIndex {
    /// Gets index's borrowed version
    pub fn as_borrowed(&self) -> RowIndex<'_> {
        match self {
            OwnedRowIndex::Position(index) => (*index).into(),
            OwnedRowIndex::Name(index) => index.as_ref().into(),
        }
    }
}

impl From<usize> for RowIndex<'_> {
    fn from(value: usize) -> Self {
        Self::Position(value)
    }
}

impl<'i> From<&'i str> for RowIndex<'i> {
    fn from(value: &'i str) -> Self {
        Self::Name(value)
    }
}

impl From<usize> for OwnedRowIndex {
    fn from(value: usize) -> Self {
        Self::Position(value)
    }
}

impl From<&str> for OwnedRowIndex {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

impl From<String> for OwnedRowIndex {
    fn from(value: String) -> Self {
        Self::Name(value.into_boxed_str())
    }
}
