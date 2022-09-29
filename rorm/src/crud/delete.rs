//! Delete builder

use crate::conditional::Condition;
use crate::crud::builder::ConditionMarker;
use crate::model::Model;
use rorm_db::error::Error;
use rorm_db::Database;
use std::future::{Future, IntoFuture};
use std::marker::PhantomData;
use std::pin::Pin;

/// Builder for delete queries
pub struct DeleteBuilder<'a, M: Model, C: ConditionMarker<'a>> {
    db: &'a Database,
    condition: C,

    _phantom: PhantomData<*const M>,
}

impl<'a, M: Model> DeleteBuilder<'a, M, ()> {
    /// Start building a delete query
    pub fn new(db: &'a Database) -> Self {
        DeleteBuilder {
            db,
            condition: (),

            _phantom: PhantomData,
        }
    }

    /// Set the condition to delete a single model instance
    pub fn instance(&'a self, model: &'a M) -> DeleteBuilder<'a, M, Condition> {
        DeleteBuilder {
            db: self.db,
            condition: model.as_condition(),

            _phantom: PhantomData,
        }
    }
}

impl<'a, M: Model> DeleteBuilder<'a, M, ()> {
    /// Add a condition to the delete query
    pub fn condition(&self, condition: Condition<'a>) -> DeleteBuilder<'a, M, Condition<'a>> {
        DeleteBuilder {
            db: self.db,
            condition,

            _phantom: PhantomData,
        }
    }
}

impl<'a, M: Model> DeleteBuilder<'a, M, ()> {
    /// Delete all columns
    pub async fn all(&self) -> Result<u64, Error> {
        self.db.delete(M::table_name(), None, None).await
    }
}

impl<'a, M: Model, C: ConditionMarker<'a>> DeleteBuilder<'a, M, C> {
    /// Perform the delete operation
    async fn exec(self) -> Result<u64, Error> {
        self.db
            .delete(M::table_name(), self.condition.as_option(), None)
            .await
    }
}

impl<'a, M: Model + 'a> IntoFuture for DeleteBuilder<'a, M, Condition<'a>> {
    type Output = Result<u64, Error>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + 'a>>;

    /// Convert a [DeleteBuilder] with a [Condition] into a [Future] implicitly
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.exec())
    }
}
