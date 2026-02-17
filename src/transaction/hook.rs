use std::future::{ready, Future};

use crate::transaction::{Transaction, TransactionError};

/// An object attached to a [`Transaction`]
/// which receives callback during the `Transaction`'s lifecycle.
pub trait TransactionHook: Send + 'static {
    /// Called when the `Transaction`'s owner calls `commit`
    /// before sending the actual `COMMIT` to the database.
    fn pre_commit<'a>(
        &'a mut self,
        _tx: &'a mut Transaction,
    ) -> impl Future<Output = Result<(), TransactionError>> + Send + use<'a, Self> {
        ready(Ok(()))
    }

    /// Called after the transaction has been commited successfully
    fn post_commit(&mut self) {}

    /// Called when the `Transaction`'s owner calls `rollback`
    /// OR the `Transaction` is dropped without being commited.
    ///
    /// This method WILL be run when the transaction is rolled back.
    /// It MAY be run before, during or after the actual database operation.
    fn on_rollback(&mut self) {}
}
