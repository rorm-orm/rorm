//! Wrapper to implement [`TransactionHook`] for a closure

use std::future::Future;

use crate::transaction::{Transaction, TransactionError, TransactionHook};

/// Wrapper to implement [`TransactionHook`] for a closure
pub struct ClosureHook<C, L> {
    closure: Option<C>,
    _lifecycle: L,
}
impl<C, L> ClosureHook<C, L> {
    /// Wraps a `closure` to run during `lifecycle` of a [`TransactionHook`]
    ///
    /// `lifecycle` is one of:
    /// - [`PreCommit`]
    /// - [`PostCommit`]
    /// - [`OnRollback`]
    pub fn new(closure: C, lifecycle: L) -> Self
    where
        Self: TransactionHook,
    {
        Self {
            closure: Some(closure),
            _lifecycle: lifecycle,
        }
    }
}

/// Marker for [`TransactionHook::pre_commit`]
///
/// It is used by [`ClosureHook`] to determine when to run the closure.
pub struct PreCommit;

/// Marker for [`TransactionHook::post_commit`]
///
/// It is used by [`ClosureHook`] to determine when to run the closure.
pub struct PostCommit;

/// Marker for [`TransactionHook::on_rollback`]
///
/// It is used by [`ClosureHook`] to determine when to run the closure.
pub struct OnRollback;

impl<T, F> TransactionHook for ClosureHook<T, PreCommit>
where
    T: FnOnce() -> F + Send + 'static,
    F: Future<Output = Result<(), TransactionError>> + Send,
{
    async fn pre_commit(&mut self, _tx: &mut Transaction) -> Result<(), TransactionError> {
        if let Some(closure) = self.closure.take() {
            closure().await?;
        }
        Ok(())
    }
}
impl<T> TransactionHook for ClosureHook<T, PostCommit>
where
    T: FnOnce() + Send + 'static,
{
    fn post_commit(&mut self) {
        if let Some(closure) = self.closure.take() {
            closure()
        }
    }
}
impl<T> TransactionHook for ClosureHook<T, OnRollback>
where
    T: FnOnce() + Send + 'static,
{
    fn on_rollback(&mut self) {
        if let Some(closure) = self.closure.take() {
            closure()
        }
    }
}
