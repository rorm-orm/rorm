//! This module holds the definition of transactions

use std::error::Error as StdError;
use std::fmt;
use std::future::Future;

use tracing::debug;

use crate::internal::any::AnyTransaction;
pub use crate::transaction::hook::TransactionHook;
use crate::transaction::hook_closure::{ClosureHook, OnRollback, PostCommit, PreCommit};
use crate::transaction::hook_storage::HookStorage;
use crate::Error;

mod hook;
mod hook_closure;
mod hook_storage;

/// Transactions can be used to provide a safe way to execute multiple SQL operations
/// after another with a way to go back to the start without something changed in the
/// database.
///
/// Can be obtained using [`Database::start_transaction`](crate::Database::start_transaction).
#[must_use = "A transaction needs to be committed."]
pub struct Transaction {
    pub(crate) sqlx: AnyTransaction,
    hooks: Option<HookStorage>,
}

impl Transaction {
    pub(crate) fn new(sqlx: AnyTransaction) -> Self {
        Self { sqlx, hooks: None }
    }

    /// This function commits the transaction.
    pub async fn commit(mut self) -> Result<(), TransactionError> {
        let mut hooks = self.hooks.take();

        if let Some(hooks) = hooks.as_mut() {
            hooks.pre_commit(&mut self).await?;

            if let Some(invalid_hooks) = self.hooks.as_mut() {
                debug!("Some transaction hook added additional hooks during pre-commit. This is not supported and will be ignored.");

                // Prevent `Drop` impl from calling `on_rollback`.
                invalid_hooks.clear();
            }
        }

        let result = self.sqlx.commit().await;

        if let Some(hooks) = hooks.as_mut() {
            if result.is_ok() {
                hooks.post_commit();

                // Prevent `Drop` impl from calling `on_rollback`.
                hooks.clear();
            }
        }

        result
            .map_err(Error::SqlxError)
            .map_err(TransactionError::Database)
    }

    /// Use this function to abort the transaction.
    pub async fn rollback(self) -> Result<(), Error> {
        self.sqlx.rollback().await.map_err(Error::SqlxError)
    }
}

// This impl should be on `Transaction` itself.
// However, the `sqlx` field has to be consumed by ownership
// which prevents `Transaction` from implementing `Drop`.
impl Drop for HookStorage {
    fn drop(&mut self) {
        // `Transaction::commit` will clear all hooks, so this call would become a no-op.
        self.on_rollback();
    }
}

impl Transaction {
    /// Accesses the simple API for adding hooks to the transaction
    ///
    /// If you reach the API's limits, consider [`Transaction::adv_hooks`].
    pub fn hooks(&mut self) -> SimpleHooksApi<'_> {
        SimpleHooksApi(self.hooks.get_or_insert_default())
    }

    /// Accesses the advanced API for adding hooks to the transaction
    ///
    /// If you're new to transaction hooks, consider [`Transaction::hooks`].
    pub fn adv_hooks(&mut self) -> AdvancedHooksApi<'_> {
        AdvancedHooksApi(self.hooks.get_or_insert_default())
    }
}

/// Simple API for adding hooks to [`Transaction`]s
///
/// A hook is a closure which is called before or after a transaction has been commited.
pub struct SimpleHooksApi<'a>(&'a mut HookStorage);
impl SimpleHooksApi<'_> {
    /// Adds an async closure which is run before the transaction is commited.
    ///
    /// Note, the transaction could still fail due to a database error or a hook error.
    pub fn pre_commit<F>(&mut self, hook: impl FnOnce() -> F + Send + 'static) -> &mut Self
    where
        F: Future<Output = Result<(), TransactionError>> + Send,
    {
        self.0
            .get_or_insert()
            .push(ClosureHook::new(hook, PreCommit));
        self
    }

    /// Adds a closure which is run after the transaction has been commited successfully.
    pub fn post_commit(&mut self, hook: impl FnOnce() + Send + 'static) -> &mut Self {
        self.0
            .get_or_insert()
            .push(ClosureHook::new(hook, PostCommit));
        self
    }

    /// Adds a closure which is run when the transaction is rolled back.
    ///
    /// It MAY be called before, during or after the actual database operation.
    pub fn on_rollback(&mut self, hook: impl FnOnce() + Send + 'static) -> &mut Self {
        self.0
            .get_or_insert()
            .push(ClosureHook::new(hook, OnRollback));
        self
    }
}

