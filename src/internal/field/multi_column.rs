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
//! use rorm::fields::proxy;
//! use rorm::fields::proxy::{FieldProxy, FieldProxyImpl};
//! use rorm::fields::traits::{Array, FieldColumns, FieldType};
//! use rorm::fields::utils::column_name::ColumnName;
//! use rorm::fields::utils::const_fn::{ConstFn, Contains};
//! use rorm::internal::const_concat::ConstString;
//! use rorm::internal::field::decoder::FieldDecoder;
//! use rorm::internal::field::multi_column::{slice_for_check, ArrayBuilder};
//! use rorm::internal::field::{ContainerFieldType, Field};
//! use rorm::internal::hmr::annotations::Annotations;
//! use rorm::internal::hmr::Source;
//! use rorm::internal::query_context::QueryContext;
//! use rorm::internal::relation_path::Path;
//! use rorm::internal::ConstRef;
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
//! impl<F, P> ContainerFieldType<F, P> for MyMcf
//! where
//!     Self: FieldType,
//!     F: Field<Type = Self>,
//!     P: Path<Current = F::Model>,
//! {
//!     type Target = MyMcfFields<F, P>;
//! }
//!
//! pub struct MyMcfFields<F: Field<Type = MyMcf>, P: Path> {
//!     pub foo: FieldProxy<(MyMcf_foo_Field<F>, P)>,
//!     pub bar: FieldProxy<(MyMcf_bar_Field<F>, P)>,
//!     pub baz: FieldProxy<(MyMcf_baz_Field<F>, P)>,
//! }
//! impl<F: Field<Type = MyMcf>, P: Path> ConstRef for MyMcfFields<F, P> {
//!     const REF: &'static Self = &Self {
//!         foo: proxy::new(),
//!         bar: proxy::new(),
//!         baz: proxy::new(),
//!     };
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
//!         // TODO: Path should be tweaked
//!         MyMcfDecoder {
//!             foo: <<i32 as FieldType>::Decoder as FieldDecoder>::new(&mut *ctx, proxy::new::<(MyMcf_foo_Field<I::Field>, I::Path)>()),
//!             bar: <<String as FieldType>::Decoder as FieldDecoder>::new(&mut *ctx, proxy::new::<(MyMcf_bar_Field<I::Field>, I::Path)>()),
//!             baz: <<bool as FieldType>::Decoder as FieldDecoder>::new(&mut *ctx, proxy::new::<(MyMcf_baz_Field<I::Field>, I::Path)>()),
//!         }
//!     }
//! }
//!
//! const_fn! {
//!     pub fn get_MyMcf_names(#[raw] T: (ColumnName,)) -> FieldColumns<MyMcf, ColumnName> {
//!         let mut builder = ArrayBuilder::new([ColumnName::placeholder(); 3]);
//!         builder.extend_const(
//!             <<<i32 as FieldType>::GetNames as ConstFn<_, _>>::Body<(
//!                 <get_MyMcf_foo_name as ConstFn<_, _>>::Body<T>,
//!             )> as Contains<_>>::ITEM,
//!         );
//!         builder.extend_const(
//!             <<<String as FieldType>::GetNames as ConstFn<_, _>>::Body<(
//!                 <get_MyMcf_bar_name as ConstFn<_, _>>::Body<T>,
//!             )> as Contains<_>>::ITEM,
//!         );
//!         builder.extend_const(
//!             <<<bool as FieldType>::GetNames as ConstFn<_, _>>::Body<(
//!                 <get_MyMcf_baz_name as ConstFn<_, _>>::Body<T>,
//!             )> as Contains<_>>::ITEM,
//!         );
//!         builder.finish_const()
//!     }
//! }
//!
//! const_fn! {
//!     #[allow(non_snake_case)]
//!     pub fn get_MyMcf_foo_name(column_name: ColumnName) -> ColumnName {
//!         column_name.join("foo")
//!     }
//! }
//! const_fn! {
//!     #[allow(non_snake_case)]
//!     pub fn get_MyMcf_bar_name(column_name: ColumnName) -> ColumnName {
//!         column_name.join("bar")
//!     }
//! }
//! const_fn! {
//!     #[allow(non_snake_case)]
//!     pub fn get_MyMcf_baz_name(column_name: ColumnName) -> ColumnName {
//!         column_name.join("baz")
//!     }
//! }
//!
//! const_fn! {
//!     pub fn get_MyMcf_annotations(#[raw] T: (Annotations,)) -> FieldColumns<MyMcf, Annotations> {
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
//!     }
//! }
//!
//! const_fn! {
//!     pub fn check_MyMcf(#[raw] T: (Annotations, FieldColumns<MyMcf, Annotations>)) -> Result<(), ConstString<1024>> {
//!         'result: {
//!             let result = <<<i32 as FieldType>::Check as ConstFn<_, _>>::Body<
//!                 <slice_for_check<3, 1, 0> as ConstFn<_, _>>::Body<T>,
//!             > as Contains<_>>::ITEM;
//!             if matches!(result, Err(_)) {
//!                 break 'result result;
//!             }
//!
//!             let result = <<<String as FieldType>::Check as ConstFn<_, _>>::Body<
//!                 <slice_for_check<3, 1, 0> as ConstFn<_, _>>::Body<T>,
//!             > as Contains<_>>::ITEM;
//!             if matches!(result, Err(_)) {
//!                 break 'result result;
//!             }
//!
//!             let result = <<<bool as FieldType>::Check as ConstFn<_, _>>::Body<
//!                 <slice_for_check<3, 1, 0> as ConstFn<_, _>>::Body<T>,
//!             > as Contains<_>>::ITEM;
//!             if matches!(result, Err(_)) {
//!                 break 'result result;
//!             }
//!
//!             Ok(())
//!         }
//!     }
//! }
//!
//! pub struct MyMcf_foo_Field<F>(PhantomData<F>);
//! impl<F> Copy for MyMcf_foo_Field<F> {}
//! impl<F> Clone for MyMcf_foo_Field<F> {
//!     fn clone(&self) -> Self {
//!         *self
//!     }
//! }
//! impl<F> Field for MyMcf_foo_Field<F>
//! where
//!     F: Field<Type = MyMcf>
//! {
//!     type Type = i32;
//!     type Model = F::Model;
//!     const INDEX: usize = 0;
//!     const NAME: ColumnName = F::EFFECTIVE_NAMES[Self::INDEX];
//!     const EXPLICIT_ANNOTATIONS: Annotations = F::EFFECTIVE_ANNOTATIONS[Self::INDEX];
//!     const SOURCE: Source = F::SOURCE;
//!     fn new() -> Self {
//!         Self(PhantomData)
//!     }
//! }
//!
//! pub struct MyMcf_bar_Field<F>(PhantomData<F>);
//! impl<F> Copy for MyMcf_bar_Field<F> {}
//! impl<F> Clone for MyMcf_bar_Field<F> {
//!     fn clone(&self) -> Self {
//!         *self
//!     }
//! }
//! impl<F> Field for MyMcf_bar_Field<F>
//! where
//!     F: Field<Type = MyMcf>
//! {
//!     type Type = String;
//!     type Model = F::Model;
//!     const INDEX: usize = 1;
//!     const NAME: ColumnName = F::EFFECTIVE_NAMES[Self::INDEX];
//!     const EXPLICIT_ANNOTATIONS: Annotations = F::EFFECTIVE_ANNOTATIONS[Self::INDEX];
//!     const SOURCE: Source = F::SOURCE;
//!     fn new() -> Self {
//!         Self(PhantomData)
//!     }
//! }
//!
//! pub struct MyMcf_baz_Field<F>(PhantomData<F>);
//! impl<F> Copy for MyMcf_baz_Field<F> {}
//! impl<F> Clone for MyMcf_baz_Field<F> {
//!     fn clone(&self) -> Self {
//!         *self
//!     }
//! }
//! impl<F> Field for MyMcf_baz_Field<F>
//! where
//!     F: Field<Type = MyMcf>
//! {
//!     type Type = bool;
//!     type Model = F::Model;
//!     const INDEX: usize = 2;
//!     const NAME: ColumnName = F::EFFECTIVE_NAMES[Self::INDEX];
//!     const EXPLICIT_ANNOTATIONS: Annotations = F::EFFECTIVE_ANNOTATIONS[Self::INDEX];
//!     const SOURCE: Source = F::SOURCE;
//!     fn new() -> Self {
//!         Self(PhantomData)
//!     }
//! }
//! ```

use crate::const_fn;
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

const_fn! {
    /// Helper for a multi-column `FieldType`'s `Check` implementation.
    ///
    /// The multi-column type would have to "call" each subfield's check function.
    /// Each of which takes an array of annotations of some unique length `M`.
    /// The multi-column type only has an array of length `N` which is the sum of all `M`s.
    ///
    /// This function takes the field's array of length `N` and extracts a subarray of length `M`
    /// starting at position `I`.
    pub fn slice_for_check<const N: usize, const M: usize, const I: usize>(single: Annotations, items: [Annotations; N]) -> (Annotations, [Annotations; M]) {
        let mut array = [Annotations::empty(); M];

        let mut i = 0;
        while i < M {
            array[i] = items[i + I];

            i += 1;
        }

        (single, array)
    }
}
