//! Trait for selecting stuff

use std::marker::PhantomData;

use crate::aggregate::{AggregatedColumn, AggregationFunc};
use crate::crud::decoder::{Decoder, DirectDecoder};
use crate::fields::traits::FieldType;
use crate::internal::field::as_db_type::AsDbType;
use crate::internal::field::decoder::FieldDecoder;
use crate::internal::field::{Field, FieldProxy, SingleColumnField};
use crate::internal::query_context::QueryContext;
use crate::internal::relation_path::{Path, PathImpl, PathStep, ResolvedRelatedField};
use crate::model::{Model, PatchSelector};
use crate::Patch;

/// Something which "selects" a value from a certain table,
/// by configuring a [`QueryContext`] and providing a [`Decoder`]
pub trait Selector {
    /// The value selected by this selector
    type Result;

    /// [`Model`] from whose table to select from
    type Model: Model;

    /// [`Decoder`] to decode the selected value from a [`&Row`](rorm_db::Row)
    type Decoder: Decoder<Result = Self::Result>;

    /// Can this selector be used in insert queries to specify the returning expression?
    const INSERT_COMPATIBLE: bool;

    /// Constructs a decoder and configures a [`QueryContext`] to query the required columns
    fn select(self, ctx: &mut QueryContext) -> Self::Decoder;
}

impl<F, P> Selector for FieldProxy<F, P>
where
    P: Path,
    F: Field,
{
    type Result = F::Type;
    type Model = P::Origin;
    type Decoder = <F::Type as FieldType>::Decoder;
    const INSERT_COMPATIBLE: bool = P::IS_ORIGIN;

    fn select(self, ctx: &mut QueryContext) -> Self::Decoder {
        FieldDecoder::new(ctx, FieldProxy::<F, P>::new())
    }
}

#[doc(hidden)]
impl<F, P> FieldProxy<F, P>
where
    F: Field,
    P: Path,
    PathStep<F, P>: PathImpl<F::Type>,
{
    pub fn select_as<Ptch>(self) -> PatchSelector<Ptch, PathStep<F, P>>
    where
        Ptch: Patch<Model = <ResolvedRelatedField<F, P> as Field>::Model>,
    {
        PatchSelector::new()
    }
}

impl<A, F, P> Selector for AggregatedColumn<A, F, P>
where
    A: AggregationFunc,
    F: SingleColumnField,
    F::Type: AsDbType,
    P: Path,
{
    type Result = A::Result<<F::Type as AsDbType>::Primitive>;
    type Model = P::Origin;
    type Decoder = DirectDecoder<Self::Result>;
    const INSERT_COMPATIBLE: bool = false;

    fn select(self, ctx: &mut QueryContext) -> Self::Decoder {
        let (index, column) = ctx.select_aggregation::<A, F, P>();
        DirectDecoder {
            result: PhantomData,
            column,
            index,
        }
    }
}

macro_rules! selectable {
    ($($index:tt : $S:ident,)+) => {
        impl<M: Model, $($S: Selector<Model = M>),+> Selector for ($($S,)+)
        {
            type Result = ($(
                $S::Result,
            )+);

            type Model = M;

            type Decoder = ($(
                $S::Decoder,
            )+);

            const INSERT_COMPATIBLE: bool = $($S::INSERT_COMPATIBLE &&)+ true;

            fn select(self, ctx: &mut QueryContext) -> Self::Decoder {
                ($(
                    self.$index.select(ctx),
                )+)
            }
        }
    };
}
rorm_macro::impl_tuple!(selectable, 1..33);
