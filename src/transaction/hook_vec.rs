use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use crate::transaction::{HookError, Transaction, TransactionError, TransactionHook};

#[derive(Default)]
pub struct HookVec {
    by_type: HashMap<TypeId, Box<dyn VecOfHooks>>,
}

impl HookVec {
    pub fn get_or_insert<T: TransactionHook>(&mut self) -> &mut Vec<T> {
        self.by_type
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(Vec::<T>::new()))
            .downcast()
            .unwrap()
    }

    pub async fn pre_commit(&mut self, tx: &mut Transaction) -> Result<(), TransactionError> {
        for vec in self.by_type.values_mut() {
            vec.pre_commit(tx).await?;
        }
        Ok(())
    }

    pub async fn pre_rollback(&mut self) -> Result<(), HookError> {
        for vec in self.by_type.values_mut() {
            vec.pre_rollback().await?;
        }
        Ok(())
    }

    pub fn post_commit(&mut self) {
        for vec in self.by_type.values_mut() {
            vec.post_commit();
        }
    }

    pub fn post_rollback(&mut self) {
        for vec in self.by_type.values_mut() {
            vec.post_rollback();
        }
    }
}

trait VecOfHooks: Any + Send + 'static {
    fn pre_commit<'a>(
        &'a mut self,
        tx: &'a mut Transaction,
    ) -> Pin<Box<dyn Future<Output = Result<(), TransactionError>> + Send + 'a>>;
    fn pre_rollback(&mut self) -> Pin<Box<dyn Future<Output = Result<(), HookError>> + Send + '_>>;
    fn post_commit(&mut self);
    fn post_rollback(&mut self);
}
impl dyn VecOfHooks {
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

    fn pre_rollback(&mut self) -> Pin<Box<dyn Future<Output = Result<(), HookError>> + Send + '_>> {
        Box::pin(async {
            for hook in self {
                hook.pre_rollback().await?;
            }
            Ok(())
        })
    }

    fn post_commit(&mut self) {
        for hook in self {
            hook.post_commit();
        }
    }

    fn post_rollback(&mut self) {
        for hook in self {
            hook.post_rollback();
        }
    }
}
