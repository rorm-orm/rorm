//! Query builder and macro

use std::marker::PhantomData;
use std::ops::{Range, RangeInclusive, Sub};

use rorm_db::database;
use rorm_db::database::ColumnSelector;
use rorm_db::error::Error;
use rorm_db::executor::{All, Executor, One, Optional, Stream};
use rorm_db::row::Row;
use rorm_db::sql::limit_clause::LimitClause;
use rorm_db::sql::ordering::{OrderByEntry, Ordering};

use crate::conditions::Condition;
use crate::crud::builder::ConditionMarker;
use crate::crud::selectable::Selectable;
use crate::internal::field::{FieldProxy, RawField};
use crate::internal::query_context::QueryContextBuilder;
use crate::internal::relation_path::Path;
use crate::model::{Model, Patch};
use crate::sealed;

/// Builder for select queries
///
/// Is is recommended to start a builder using [`query!`](macro@crate::query).
///
/// ## Generics
/// - `'rf`
///
///     Lifetime of external values (eg: condition values).
///
/// - `E`: [`Executor`]
///
///     The executor to query with.
///
/// - `M`: [`Model`]
///
///     The model from whose table to select.
///
/// - `S`: [`Selector<M>`]
///
///     The columns to be selected and a type to convert the rows into.
///
/// - `C`: [`ConditionMarker<'rf>`](ConditionMarker)
///
///     An optional condition to filter the query by.
///
/// - `LO`: [`LimOffMarker`](LimOffMarker)
///
///     An optional limit and or offset to control the amount of queried rows.
#[must_use]
pub struct QueryBuilder<'rf, E, M, S, C, LO> {
    executor: E,
    ctx: QueryContextBuilder,
    selector: S,
    _phantom: PhantomData<&'rf M>,

    condition: C,
    lim_off: LO,
    ordering: Vec<OrderByEntry<'static>>,
}

/// Specifies which columns to query and how to decode the rows into what.
pub trait Selector<M: Model> {
    /// Type as which rows should be decoded
    type Result;

    /// Decode a row
    fn decode(row: Row) -> Result<Self::Result, Error>;

    /// Columns to query
    fn columns(&self, builder: &mut QueryContextBuilder) -> &[ColumnSelector<'static>];
}

impl<'ex, 'rf, E, M, S> QueryBuilder<'rf, E, M, S, (), ()>
where
    E: Executor<'ex>,
    M: Model,
    S: Selector<M>,
{
    /// Start building a query using a generic [`Selector`]
    pub fn new(executor: E, selector: S) -> Self {
        QueryBuilder {
            executor,
            ctx: QueryContextBuilder::new(),
            selector,
            _phantom: PhantomData,

            condition: (),
            lim_off: (),
            ordering: Vec::new(),
        }
    }
}

impl<'rf, E, M, S, LO> QueryBuilder<'rf, E, M, S, (), LO> {
    /// Add a condition to the query
    pub fn condition<C: Condition<'rf>>(self, condition: C) -> QueryBuilder<'rf, E, M, S, C, LO> {
        #[rustfmt::skip]
        let QueryBuilder { executor, ctx, selector, _phantom, lim_off, ordering, .. } = self;
        #[rustfmt::skip]
        return QueryBuilder { executor, ctx, selector, _phantom, condition, lim_off, ordering, };
    }
}

impl<'rf, E, M, S, C, O> QueryBuilder<'rf, E, M, S, C, O>
where
    O: OffsetMarker,
{
    /// Add a limit to the query
    pub fn limit(self, limit: u64) -> QueryBuilder<'rf, E, M, S, C, Limit<O>> {
        #[rustfmt::skip]
        let QueryBuilder { executor, ctx, selector, _phantom, condition,  lim_off, ordering, } = self;
        #[rustfmt::skip]
        return QueryBuilder { executor, ctx, selector, _phantom, condition, lim_off: Limit { limit, offset: lim_off }, ordering, };
    }
}

