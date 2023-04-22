//! Trait for selecting stuff

use std::marker::PhantomData;

use rorm_db::database::ColumnSelector;
use rorm_db::{Error, Row};

use crate::aggregate::{AggregatedColumn, AggregationFunc};
use crate::internal::field::as_db_type::AsDbType;
use crate::internal::field::{AbstractField, AliasedField, Field, FieldProxy, RawField};
use crate::internal::query_context::QueryContext;
use crate::internal::relation_path::{Path, PathImpl, PathStep, ResolvedRelatedField};
use crate::model::Model;
use crate::Patch;

/// Anything which can be used to select a single item
pub trait Selectable {
    /// The type resulting from this select item
    type Result;

    /// Model from whose table to select from
    type Model: Model;

    /// Prepare the context to handle the select
    ///
    /// i.e. push
    /// - the actual [columns]() to select
    /// - potential joins
    fn prepare(&self, context: &mut QueryContext);

    /// Retrieve the result from a row
    fn decode(&self, row: &Row) -> Result<Self::Result, Error>;
}

impl<F, P> Selectable for FieldProxy<F, P>
where
    P: Path,
    F: AbstractField + AliasedField<P>,
{
    type Result = F::Type;

    type Model = P::Origin;

    fn prepare(&self, context: &mut QueryContext) {
        let columns = <F as AbstractField>::COLUMNS;
        let aliases = <F as AliasedField<P>>::COLUMNS;
        for (column, alias) in columns.iter().zip(aliases) {
            context.add_select(ColumnSelector {
                table_name: Some(P::ALIAS),
                column_name: column,
                select_alias: Some(alias),
                aggregation: None,
            });
        }
        P::add_to_context(context);
    }

    fn decode(&self, row: &Row) -> Result<Self::Result, Error> {
        F::get_by_alias(row)
    }
}

#[doc(hidden)]
pub struct PatchFieldSelector<T, P> {
    decode: fn(&Row) -> Result<T, Error>,
    columns: &'static [&'static str],
    aliases: &'static [&'static str],
    path: PhantomData<P>,
}
impl<T, P> PatchFieldSelector<T, P>
where
    P: Path,
{
    pub fn new<F>(_proxy: FieldProxy<F, F::Model>) -> Self
    where
        F: RawField<Type = T>,
        F: AbstractField,
        F: AliasedField<P>,
    {
        Self {
            decode: F::get_by_alias,
            columns: <F as AbstractField>::COLUMNS,
            aliases: <F as AliasedField<P>>::COLUMNS,
            path: PhantomData,
        }
    }
}
impl<T, P> Selectable for PatchFieldSelector<T, P>
where
    P: Path,
{
    type Result = T;

    type Model = P::Origin;

    fn prepare(&self, context: &mut QueryContext) {
        for (column, alias) in self.columns.iter().zip(self.aliases) {
            context.add_select(ColumnSelector {
                table_name: Some(P::ALIAS),
                column_name: column,
                select_alias: Some(alias),
                aggregation: None,
            });
        }
        P::add_to_context(context);
    }

    fn decode(&self, row: &Row) -> Result<Self::Result, Error> {
        (self.decode)(row)
    }
}

#[doc(hidden)]
impl<F, P> FieldProxy<F, P>
where
    F: RawField,
    P: Path,
    PathStep<F, P>: PathImpl<F::Type>,
{
    pub fn select_as<Ptch>(self) -> Ptch::Selector<PathStep<F, P>>
    where
        Ptch: Patch<Model = <ResolvedRelatedField<F, P> as RawField>::Model>,
    {
        Ptch::select()
    }
}

impl<A, F, P> Selectable for AggregatedColumn<A, F, P>
where
    A: AggregationFunc,
    F: Field,
    F::Type: AsDbType,
    P: Path,
{
    type Result = A::Result<<F::Type as AsDbType>::Primitive>;

    type Model = P::Origin;

    fn prepare(&self, context: &mut QueryContext) {
        context.add_select(ColumnSelector {
            table_name: Some(P::ALIAS),
            column_name: F::NAME,
            select_alias: Some(Self::SELECT_ALIAS),
            aggregation: Some(A::SQL),
        });
        P::add_to_context(context);
    }

    fn decode(&self, row: &Row) -> Result<Self::Result, Error> {
        row.get(Self::SELECT_ALIAS)
    }
}

macro_rules! selectable {
    ($($index:tt : $S:ident,)+) => {
        impl<M: Model, $($S: Selectable<Model = M>),+> Selectable for ($($S,)+)
        {
            type Result = ($(
                $S::Result,
            )+);

            type Model = M;

            fn prepare(&self, context: &mut QueryContext) {
                $(
                    self.$index.prepare(context);
                )+
            }

            fn decode(&self, row: &Row) -> Result<Self::Result, Error> {
                Ok(($(
                    self.$index.decode(&row)?,
                )+))
            }
        }
    };
}
rorm_macro::impl_tuple!(selectable, 1..33);
