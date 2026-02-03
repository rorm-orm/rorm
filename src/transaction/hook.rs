use std::future::{ready, Future};

use crate::transaction::{HookError, Transaction, TransactionError};

/// An object attached to a [`Transaction`]
/// which receives callback during the `Transaction`'s lifecycle.
///
/// TODO: more docs
pub trait TransactionHook: Send + 'static {
    /// Called when the `Transaction`'s owner calls `commit`
    /// BUT BEFORE sending the commit to the database.
    fn pre_commit<'a>(
        &'a mut self,
        _tx: &'a mut Transaction,
    ) -> impl Future<Output = Result<(), TransactionError>> + Send + use<'a, Self> {
        ready(Ok(()))
    }

    /// Called when the `Transaction`'s owner calls `rollback`
    /// OR the `Transaction` is dropped without being commited
    /// BUT BEFORE the rollback is sent to the database.
    fn pre_rollback(
        &mut self,
    ) -> impl Future<Output = Result<(), HookError>> + Send + use<'_, Self> {
        ready(Ok(()))
    }

    /// Called after the transaction has been commited successfully
    fn post_commit(&mut self) {}

    /// Called after the transaction has been rolled back
    fn post_rollback(&mut self) {}
}

pub struct ClosureHook<C, L> {
    closure: Option<C>,
    _lifecycle: L,
}
impl<C, L> ClosureHook<C, L> {
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

pub struct PreFinish;
pub struct PreCommit;
pub struct PreRollback;
pub struct PostFinish;
pub struct PostCommit;
pub struct PostRollback;

impl<T, F> TransactionHook for ClosureHook<T, PreFinish>
where
    T: FnOnce(bool) -> F + Send + 'static,
    F: Future<Output = Result<(), HookError>> + Send,
{
    async fn pre_commit(&mut self, _tx: &mut Transaction) -> Result<(), TransactionError> {
        if let Some(closure) = self.closure.take() {
            closure(true).await?;
        }
        Ok(())
    }

    async fn pre_rollback(&mut self) -> Result<(), HookError> {
        if let Some(closure) = self.closure.take() {
            closure(false).await?;
        }
        Ok(())
    }
}
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
impl<T, F> TransactionHook for ClosureHook<T, PreRollback>
where
    T: FnOnce() -> F + Send + 'static,
    F: Future<Output = Result<(), HookError>> + Send,
{
    async fn pre_rollback(&mut self) -> Result<(), HookError> {
        if let Some(closure) = self.closure.take() {
            closure().await?;
        }
        Ok(())
    }
}
impl<T> TransactionHook for ClosureHook<T, PostFinish>
where
    T: FnOnce(bool) + Send + 'static,
{
    fn post_commit(&mut self) {
        if let Some(closure) = self.closure.take() {
            closure(true)
        }
    }

    fn post_rollback(&mut self) {
        if let Some(closure) = self.closure.take() {
            closure(false)
        }
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
impl<T> TransactionHook for ClosureHook<T, PostRollback>
where
    T: FnOnce() + Send + 'static,
{
    fn post_rollback(&mut self) {
        if let Some(closure) = self.closure.take() {
            closure()
        }
    }
}
