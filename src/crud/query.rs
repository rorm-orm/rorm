//! Query builder and macro

use std::ops::{Range, RangeInclusive, Sub};

use rorm_db::database;
use rorm_db::error::Error;
use rorm_db::executor::{All, Executor, One, Optional, Stream};
use rorm_db::sql::limit_clause::LimitClause;
use rorm_db::sql::ordering::{OrderByEntry, Ordering};

use crate::conditions::Condition;
use crate::crud::builder::ConditionMarker;
use crate::crud::decoder::Decoder;
use crate::crud::selector::Selector;
use crate::internal::field::{Field, FieldProxy};
use crate::internal::query_context::QueryContext;
use crate::internal::relation_path::Path;
use crate::model::Model;
use crate::sealed;

/// Builder for select queries
///
/// Is is recommended to start a builder using [`query!`](macro@crate::query).
///
/// - `E`: [`Executor`]
///
///     The executor to query with.
///
/// - `S`: [`Selector`]
///
///     The columns to be selected and a type to convert the rows into.
///
/// - `C`: [`ConditionMarker`]
///
///     An optional condition to filter the query by.
///
/// - `LO`: [`LimOffMarker`]
///
///     An optional limit and or offset to control the amount of queried rows.
#[must_use]
pub struct QueryBuilder<E, S, C, LO> {
    executor: E,
    ctx: QueryContext,
    selector: S,
    condition: C,
    lim_off: LO,
    ordering: Vec<OrderByEntry<'static>>,
}

impl<'ex, E, S> QueryBuilder<E, S, (), ()>
where
    E: Executor<'ex>,
    S: Selector,
{
    /// Start building a query using a generic [`Selector`](Selector)
    pub fn new(executor: E, selector: S, _: <S::Model as Model>::QueryPermission) -> Self {
        QueryBuilder {
            executor,
            ctx: QueryContext::new(),
            selector,
            condition: (),
            lim_off: (),
            ordering: Vec::new(),
        }
    }
}

impl<E, S, LO> QueryBuilder<E, S, (), LO> {
    /// Add a condition to the query
    pub fn condition<'c, C: Condition<'c>>(self, condition: C) -> QueryBuilder<E, S, C, LO> {
        #[rustfmt::skip]
        let QueryBuilder { executor, ctx, selector, lim_off, ordering, .. } = self;
        #[rustfmt::skip]
        return QueryBuilder { executor, ctx, selector, condition, lim_off, ordering, };
    }
}

impl<E, S, C, O> QueryBuilder<E, S, C, O>
where
    O: OffsetMarker,
{
    /// Add a limit to the query
    pub fn limit(self, limit: u64) -> QueryBuilder<E, S, C, Limit<O>> {
        #[rustfmt::skip]
        let QueryBuilder { executor, ctx, selector, condition,  lim_off, ordering, } = self;
        #[rustfmt::skip]
        return QueryBuilder { executor, ctx, selector, condition, lim_off: Limit { limit, offset: lim_off }, ordering, };
    }
}

impl<E, S, C, LO> QueryBuilder<E, S, C, LO>
where
    LO: AcceptsOffset,
{
    /// Add a offset to the query
    pub fn offset(self, offset: u64) -> QueryBuilder<E, S, C, LO::Result> {
        #[rustfmt::skip]
        let QueryBuilder { executor, ctx, selector, condition, lim_off, ordering, .. } = self;
        let lim_off = lim_off.add_offset(offset);
        #[rustfmt::skip]
        return QueryBuilder { executor, ctx, selector, condition, lim_off, ordering, };
    }
}

impl<E, S, C> QueryBuilder<E, S, C, ()> {
    /// Add a offset to the query
    pub fn range(self, range: impl FiniteRange<u64>) -> QueryBuilder<E, S, C, Limit<u64>> {
        #[rustfmt::skip]
        let QueryBuilder { executor, ctx, selector, condition, ordering,  .. } = self;
        let limit = Limit {
            limit: range.len(),
            offset: range.start(),
        };
        #[rustfmt::skip]
        return QueryBuilder { executor, ctx, selector, condition, lim_off: limit, ordering, };
    }
}