impl<'rf, E, M, S, C, LO> QueryBuilder<'rf, E, M, S, C, LO>
where
    LO: AcceptsOffset,
{
    /// Add a offset to the query
    pub fn offset(self, offset: u64) -> QueryBuilder<'rf, E, M, S, C, LO::Result> {
        #[rustfmt::skip]
        let QueryBuilder { executor, ctx, selector, _phantom, condition, lim_off, ordering, .. } = self;
        let lim_off = lim_off.add_offset(offset);
        #[rustfmt::skip]
        return QueryBuilder { executor, ctx, selector, _phantom, condition, lim_off, ordering, };
    }
}

impl<'rf, E, M, S, C> QueryBuilder<'rf, E, M, S, C, ()> {
    /// Add a offset to the query
    pub fn range(self, range: impl FiniteRange<u64>) -> QueryBuilder<'rf, E, M, S, C, Limit<u64>> {
        #[rustfmt::skip]
        let QueryBuilder { executor, ctx, selector, _phantom, condition, ordering,  .. } = self;
        let limit = Limit {
            limit: range.len(),
            offset: range.start(),
        };
        #[rustfmt::skip]
        return QueryBuilder { executor, ctx, selector, _phantom, condition, lim_off: limit, ordering, };
    }
}

impl<'rf, E, M, S, C, LO> QueryBuilder<'rf, E, M, S, C, LO> {
    /// Order the query by a field
    ///
    /// You can add multiple orderings from most to least significant.
    pub fn order_by<F, P>(mut self, _field: FieldProxy<F, P>, order: Ordering) -> Self
    where
        F: RawField,
        P: Path<Origin = M>,
    {
        P::add_to_join_builder(&mut self.ctx);
        self.ordering.push(OrderByEntry {
            ordering: order,
            table_name: Some(P::ALIAS),
            column_name: F::NAME,
        });
        self
    }

    /// Order the query ascending by a field
    ///
    /// You can add multiple orderings from most to least significant.
    pub fn order_asc<F, P>(self, field: FieldProxy<F, P>) -> Self
    where
        F: RawField,
        P: Path<Origin = M>,
    {
        self.order_by(field, Ordering::Asc)
    }

    /// Order the query descending by a field
    ///
    /// You can add multiple orderings from most to least significant.
    pub fn order_desc<F, P>(self, field: FieldProxy<F, P>) -> Self
    where
        F: RawField,
        P: Path<Origin = M>,
    {
        self.order_by(field, Ordering::Desc)
    }
}

impl<'ex, 'rf, E, M, S, C, LO> QueryBuilder<'rf, E, M, S, C, LO>
where
    'ex: 'rf,
    E: Executor<'ex>,
    M: Model,
    S: Selector<M>,
    C: ConditionMarker<'rf>,
{
    /// Retrieve and decode all matching rows
    pub async fn all(mut self) -> Result<Vec<S::Result>, Error>
    where
        LO: LimitMarker,
    {
        let columns = self.selector.columns(&mut self.ctx);
        self.condition.add_to_builder(&mut self.ctx);
        let context = self.ctx.finish();
        let joins = context.get_joins();

        let condition = self.condition.into_option();
        let condition = condition
            .as_ref()
            .map(|condition| condition.as_sql(&context));

        database::query::<All>(
            self.executor,
            M::TABLE,
            columns,
            &joins,
            condition.as_ref(),
            self.ordering.as_slice(),
            self.lim_off.into_option(),
        )
        .await?
        .into_iter()
        .map(|x| S::decode(x).map_err(|_| Error::DecodeError("Could not decode row".to_string())))
        .collect::<Result<Vec<_>, _>>()
    }

    /// Retrieve and decode the query as a stream
    pub fn stream(mut self) -> QueryStream<'rf, M, S>
    where
        S: 'rf,
        LO: LimitMarker,
    {
        let columns = self.selector.columns(&mut self.ctx);
        self.condition.add_to_builder(&mut self.ctx);
        QueryStream::new(
            self.ctx.finish(),
            |ctx| ctx.get_joins(),
            self.condition.into_option(),
            |conditions, joins| {
                database::query::<Stream>(
                    self.executor,
                    M::TABLE,
                    columns,
                    joins,
                    conditions,
                    self.ordering.as_slice(),
                    self.lim_off.into_option(),
                )
            },
        )
    }

    /// Retrieve and decode exactly one matching row
    ///
    /// An error is returned if no value could be retrieved.
    pub async fn one(mut self) -> Result<S::Result, Error>
    where
        LO: OffsetMarker,
    {
        let columns = self.selector.columns(&mut self.ctx);
        self.condition.add_to_builder(&mut self.ctx);
        let context = self.ctx.finish();
        let joins = context.get_joins();

        let condition = self.condition.into_option();
        let condition = condition
            .as_ref()
            .map(|condition| condition.as_sql(&context));

        let row = database::query::<One>(
            self.executor,
            M::TABLE,
            columns,
            &joins,
            condition.as_ref(),
            self.ordering.as_slice(),
            self.lim_off.into_option(),
        )
        .await?;
        S::decode(row).map_err(|_| Error::DecodeError("Could not decode row".to_string()))
    }

    /// Try to retrieve and decode a matching row
    pub async fn optional(mut self) -> Result<Option<S::Result>, Error>
    where
        LO: OffsetMarker,
    {
        let columns = self.selector.columns(&mut self.ctx);
        self.condition.add_to_builder(&mut self.ctx);
        let context = self.ctx.finish();
        let joins = context.get_joins();

        let condition = self.condition.into_option();
        let condition = condition
            .as_ref()
            .map(|condition| condition.as_sql(&context));

        let row = database::query::<Optional>(
            self.executor,
            M::TABLE,
            columns,
            &joins,
            condition.as_ref(),
            self.ordering.as_slice(),
            self.lim_off.into_option(),
        )
        .await?;
        match row {
            None => Ok(None),
            Some(row) => {
                Ok(Some(S::decode(row).map_err(|_| {
                    Error::DecodeError("Could not decode row".to_string())
                })?))
            }
        }
    }
}

