use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::transaction::TransactionHook;

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

    pub fn on_commit(&mut self) {
        for vec in self.by_type.values_mut() {
            vec.on_commit();
        }
    }

    pub fn on_rollback(&mut self) {
        for vec in self.by_type.values_mut() {
            vec.on_rollback();
        }
    }
}

trait VecOfHooks: Any + Send + 'static {
    fn on_commit(&mut self);
    fn on_rollback(&mut self);
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
    fn on_commit(&mut self) {
        for hook in self {
            hook.on_commit();
        }
    }

    fn on_rollback(&mut self) {
        for hook in self {
            hook.on_rollback();
        }
    }
}
