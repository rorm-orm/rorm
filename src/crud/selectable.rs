//! Trait for the items inside [`SelectTuple`](crate::crud::query::SelectTuple)

use std::marker::PhantomData;

use rorm_db::database::ColumnSelector;
use rorm_db::{Error, Row};

use crate::aggregate::{AggregatedColumn, AggregationFunc};
use crate::internal::field::as_db_type::AsDbType;
use crate::internal::field::{AbstractField, AliasedField, Field, FieldProxy, RawField};
use crate::internal::query_context::QueryContext;
use crate::internal::relation_path::Path;
use crate::model::Model;
use crate::{const_concat, Patch};

/// Anything which can be used to select a single item
pub trait Selectable {
    /// The type resulting from this select item
    type Result;

    /// Model from whose table to select from
    type Model: Model;

    /// Push your rorm-sql `ColumnSelector` to the list
    ///
    /// Used to populate [`SelectTuple`](crate::crud::query::SelectTuple)'s `columns` field.
    fn push_selector(selectors: &mut Vec<ColumnSelector<'static>>);

    /// Wrap [`Selectable::push_selector`] to produce a [`Vec`]
    fn selector() -> Vec<ColumnSelector<'static>> {
        let mut columns = Vec::new();
        Self::push_selector(&mut columns);
        columns
    }

    /// Prepare the context to handle the select i.e. register potential joins
    fn prepare(context: &mut QueryContext);

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

    fn push_selector(selectors: &mut Vec<ColumnSelector<'static>>) {
        let columns = <F as AbstractField>::COLUMNS;
        let aliases = <F as AliasedField<P>>::COLUMNS;
        for (column, alias) in columns.iter().zip(aliases) {
            selectors.push(ColumnSelector {
                table_name: Some(P::ALIAS),
                column_name: column,
                select_alias: Some(alias),
                aggregation: None,
            });
        }
    }

    fn prepare(context: &mut QueryContext) {
        P::add_to_context(context);
    }

    fn decode(&self, row: &Row) -> Result<Self::Result, Error> {
        F::get_by_alias(row)
    }
}

impl<A, F, P> AggregatedColumn<A, F, P>
where
    A: AggregationFunc,
    F: RawField,
    P: Path,
{
    const SELECT_ALIAS: &'static str = const_concat!(&[P::ALIAS, "__", F::NAME, "___", A::NAME]);
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

    fn push_selector(selectors: &mut Vec<ColumnSelector<'static>>) {
        selectors.push(ColumnSelector {
            table_name: Some(P::ALIAS),
            column_name: F::NAME,
            select_alias: Some(Self::SELECT_ALIAS),
            aggregation: Some(A::SQL),
        });
    }

    fn prepare(context: &mut QueryContext) {
        P::add_to_context(context);
    }

    fn decode(&self, row: &Row) -> Result<Self::Result, Error> {
        row.get(Self::SELECT_ALIAS)
    }
}

impl<M: Model, P: Patch<Model = M>> Selectable for PhantomData<P> {
    type Result = P;

    type Model = M;

    fn push_selector(selectors: &mut Vec<ColumnSelector<'static>>) {
        selectors.extend(P::COLUMNS.iter().map(|x| ColumnSelector {
            table_name: Some(P::Model::TABLE),
            column_name: x,
            select_alias: None,
            aggregation: None,
        }));
    }

    fn prepare(_context: &mut QueryContext) {}

    fn decode(&self, _row: &Row) -> Result<Self::Result, Error> {
        P::from_row(todo!())
    }
}