/// Create a SELECT query.
///
/// 1. Give a reference to your db and the patch to query.
///     If you just need a few fields and don't want to create a patch for it,
///     you can specify these fields directly as a tuple as well.
///
///     `query!(&db, MyModelType)`
///
///     `query!(&db, (MyModelType::F.some_field, MyModelType::F.another_field, ))`
///
/// 2. Set a condition which rows to query.
///
///     `.condition(MyModelType::F.some_field.equals("some_value"))`
///
/// 3. *Optionally* add a limit or offset to restrict your query size.
///
///     `.limit(5)`
///
///     `.offset(2)`
///
///     `.range(2..7)`
///
/// 5. Finally specify how to get the queries results. This will also execute the query.
///     - Get [`all`](QueryBuilder::all) matching rows in a vector.
///
///         `.all().await`
///
///     - Get all matching rows in an async [`stream`](QueryBuilder::stream).
///
///         `.stream()`
///
///     - Just get exactly [`one`](QueryBuilder::one) row.
///
///         `.one().await`
///
///     - Get one row if any. ([`optional`](QueryBuilder::optional))
///
///         `.optional().await`
///
///     Each of these methods decodes the database's rows into the patch you specified in step 1.
///     If you want to work with raw rows, each of the methods in step 4 has a `*_as_row` twin.
///
/// Example:
/// ```no_run
/// # use rorm::{Model, Database, query};
/// #
/// # #[derive(Model)]
/// # struct User {
/// #     #[rorm(id)]
/// #     id: i64,
/// #
/// #     #[rorm(max_length = 255)]
/// #     username: String,
/// #
/// #     #[rorm(max_length = 255)]
/// #     password: String,
/// # }
/// #
/// #
/// # async fn shame_user(_user: &User) {}
/// #
/// pub async fn shame_users(db: &Database) {
///     for (id, password) in query!(db, (User::F.id, User::F.password)).all().await.unwrap() {
///         if password == "password" {
///             let user = query!(db, User)
///                 .condition(User::F.id.equals(id))
///                 .one()
///                 .await
///                 .unwrap();
///             shame_user(&user).await;
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! query {
    (replace $anything:tt with $result:tt) => { $result };
    ($db:expr, $patch:path) => {
        $crate::crud::query::QueryBuilder::new(
            $db,
            $crate::crud::query::SelectPatch::<$patch>::new(),
        )
    };
    ($db:expr, ($($field:expr),+$(,)?)) => {
        $crate::crud::query::QueryBuilder::new(
            $db,
            $crate::crud::query::SelectTuple::from(($($field,)+)),
        )
    };
}

