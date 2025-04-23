//! Draft of and utilities for implementing multi-column fields
//!
//! Current draft:
//! ```
//! use std::array;
//! use std::marker::PhantomData;
//!
//! use rorm::conditions::Value;
//! use rorm::crud::decoder::Decoder;
//! use rorm::db::row::RowError;
//! use rorm::db::sql::value::NullType;
//! use rorm::fields::proxy::{FieldProxy, FieldProxyImpl};
//! use rorm::fields::traits::{Array, FieldColumns, FieldType};
//! use rorm::fields::utils::const_fn::{ConstFn, Contains};
//! use rorm::internal::const_concat::ConstString;
//! use rorm::internal::field::decoder::FieldDecoder;
//! use rorm::internal::field::mulit_column::{slice_for_check, ArrayBuilder};
//! use rorm::internal::field::Field;
//! use rorm::internal::hmr::annotations::Annotations;
//! use rorm::internal::query_context::QueryContext;
//! use rorm::{const_fn, Row};
//!
//! pub struct MyMcf {
//!     pub foo: i32,
//!     pub bar: String,
//!     pub baz: bool,
//! }
//!
//! impl FieldType for MyMcf {
//!     type Columns = Array<3>;
//!     const NULL: FieldColumns<Self, NullType> = {
//!         let mut builder = ArrayBuilder::new([NullType::Bool; 3]);
//!         builder.extend_const(i32::NULL);
//!         builder.extend_const(String::NULL);
//!         builder.extend_const(bool::NULL);
//!         builder.finish_const()
//!     };
//!
//!     fn into_values<'a>(self) -> FieldColumns<Self, Value<'a>> {
//!         let mut builder = ArrayBuilder::new(array::from_fn(|_| Value::Bool(false)));
//!         builder.extend(self.foo.into_values());
//!         builder.extend(self.bar.into_values());
//!         builder.extend(self.baz.into_values());
//!         builder.finish()
//!     }
//!
//!     fn as_values(&self) -> FieldColumns<Self, Value<'_>> {
//!         let mut builder = ArrayBuilder::new(array::from_fn(|_| Value::Bool(false)));
//!         builder.extend(self.foo.as_values());
//!         builder.extend(self.bar.as_values());
//!         builder.extend(self.baz.as_values());
//!         builder.finish()
//!     }
//!
//!     type Decoder = MyMcfDecoder;
//!     type GetNames = get_MyMcf_names;
//!     type GetAnnotations = get_MyMcf_annotations;
//!     type Check = check_MyMcf;
//! }
//!
//! pub struct MyMcfDecoder {
//!     pub foo: <i32 as FieldType>::Decoder,
//!     pub bar: <String as FieldType>::Decoder,
//!     pub baz: <bool as FieldType>::Decoder,
//! }
//!
//! impl Decoder for MyMcfDecoder {
//!     type Result = MyMcf;
//!
//!     fn by_name<'index>(&'index self, row: &'_ Row) -> Result<Self::Result, RowError<'index>> {
//!         Ok(MyMcf {
//!             foo: self.foo.by_name(row)?,
//!             bar: self.bar.by_name(row)?,
//!             baz: self.baz.by_name(row)?,
//!         })
//!     }
//!
//!     fn by_index<'index>(&'index self, row: &'_ Row) -> Result<Self::Result, RowError<'index>> {
//!         Ok(MyMcf {
//!             foo: self.foo.by_index(row)?,
//!             bar: self.bar.by_index(row)?,
//!             baz: self.baz.by_index(row)?,
//!         })
//!     }
//! }
//!
//! impl FieldDecoder for MyMcfDecoder {
//!     fn new<I>(ctx: &mut QueryContext, _: FieldProxy<I>) -> Self
//!     where
//!         I: FieldProxyImpl<Field: Field<Type= Self::Result>>,
//!     {
//!         todo!()
//!     }
//! }
//!
//! const_fn! {
//!     pub fn get_MyMcf_names(field_name: &'static str) -> FieldColumns<MyMcf, &'static str> {
//!         ["foo", "bar", "baz"]
//!     }
//! }
//!
//! pub struct get_MyMcf_annotations;
//! impl ConstFn<(Annotations,), FieldColumns<MyMcf, Annotations>> for get_MyMcf_annotations {
//!     type Body<T: Contains<(Annotations,)>> = get_MyMcf_annotations_Body<T>;
//! }
//! pub struct get_MyMcf_annotations_Body<T>(PhantomData<T>);
//! impl<T: Contains<(Annotations,)>> Contains<FieldColumns<MyMcf, Annotations>>
//! for get_MyMcf_annotations_Body<T>
//! {
//!     const ITEM: FieldColumns<MyMcf, Annotations> = {
//!         let mut builder = ArrayBuilder::new([Annotations::empty(); 3]);
//!         builder.extend_const(
//!             <<<i32 as FieldType>::GetAnnotations as ConstFn<_, _>>::Body<T> as Contains<_>>::ITEM,
//!         );
//!         builder.extend_const(
//!             <<<String as FieldType>::GetAnnotations as ConstFn<_, _>>::Body<T> as Contains<_>>::ITEM,
//!         );
//!         builder.extend_const(
//!             <<<bool as FieldType>::GetAnnotations as ConstFn<_, _>>::Body<T> as Contains<_>>::ITEM,
//!         );
//!         builder.finish_const()
//!     };
//! }
//!
//! pub struct check_MyMcf;
//! impl ConstFn<(Annotations, FieldColumns<MyMcf, Annotations>), Result<(), ConstString<1024>>>
//! for check_MyMcf
//! {
//!     type Body<T: Contains<(Annotations, FieldColumns<MyMcf, Annotations>)>> = check_MyMcf_Body<T>;
//! }
//! pub struct check_MyMcf_Body<T>(PhantomData<T>);
//! impl<T: Contains<(Annotations, FieldColumns<MyMcf, Annotations>)>>
//! Contains<Result<(), ConstString<1024>>> for check_MyMcf_Body<T>
//! {
//!     const ITEM: Result<(), ConstString<1024>> = 'result: {
//!         let result = <<<i32 as FieldType>::Check as ConstFn<_, _>>::Body<
//!             <slice_for_check<3, 1, 0> as ConstFn<_, _>>::Body<T>,
//!         > as Contains<_>>::ITEM;
//!         if matches!(result, Err(_)) {
//!             break 'result result;
//!         }
//!
//!         let result = <<<String as FieldType>::Check as ConstFn<_, _>>::Body<
//!             <slice_for_check<3, 1, 0> as ConstFn<_, _>>::Body<T>,
//!         > as Contains<_>>::ITEM;
//!         if matches!(result, Err(_)) {
//!             break 'result result;
//!         }
//!
//!         let result = <<<bool as FieldType>::Check as ConstFn<_, _>>::Body<
//!             <slice_for_check<3, 1, 0> as ConstFn<_, _>>::Body<T>,
//!         > as Contains<_>>::ITEM;
//!         if matches!(result, Err(_)) {
//!             break 'result result;
//!         }
//!
//!         Ok(())
//!     };
//! }
//! ```