macro_rules! impl_select_tuple {
    ($C:literal, ($($index:tt : $S:ident,)+)) => {
        impl<M: Model, $($S: Selectable<Model = M>),+> Selectable for ($($S,)+)
        {
            type Result = ($(
                $S::Result,
            )+);

            type Model = M;

            fn push_selector(selectors: &mut Vec<ColumnSelector<'static>>) {
                $(
                    $S::push_selector(selectors);
                )+
            }

            fn prepare(context: &mut QueryContext) {
                $(
                    $S::prepare(context);
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
impl_select_tuple!(1, (0: S0,));
impl_select_tuple!(2, (0: S0, 1: S1,));
impl_select_tuple!(3, (0: S0, 1: S1, 2: S2,));
impl_select_tuple!(4, (0: S0, 1: S1, 2: S2, 3: S3,));
impl_select_tuple!(5, (0: S0, 1: S1, 2: S2, 3: S3, 4: S4,));
impl_select_tuple!(6, (0: S0, 1: S1, 2: S2, 3: S3, 4: S4, 5: S5,));
impl_select_tuple!(7, (0: S0, 1: S1, 2: S2, 3: S3, 4: S4, 5: S5, 6: S6,));
impl_select_tuple!(8, (0: S0, 1: S1, 2: S2, 3: S3, 4: S4, 5: S5, 6: S6, 7: S7,));
impl_select_tuple!(
    9,
    (
        0: S0,
        1: S1,
        2: S2,
        3: S3,
        4: S4,
        5: S5,
        6: S6,
        7: S7,
        8: S8,
    )
);
impl_select_tuple!(
    10,
    (
        0: S0,
        1: S1,
        2: S2,
        3: S3,
        4: S4,
        5: S5,
        6: S6,
        7: S7,
        8: S8,
        9: S9,
    )
);
impl_select_tuple!(
    11,
    (
        0: S0,
        1: S1,
        2: S2,
        3: S3,
        4: S4,
        5: S5,
        6: S6,
        7: S7,
        8: S8,
        9: S9,
        10: S10,
    )
);
impl_select_tuple!(
    12,
    (
        0: S0,
        1: S1,
        2: S2,
        3: S3,
        4: S4,
        5: S5,
        6: S6,
        7: S7,
        8: S8,
        9: S9,
        10: S10,
        11: S11,
    )
);
impl_select_tuple!(
    13,
    (
        0: S0,
        1: S1,
        2: S2,
        3: S3,
        4: S4,
        5: S5,
        6: S6,
        7: S7,
        8: S8,
        9: S9,
        10: S10,
        11: S11,
        12: S12,
    )
);
impl_select_tuple!(
    14,
    (
        0: S0,
        1: S1,
        2: S2,
        3: S3,
        4: S4,
        5: S5,
        6: S6,
        7: S7,
        8: S8,
        9: S9,
        10: S10,
        11: S11,
        12: S12,
        13: S13,
    )
);
impl_select_tuple!(
    15,
    (
        0: S0,
        1: S1,
        2: S2,
        3: S3,
        4: S4,
        5: S5,
        6: S6,
        7: S7,
        8: S8,
        9: S9,
        10: S10,
        11: S11,
        12: S12,
        13: S13,
        14: S14,
    )
);
impl_select_tuple!(
    16,
    (
        0: S0,
        1: S1,
        2: S2,
        3: S3,
        4: S4,
        5: S5,
        6: S6,
        7: S7,
        8: S8,
        9: S9,
        10: S10,
        11: S11,
        12: S12,
        13: S13,
        14: S14,
        15: S15,
    )
);
impl_select_tuple!(
    17,
    (
        0: S0,
        1: S1,
        2: S2,
        3: S3,
        4: S4,
        5: S5,
        6: S6,
        7: S7,
        8: S8,
        9: S9,
        10: S10,
        11: S11,
        12: S12,
        13: S13,
        14: S14,
        15: S15,
        16: S16,
    )
);
impl_select_tuple!(
    18,
    (
        0: S0,
        1: S1,
        2: S2,
        3: S3,
        4: S4,
        5: S5,
        6: S6,
        7: S7,
        8: S8,
        9: S9,
        10: S10,
        11: S11,
        12: S12,
        13: S13,
        14: S14,
        15: S15,
        16: S16,
        17: S17,
    )
);
impl_select_tuple!(
    19,
    (
        0: S0,
        1: S1,
        2: S2,
        3: S3,
        4: S4,
        5: S5,
        6: S6,
        7: S7,
        8: S8,
        9: S9,
        10: S10,
        11: S11,
        12: S12,
        13: S13,
        14: S14,
        15: S15,
        16: S16,
        17: S17,
        18: S18,
    )
);
impl_select_tuple!(
    20,
    (
        0: S0,
        1: S1,
        2: S2,
        3: S3,
        4: S4,
        5: S5,
        6: S6,
        7: S7,
        8: S8,
        9: S9,
        10: S10,
        11: S11,
        12: S12,
        13: S13,
        14: S14,
        15: S15,
        16: S16,
        17: S17,
        18: S18,
        19: S19,
    )
);
impl_select_tuple!(
    21,
    (
        0: S0,
        1: S1,
        2: S2,
        3: S3,
        4: S4,
        5: S5,
        6: S6,
        7: S7,
        8: S8,
        9: S9,
        10: S10,
        11: S11,
        12: S12,
        13: S13,
        14: S14,
        15: S15,
        16: S16,
        17: S17,
        18: S18,
        19: S19,
        20: S20,
    )
);
impl_select_tuple!(
    22,
    (
        0: S0,
        1: S1,
        2: S2,
        3: S3,
        4: S4,
        5: S5,
        6: S6,
        7: S7,
        8: S8,
        9: S9,
        10: S10,
        11: S11,
        12: S12,
        13: S13,
        14: S14,
        15: S15,
        16: S16,
        17: S17,
        18: S18,
        19: S19,
        20: S20,
        21: S21,
    )
);
impl_select_tuple!(
    23,
    (
        0: S0,
        1: S1,
        2: S2,
        3: S3,
        4: S4,
        5: S5,
        6: S6,
        7: S7,
        8: S8,
        9: S9,
        10: S10,
        11: S11,
        12: S12,
        13: S13,
        14: S14,
        15: S15,
        16: S16,
        17: S17,
        18: S18,
        19: S19,
        20: S20,
        21: S21,
        22: S22,
    )
);
impl_select_tuple!(
    24,
    (
        0: S0,
        1: S1,
        2: S2,
        3: S3,
        4: S4,
        5: S5,
        6: S6,
        7: S7,
        8: S8,
        9: S9,
        10: S10,
        11: S11,
        12: S12,
        13: S13,
        14: S14,
        15: S15,
        16: S16,
        17: S17,
        18: S18,
        19: S19,
        20: S20,
        21: S21,
        22: S22,
        23: S23,
    )
);
impl_select_tuple!(
    25,
    (
        0: S0,
        1: S1,
        2: S2,
        3: S3,
        4: S4,
        5: S5,
        6: S6,
        7: S7,
        8: S8,
        9: S9,
        10: S10,
        11: S11,
        12: S12,
        13: S13,
        14: S14,
        15: S15,
        16: S16,
        17: S17,
        18: S18,
        19: S19,
        20: S20,
        21: S21,
        22: S22,
        23: S23,
        24: S24,
    )
);
impl_select_tuple!(
    26,
    (
        0: S0,
        1: S1,
        2: S2,
        3: S3,
        4: S4,
        5: S5,
        6: S6,
        7: S7,
        8: S8,
        9: S9,
        10: S10,
        11: S11,
        12: S12,
        13: S13,
        14: S14,
        15: S15,
        16: S16,
        17: S17,
        18: S18,
        19: S19,
        20: S20,
        21: S21,
        22: S22,
        23: S23,
        24: S24,
        25: S25,
    )
);
impl_select_tuple!(
    27,
    (
        0: S0,
        1: S1,
        2: S2,
        3: S3,
        4: S4,
        5: S5,
        6: S6,
        7: S7,
        8: S8,
        9: S9,
        10: S10,
        11: S11,
        12: S12,
        13: S13,
        14: S14,
        15: S15,
        16: S16,
        17: S17,
        18: S18,
        19: S19,
        20: S20,
        21: S21,
        22: S22,
        23: S23,
        24: S24,
        25: S25,
        26: S26,
    )
);
impl_select_tuple!(
    28,
    (
        0: S0,
        1: S1,
        2: S2,
        3: S3,
        4: S4,
        5: S5,
        6: S6,
        7: S7,
        8: S8,
        9: S9,
        10: S10,
        11: S11,
        12: S12,
        13: S13,
        14: S14,
        15: S15,
        16: S16,
        17: S17,
        18: S18,
        19: S19,
        20: S20,
        21: S21,
        22: S22,
        23: S23,
        24: S24,
        25: S25,
        26: S26,
        27: S27,
    )
);
impl_select_tuple!(
    29,
    (
        0: S0,
        1: S1,
        2: S2,
        3: S3,
        4: S4,
        5: S5,
        6: S6,
        7: S7,
        8: S8,
        9: S9,
        10: S10,
        11: S11,
        12: S12,
        13: S13,
        14: S14,
        15: S15,
        16: S16,
        17: S17,
        18: S18,
        19: S19,
        20: S20,
        21: S21,
        22: S22,
        23: S23,
        24: S24,
        25: S25,
        26: S26,
        27: S27,
        28: S28,
    )
);
impl_select_tuple!(
    30,
    (
        0: S0,
        1: S1,
        2: S2,
        3: S3,
        4: S4,
        5: S5,
        6: S6,
        7: S7,
        8: S8,
        9: S9,
        10: S10,
        11: S11,
        12: S12,
        13: S13,
        14: S14,
        15: S15,
        16: S16,
        17: S17,
        18: S18,
        19: S19,
        20: S20,
        21: S21,
        22: S22,
        23: S23,
        24: S24,
        25: S25,
        26: S26,
        27: S27,
        28: S28,
        29: S29,
    )
);
impl_select_tuple!(
    31,
    (
        0: S0,
        1: S1,
        2: S2,
        3: S3,
        4: S4,
        5: S5,
        6: S6,
        7: S7,
        8: S8,
        9: S9,
        10: S10,
        11: S11,
        12: S12,
        13: S13,
        14: S14,
        15: S15,
        16: S16,
        17: S17,
        18: S18,
        19: S19,
        20: S20,
        21: S21,
        22: S22,
        23: S23,
        24: S24,
        25: S25,
        26: S26,
        27: S27,
        28: S28,
        29: S29,
        30: S30,
    )
);
impl_select_tuple!(
    32,
    (
        0: S0,
        1: S1,
        2: S2,
        3: S3,
        4: S4,
        5: S5,
        6: S6,
        7: S7,
        8: S8,
        9: S9,
        10: S10,
        11: S11,
        12: S12,
        13: S13,
        14: S14,
        15: S15,
        16: S16,
        17: S17,
        18: S18,
        19: S19,
        20: S20,
        21: S21,
        22: S22,
        23: S23,
        24: S24,
        25: S25,
        26: S26,
        27: S27,
        28: S28,
        29: S29,
        30: S30,
        31: S31,
    )
);