/// Sadly ouroboros doesn't handle the lifetime bounds required for the QueryStream very well.
/// This module's code is copied from ouroboros' expanded macro and the tailored to fit the lifetime bounds.
mod query_stream {
    use std::marker::PhantomData;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    use ouroboros::macro_help::{aliasable_boxed, change_lifetime, AliasableBox};
    use rorm_db::database::JoinTable;
    use rorm_db::executor::{QueryStrategyResult, Stream};
    use rorm_db::sql::conditional::Condition as SqlCondition;
    use rorm_db::Error;

    use crate::conditions::Condition;
    use crate::crud::query::Selector;
    use crate::internal::query_context::QueryContext;
    use crate::Model;

    #[pin_project::pin_project]
    #[allow(dead_code)] // The field's are never "read" because they are aliased before being assigned to the struct
    pub struct QueryStream<'rf, M, S> {
        ctx: AliasableBox<QueryContext>,

        owned_condition: AliasableBox<Option<Box<dyn Condition<'rf>>>>,
        sql_condition: AliasableBox<Option<SqlCondition<'rf>>>,
        joins: AliasableBox<Vec<JoinTable<'rf, 'rf>>>,

        selector: PhantomData<(M, S)>,

        #[pin]
        stream: <Stream as QueryStrategyResult>::Result<'rf>,
    }

    impl<'stream, M, S> QueryStream<'stream, M, S> {
        pub(crate) fn new<'until_build>(
            ctx: QueryContext,
            joins_builder: impl FnOnce(&'stream QueryContext) -> Vec<JoinTable<'stream, 'stream>>
                + 'until_build,
            condition: Option<Box<dyn Condition<'stream>>>,
            stream_builder: impl FnOnce(
                    Option<&'stream SqlCondition<'stream>>,
                    &'stream Vec<JoinTable<'stream, 'stream>>,
                ) -> <Stream as QueryStrategyResult>::Result<'stream>
                + 'until_build,
        ) -> Self
        where
            'stream: 'until_build,
        {
            unsafe {
                let ctx = aliasable_boxed(ctx);
                let ctx_illegal_static_reference = change_lifetime(&*ctx);

                let joins = joins_builder(ctx_illegal_static_reference);
                let joins = aliasable_boxed(joins);
                let joins_illegal_static_reference = change_lifetime(&*joins);

                let owned_condition = aliasable_boxed(condition);
                let owned_condition_illegal_static_reference = change_lifetime(&*owned_condition);

                let sql_condition = aliasable_boxed(
                    owned_condition_illegal_static_reference
                        .as_ref()
                        .map(|cond| cond.as_sql(ctx_illegal_static_reference)),
                );
                let sql_condition_illegal_static_reference = change_lifetime(&*sql_condition);

                let stream = stream_builder(
                    sql_condition_illegal_static_reference.as_ref(),
                    joins_illegal_static_reference,
                );

                Self {
                    ctx,
                    joins,
                    owned_condition,
                    sql_condition,
                    selector: PhantomData,
                    stream,
                }
            }
        }
    }

    impl<'stream, M: Model, S: Selector<M>> futures::stream::Stream for QueryStream<'stream, M, S> {
        type Item = Result<S::Result, Error>;

        fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            self.project()
                .stream
                .as_mut()
                .poll_next(cx)
                .map(|option| option.map(|result| result.and_then(S::decode)))
        }
    }
}
use query_stream::QueryStream;

/// Finite alternative to [`RangeBounds`](std::ops::RangeBounds)
///
/// It unifies [`Range`] and [`RangeInclusive`]
#[allow(clippy::len_without_is_empty)] // Since it is generic, there is no trivial way to compare with zero
pub trait FiniteRange<T> {
    // and I don't see why I should use an "IsZero" trait, just to satisfy clippy.
    /// The lower bound of the range (inclusive)
    fn start(&self) -> T;

    /// The upper bound of the range (exclusive)
    fn end(&self) -> T;