use std::marker::PhantomData;

use crate::fields::utils::const_fn::{ConstFn, Contains};
use crate::internal::hmr::annotations::Annotations;

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
            panic!();
        }
        for item in other {
            self.array[self.index] = item;
            self.index += 1;
        }
    }

    /// Returns the final array
    pub fn finish(self) -> [T; N] {
        if self.index != N {
            panic!();
        }
        self.array
    }

    /// Extends `self` by another array
    pub const fn extend_const<const M: usize>(&mut self, other: [T; M])
    where
        T: Copy,
    {
        if M > N - self.index {
            panic!();
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
            panic!();
        }
        self.array
    }
}

#[allow(non_camel_case_types)]
pub struct slice_for_check<const N: usize, const M: usize, const I: usize>;
impl<const N: usize, const M: usize, const I: usize>
    ConstFn<(Annotations, [Annotations; N]), (Annotations, [Annotations; M])>
    for slice_for_check<N, M, I>
{
    type Body<T: Contains<(Annotations, [Annotations; N])>> = slice_for_check_Body<N, M, I, T>;
}
#[doc(hidden)]
#[allow(non_camel_case_types)]
pub struct slice_for_check_Body<const N: usize, const M: usize, const I: usize, T>(PhantomData<T>);
impl<
        const N: usize,
        const M: usize,
        const I: usize,
        T: Contains<(Annotations, [Annotations; N])>,
    > Contains<(Annotations, [Annotations; M])> for slice_for_check_Body<N, M, I, T>
{
    const ITEM: (Annotations, [Annotations; M]) = {
        let (single, items) = T::ITEM;
        let mut array = [Annotations::empty(); M];

        let mut i = 0;
        while i < M {
            array[i] = items[i + I];

            i += 1;
        }

        (single, array)
    };
}