impl<E, S, C, LO> QueryBuilder<E, S, C, LO>
where
    S: Selector,
{
    /// Order the query by a field
    ///
    /// You can add multiple orderings from most to least significant.
    pub fn order_by<F, P>(mut self, _field: FieldProxy<F, P>, order: Ordering) -> Self
    where
        F: Field,
        P: Path<Origin = S::Model>,
    {
        P::add_to_context(&mut self.ctx);
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
        F: Field,
        P: Path<Origin = S::Model>,
    {
        self.order_by(field, Ordering::Asc)
    }

    /// Order the query descending by a field
    ///
    /// You can add multiple orderings from most to least significant.
    pub fn order_desc<F, P>(self, field: FieldProxy<F, P>) -> Self
    where
        F: Field,
        P: Path<Origin = S::Model>,
    {
        self.order_by(field, Ordering::Desc)
    }
}

impl<'e, 'c, E, S, C, LO> QueryBuilder<E, S, C, LO>
where
    E: Executor<'e>,
    S: Selector,
    C: ConditionMarker<'c>,
{
    /// Retrieve and decode all matching rows
    pub async fn all(mut self) -> Result<Vec<S::Result>, Error>
    where
        LO: LimitMarker,
    {
        let decoder = self.selector.select(&mut self.ctx);
        self.condition.add_to_builder(&mut self.ctx);

        let columns = self.ctx.get_selects();
        let joins = self.ctx.get_joins();

        let condition = self.condition.into_option();
        let condition = condition
            .as_ref()
            .map(|condition| condition.as_sql(&self.ctx));

        database::query::<All>(
            self.executor,
            S::Model::TABLE,
            &columns,
            &joins,
            condition.as_ref(),
            self.ordering.as_slice(),
            self.lim_off.into_option(),
        )
        .await?
        .into_iter()
        .map(|x| {
            decoder
                .by_name(&x)
                .map_err(|_| Error::DecodeError("Could not decode row".to_string()))
        })
        .collect::<Result<Vec<_>, _>>()
    }

    /// Retrieve and decode the query as a stream
    pub fn stream<'stream>(mut self) -> QueryStream<'stream, 'c, S::Decoder>
    where
        'e: 'stream,
        'c: 'stream,
        S: 'stream,
        LO: LimitMarker,
    {
        let decoder = self.selector.select(&mut self.ctx);
        self.condition.add_to_builder(&mut self.ctx);

        QueryStream::new(
            decoder,
            self.ctx,
            self.condition.into_option(),
            move |ctx, conditions| {
                let condition = conditions.map(|c| c.as_sql(ctx));
                database::query::<Stream>(
                    self.executor,
                    S::Model::TABLE,
                    ctx.get_selects().as_slice(),
                    ctx.get_joins().as_slice(),
                    condition.as_ref(),
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
        let decoder = self.selector.select(&mut self.ctx);
        self.condition.add_to_builder(&mut self.ctx);

        let columns = self.ctx.get_selects();
        let joins = self.ctx.get_joins();

        let condition = self.condition.into_option();
        let condition = condition
            .as_ref()
            .map(|condition| condition.as_sql(&self.ctx));

        let row = database::query::<One>(
            self.executor,
            S::Model::TABLE,
            &columns,
            &joins,
            condition.as_ref(),
            self.ordering.as_slice(),
            self.lim_off.into_option(),
        )
        .await?;
        decoder
            .by_name(&row)
            .map_err(|_| Error::DecodeError("Could not decode row".to_string()))
    }

    /// Try to retrieve and decode a matching row
    pub async fn optional(mut self) -> Result<Option<S::Result>, Error>
    where
        LO: OffsetMarker,
    {
        let decoder = self.selector.select(&mut self.ctx);
        self.condition.add_to_builder(&mut self.ctx);

        let columns = self.ctx.get_selects();
        let joins = self.ctx.get_joins();

        let condition = self.condition.into_option();
        let condition = condition
            .as_ref()
            .map(|condition| condition.as_sql(&self.ctx));

        let row = database::query::<Optional>(
            self.executor,
            S::Model::TABLE,
            &columns,
            &joins,
            condition.as_ref(),
            self.ordering.as_slice(),
            self.lim_off.into_option(),
        )
        .await?;
        match row {
            None => Ok(None),
            Some(row) => {
                Ok(Some(decoder.by_name(&row).map_err(|_| {
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
/// # use rorm::{Model, Database, query, FieldAccess};
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
    (__internal $model:ident::F => $($segments:ident,)*) => {
        $($segments::)* $model
    };
    (__internal $segment:ident::$($remainder:ident)::+ => $($segments:ident,)*) => {
        query!(__internal $($remainder)::+ => $($segments,)* $segment,)
    };
    ($db:expr, ($(
        $($model:ident)::+.$($field:ident).+ $(($($args:tt)?))? $(as $patch:ty)?
    ),+ $(,)?)) => {
        $crate::query!(
            $db,
            ($(
                $($model)::+.$($field).+ $(($($args)?))? $(as $patch)?
            ),+),
            perm = ($(
                <query!(__internal $($model)::+ =>) as $crate::model::Model>::permissions()
                    .query_permission(),
            )+).0
        )
    };
    ($db:expr, ($(
        $($model:ident)::+.$($field:ident).+ $(($($args:tt)?))? $(as $patch:ty)?
    ),+$(,)?), perm = $perm:expr) => {
        $crate::crud::query::QueryBuilder::new(
            $db,
            ($(
                $($model)::+.$($field).+ $(($($args)?))? $(.select_as::<$patch>())?,
            )+),
            $perm,
        )
    };
    ($db:expr, $patch:ty) => {
        $crate::query!(
            $db,
            $patch,
            perm = <<$patch as $crate::model::Patch>::Model as $crate::model::Model>::permissions()
                .query_permission()
        )
    };
    ($db:expr, $patch:ty, perm = $perm:expr) => {
        $crate::crud::query::QueryBuilder::new(
            $db,
            $crate::model::PatchSelector::<$patch>::new(),
            $perm,
        )
    };
}

/// Sadly ouroboros doesn't handle the lifetime bounds required for the QueryStream very well.
/// This module's code is copied from ouroboros' expanded macro and the tailored to fit the lifetime bounds.
mod query_stream {
    use std::pin::Pin;
    use std::task::{Context, Poll};

    use rorm_db::executor::{QueryStrategyResult, Stream};
    use rorm_db::Error;

    use crate::conditions::Condition;
    use crate::crud::decoder::Decoder;
    use crate::internal::query_context::QueryContext;

    /// Self-referential struct storing the query's data next to the stream which borrows it.
    ///
    /// ## Lifetimes
    /// - `'this` is the self-referential struct's lifetime
    /// - `'cond` is the [`dyn Condition<'cond>`](Condition)'s lifetime
    ///     which needs to be separate from `'this` because it is invariant.
    #[pin_project::pin_project]
    #[allow(dead_code)] // The field's are never "read" because they are aliased before being assigned to the struct
    pub struct QueryStream<'this, 'cond: 'this, D> {
        decoder: D,

        ctx: Box<QueryContext>,

        condition: Option<Box<dyn Condition<'cond>>>,

        #[pin]
        stream: <Stream as QueryStrategyResult>::Result<'this>,
    }

    impl<'this, 'cond: 'this, D> QueryStream<'this, 'cond, D> {
        pub(crate) fn new(
            decoder: D,
            ctx: QueryContext,
            condition: Option<Box<dyn Condition<'cond>>>,
            stream_builder: impl FnOnce(
                &'this QueryContext,
                Option<&'this dyn Condition<'cond>>,
            ) -> <Stream as QueryStrategyResult>::Result<'this>,
        ) -> Self {
            unsafe fn change_lifetime<'old, 'new: 'old, T: 'new + ?Sized>(
                data: &'old T,
            ) -> &'new T {
                &*(data as *const _)
            }

            unsafe {
                let ctx = Box::new(ctx);
                let ctx_ref: &'this QueryContext = change_lifetime(ctx.as_ref());

                let condition_ref: Option<&'this dyn Condition<'cond>> = condition
                    .as_deref()
                    .map(|condition| change_lifetime(condition));

                let stream = stream_builder(ctx_ref, condition_ref);

                Self {
                    ctx,
                    condition,
                    decoder,
                    stream,
                }
            }
        }
    }

    impl<'this, 'cond: 'this, D: Decoder> futures::stream::Stream for QueryStream<'this, 'cond, D> {
        type Item = Result<D::Result, Error>;

        fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            let mut projection = self.project();
            projection.stream.as_mut().poll_next(cx).map(|option| {
                option.map(|result| result.and_then(|row| projection.decoder.by_name(&row)))
            })
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
    sealed!(trait);
}
impl LimOffMarker for () {
    sealed!(impl);
}
impl<O: OffsetMarker> LimOffMarker for Limit<O> {
    sealed!(impl);
}
impl LimOffMarker for u64 {
    sealed!(impl);
}

/// Marker for the generic parameter storing a limit.
///
/// Valid values are `()`, `Limit<()>` and `Limit<u64>`.
pub trait LimitMarker: LimOffMarker {
    sealed!(trait);

    /// Convert the generic limit into [`Option<LimitClause>`]
    fn into_option(self) -> Option<LimitClause>;
}
impl LimitMarker for () {
    sealed!(impl);

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
    sealed!(impl);

    fn into_option(self) -> Option<LimitClause> {
        Some(LimitClause {
            limit: self.limit,
            offset: self.offset.into_option(),
        })
    }
}

/// Unification of `()` and `Limit<()>`
pub trait AcceptsOffset: LimOffMarker {
    sealed!(trait);

    /// The resulting type i.e. `u64` or `Limit<u64>`
    type Result: LimOffMarker;
    /// "Add" the offset to the type
    fn add_offset(self, offset: u64) -> Self::Result;
}
impl AcceptsOffset for () {
    sealed!(impl);
    type Result = u64;
    fn add_offset(self, offset: u64) -> Self::Result {
        offset
    }
}
impl AcceptsOffset for Limit<()> {
    sealed!(impl);
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
    sealed!(trait);

    /// Convert the generic offset into [`Option<u64>`]
    fn into_option(self) -> Option<u64>;
}
impl OffsetMarker for () {
    sealed!(impl);
    fn into_option(self) -> Option<u64> {
        None
    }
}
impl OffsetMarker for u64 {
    sealed!(impl);
    fn into_option(self) -> Option<u64> {
        Some(self)
    }
}