/// Advanced API for adding hooks to [`Transaction`]s
///
/// A [`TransactionHook`] is a type which is called before and after a transaction has been finished.
///
/// A `Transaction` can store many instances of many `TransactionHook` types.
///
/// This API provides convenience methods for two common patters:
/// - [`push`](Self::push) for adding many instances (potentially of the same type)
/// - [`get_or_insert_default`](Self::get_or_insert_default) and [`get_or_insert_with`](Self::get_or_insert_with)
///   when you only want a single instance of your hook type but want to extend it several times.
///
/// If these APIs are not flexible enough, you can use [`get_all`](Self::get_all) to access the raw
/// storage of `TransactionHook`s of a single type.
pub struct AdvancedHooksApi<'a>(&'a mut HookStorage);
impl AdvancedHooksApi<'_> {
    /// Adds a hook which is called if the transaction has been finished.
    pub fn push<T: TransactionHook>(&mut self, hook: T) {
        self.get_all().push(hook);
    }

    /// Gets the hook of type `T`.
    ///
    /// Adds its [`Default`] value if no value has been added yet.
    pub fn get_or_insert_default<T: TransactionHook + Default>(&mut self) -> &mut T {
        self.get_or_insert_with(T::default)
    }

    /// Gets the hook of type `T`.
    ///
    /// Calls `init` to add a value if no value has been added yet.
    pub fn get_or_insert_with<T: TransactionHook>(&mut self, init: impl FnOnce() -> T) -> &mut T {
        let vec = self.get_all();
        if vec.is_empty() {
            vec.push(init());
        }
        &mut vec[0]
    }

    /// Gets all hooks of type `T`.
    pub fn get_all<T: TransactionHook>(&mut self) -> &mut Vec<T> {
        self.0.get_or_insert()
    }
}

/// Error for committing a [`Transaction`]
#[derive(Debug)]
pub enum TransactionError {
    /// Error returned by the database
    Database(Error),

    /// Arbitrary error returned by a hook
    Hook(HookError),
}
/// Arbitrary error returned by a hook
pub type HookError = Box<dyn StdError + Send + Sync>;

impl From<Error> for TransactionError {
    fn from(value: Error) -> Self {
        Self::Database(value)
    }
}
impl From<HookError> for TransactionError {
    fn from(value: HookError) -> Self {
        Self::Hook(value)
    }
}
impl fmt::Display for TransactionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionError::Database(x) => fmt::Display::fmt(x, f),
            TransactionError::Hook(x) => fmt::Display::fmt(x, f),
        }
    }
}
impl StdError for TransactionError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            TransactionError::Database(x) => Some(x),
            TransactionError::Hook(x) => Some(x.as_ref()),
        }
    }
}

/// Either an owned or borrowed [`Transaction`].
///
/// "Guarding" a piece of code which has to be run in an transaction
/// (see [`Executor::ensure_transaction`](crate::executor::Executor::ensure_transaction))
#[must_use = "The potentially owned transaction needs to be committed."]
pub enum TransactionGuard<'tr> {
    /// An owned transaction
    Owned(Transaction),

    /// A borrowed transaction
    Borrowed(&'tr mut Transaction),
}

impl TransactionGuard<'_> {
    /// Get a reference to the guarded transaction
    pub fn get_transaction(&mut self) -> &mut Transaction {
        match self {
            TransactionGuard::Owned(tr) => tr,
            TransactionGuard::Borrowed(tr) => tr,
        }
    }

    /// Consume the guard, committing the potentially owned transaction.
    pub async fn commit(self) -> Result<(), TransactionError> {
        if let TransactionGuard::Owned(tr) = self {
            tr.commit().await
        } else {
            Ok(())
        }
    }
}
