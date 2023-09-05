//! [`FieldDecoder`] trait, some simple impls and a macro to create new ones

use std::marker::PhantomData;

use rorm_db::row::DecodeOwned;

use crate::crud::decoder::{Decoder, DirectDecoder, NoopDecoder};
use crate::internal::field::{FieldProxy, RawField};
use crate::internal::query_context::QueryContext;
use crate::internal::relation_path::Path;

/// [`Decoder`] for a single field's [`RawField::Type`](RawField::Type)
pub trait FieldDecoder: Decoder {
    /// Construct decoder for a specific field and path
    fn new<F, P>(ctx: &mut QueryContext, _: FieldProxy<F, P>) -> Self
    where
        F: RawField<Type = Self::Result>,
        P: Path;
}

impl<T> FieldDecoder for DirectDecoder<T>
where
    T: DecodeOwned,
{
    fn new<F, P>(ctx: &mut QueryContext, _: FieldProxy<F, P>) -> Self
    where
        F: RawField<Type = Self::Result>,
        P: Path,
    {
        let (index, column) = ctx.select_field::<F, P>();
        Self {
            result: PhantomData,
            column,
            index,
        }
    }
}

impl<T> FieldDecoder for NoopDecoder<T>
where
    T: Default,
{
    fn new<F, P>(_: &mut QueryContext, _: FieldProxy<F, P>) -> Self
    where
        F: RawField<Type = Self::Result>,
        P: Path,
    {
        Self(PhantomData)
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! new_converting_decoder {
    ($(#[$attr:meta])* $vis:vis $decoder:ident$(<$($generic:ident$(: $bound1:ident $(+ $boundN:ident)*)?),+>)?, |$convert_arg:ident: $primitive:ty| -> $result:ty $convert_block:block) => {
        $(#[$attr])*
        $vis struct $decoder$(<$($generic),+>)? {
            column: String,
            index: usize,
            $(generics: ::std::marker::PhantomData<($($generic,)+)>)?
        }
        impl$(<$($generic),*>)? $crate::crud::decoder::Decoder for $decoder$(<$($generic),+>)?
        where
            $($($(
            $generic: $bound1,
            $($generic: $boundN,)*
            )?)+)?
        {
            type Result = $result;

            fn by_name(&self, row: &$crate::Row) -> Result<Self::Result, $crate::Error> {
                let $convert_arg: $primitive = row.get(self.column.as_str())?;
                $convert_block
            }

            fn by_index(&self, row: &$crate::Row) -> Result<Self::Result, $crate::Error> {
                let $convert_arg: $primitive = row.get(self.index)?;
                $convert_block
            }
        }
        impl$(<$($generic),*>)? $crate::internal::field::decoder::FieldDecoder for $decoder$(<$($generic),+>)?
        where
            $($($(
            $generic: $bound1,
            $($generic: $boundN,)*
            )?)+)?
        {
            fn new<F, P>(ctx: &mut $crate::internal::query_context::QueryContext, _: $crate::internal::field::FieldProxy<F, P>) -> Self
            where
                F: $crate::internal::field::RawField<Type = $result>,
                P: $crate::internal::relation_path::Path
            {
                let (index, column) = ctx.select_field::<F, P>();
                Self {
                    column,
                    index,
                    $(generics: ::std::marker::PhantomData::<($($generic,)+)>)?
                }
            }
        }
    };
}