    /// The number of items contained in this range
    fn len(&self) -> T
    where
        T: Sub<T, Output = T> + Copy,
    {
        self.end() - self.start()
    }
}
impl<T: Copy> FiniteRange<T> for Range<T> {
    fn start(&self) -> T {
        self.start
    }

    fn end(&self) -> T {
        self.end
    }
}
impl FiniteRange<u64> for RangeInclusive<u64> {
    fn start(&self) -> u64 {
        *self.start()
    }

    fn end(&self) -> u64 {
        *self.end() + 1
    }
}

/// Unification of [`LimitMarker`] and [`OffsetMarker`]
pub trait LimOffMarker: 'static {
    sealed!();
}
impl LimOffMarker for () {}
impl<O: OffsetMarker> LimOffMarker for Limit<O> {}
impl LimOffMarker for u64 {}

/// Marker for the generic parameter storing a limit.
///
/// Valid values are `()`, `Limit<()>` and `Limit<u64>`.
pub trait LimitMarker: LimOffMarker {
    sealed!();

    /// Convert the generic limit into [`Option<LimitClause>`]
    fn into_option(self) -> Option<LimitClause>;
}
impl LimitMarker for () {
    fn into_option(self) -> Option<LimitClause> {
        None
    }
}
/// A query limit and optional offset
pub struct Limit<O: OffsetMarker> {
    /// Number of rows to query
    limit: u64,

    /// Optional offset to begin query at
    offset: O,
}
impl<O: OffsetMarker> LimitMarker for Limit<O> {
    fn into_option(self) -> Option<LimitClause> {
        Some(LimitClause {
            limit: self.limit,
            offset: self.offset.into_option(),
        })
    }
}

/// Unification of `()` and `Limit<()>`
pub trait AcceptsOffset: LimOffMarker {
    sealed!();

    /// The resulting type i.e. `u64` or `Limit<u64>`
    type Result: LimOffMarker;
    /// "Add" the offset to the type
    fn add_offset(self, offset: u64) -> Self::Result;
}
impl AcceptsOffset for () {
    type Result = u64;
    fn add_offset(self, offset: u64) -> Self::Result {
        offset
    }
}
impl AcceptsOffset for Limit<()> {
    type Result = Limit<u64>;
    fn add_offset(self, offset: u64) -> Self::Result {
        let Limit { limit, offset: _ } = self;
        Limit { limit, offset }
    }
}

/// Marker for the generic parameter storing a limit's offset.
///
/// Valid values are `()` and `u64`.
pub trait OffsetMarker: LimOffMarker {
    sealed!();

    /// Convert the generic offset into [`Option<u64>`]
    fn into_option(self) -> Option<u64>;
}
impl OffsetMarker for () {
    fn into_option(self) -> Option<u64> {
        None
    }
}
impl OffsetMarker for u64 {
    fn into_option(self) -> Option<u64> {
        Some(self)
    }
}

/// The [`Selector`] for patches.
pub struct SelectPatch<P: Patch> {
    patch: PhantomData<P>,
    columns: Vec<ColumnSelector<'static>>,
}
impl<P: Patch> SelectPatch<P> {
    /// Create a SelectPatch
    pub fn new() -> Self {
        SelectPatch {
            patch: PhantomData,
            columns: P::COLUMNS
                .iter()
                .map(|x| ColumnSelector {
                    table_name: Some(P::Model::TABLE),
                    column_name: x,
                    select_alias: None,
                    aggregation: None,
                })
                .collect(),
        }
    }
}
impl<P: Patch> Default for SelectPatch<P> {
    fn default() -> Self {
        Self::new()
    }
}
impl<M: Model, P: Patch<Model = M>> Selector<M> for SelectPatch<P> {
    type Result = P;

    fn decode(row: Row) -> Result<Self::Result, Error> {
        P::from_row(row)
    }

    fn columns(&self, _builder: &mut QueryContextBuilder) -> &[ColumnSelector<'static>] {
        &self.columns
    }
}

