//! [`FieldProxy`] and some utility functions which are used by rorm's various macros
#![allow(missing_docs)]

use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::Deref;

use rorm_db::sql::aggregation::SelectAggregator;

use crate::crud::selector::{AggregatedColumn, PathedSelector, Selector};
use crate::fields::traits::{Array, FieldColumns, FieldType};
use crate::fields::utils::column_name::ColumnName;
use crate::internal::field::Field;
use crate::internal::relation_path::{Path, PathField};
use crate::internal::ConstRef;
use crate::sealed;

#[macro_export]
macro_rules! declare_proxy_layer {
    (
        $(#[doc = $doc:literal])*
        $name:ident<$($T:ident,)* _>
    ) => {
        $(#[doc = $doc])*
        pub struct $name<
            $($T,)*
            __L = $crate::fields::proxy::LayerStackBase
        >(
            ::std::marker::PhantomData<
                ::std::mem::ManuallyDrop<($($T,)* __L, )>
            >
        );
        impl<$($T,)* __L> $crate::internal::ConstRef for $name<$($T,)* __L>
        where
            __L: 'static,
            $(
                $T: 'static,
            )*
        {
            const REF: &'static Self = &Self(::std::marker::PhantomData);
        }
        impl<$($T,)* __L> FieldProxyLayerStack for $name<$($T,)* __L>
        where
            __L:  FieldProxyLayerStack,
            $(
                $T: 'static,
            )*
        {
            type SetImpl<__I: FieldProxyImpl> = $name<$($T,)* __I>;

            type PopSelf = __L;

            type ReplaceBase<NewBase: FieldProxyLayerStack> =
                $name<$($T,)* <__L as FieldProxyLayerStack>::ReplaceBase<NewBase>>;
        }
        impl<$($T,)* __I> ::std::ops::Deref for $name<$($T,)* __I>
        where
            __I: FieldProxyImpl,
        {
            type Target = __I::DerefTarget;

            fn deref(&self) -> &Self::Target {
                $crate::internal::ConstRef::REF
            }
        }
    };
}

/// A stack of layers a field proxy can be constructed from
pub trait FieldProxyLayerStack: 'static {
    type SetImpl<I: FieldProxyImpl>: ConstRef;

    type PopSelf: FieldProxyLayerStack;

    /// Replaces the final type, a proxy (build from this layer) `deref`s to, with `E`
    type ReplaceBase<NewBase: FieldProxyLayerStack>: FieldProxyLayerStack;
}

pub trait FieldProxyLayerStackBase: 'static {
    type SetImpl<I: FieldProxyImpl>: ConstRef;
}
impl<T: FieldProxyLayerStackBase> FieldProxyLayerStack for T {
    type SetImpl<I: FieldProxyImpl> = <T as FieldProxyLayerStackBase>::SetImpl<I>;
    type PopSelf = T;
    type ReplaceBase<NewBase: FieldProxyLayerStack> = NewBase;
}

pub struct LayerStackBase(());
impl ConstRef for LayerStackBase {
    const REF: &'static Self = &Self(());
}
impl FieldProxyLayerStackBase for LayerStackBase {
    type SetImpl<I: FieldProxyImpl> = LayerStackBase;
}

declare_proxy_layer!(NumberProxy<_>);
impl<I: FieldProxyImpl, L> NumberProxy<(L, I)> {
    pub fn greater_than(&self, _value: ()) -> () {
        let condition = ();
        condition
    }
}

declare_proxy_layer!(StringProxy<_>);
impl<I: FieldProxyImpl> StringProxy<I> {
    pub fn contains(&self, _value: ()) -> () {
        let condition = ();
        condition
    }
}

declare_proxy_layer!(EqualsProxy<_>);
impl<I: FieldProxyImpl> EqualsProxy<I> {
    pub fn equals(&self, _value: ()) -> () {
        let condition = ();
        condition
    }
}

/// This unit struct acts as a proxy exposing a model's field (the field's declaration not its value)
/// as a value to pass around and call methods on.
///
/// It also constructs JOIN paths by following relations between models.
///
/// TODO: more docs
pub struct FieldProxy<T>(PhantomData<ManuallyDrop<T>>);

impl<I> FieldProxy<I>
where
    I: FieldProxyImpl<Field: Field<Type: FieldType<Columns = Array<1>>>>,
{
    /// Returns the count of the number of times that the column is not null.
    pub fn count(self) -> AggregatedColumn<I, i64> {
        AggregatedColumn {
            sql: SelectAggregator::Count,
            alias: "count",
            field: self,
            result: PhantomData,
        }
    }
}

impl<F, P, I> FieldProxy<I>
where
    F: Field + PathField<<F as Field>::Type>,
    P: Path<Current = <F::ParentField as Field>::Model>,
    I: FieldProxyImpl<Field = F, Path = P>,
{
    /// Query the model this field points to using `selector`
    pub fn query_as<S>(self, selector: S) -> PathedSelector<S, <I::Path as Path>::Step<I::Field>>
    where
        S: Selector<Model = <F::ChildField as Field>::Model>,
    {
        PathedSelector {
            selector,
            path: Default::default(),
        }
    }
}

impl<I: FieldProxyImpl> Deref for FieldProxy<I> {
    type Target = I::DerefTarget;

    fn deref(&self) -> &Self::Target {
        ConstRef::REF
    }
}

// SAFETY:
// struct contains no data
unsafe impl<T> Send for FieldProxy<T> {}
unsafe impl<T> Sync for FieldProxy<T> {}

impl<T> Clone for FieldProxy<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for FieldProxy<T> {}

/// Implementation detail of [`FieldProxy`], `FieldProxy`'s generic must implement this trait.
///
/// This trait is not relevant for the average rorm user.
pub trait FieldProxyImpl: 'static {
    sealed!(trait);

    /// Field which is proxied
    type Field: Field;

    /// Path the field is accessed through
    type Path: Path;

    /// Target type this field proxy should deref to
    ///
    /// (i.e. the `Deref::Target`)
    type DerefTarget: ConstRef;
}

impl<F, P, L> FieldProxyImpl for (F, P, L)
where
    F: Field,
    P: Path,
    L: FieldProxyLayerStack,
{
    sealed!(impl);

    type Field = F;
    type Path = P;
    type DerefTarget = L::SetImpl<(F, P, L::PopSelf)>;
}

/// Construct a new `FieldProxy`
///
/// *Not relevant for the average rorm user*
///
/// This function is used by the `#[derive(Model)]` macro to populate the Fields struct.
pub const fn new<I: 'static>() -> FieldProxy<I> {
    FieldProxy(PhantomData)
}

/// Get a [`Field`]'s `INDEX` from a `FieldProxy`
///
/// *Not relevant for the average rorm user*
///
/// This function is used by the [`get_field`](crate::get_field) and [`field`](crate::field) macros.
pub const fn index<I: FieldProxyImpl>(_: fn() -> FieldProxy<I>) -> usize {
    <I::Field as Field>::INDEX
}

/// Get the names of the columns which store the field
///
/// *Not relevant for the average rorm user*
///
/// This function is used by the `#[derive(Patch)]` macro to gather a list of all columns.
pub const fn columns<I: FieldProxyImpl>(
    _: fn() -> FieldProxy<I>,
) -> FieldColumns<<I::Field as Field>::Type, ColumnName> {
    <I::Field as Field>::EFFECTIVE_NAMES
}
