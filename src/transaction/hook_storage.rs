//! Internal storage of [`TransactionHook`]s

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use crate::transaction::{Transaction, TransactionError, TransactionHook};

/// Internal storage of [`TransactionHook`]s
#[derive(Default)]
pub struct HookStorage {
    /// Set of `Vec<impl TransactionHook>`s
    by_type: HashMap<TypeId, Box<dyn VecOfHooks>>,
}

impl HookStorage {
    /// Retrieves a `Vec` to store hooks of type `T` in
    pub fn get_or_insert<T: TransactionHook>(&mut self) -> &mut Vec<T> {
        self.by_type
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(Vec::<T>::new()))
            .downcast()
            .unwrap()
    }

    /// Removes all hooks
    pub fn clear(&mut self) {
        self.by_type.clear();
    }

    /// Executes every hook's [`TransactionHook::pre_commit`] method
    pub async fn pre_commit(&mut self, tx: &mut Transaction) -> Result<(), TransactionError> {
        for vec in self.by_type.values_mut() {
            vec.pre_commit(tx).await?;
        }
        Ok(())
    }

    /// Executes every hook's [`TransactionHook::post_commit`] method
    pub fn post_commit(&mut self) {
        for vec in self.by_type.values_mut() {
            vec.post_commit();
        }
    }

    /// Executes every hook's [`TransactionHook::on_rollback`] method
    pub fn on_rollback(&mut self) {
        for vec in self.by_type.values_mut() {
            vec.on_rollback();
        }
    }
}

/// Dyn-compatible trait implemented for `Vec<impl TransactionHook>`.
///
/// It is a sub-trait of `Any` and implements a `downcast` method for its `dyn` type.
trait VecOfHooks: Any + Send + 'static {
    /// Executes every hook's [`TransactionHook::pre_commit`] method
    fn pre_commit<'a>(
        &'a mut self,
        tx: &'a mut Transaction,
    ) -> Pin<Box<dyn Future<Output = Result<(), TransactionError>> + Send + 'a>>;

    /// Executes every hook's [`TransactionHook::post_commit`] method
    fn post_commit(&mut self);

    /// Executes every hook's [`TransactionHook::on_rollback`] method
    fn on_rollback(&mut self);
}
impl dyn VecOfHooks {
    /// Tries to cast the `dyn` type into the concrete `Vec<impl TransactionHook>`
    fn downcast<T: TransactionHook>(&mut self) -> Option<&mut Vec<T>> {
        let this: &mut dyn Any = self;
        this.downcast_mut()
    }
}
impl<T> VecOfHooks for Vec<T>
where
    T: TransactionHook,
{
    fn pre_commit<'a>(
        &'a mut self,
        tx: &'a mut Transaction,
    ) -> Pin<Box<dyn Future<Output = Result<(), TransactionError>> + Send + 'a>> {
        Box::pin(async {
            for hook in self {
                hook.pre_commit(tx).await?;
            }
            Ok(())
        })
    }

    fn post_commit(&mut self) {
        for hook in self {
            hook.post_commit();
        }
    }

    fn on_rollback(&mut self) {
        for hook in self {
            hook.on_rollback();
        }
    }
}
