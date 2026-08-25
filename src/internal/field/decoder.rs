//! [`FieldDecoder`] trait, some simple impls and a macro to create new ones

use std::marker::PhantomData;

use rorm_db::row::DecodeOwned;

use crate::crud::decoder::{Decoder, DirectDecoder, NoopDecoder};
use crate::fields::proxy::FieldProxyImpl;
use crate::internal::field::{Field, FieldProxy};
use crate::internal::query_context::QueryContext;

/// [`Decoder`] for a single field's [`Field::Type`]
pub trait FieldDecoder: Decoder {
    /// Construct decoder for a specific field and path
    fn new<I>(ctx: &mut QueryContext, _: FieldProxy<I>) -> Self
    where
        I: FieldProxyImpl<Field: Field<Type = Self::Result>>;
}

impl<T> FieldDecoder for DirectDecoder<T>
where
    T: DecodeOwned,
{
    fn new<I>(ctx: &mut QueryContext, _: FieldProxy<I>) -> Self
    where
        I: FieldProxyImpl<Field: Field<Type = Self::Result>>,
    {
        let (index, column) = ctx.select_field::<I::Field, I::Path>();
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
    fn new<I>(_: &mut QueryContext, _: FieldProxy<I>) -> Self
    where
        I: FieldProxyImpl<Field: Field<Type = Self::Result>>,
    {
        Self(PhantomData)
    }
}

/// Syntax example
/// ```
/// new_converting_decoder!(
///     /// My custom decoder
///     pub MyCustomDecoder,
///     |x: i64| -> MyCustomType {Ok(x)}
/// );
/// ```
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

            fn by_name<'index>(&'index self, row: &'_ $crate::Row) -> Result<Self::Result, $crate::db::row::RowError<'index>> {
                let $convert_arg: $primitive = row.get(self.column.as_str())?;
                let convert_result = $convert_block;
                convert_result.map_err(|error| $crate::db::row::RowError::Decode {
                    index: self.column.as_str().into(),
                    source: error.into(),
                })
            }

            fn by_index<'index>(&'index self, row: &'_ $crate::Row) -> Result<Self::Result, $crate::db::row::RowError<'index>> {
                let $convert_arg: $primitive = row.get(self.index)?;
                let convert_result = $convert_block;
                convert_result.map_err(|error| $crate::db::row::RowError::Decode {
                    index: self.index.into(),
                    source: error.into(),
                })
            }
        }
        impl$(<$($generic),*>)? $crate::internal::field::decoder::FieldDecoder for $decoder$(<$($generic),+>)?
        where
            $($($(
            $generic: $bound1,
            $($generic: $boundN,)*
            )?)+)?
        {
            fn new<I>(
                ctx: &mut $crate::internal::query_context::QueryContext,
                _: $crate::fields::proxy::FieldProxy<I>
            ) -> Self
            where
                I: $crate::fields::proxy::FieldProxyImpl<
                    Field: $crate::internal::field::Field<
                        Type = Self::Result
                    >,
                >,
            {
                let (index, column) = ctx.select_field::<I::Field, I::Path>();
                Self {
                    column,
                    index,
                    $(generics: ::std::marker::PhantomData::<($($generic,)+)>)?
                }
            }
        }
    };
}
