//! Query builder and macro
use std::marker::PhantomData;

use futures::stream::BoxStream;
use futures::{Stream, StreamExt};
use rorm_db::row::FromRow;
use rorm_db::transaction::Transaction;
use rorm_db::{conditional::Condition, error::Error, row::Row, Database};

use crate::crud::builder::{ConditionMarker, TransactionMarker};
use crate::model::{Model, Patch};

/// Specifies which columns to query and the rows should be decoded as.
pub trait Selector<M: Model> {
    /// Type as which rows should be decoded
    type Result: FromRow;

    /// Columns to query
    fn columns(&self) -> &[&'static str];
}

/// The [Selector] for patches.
///
/// # Why implement `Selector` on `SelectPatch<P>` instead of `P` directly?
/// Since a selector is used by reference, it needs a runtime value.
/// But there wouldn't be any data to create a patch's instance with.
/// On top of that all that data would be ignored anyway,
/// because the columns to query are stored in the patch type.
///
/// => So create a struct without data to "wrap" the patch type.
pub struct SelectPatch<P: Patch>(PhantomData<P>);
impl<P: Patch> SelectPatch<P> {
    /// Create a SelectPatch
    pub const fn new() -> Self {
        SelectPatch(PhantomData)
    }
}
impl<M: Model, P: Patch<Model = M>> Selector<M> for SelectPatch<P> {
    type Result = P;
    fn columns(&self) -> &[&'static str] {
        P::COLUMNS
    }
}

/// Builder for creating queries
pub struct QueryBuilder<
    'db,
    'a,
    M: Model,
    S: Selector<M>,
    C: ConditionMarker<'a>,
    T: TransactionMarker<'a, 'db>,
> {
    db: &'db Database,
    selector: &'a S,
    _phantom: PhantomData<M>,

    condition: C,
    transaction: T,
}

impl<'db, 'rf, M: Model, S: Selector<M>> QueryBuilder<'db, 'rf, M, S, (), ()> {
    /// Start building a query using a generic [Selector]
    pub fn new(db: &'db Database, selector: &'rf S) -> QueryBuilder<'db, 'rf, M, S, (), ()> {
        QueryBuilder {
            db,
            selector,
            _phantom: PhantomData,

            condition: (),
            transaction: (),
        }
    }
}

impl<'db, 'a, M: Model, S: Selector<M>, T: TransactionMarker<'a, 'db>>
    QueryBuilder<'db, 'a, M, S, (), T>
{
    /// Add a condition to the query
    pub fn condition(
        self,
        condition: Condition<'a>,
    ) -> QueryBuilder<'db, 'a, M, S, Condition<'a>, T> {
        #[rustfmt::skip]
        let QueryBuilder { db, selector, _phantom, transaction, .. } = self;
        #[rustfmt::skip]
        return QueryBuilder { db, selector, _phantom, condition, transaction, };
    }
}

impl<'db, 'a, M: Model, S: Selector<M>, C: ConditionMarker<'a>> QueryBuilder<'db, 'a, M, S, C, ()> {
    /// Add a transaction to the query
    pub fn transaction(
        self,
        transaction: &'a mut Transaction<'db>,
    ) -> QueryBuilder<'db, 'a, M, S, C, &'a mut Transaction<'db>> {
        #[rustfmt::skip]
        let QueryBuilder { db, selector, _phantom, condition, .. } = self;
        #[rustfmt::skip]
        return QueryBuilder { db, selector, _phantom, condition, transaction, };
    }
}

impl<
        'rf,
        'db: 'rf,
        M: Model,
        S: Selector<M>,
        C: ConditionMarker<'rf>,
        T: TransactionMarker<'rf, 'db>,
    > QueryBuilder<'db, 'rf, M, S, C, T>
{
    /// Retrieve all matching rows
    pub async fn all_as_rows(self) -> Result<Vec<Row>, Error> {
        self.db
            .query_all(
                M::table_name(),
                self.selector.columns(),
                self.condition.as_option(),
                self.transaction.into_option(),
            )
            .await
    }

    /// Retrieve exactly one matching row
    ///
    /// An error is returned if no value could be retrieved.
    pub async fn one_as_row(self) -> Result<Row, Error> {
        self.db
            .query_one(
                M::table_name(),
                self.selector.columns(),
                self.condition.as_option(),
                self.transaction.into_option(),
            )
            .await
    }

    /// Retrieve the query as a stream of rows
    pub fn stream_as_row(self) -> BoxStream<'rf, Result<Row, Error>> {
        self.db.query_stream(
            M::table_name(),
            self.selector.columns(),
            self.condition.as_option(),
            self.transaction.into_option(),
        )
    }

    /// Try to retrieve the a matching row
    pub async fn optional_as_row(self) -> Result<Option<Row>, Error> {
        self.db
            .query_optional(
                M::table_name(),
                self.selector.columns(),
                self.condition.as_option(),
                self.transaction.into_option(),
            )
            .await
    }

    /// Retrieve and decode all matching rows
    pub async fn all(self) -> Result<Vec<S::Result>, Error> {
        self.all_as_rows()
            .await?
            .into_iter()
            .map(|x| {
                S::Result::from_row(x)
                    .map_err(|_| Error::DecodeError("Could not decode row".to_string()))
            })
            .collect::<Result<Vec<_>, _>>()
    }

    /// Retrieve and decode exactly one matching row
    ///
    /// An error is returned if no value could be retrieved.
    pub async fn one(self) -> Result<S::Result, Error> {
        S::Result::from_row(self.one_as_row().await?)
            .map_err(|_| Error::DecodeError("Could not decode row".to_string()))
    }

    /// Retrieve and decode the query as a stream
    pub fn stream(self) -> impl Stream<Item = Result<S::Result, Error>> + 'rf {
        self.stream_as_row().map(|row| {
            S::Result::from_row(row?)
                .map_err(|_| Error::DecodeError("Could not decode row".to_string()))
        })
    }

    /// Try to retrieve and decode the a matching row
    pub async fn optional(self) -> Result<Option<S::Result>, Error> {
        match self.optional_as_row().await? {
            None => Ok(None),
            Some(row) => {
                Ok(Some(S::Result::from_row(row).map_err(|_| {
                    Error::DecodeError("Could not decode row".to_string())
                })?))
            }
        }
    }
}

/// Slightly less verbose macro to start a [`QueryBuilder`] from a model or patch
#[macro_export]
macro_rules! query {
    ($db:expr, $patch:path) => {
        $crate::crud::query::QueryBuilder::new(
            $db,
            &$crate::crud::query::SelectPatch::<$patch>::new(),
        )
    };
}
