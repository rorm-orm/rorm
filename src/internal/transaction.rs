use crate::internal::any::AnyTransaction;
use crate::transaction::Transaction;
use crate::Error;

pub(crate) type Impl = AnyTransaction;

/// Implementation of [Transaction::commit]
pub(crate) async fn commit(transaction: Transaction) -> Result<(), Error> {
    transaction.0.commit().await.map_err(Error::SqlxError)
}

/// Implementation of [Transaction::rollback]
pub(crate) async fn rollback(transaction: Transaction) -> Result<(), Error> {
    transaction.0.rollback().await.map_err(Error::SqlxError)
}
