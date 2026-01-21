pub trait TransactionHook: Send + 'static {
    fn on_commit(&mut self) {}
    fn on_rollback(&mut self) {}
}

pub(super) struct ClosureOnFinish<T>(Option<T>);
pub(super) struct ClosureOnCommit<T>(Option<T>);
pub(super) struct ClosureOnRollback<T>(Option<T>);

impl<T> ClosureOnFinish<T> {
    pub fn new(hook: T) -> Self {
        Self(Some(hook))
    }
}
impl<T> ClosureOnCommit<T> {
    pub fn new(hook: T) -> Self {
        Self(Some(hook))
    }
}
impl<T> ClosureOnRollback<T> {
    pub fn new(hook: T) -> Self {
        Self(Some(hook))
    }
}

impl<T> TransactionHook for ClosureOnFinish<T>
where
    T: FnOnce(bool) + Send + 'static,
{
    fn on_commit(&mut self) {
        if let Some(closure) = self.0.take() {
            closure(true)
        }
    }

    fn on_rollback(&mut self) {
        if let Some(closure) = self.0.take() {
            closure(false)
        }
    }
}
impl<T> TransactionHook for ClosureOnCommit<T>
where
    T: FnOnce() + Send + 'static,
{
    fn on_commit(&mut self) {
        if let Some(closure) = self.0.take() {
            closure()
        }
    }
}
impl<T> TransactionHook for ClosureOnRollback<T>
where
    T: FnOnce() + Send + 'static,
{
    fn on_rollback(&mut self) {
        if let Some(closure) = self.0.take() {
            closure()
        }
    }
}
