//! [`Decoder`] trait and some basic implementations

use std::marker::PhantomData;

use rorm_db::row::{DecodeOwned, RowError};
use rorm_db::Row;

/// Something which decodes a [value](Self::Result) from a [`&Row`](rorm_db::Row)
///
/// It is basically a closure `Fn(&Row) -> Result<Self::Result, Error>`.
/// Sadly we need to support decoding via indexes so this trait actually has two method.
/// One for decoding [`by_name`](Self::by_name) and another one for decoding [`by_index`](Self::by_index).
///
/// This trait does not manage
/// a) how the decoder is constructed
/// and b) that the row contains the columns which the decoder will access
///
/// These concerns are delegated to further sub-traits, namely:
/// - [`Selector`](super::selector::Selector) which constructs a [`Decoder`] and configures the [`QueryContext`](crate::internal::query_context::QueryContext) appropriately
/// - [`FieldDecoder`](crate::internal::field::decoder::FieldDecoder) which decodes and is associated to single fields through [`FieldType::Decoder`](crate::fields::traits::FieldType::Decoder)
pub trait Decoder {
    /// The value decoded from a row
    type Result;

    /// Decode a value from a row using select aliases to access the columns
    fn by_name<'index>(&'index self, row: &'_ Row) -> Result<Self::Result, RowError<'index>>;

    /// Decode a value from a row using indexes to access the columns
    fn by_index<'index>(&'index self, row: &'_ Row) -> Result<Self::Result, RowError<'index>>;
}

impl<D: Decoder> Decoder for &'_ D {
    type Result = D::Result;

    fn by_name<'index>(&'index self, row: &'_ Row) -> Result<Self::Result, RowError<'index>> {
        D::by_name(self, row)
    }

    fn by_index<'index>(&'index self, row: &'_ Row) -> Result<Self::Result, RowError<'index>> {
        D::by_index(self, row)
    }
}

/// A [`Decoder`] which directly decodes a [`T: DecodedOwned`](DecodeOwned)
pub struct DirectDecoder<T> {
    pub(crate) result: PhantomData<T>,
    pub(crate) column: String,
    pub(crate) index: usize,
}
impl<T> Decoder for DirectDecoder<T>
where
    T: DecodeOwned,
{
    type Result = T;

    fn by_name<'index>(&'index self, row: &'_ Row) -> Result<Self::Result, RowError<'index>> {
        row.get(self.column.as_str())
    }

    fn by_index<'index>(&'index self, row: &'_ Row) -> Result<Self::Result, RowError<'index>> {
        row.get(self.index)
    }
}

/// A [`Decoder`] which "decodes" a value by using the [`Default`] trait
///
/// This is a "noop" which doesn't touch the [`&Row`](rorm_db::Row) at all
pub struct NoopDecoder<T>(pub(crate) PhantomData<T>);
impl<T> Decoder for NoopDecoder<T>
where
    T: Default,
{
    type Result = T;

    fn by_name<'index>(&'index self, _row: &'_ Row) -> Result<Self::Result, RowError<'index>> {
        Ok(Default::default())
    }

    fn by_index<'index>(&'index self, _row: &'_ Row) -> Result<Self::Result, RowError<'index>> {
        Ok(Default::default())
    }
}

macro_rules! decoder {
    ($($index:tt : $S:ident,)+) => {
        impl<$($S: Decoder),+> Decoder for ($($S,)+) {
            type Result = ($(
                $S::Result,
            )+);

            fn by_name<'index>(&'index self, row: &'_ Row) -> Result<Self::Result, RowError<'index>> {
                Ok(($(
                    self.$index.by_name(row)?,
                )+))
            }

            fn by_index<'index>(&'index self, row: &'_ Row) -> Result<Self::Result, RowError<'index>> {
                Ok(($(
                    self.$index.by_index(row)?,
                )+))
            }
        }
    };
}
rorm_macro::impl_tuple!(decoder, 1..33);

/// Extension trait for [`Decoder`]
///
/// It provides combinators to tweak a decoder's behaviour.
///
/// This is an extension trait instead of part of the base trait,
/// because I'm not sure yet, if, how and by whom those combinators would be used.
pub trait DecoderExt: Decoder + Sized {
    /// Borrows the decoder
    ///
    /// This method is an alternative to taking a reference
    /// which might look awkward in a builder expression.
    fn by_ref(&self) -> &Self {
        self
    }

    /// Construct a decoder which applies a function to the result of `Self`.
    fn map<F, U>(self, f: F) -> Map<Self, F>
    where
        F: Fn(Self::Result) -> U,
    {
        Map {
            decoder: self,
            function: f,
        }
    }

    /// Construct a decoder which handles a `RowError::UnexpectedNull` by producing `None`
    fn optional(self) -> Optional<Self> {
        Optional { decoder: self }
    }

    // TODO: Where should RowError get its lifetime from?
    //
    // fn and_then<F, T>(self, f: F) -> AndThen<Self, F>
    // where
    //     F: Fn(Self::Result) -> Result<T, RowError<'static>>,
    // {
    //     AndThen {
    //         decoder: self,
    //         function: f,
    //     }
    // }
}

/// [`Decoder`] returned by [`DecoderExt::map`]
pub struct Map<D, F> {
    decoder: D,
    function: F,
}
impl<D, F, T> Decoder for Map<D, F>
where
    D: Decoder,
    F: Fn(D::Result) -> T,
{
    type Result = T;

    fn by_name<'index>(&'index self, row: &'_ Row) -> Result<Self::Result, RowError<'index>> {
        self.decoder.by_name(row).map(&self.function)
    }

    fn by_index<'index>(&'index self, row: &'_ Row) -> Result<Self::Result, RowError<'index>> {
        self.decoder.by_index(row).map(&self.function)
    }
}

/// [`Decoder`] returned by [`DecoderExt::optional`]
pub struct Optional<D> {
    decoder: D,
}
impl<D> Decoder for Optional<D>
where
    D: Decoder,
{
    type Result = Option<D::Result>;

    fn by_name<'index>(&'index self, row: &'_ Row) -> Result<Self::Result, RowError<'index>> {
        match self.decoder.by_name(row) {
            Ok(result) => Ok(Some(result)),
            Err(RowError::UnexpectedNull { .. }) => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn by_index<'index>(&'index self, row: &'_ Row) -> Result<Self::Result, RowError<'index>> {
        match self.decoder.by_index(row) {
            Ok(result) => Ok(Some(result)),
            Err(RowError::UnexpectedNull { .. }) => Ok(None),
            Err(error) => Err(error),
        }
    }
}

// /// [`Decoder`] returned by [`DecoderExt::and_then`]
// pub struct AndThen<D, F> {
//     decoder: D,
//     function: F,
// }
//
// impl<D, F, T> Decoder for AndThen<D, F>
// where
//     D: Decoder,
//     F: Fn(D::Result) -> Result<T, RowError<'static>>,
// {
//     type Result = T;
//
//     TODO: RowError requires a single index, what to do when `D` decodes more than one column?
//
//     fn by_name<'index>(&'index self, row: &'_ Row) -> Result<Self::Result, RowError<'index>> {
//         self.decoder.by_name(row).and_then(&self.function)
//     }
//
//     fn by_index<'index>(&'index self, row: &'_ Row) -> Result<Self::Result, RowError<'index>> {
//         self.decoder.by_name(row).and_then(&self.function)
//     }
// }
