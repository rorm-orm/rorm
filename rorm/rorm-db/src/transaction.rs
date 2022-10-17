use sqlx::Any;

use crate::Error;

/**
Transactions can be used to provide a safe way to execute multiple SQL operations
after another with a way to go back to the start without something changed in the
database.

Can be obtained using [crate::Database::start_transaction].
*/
pub struct Transaction<'db> {
    pub(crate) tx: sqlx::Transaction<'db, Any>,
}

impl<'db> Transaction<'db> {
    /**
    This function commits the transaction.
    */
    pub async fn commit(self) -> Result<(), Error> {
        self.tx.commit().await.map_err(|err| Error::SqlxError(err))
    }

    /**
    Use this function to abort the transaction.
    */
    pub async fn rollback(self) -> Result<(), Error> {
        self.tx
            .rollback()
            .await
            .map_err(|err| Error::SqlxError(err))
    }
}
