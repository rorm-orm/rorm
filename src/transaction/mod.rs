//! This module holds the definition of transactions

use crate::internal::any::AnyTransaction;
pub use crate::transaction::hook::TransactionHook;
use crate::transaction::hook::{ClosureOnCommit, ClosureOnFinish, ClosureOnRollback};
use crate::transaction::hook_vec::HookVec;
use crate::Error;

mod hook;
mod hook_vec;

/// Transactions can be used to provide a safe way to execute multiple SQL operations
/// after another with a way to go back to the start without something changed in the
/// database.
///
/// Can be obtained using [`Database::start_transaction`](crate::Database::start_transaction).
#[must_use = "A transaction needs to be committed."]
pub struct Transaction {
    pub(crate) sqlx: AnyTransaction,
    hooks: Option<HookVec>,
}

impl Transaction {
    pub(crate) fn new(sqlx: AnyTransaction) -> Self {
        Self { sqlx, hooks: None }
    }

    /// This function commits the transaction.
    pub async fn commit(self) -> Result<(), Error> {
        let result = self.sqlx.commit().await;

        if let Some(mut hooks) = self.hooks {
            if result.is_ok() {
                hooks.on_commit();
            } else {
                hooks.on_rollback();
            }
        }

        result.map_err(Error::SqlxError)
    }

    /// Use this function to abort the transaction.
    pub async fn rollback(self) -> Result<(), Error> {
        let result = self.sqlx.rollback().await;

        if let Some(mut hooks) = self.hooks {
            hooks.on_rollback();
        }

        result.map_err(Error::SqlxError)
    }

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
/// A hook is a closure which is called after a transaction has been finished.
pub struct SimpleHooksApi<'a>(&'a mut HookVec);
impl SimpleHooksApi<'_> {
    /// Adds a closure which is run if the transaction has been finished.
    ///
    /// The closure will be called with `true` if the transaction was successful and `false` otherwise.
    pub fn on_finish(&mut self, hook: impl FnOnce(bool) + Send + 'static) -> &mut Self {
        self.0.get_or_insert().push(ClosureOnFinish::new(hook));
        self
    }

    /// Adds a closure which is run if the transaction has been committed successfully.
    pub fn on_commit(&mut self, hook: impl FnOnce() + Send + 'static) -> &mut Self {
        self.0.get_or_insert().push(ClosureOnCommit::new(hook));
        self
    }

    /// Adds a closure which is run if the transaction has been rolled back.
    pub fn on_rollback(&mut self, hook: impl FnOnce() + Send + 'static) -> &mut Self {
        self.0.get_or_insert().push(ClosureOnRollback::new(hook));
        self
    }
}

/// Advanced API for adding hooks to [`Transaction`]s
///
/// A [`TransactionHook`] is a type which is called after a transaction has been finished.
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
pub struct AdvancedHooksApi<'a>(&'a mut HookVec);
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
    pub async fn commit(self) -> Result<(), Error> {
        if let TransactionGuard::Owned(tr) = self {
            tr.commit().await
        } else {
            Ok(())
        }
    }
}
