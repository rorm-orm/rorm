//! Trait for the items inside [`SelectTuple`](crate::crud::query::SelectTuple)

use rorm_db::database::ColumnSelector;
use rorm_db::{Error, Row};

use crate::aggregate::{AggregatedColumn, AggregationFunc};
use crate::const_concat;
use crate::internal::field::as_db_type::AsDbType;
use crate::internal::field::{AbstractField, AliasedField, Field, FieldProxy};
use crate::internal::query_context::QueryContextBuilder;
use crate::internal::relation_path::Path;
use crate::model::Model;

/// Anything which can be used to select a single item
pub trait Selectable {
    /// The type resulting from this select item
    type Result;

    /// The table i.e. model this selects from
    type Table: Model;

    /// Number of columns to be selected
    const COLUMNS: usize = 1;

    /// Helper to store an alias in
    ///
    /// Used in [`AggregatedColumn`] to store the `const` generated alias
    const SELECT_ALIAS: &'static str = "";

    /// Push your rorm-sql `ColumnSelector` to the list
    ///
    /// Used to populate [`SelectTuple`](crate::crud::query::SelectTuple)'s `columns` field.
    fn push_selector(selectors: &mut Vec<ColumnSelector<'static>>);

    /// Prepare the context to handle the select i.e. register potential joins
    fn prepare(builder: &mut QueryContextBuilder);

    /// Retrieve the result from a row
    fn decode(row: &Row) -> Result<Self::Result, Error>;
}

impl<F, P> Selectable for FieldProxy<F, P>
where
    P: Path,
    F: AbstractField + AliasedField<P>,
{
    type Result = F::Type;

    type Table = P::Origin;

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

    fn prepare(builder: &mut QueryContextBuilder) {
        builder.add_field_proxy::<F, P>()
    }

    fn decode(row: &Row) -> Result<Self::Result, Error> {
        F::get_by_alias(row)
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

    type Table = P::Origin;

    const SELECT_ALIAS: &'static str = const_concat!(&[P::ALIAS, "__", F::NAME, "___", A::NAME]);

    fn push_selector(selectors: &mut Vec<ColumnSelector<'static>>) {
        selectors.push(ColumnSelector {
            table_name: Some(P::ALIAS),
            column_name: F::NAME,
            select_alias: Some(Self::SELECT_ALIAS),
            aggregation: Some(A::SQL),
        });
    }

    fn prepare(builder: &mut QueryContextBuilder) {
        builder.add_field_proxy::<F, P>()
    }

    fn decode(row: &Row) -> Result<Self::Result, Error> {
        row.get(Self::SELECT_ALIAS)
    }
}
