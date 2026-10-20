//! [`FieldDecoder`] trait, some simple impls and a macro to create new ones

/// Syntax example
/// ```text
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

            fn decode<'index>(&'index self, row: &'_ $crate::Row) -> Result<Self::Result, $crate::db::row::RowError<'index>> {
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
                let index = ctx.select_field::<I::Field, I::Path>();
                Self {
                    index,
                    $(generics: ::std::marker::PhantomData::<($($generic,)+)>)?
                }
            }
        }
    };
}