/// The [`Selector`] for tuple
///
/// Implemented for tuple of size 8 or less.
pub struct SelectTuple<T> {
    #[allow(dead_code)]
    tuple: T,
    columns: Vec<ColumnSelector<'static>>,
}
macro_rules! impl_select_tuple {
    ($C:literal, ($($S:ident,)+)) => {
        impl<$($S: Selectable),+> From<($($S,)+)> for SelectTuple<($($S,)+)> {
            fn from(tuple: ($($S,)+)) -> SelectTuple<($($S,)+)>{
                let mut selectors = Vec::new();
                $(
                    $S::push_selector(&mut selectors);
                )+
                Self {
                    tuple,
                    columns: selectors,
                }
            }
        }
        impl<M: Model, $($S: Selectable<Table = M>),+> Selector<M> for SelectTuple<($($S,)+)>
        {
            type Result = ($(
                $S::Result,
            )+);

            fn decode(row: Row) -> Result<Self::Result, Error> {
                Ok(($(
                    $S::decode(&row)?,
                )+))
            }

            fn columns(&self, builder: &mut QueryContextBuilder) -> &[ColumnSelector<'static>] {
                $(
                    $S::prepare(builder);
                )+
                &self.columns
            }
        }
    };
}
impl_select_tuple!(1, (S0,));
impl_select_tuple!(2, (S0, S1,));
impl_select_tuple!(3, (S0, S1, S2,));
impl_select_tuple!(4, (S0, S1, S2, S3,));
impl_select_tuple!(5, (S0, S1, S2, S3, S4,));
impl_select_tuple!(6, (S0, S1, S2, S3, S4, S5,));
impl_select_tuple!(7, (S0, S1, S2, S3, S4, S5, S6,));
impl_select_tuple!(8, (S0, S1, S2, S3, S4, S5, S6, S7,));
impl_select_tuple!(9, (S0, S1, S2, S3, S4, S5, S6, S7, S8,));
impl_select_tuple!(10, (S0, S1, S2, S3, S4, S5, S6, S7, S8, S9,));
impl_select_tuple!(11, (S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10,));
impl_select_tuple!(12, (S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11,));
impl_select_tuple!(13, (S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12,));
impl_select_tuple!(
    14,
    (S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13,)
);
impl_select_tuple!(
    15,
    (S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14,)
);
impl_select_tuple!(
    16,
    (S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15,)
);
impl_select_tuple!(
    17,
    (S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16,)
);
impl_select_tuple!(
    18,
    (S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17,)
);
impl_select_tuple!(
    19,
    (S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18,)
);
impl_select_tuple!(
    20,
    (S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19,)
);
impl_select_tuple!(
    21,
    (
        S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19,
        S20,
    )
);
impl_select_tuple!(
    22,
    (
        S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19,
        S20, S21,
    )
);
impl_select_tuple!(
    23,
    (
        S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19,
        S20, S21, S22,
    )
);
impl_select_tuple!(
    24,
    (
        S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19,
        S20, S21, S22, S23,
    )
);
impl_select_tuple!(
    25,
    (
        S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19,
        S20, S21, S22, S23, S24,
    )
);
impl_select_tuple!(
    26,
    (
        S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19,
        S20, S21, S22, S23, S24, S25,
    )
);
impl_select_tuple!(
    27,
    (
        S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19,
        S20, S21, S22, S23, S24, S25, S26,
    )
);
impl_select_tuple!(
    28,
    (
        S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19,
        S20, S21, S22, S23, S24, S25, S26, S27,
    )
);
impl_select_tuple!(
    29,
    (
        S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19,
        S20, S21, S22, S23, S24, S25, S26, S27, S28,
    )
);
impl_select_tuple!(
    30,
    (
        S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19,
        S20, S21, S22, S23, S24, S25, S26, S27, S28, S29,
    )
);
impl_select_tuple!(
    31,
    (
        S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19,
        S20, S21, S22, S23, S24, S25, S26, S27, S28, S29, S30,
    )
);
impl_select_tuple!(
    32,
    (
        S0, S1, S2, S3, S4, S5, S6, S7, S8, S9, S10, S11, S12, S13, S14, S15, S16, S17, S18, S19,
        S20, S21, S22, S23, S24, S25, S26, S27, S28, S29, S30, S31,
    )
);
